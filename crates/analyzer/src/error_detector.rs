//! Detect error categorization patterns

use crate::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{Item, ItemEnum};
use walkdir::WalkDir;

/// Result of error detection
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// Error metadata import path
    pub metadata_import: Option<String>,
    /// Categorization rules (category → error codes)
    pub categorization: HashMap<String, Vec<String>>,
    /// Confidence score
    pub confidence: f32,
}

/// Detect error categorization patterns
pub fn detect_errors(repo_path: &Path, crate_name: &str) -> Result<ErrorInfo> {
    // Try to find error enums in first SDK crate
    let error_variants = find_error_variants(repo_path, crate_name)?;

    let categorization = categorize_errors(&error_variants);
    let confidence = calculate_error_confidence(&categorization, &error_variants);

    // Try to detect metadata import
    let metadata_import = detect_metadata_import(repo_path, crate_name)?;

    Ok(ErrorInfo {
        metadata_import,
        categorization,
        confidence,
    })
}

/// Find error enum variants in SDK source
fn find_error_variants(repo_path: &Path, crate_name: &str) -> Result<Vec<String>> {
    let mut variants = Vec::new();

    // Common locations for error definitions
    let search_paths = [
        repo_path.join(crate_name),
        repo_path.join("crates").join(crate_name),
        repo_path.join("sdk").join(crate_name),
    ];

    for search_path in &search_paths {
        let src_dir = search_path.join("src");
        if !src_dir.exists() {
            continue;
        }

        // Walk source files
        for entry in WalkDir::new(&src_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.extension().and_then(|s| s.to_str()) == Some("rs")
                    && (path.file_name().and_then(|s| s.to_str()) == Some("error.rs")
                        || path.file_name().and_then(|s| s.to_str()) == Some("errors.rs")
                        || path
                            .to_string_lossy()
                            .contains(&format!("{crate_name}/src/types/error")))
            })
        {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                variants.extend(extract_error_variants(&content));
            }
        }
    }

    Ok(variants)
}

/// Extract error enum variants from source
fn extract_error_variants(source: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Parse source
    let Ok(syntax_tree) = syn::parse_file(source) else {
        return variants;
    };

    // Look for error enums
    for item in syntax_tree.items {
        if let Item::Enum(ItemEnum {
            ident,
            variants: enum_variants,
            ..
        }) = item
        {
            // Check if this looks like an error enum
            let ident_str = ident.to_string();
            if ident_str.contains("Error")
                || ident_str.contains("Exception")
                || ident_str.ends_with("Kind")
            {
                for variant in enum_variants {
                    variants.push(variant.ident.to_string());
                }
            }
        }
    }

    variants
}

/// Categorize error variants using heuristics
fn categorize_errors(variants: &[String]) -> HashMap<String, Vec<String>> {
    let mut categorization: HashMap<String, Vec<String>> = HashMap::new();

    // Heuristic patterns for common error categories
    let patterns = [
        (
            "not_found",
            vec!["NotFound", "NoSuch*", "ResourceNotFound*", "*NotExist*"],
        ),
        (
            "already_exists",
            vec![
                "AlreadyExists",
                "ResourceExists*",
                "*AlreadyExists",
                "*InUse",
                "Conflict",
            ],
        ),
        (
            "permission_denied",
            vec![
                "AccessDenied",
                "*Unauthorized",
                "Forbidden",
                "PermissionDenied",
            ],
        ),
        (
            "validation",
            vec![
                "Invalid*",
                "Malformed*",
                "ValidationException",
                "*Validation*",
            ],
        ),
        (
            "failed_precondition",
            vec!["PreconditionFailed", "ConditionNotMet", "*Precondition*"],
        ),
        (
            "resource_exhausted",
            vec![
                "LimitExceeded",
                "*Limit*",
                "QuotaExceeded",
                "TooMany*",
                "Throttl*",
            ],
        ),
        (
            "unavailable",
            vec!["ServiceUnavailable", "Unavailable", "*Unavailable"],
        ),
        (
            "deadline_exceeded",
            vec!["Timeout", "DeadlineExceeded", "*Timeout*"],
        ),
    ];

    for variant in variants {
        for (category, category_patterns) in &patterns {
            if matches_any_pattern(variant, category_patterns) {
                categorization
                    .entry(category.to_string())
                    .or_default()
                    .push(variant.clone());
            }
        }
    }

    categorization
}

/// Check if a variant matches any pattern (supports wildcards)
fn matches_any_pattern(variant: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            // Contains
            if let Some(inner) = pattern.strip_prefix('*').and_then(|p| p.strip_suffix('*')) {
                variant.contains(inner)
            } else {
                false
            }
        } else if let Some(suffix) = pattern.strip_prefix('*') {
            // Suffix
            variant.ends_with(suffix)
        } else if let Some(prefix) = pattern.strip_suffix('*') {
            // Prefix
            variant.starts_with(prefix)
        } else {
            // Exact match
            variant == *pattern
        }
    })
}

/// Calculate confidence in error categorization
fn calculate_error_confidence(
    categorization: &HashMap<String, Vec<String>>,
    all_variants: &[String],
) -> f32 {
    if all_variants.is_empty() {
        return 0.3; // Low confidence - no errors found
    }

    let categorized_count: usize = categorization.values().map(|v| v.len()).sum();
    let coverage = categorized_count as f32 / all_variants.len() as f32;

    // Error categorization is inherently low confidence
    // Even good coverage gets max 0.7 since it needs manual review
    match coverage {
        c if c >= 0.8 => 0.7,
        c if c >= 0.6 => 0.6,
        c if c >= 0.4 => 0.5,
        c if c >= 0.2 => 0.4,
        _ => 0.3,
    }
}

/// Detect error metadata import path
fn detect_metadata_import(_repo_path: &Path, crate_name: &str) -> Result<Option<String>> {
    // Common patterns for error metadata
    let _patterns = [
        format!(
            "{}_types::error::metadata::ProvideErrorMetadata",
            crate_name.replace('-', "_")
        ),
        "smithy_types::error::metadata::ProvideErrorMetadata".to_string(),
        "aws_smithy_types::error::metadata::ProvideErrorMetadata".to_string(),
    ];

    // For MVP, return None and let user fill in
    // In future, could search source files for actual imports
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_any_pattern("NotFound", &["NotFound"]));
        assert!(matches_any_pattern("NoSuchBucket", &["NoSuch*"]));
        assert!(matches_any_pattern(
            "AccessUnauthorized",
            &["*Unauthorized"]
        ));
        assert!(matches_any_pattern("InvalidRequest", &["Invalid*"]));
        assert!(matches_any_pattern("LimitExceeded", &["*Limit*"]));

        assert!(!matches_any_pattern("Success", &["NotFound", "Error*"]));
    }

    #[test]
    fn test_categorize_errors() {
        let variants = vec![
            "NotFound".to_string(),
            "NoSuchBucket".to_string(),
            "AccessDenied".to_string(),
            "InvalidRequest".to_string(),
            "ResourceAlreadyExists".to_string(),
        ];

        let categorization = categorize_errors(&variants);

        assert!(categorization.contains_key("not_found"));
        assert!(categorization.contains_key("permission_denied"));
        assert!(categorization.contains_key("validation"));
        assert!(categorization.contains_key("already_exists"));
    }

    #[test]
    fn test_extract_error_variants() {
        let source = r#"
            pub enum ServiceError {
                NotFound,
                AccessDenied,
                InvalidInput,
            }
        "#;

        let variants = extract_error_variants(source);
        assert_eq!(variants.len(), 3);
        assert!(variants.contains(&"NotFound".to_string()));
        assert!(variants.contains(&"AccessDenied".to_string()));
        assert!(variants.contains(&"InvalidInput".to_string()));
    }

    #[test]
    fn test_error_confidence() {
        let mut categorization = HashMap::new();
        categorization.insert("not_found".to_string(), vec!["NotFound".to_string()]);
        categorization.insert(
            "permission_denied".to_string(),
            vec!["AccessDenied".to_string()],
        );

        let all_variants = vec!["NotFound".to_string(), "AccessDenied".to_string()];

        let confidence = calculate_error_confidence(&categorization, &all_variants);
        assert!((0.6..=0.7).contains(&confidence));
    }
}
