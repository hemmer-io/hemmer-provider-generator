//! Detect SDK configuration patterns

use crate::{workspace_detector::WorkspaceInfo, Result};
use hemmer_provider_generator_common::ProviderConfigAttr;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Result of config detection
#[derive(Debug, Clone)]
pub struct ConfigInfo {
    /// Config crate name (e.g., "aws-config")
    pub config_crate: Option<String>,
    /// Confidence in config crate detection
    pub config_crate_confidence: f32,
    /// Detected config attributes
    pub attributes: Vec<ProviderConfigAttr>,
    /// Confidence in attribute detection
    pub attributes_confidence: f32,
    /// Whether SDK uses async clients
    pub async_client: bool,
}

/// Detect configuration patterns from workspace
pub fn detect_config(repo_path: &Path, workspace: &WorkspaceInfo) -> Result<ConfigInfo> {
    // Find config crate
    let config_crate = workspace.config_crate().map(|p| p.name.clone());
    let config_crate_confidence = if config_crate.is_some() {
        0.95
    } else {
        0.0
    };

    // Try to detect config attributes if config crate exists
    let (attributes, attributes_confidence) = if let Some(ref cfg_crate) = config_crate {
        detect_config_attributes(repo_path, cfg_crate)?
    } else {
        (vec![], 0.0)
    };

    // Check for async patterns (default to true for modern SDKs)
    let async_client = detect_async_pattern(repo_path)?;

    Ok(ConfigInfo {
        config_crate,
        config_crate_confidence,
        attributes,
        attributes_confidence,
        async_client,
    })
}

/// Detect configuration attributes from config crate
fn detect_config_attributes(
    repo_path: &Path,
    config_crate: &str,
) -> Result<(Vec<ProviderConfigAttr>, f32)> {
    let mut attributes = Vec::new();

    // Common attributes to look for
    let common_attrs = [
        ("region", "Region to use for requests"),
        ("profile", "Named profile to use"),
        ("endpoint", "Custom endpoint URL"),
        ("credentials", "Authentication credentials"),
        ("timeout", "Request timeout"),
    ];

    // Search for builder patterns in config crate
    if let Some(found_attrs) = search_for_builder_methods(repo_path, config_crate, &common_attrs)?
    {
        attributes.extend(found_attrs);
    }

    // Deduplicate attributes by name (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    attributes.retain(|attr| seen.insert(attr.name.clone()));

    let confidence = if attributes.is_empty() {
        0.0
    } else {
        0.7 // Medium confidence - attributes detected but need review
    };

    Ok((attributes, confidence))
}

/// Search for builder methods in config crate source
fn search_for_builder_methods(
    repo_path: &Path,
    config_crate: &str,
    common_attrs: &[(&str, &str)],
) -> Result<Option<Vec<ProviderConfigAttr>>> {
    // Find config crate directory
    let config_dir = find_crate_dir(repo_path, config_crate)?;
    let Some(config_dir) = config_dir else {
        return Ok(None);
    };

    let src_dir = config_dir.join("src");
    if !src_dir.exists() {
        return Ok(None);
    }

    let mut found_attrs = Vec::new();

    // Walk source files looking for builder patterns
    for entry in WalkDir::new(&src_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            for (attr_name, description) in common_attrs {
                if content.contains(&format!("pub fn {attr_name}"))
                    || content.contains(&format!("fn {attr_name}"))
                {
                    found_attrs.push(ProviderConfigAttr {
                        name: attr_name.to_string(),
                        description: description.to_string(),
                        required: false,
                        setter_snippet: None,
                        value_extractor: None,
                    });
                }
            }
        }
    }

    Ok(Some(found_attrs))
}

/// Find crate directory by name
fn find_crate_dir(repo_path: &Path, crate_name: &str) -> Result<Option<std::path::PathBuf>> {
    // Try common locations
    let candidates = [
        repo_path.join(crate_name),
        repo_path.join("crates").join(crate_name),
        repo_path.join("sdk").join(crate_name),
    ];

    for candidate in &candidates {
        if candidate.join("Cargo.toml").exists() {
            return Ok(Some(candidate.clone()));
        }
    }

    Ok(None)
}

/// Detect async pattern usage in SDK
fn detect_async_pattern(_repo_path: &Path) -> Result<bool> {
    // For MVP, default to async (most modern SDKs use async)
    // In future, could parse source to check for .await usage
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace_detector::PackageInfo;

    #[test]
    fn test_config_crate_detection() {
        let workspace = WorkspaceInfo {
            packages: vec![
                PackageInfo {
                    name: "my-sdk-s3".to_string(),
                    version: "1.0.0".to_string(),
                    manifest_path: "".to_string(),
                },
                PackageInfo {
                    name: "my-config".to_string(),
                    version: "1.0.0".to_string(),
                    manifest_path: "".to_string(),
                },
            ],
            sdk_version: "1.0.0".to_string(),
            is_workspace: true,
        };

        // Config crate should be detected
        assert!(workspace.config_crate().is_some());
        assert_eq!(workspace.config_crate().unwrap().name, "my-config");
    }

    #[test]
    fn test_async_detection_default() {
        let result = detect_async_pattern(Path::new(".")).unwrap();
        assert!(result); // Default to true
    }
}
