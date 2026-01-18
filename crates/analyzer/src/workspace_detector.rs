//! Cargo workspace detection and analysis

use crate::{AnalyzerError, Result};
use cargo_metadata::{MetadataCommand, Package};
use std::path::Path;

/// Information about a Cargo workspace
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// All packages in the workspace
    pub packages: Vec<PackageInfo>,
    /// SDK version (from workspace metadata or first package)
    #[allow(dead_code)]
    pub sdk_version: String,
    /// Whether this is a workspace or single crate
    pub is_workspace: bool,
}

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub manifest_path: String,
}

impl WorkspaceInfo {
    /// Detect workspace structure from a path
    pub fn detect(repo_path: &Path) -> Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path(repo_path.join("Cargo.toml"))
            .exec()
            .map_err(|e| AnalyzerError::CargoMetadata(e.to_string()))?;

        let packages: Vec<PackageInfo> = metadata
            .workspace_packages()
            .iter()
            .map(|p| PackageInfo::from_package(p))
            .collect();

        if packages.is_empty() {
            return Err(AnalyzerError::NoSdkCrates);
        }

        // Determine SDK version (use first package version)
        let sdk_version = packages
            .first()
            .map(|p| p.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string());

        let is_workspace = metadata.workspace_members.len() > 1;

        Ok(Self {
            packages,
            sdk_version,
            is_workspace,
        })
    }

    /// Filter packages to likely SDK service crates
    /// Uses positive pattern matching for known SDK prefixes, then excludes infrastructure
    pub fn sdk_crates(&self) -> Vec<&PackageInfo> {
        self.packages
            .iter()
            .filter(|p| {
                let name = p.name.as_str();

                // First, check if it matches known SDK service patterns (positive matching)
                let matches_sdk_pattern = name.starts_with("aws-sdk-")
                    || name.starts_with("google-cloud-")
                    || name.starts_with("azure-sdk-")
                    || name.starts_with("gcp-sdk-");

                // For known patterns, only exclude if it's clearly infrastructure
                if matches_sdk_pattern {
                    return !Self::is_infrastructure_crate(name);
                }

                // For other crates, apply standard filtering
                !Self::is_non_sdk_crate(name) && !Self::is_infrastructure_crate(name)
            })
            .collect()
    }

    /// Check if crate name indicates a non-SDK crate (examples, tests, tools)
    fn is_non_sdk_crate(name: &str) -> bool {
        name.contains("example")
            || name.contains("test")
            || name.contains("tool")
            || name.contains("codegen")
            || name.ends_with("-macro")
    }

    /// Check if crate is infrastructure/support library rather than service crate
    fn is_infrastructure_crate(name: &str) -> bool {
        // Common infrastructure suffixes
        if name.ends_with("-config")
            || name.ends_with("-types")
            || name.ends_with("-core")
            || name.ends_with("-derive")
        {
            return true;
        }

        // Filter out generated crates with version suffixes (e.g., google-cloud-language-v2)
        // Check for pattern like -v1, -v2, -v1alpha, etc.
        if let Some(dash_pos) = name.rfind("-v") {
            let version_part = &name[dash_pos + 2..];
            if version_part.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                return true;
            }
        }

        // Common infrastructure components
        let infra_components = [
            "smithy",      // aws-smithy-*
            "runtime",     // aws-runtime, kube-runtime
            "credential",  // aws-credential-types
            "sigv4",       // aws-sigv4
            "auth",        // google-cloud-auth
            "base",        // google-cloud-base
            "gax",         // google-cloud-gax (Google API Extensions)
            "internal",    // gax-internal
            "lro",         // Long-running operations
            "wkt",         // Well-known types
            "util",        // test-utils, utilities
            "generated",   // Generated code directories
            "guide",       // Documentation/guides
            "integration", // Integration tests
            "root",        // Root/meta crates
            "validation",  // Validation helpers
        ];

        infra_components.iter().any(|component| name.contains(component))
    }

    /// Find config crate if it exists
    pub fn config_crate(&self) -> Option<&PackageInfo> {
        self.packages
            .iter()
            .find(|p| p.name.ends_with("-config") || p.name.contains("config"))
    }

    /// Detect if this is a monolithic SDK (single client for all resources)
    /// Returns true if < 3 service crates found (e.g., Kubernetes has kube-client)
    pub fn is_monolithic(&self) -> bool {
        self.sdk_crates().len() < 3
    }

    /// Get the main client crate for monolithic SDKs
    pub fn main_client_crate(&self) -> Option<&PackageInfo> {
        let sdk_crates = self.sdk_crates();
        if sdk_crates.len() == 1 {
            Some(sdk_crates[0])
        } else {
            // Look for crates with "client" in the name
            sdk_crates
                .iter()
                .find(|p| p.name.contains("client"))
                .copied()
                .or_else(|| sdk_crates.first().copied())
        }
    }
}

impl PackageInfo {
    fn from_package(package: &Package) -> Self {
        Self {
            name: package.name.clone(),
            version: package.version.to_string(),
            manifest_path: package.manifest_path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_crate_filtering() {
        let workspace = WorkspaceInfo {
            packages: vec![
                PackageInfo {
                    name: "aws-sdk-s3".to_string(),
                    version: "1.0.0".to_string(),
                    manifest_path: "".to_string(),
                },
                PackageInfo {
                    name: "aws-sdk-ec2".to_string(),
                    version: "1.0.0".to_string(),
                    manifest_path: "".to_string(),
                },
                PackageInfo {
                    name: "aws-config".to_string(),
                    version: "1.0.0".to_string(),
                    manifest_path: "".to_string(),
                },
                PackageInfo {
                    name: "examples".to_string(),
                    version: "1.0.0".to_string(),
                    manifest_path: "".to_string(),
                },
            ],
            sdk_version: "1.0.0".to_string(),
            is_workspace: true,
        };

        let sdk_crates = workspace.sdk_crates();
        assert_eq!(sdk_crates.len(), 2);
        assert!(sdk_crates.iter().any(|p| p.name == "aws-sdk-s3"));
        assert!(sdk_crates.iter().any(|p| p.name == "aws-sdk-ec2"));
    }

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

        let config = workspace.config_crate();
        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "my-config");
    }
}
