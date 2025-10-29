//! OpenAPI spec file parser

use super::types::OpenApiSpec;
use hemmer_provider_generator_common::{GeneratorError, Result, ServiceDefinition};
use std::fs;
use std::path::Path;

/// OpenAPI specification parser
///
/// Reads and parses OpenAPI 3.0 specifications from Kubernetes, Azure, or any
/// OpenAPI 3.0 compliant API.
pub struct OpenApiParser {
    /// Loaded OpenAPI spec
    spec: OpenApiSpec,

    /// Service name (e.g., "kubernetes", "compute")
    service_name: String,

    /// API version
    api_version: String,

    /// Provider type hint (optional)
    provider_hint: Option<ProviderHint>,
}

/// Provider type hint for OpenAPI specs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderHint {
    /// Kubernetes API
    Kubernetes,

    /// Azure Resource Manager
    Azure,

    /// Generic OpenAPI spec
    Generic,
}

impl OpenApiParser {
    /// Load OpenAPI spec from file path
    ///
    /// # Example
    /// ```rust,ignore
    /// let parser = OpenApiParser::from_file(
    ///     "k8s-openapi.json",
    ///     "kubernetes",
    ///     "1.27.0"
    /// )?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        service_name: &str,
        api_version: &str,
    ) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            GeneratorError::Parse(format!(
                "Failed to read OpenAPI file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        Self::from_json(&content, service_name, api_version)
    }

    /// Parse OpenAPI spec from JSON string
    pub fn from_json(json: &str, service_name: &str, api_version: &str) -> Result<Self> {
        let spec: OpenApiSpec = serde_json::from_str(json)
            .map_err(|e| GeneratorError::Parse(format!("Failed to parse OpenAPI JSON: {}", e)))?;

        Ok(Self {
            spec,
            service_name: service_name.to_string(),
            api_version: api_version.to_string(),
            provider_hint: None,
        })
    }

    /// Set provider hint for better parsing
    pub fn with_provider_hint(mut self, hint: ProviderHint) -> Self {
        self.provider_hint = Some(hint);
        self
    }

    /// Parse OpenAPI spec into ServiceDefinition IR
    pub fn parse(&self) -> Result<ServiceDefinition> {
        // Use the converter module to transform OpenAPI -> ServiceDefinition
        super::converter::convert_openapi_to_service_definition(
            &self.spec,
            &self.service_name,
            &self.api_version,
            self.provider_hint,
        )
    }

    /// Get reference to the underlying OpenAPI spec
    pub fn spec(&self) -> &OpenApiSpec {
        &self.spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_openapi() {
        let openapi_json = r#"{
            "openapi": "3.0.0",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {}
        }"#;

        let parser = OpenApiParser::from_json(openapi_json, "test", "1.0.0");
        assert!(parser.is_ok());

        let parser = parser.unwrap();
        assert_eq!(parser.spec.openapi, "3.0.0");
        assert_eq!(parser.spec.info.title, "Test API");
    }
}
