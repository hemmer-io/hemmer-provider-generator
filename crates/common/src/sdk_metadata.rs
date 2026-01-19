//! SDK metadata loading from YAML files
//!
//! This module provides functionality to load provider SDK configuration from
//! external YAML metadata files instead of hardcoding them in Rust.

use crate::{ConfigCodegen, GeneratorError, ProviderConfigAttr, ProviderSdkConfig, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Root structure for provider SDK metadata YAML files
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderSdkMetadata {
    /// Metadata format version
    pub version: u32,
    /// Provider information
    pub provider: ProviderInfo,
    /// SDK configuration
    pub sdk: SdkInfo,
    /// Configuration code generation
    pub config: ConfigInfo,
    /// Error handling configuration
    pub errors: ErrorInfo,
}

/// Provider identification and display information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderInfo {
    /// Provider identifier (e.g., "aws", "gcp")
    pub name: String,
    /// Human-readable provider name (e.g., "Amazon Web Services")
    pub display_name: String,
}

/// SDK crate and client configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SdkInfo {
    /// SDK crate naming pattern (e.g., "aws-sdk-{service}")
    pub crate_pattern: String,
    /// Client type pattern (e.g., "aws_sdk_{service}::Client")
    pub client_type_pattern: String,
    /// Optional config crate name (e.g., "aws-config")
    #[serde(default)]
    pub config_crate: Option<String>,
    /// Whether SDK uses async clients
    pub async_client: bool,
    /// Optional region attribute name (e.g., "region", "location")
    #[serde(default)]
    pub region_attr: Option<String>,
    /// Additional dependencies beyond service SDK crate
    /// Format: ["aws-config = \"1\"", "aws-smithy-types = \"1\""]
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Configuration code generation patterns
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigInfo {
    /// Config initialization snippet configuration
    pub initialization: SnippetConfig,
    /// Config loading snippet configuration
    pub load: SnippetConfig,
    /// Client creation from config snippet configuration
    pub client_from_config: SnippetConfig,
    /// Provider-specific config attributes
    pub attributes: Vec<ConfigAttribute>,
}

/// Code snippet with variable naming
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnippetConfig {
    /// Code snippet template
    pub snippet: String,
    /// Variable name used in this snippet
    pub var_name: String,
}

/// Provider-specific configuration attribute
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigAttribute {
    /// Attribute name (e.g., "region", "profile")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this attribute is required
    pub required: bool,
    /// Optional code snippet for setting this value
    /// Uses {value} placeholder for the extracted JSON value
    #[serde(default)]
    pub setter: Option<String>,
    /// Optional value extraction expression
    /// Example: "as_str()", "as_i64()"
    #[serde(default)]
    pub extractor: Option<String>,
}

/// Error handling configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorInfo {
    /// Optional error metadata trait import path
    /// Example: "aws_smithy_types::error::metadata::ProvideErrorMetadata"
    #[serde(default)]
    pub metadata_import: Option<String>,
    /// Error code categorization map
    /// Maps ProviderError variant names to error code patterns
    /// Example: {"not_found": ["NotFound", "NoSuch*"]}
    #[serde(default)]
    pub categorization: HashMap<String, Vec<String>>,
}

impl ProviderSdkMetadata {
    /// Load metadata from a YAML file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            GeneratorError::Parse(format!("Failed to read metadata file {:?}: {}", path, e))
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            GeneratorError::Parse(format!(
                "Failed to parse metadata YAML from {:?}: {}",
                path, e
            ))
        })
    }

    /// Convert metadata to ProviderSdkConfig for code generation
    pub fn to_provider_config(&self) -> ProviderSdkConfig {
        ProviderSdkConfig {
            sdk_crate_pattern: self.sdk.crate_pattern.clone(),
            client_type_pattern: self.sdk.client_type_pattern.clone(),
            config_crate: self.sdk.config_crate.clone(),
            async_client: self.sdk.async_client,
            region_attr: self.sdk.region_attr.clone(),
            config_attrs: self
                .config
                .attributes
                .iter()
                .map(|attr| ProviderConfigAttr {
                    name: attr.name.clone(),
                    description: attr.description.clone(),
                    required: attr.required,
                    setter_snippet: attr.setter.clone(),
                    value_extractor: attr.extractor.clone(),
                })
                .collect(),
            config_codegen: ConfigCodegen {
                init_snippet: self.config.initialization.snippet.clone(),
                load_snippet: self.config.load.snippet.clone(),
                client_from_config: self.config.client_from_config.snippet.clone(),
                config_var_name: self.config.initialization.var_name.clone(),
                loaded_config_var_name: self.config.load.var_name.clone(),
            },
            additional_dependencies: self.sdk.dependencies.clone(),
            error_metadata_import: self.errors.metadata_import.clone(),
            error_categorization_fn: self.errors.generate_categorization_function(),
        }
    }
}

impl ErrorInfo {
    /// Generate error categorization function from the categorization map
    ///
    /// This generates a complete Rust function that categorizes SDK error codes
    /// into ProviderError variants. Supports both exact matches and wildcard patterns.
    ///
    /// Returns None if no categorization is defined.
    fn generate_categorization_function(&self) -> Option<String> {
        if self.categorization.is_empty() {
            return None;
        }

        let mut match_arms = Vec::new();

        // Map category names to ProviderError variants
        let category_to_variant = |cat: &str| -> &str {
            match cat {
                "not_found" => "NotFound",
                "already_exists" => "AlreadyExists",
                "permission_denied" => "PermissionDenied",
                "validation" => "Validation",
                "failed_precondition" => "FailedPrecondition",
                "resource_exhausted" => "ResourceExhausted",
                "unavailable" => "Unavailable",
                "deadline_exceeded" => "DeadlineExceeded",
                "unimplemented" => "Unimplemented",
                _ => "Sdk", // Fallback to generic SDK error
            }
        };

        for (category, patterns) in &self.categorization {
            let variant = category_to_variant(category);
            let conditions = self.generate_pattern_conditions(patterns);

            match_arms.push(format!(
                "        Some(c) if {} => {{\n            ProviderError::{}(message)\n        }}",
                conditions, variant
            ));
        }

        // Generate the complete function
        let function = format!(
            r#"
/// Categorize SDK error codes and convert to ProviderError
fn categorize_error_code(code: Option<&str>, message: String) -> ProviderError {{
    match code {{
{}
        _ => ProviderError::Sdk(message),
    }}
}}

/// Convert SDK error to ProviderError (which can be converted to tonic::Status)
fn sdk_error_to_provider_error<E, R>(error: &aws_smithy_runtime_api::client::result::SdkError<E, R>) -> ProviderError
where
    E: std::fmt::Debug + ProvideErrorMetadata,
    R: std::fmt::Debug,
{{
    let message = format!("{{:?}}", error);

    match error {{
        aws_smithy_runtime_api::client::result::SdkError::ServiceError(service_err) => {{
            let code = service_err.err().code();
            debug!("AWS error code: {{:?}}", code);
            categorize_error_code(code, message)
        }}
        aws_smithy_runtime_api::client::result::SdkError::TimeoutError(_) => {{
            ProviderError::DeadlineExceeded(message)
        }}
        aws_smithy_runtime_api::client::result::SdkError::DispatchFailure(_) => {{
            ProviderError::Unavailable(message)
        }}
        aws_smithy_runtime_api::client::result::SdkError::ResponseError(_) => {{
            ProviderError::Sdk(message)
        }}
        aws_smithy_runtime_api::client::result::SdkError::ConstructionFailure(_) => {{
            ProviderError::Validation(message)
        }}
        _ => ProviderError::Sdk(message),
    }}
}}
"#,
            match_arms.join("\n")
        );

        Some(function)
    }

    /// Generate pattern matching conditions from error code patterns
    ///
    /// Supports:
    /// - Exact matches: "NotFound" → `c == "NotFound"`
    /// - Prefix wildcards: "NoSuch*" → `c.starts_with("NoSuch")`
    /// - Suffix wildcards: "*InUse" → `c.ends_with("InUse")`
    /// - Contains wildcards: "*Limit*" → `c.contains("Limit")`
    fn generate_pattern_conditions(&self, patterns: &[String]) -> String {
        let conditions: Vec<String> = patterns
            .iter()
            .map(|pattern| {
                if pattern.starts_with('*') && pattern.ends_with('*') {
                    // Contains pattern: "*Limit*" → contains("Limit")
                    let inner = pattern.trim_matches('*');
                    format!("c.contains(\"{}\")", inner)
                } else if pattern.starts_with('*') {
                    // Suffix pattern: "*InUse" → ends_with("InUse")
                    let suffix = pattern.trim_start_matches('*');
                    format!("c.ends_with(\"{}\")", suffix)
                } else if pattern.ends_with('*') {
                    // Prefix pattern: "NoSuch*" → starts_with("NoSuch")
                    let prefix = pattern.trim_end_matches('*');
                    format!("c.starts_with(\"{}\")", prefix)
                } else {
                    // Exact match: "NotFound" → c == "NotFound"
                    format!("c == \"{}\"", pattern)
                }
            })
            .collect();

        conditions.join(" || ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pattern_conditions() {
        let error_info = ErrorInfo {
            metadata_import: None,
            categorization: HashMap::new(),
        };

        // Test exact match
        let patterns = vec!["NotFound".to_string()];
        assert_eq!(
            error_info.generate_pattern_conditions(&patterns),
            "c == \"NotFound\""
        );

        // Test prefix wildcard
        let patterns = vec!["NoSuch*".to_string()];
        assert_eq!(
            error_info.generate_pattern_conditions(&patterns),
            "c.starts_with(\"NoSuch\")"
        );

        // Test suffix wildcard
        let patterns = vec!["*InUse".to_string()];
        assert_eq!(
            error_info.generate_pattern_conditions(&patterns),
            "c.ends_with(\"InUse\")"
        );

        // Test contains wildcard
        let patterns = vec!["*Limit*".to_string()];
        assert_eq!(
            error_info.generate_pattern_conditions(&patterns),
            "c.contains(\"Limit\")"
        );

        // Test multiple patterns
        let patterns = vec![
            "NotFound".to_string(),
            "NoSuch*".to_string(),
            "ResourceNotFoundException".to_string(),
        ];
        assert_eq!(
            error_info.generate_pattern_conditions(&patterns),
            "c == \"NotFound\" || c.starts_with(\"NoSuch\") || c == \"ResourceNotFoundException\""
        );
    }

    #[test]
    fn test_category_to_variant_mapping() {
        let mut categorization = HashMap::new();
        categorization.insert("not_found".to_string(), vec!["NotFound".to_string()]);
        categorization.insert(
            "permission_denied".to_string(),
            vec!["AccessDenied".to_string()],
        );

        let error_info = ErrorInfo {
            metadata_import: None,
            categorization,
        };

        let function = error_info.generate_categorization_function();
        assert!(function.is_some());

        let function = function.unwrap();
        assert!(function.contains("ProviderError::NotFound"));
        assert!(function.contains("ProviderError::PermissionDenied"));
        assert!(function.contains("categorize_error_code"));
    }

    #[test]
    fn test_empty_categorization() {
        let error_info = ErrorInfo {
            metadata_import: None,
            categorization: HashMap::new(),
        };

        assert!(error_info.generate_categorization_function().is_none());
    }
}
