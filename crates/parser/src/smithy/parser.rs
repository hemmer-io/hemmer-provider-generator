//! Smithy spec file parser

use super::types::SmithyModel;
use hemmer_provider_generator_common::{GeneratorError, Result, ServiceDefinition};
use std::fs;
use std::path::Path;

/// Smithy specification parser
///
/// Reads and parses Smithy JSON AST files from the AWS api-models-aws repository
pub struct SmithyParser {
    /// Loaded Smithy model
    model: SmithyModel,

    /// Service name (e.g., "s3", "ec2")
    service_name: String,

    /// SDK version
    sdk_version: String,
}

impl SmithyParser {
    /// Load Smithy model from file path
    ///
    /// # Example
    /// ```rust,ignore
    /// let parser = SmithyParser::from_file(
    ///     "api-models-aws/s3/2006-03-01/s3-2006-03-01.json",
    ///     "s3",
    ///     "1.0.0"
    /// )?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        service_name: &str,
        sdk_version: &str,
    ) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            GeneratorError::Parse(format!(
                "Failed to read Smithy file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        Self::from_json(&content, service_name, sdk_version)
    }

    /// Parse Smithy model from JSON string
    pub fn from_json(json: &str, service_name: &str, sdk_version: &str) -> Result<Self> {
        let model: SmithyModel = serde_json::from_str(json)
            .map_err(|e| GeneratorError::Parse(format!("Failed to parse Smithy JSON: {}", e)))?;

        Ok(Self {
            model,
            service_name: service_name.to_string(),
            sdk_version: sdk_version.to_string(),
        })
    }

    /// Parse Smithy model into ServiceDefinition IR
    pub fn parse(&self) -> Result<ServiceDefinition> {
        // Use the converter module to transform Smithy -> ServiceDefinition
        super::converter::convert_smithy_to_service_definition(
            &self.model,
            &self.service_name,
            &self.sdk_version,
        )
    }

    /// Get reference to the underlying Smithy model
    pub fn model(&self) -> &SmithyModel {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_smithy() {
        let smithy_json = r#"{
            "smithy": "2.0",
            "shapes": {
                "com.example#MyService": {
                    "type": "service",
                    "version": "1.0.0",
                    "operations": []
                }
            }
        }"#;

        let parser = SmithyParser::from_json(smithy_json, "example", "1.0.0");
        assert!(parser.is_ok());

        let parser = parser.unwrap();
        assert_eq!(parser.model.smithy, "2.0");
        assert_eq!(parser.model.shapes.len(), 1);
    }

    #[test]
    fn test_extract_service_name() {
        assert_eq!(
            SmithyModel::extract_service_name("com.amazonaws.s3#S3"),
            "s3"
        );
        assert_eq!(
            SmithyModel::extract_service_name("com.amazonaws.dynamodb#DynamoDB"),
            "dynamodb"
        );
    }
}
