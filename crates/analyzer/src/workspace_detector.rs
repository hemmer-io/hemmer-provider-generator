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
    /// Excludes: examples, tests, tools, config crates, build scripts, infrastructure crates
    pub fn sdk_crates(&self) -> Vec<&PackageInfo> {
        self.packages
            .iter()
            .filter(|p| {
                let name = p.name.as_str();
                // Exclude common non-SDK patterns
                !name.contains("example")
                    && !name.contains("test")
                    && !name.contains("tool")
                    && !name.ends_with("-config")
                    && !name.ends_with("-types")
                    && !name.ends_with("-macro")
                    && !name.contains("codegen")
                    // Exclude infrastructure crates (AWS, GCP, etc.)
                    && !name.contains("smithy")      // aws-smithy-*
                    && !name.contains("runtime")     // aws-runtime, gcp-runtime, etc.
                    && !name.contains("credential")  // aws-credential-types
                    && !name.contains("sigv4")       // aws-sigv4
                    && !name.contains("auth")        // auth libraries
            })
            .collect()
    }

    /// Find config crate if it exists
    pub fn config_crate(&self) -> Option<&PackageInfo> {
        self.packages
            .iter()
            .find(|p| p.name.ends_with("-config") || p.name.contains("config"))
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
