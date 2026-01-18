//! Detect SDK crate naming patterns

use crate::workspace_detector::PackageInfo;
use regex::Regex;

/// Result of crate pattern detection
#[derive(Debug, Clone)]
pub struct CratePattern {
    /// Detected pattern (e.g., "aws-sdk-{service}")
    pub pattern: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Sample crate names used for detection
    #[allow(dead_code)]
    pub samples: Vec<String>,
}

/// Detect crate naming pattern from SDK crates
pub fn detect_pattern(crates: &[&PackageInfo]) -> CratePattern {
    if crates.is_empty() {
        return CratePattern {
            pattern: "unknown-{service}".to_string(),
            confidence: 0.0,
            samples: vec![],
        };
    }

    let samples: Vec<String> = crates.iter().map(|p| p.name.clone()).collect();

    // Try to find common prefix and suffix
    let pattern = find_common_pattern(&samples);
    let confidence = calculate_pattern_confidence(&samples, &pattern);

    CratePattern {
        pattern,
        confidence,
        samples: samples.iter().take(5).cloned().collect(), // Keep first 5 as samples
    }
}

/// Find common pattern in crate names
fn find_common_pattern(names: &[String]) -> String {
    if names.is_empty() {
        return "unknown-{service}".to_string();
    }

    if names.len() == 1 {
        // Single crate - try to infer pattern
        let name = &names[0];
        if let Some(last_dash) = name.rfind('-') {
            return format!("{}-{{service}}", &name[..last_dash]);
        }
        return "{service}".to_string();
    }

    // Find longest common prefix
    let prefix = find_common_prefix(names);

    // Find longest common suffix (excluding what looks like service name)
    let suffix = find_common_suffix(names);

    // Build pattern
    if !prefix.is_empty() && !suffix.is_empty() {
        format!("{prefix}{{service}}{suffix}")
    } else if !prefix.is_empty() {
        // Trim trailing dash/underscore
        let prefix = prefix.trim_end_matches('-').trim_end_matches('_');
        format!("{prefix}-{{service}}")
    } else if !suffix.is_empty() {
        let suffix = suffix.trim_start_matches('-').trim_start_matches('_');
        format!("{{service}}-{suffix}")
    } else {
        "{service}".to_string()
    }
}

/// Find longest common prefix in strings
fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let first = &strings[0];
    let mut prefix_len = first.len();

    for s in &strings[1..] {
        prefix_len = first
            .chars()
            .zip(s.chars())
            .take(prefix_len)
            .take_while(|(a, b)| a == b)
            .count();

        if prefix_len == 0 {
            return String::new();
        }
    }

    first.chars().take(prefix_len).collect()
}

/// Find longest common suffix in strings
fn find_common_suffix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    let first = &strings[0];
    let mut suffix_len = first.len();

    for s in &strings[1..] {
        suffix_len = first
            .chars()
            .rev()
            .zip(s.chars().rev())
            .take(suffix_len)
            .take_while(|(a, b)| a == b)
            .count();

        if suffix_len == 0 {
            return String::new();
        }
    }

    first.chars().rev().take(suffix_len).collect::<String>()
        .chars().rev().collect()
}

/// Calculate confidence score for pattern match
fn calculate_pattern_confidence(names: &[String], pattern: &str) -> f32 {
    if names.is_empty() {
        return 0.0;
    }

    // Convert pattern to regex
    let pattern_regex = pattern.replace("{service}", r"[a-z0-9_]+");
    let Ok(re) = Regex::new(&format!("^{pattern_regex}$")) else {
        return 0.0;
    };

    // Count matches
    let matches = names.iter().filter(|name| re.is_match(name)).count();
    let match_ratio = matches as f32 / names.len() as f32;

    // Confidence based on match ratio
    match match_ratio {
        r if r >= 0.95 => 1.0,
        r if r >= 0.90 => 0.95,
        r if r >= 0.80 => 0.85,
        r if r >= 0.70 => 0.75,
        r if r >= 0.60 => 0.65,
        r => r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_packages(names: &[&str]) -> Vec<PackageInfo> {
        names
            .iter()
            .map(|name| PackageInfo {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                manifest_path: "".to_string(),
            })
            .collect()
    }

    #[test]
    fn test_aws_style_pattern() {
        let packages = make_packages(&["aws-sdk-s3", "aws-sdk-ec2", "aws-sdk-lambda"]);
        let refs: Vec<_> = packages.iter().collect();
        let pattern = detect_pattern(&refs);

        assert_eq!(pattern.pattern, "aws-sdk-{service}");
        assert!(pattern.confidence >= 0.95);
    }

    #[test]
    fn test_gcp_style_pattern() {
        let packages = make_packages(&["gcp-sdk-storage", "gcp-sdk-compute", "gcp-sdk-bigquery"]);
        let refs: Vec<_> = packages.iter().collect();
        let pattern = detect_pattern(&refs);

        assert_eq!(pattern.pattern, "gcp-sdk-{service}");
        assert!(pattern.confidence >= 0.95);
    }

    #[test]
    fn test_single_crate() {
        let packages = make_packages(&["kubernetes-client"]);
        let refs: Vec<_> = packages.iter().collect();
        let pattern = detect_pattern(&refs);

        assert_eq!(pattern.pattern, "kubernetes-{service}");
    }

    #[test]
    fn test_common_prefix_suffix() {
        assert_eq!(
            find_common_prefix(&["aws-sdk-s3".to_string(), "aws-sdk-ec2".to_string()]),
            "aws-sdk-"
        );

        assert_eq!(
            find_common_suffix(&["s3-client".to_string(), "ec2-client".to_string()]),
            "-client"
        );
    }

    #[test]
    fn test_confidence_calculation() {
        let names = vec![
            "aws-sdk-s3".to_string(),
            "aws-sdk-ec2".to_string(),
            "aws-sdk-lambda".to_string(),
            "aws-config".to_string(), // Outlier
        ];

        // 3/4 = 0.75 match ratio
        let confidence = calculate_pattern_confidence(&names, "aws-sdk-{service}");
        assert!((confidence - 0.75).abs() < 0.01);
    }
}
