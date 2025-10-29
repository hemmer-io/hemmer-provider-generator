//! Discovery document parser

use super::types::DiscoveryDoc;
use hemmer_provider_generator_common::{GeneratorError, Result, ServiceDefinition};
use std::fs;
use std::path::Path;

/// Google Discovery Document parser
///
/// Reads and parses Google Cloud Discovery Documents for services like
/// Cloud Storage, Compute Engine, BigQuery, etc.
pub struct DiscoveryParser {
    /// Loaded Discovery document
    doc: DiscoveryDoc,

    /// Service name (e.g., "storage", "compute")
    service_name: String,

    /// API version (e.g., "v1", "v2")
    api_version: String,
}

impl DiscoveryParser {
    /// Load Discovery document from file path
    ///
    /// # Example
    /// ```rust,ignore
    /// let parser = DiscoveryParser::from_file(
    ///     "storage-v1.json",
    ///     "storage",
    ///     "v1"
    /// )?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        service_name: &str,
        api_version: &str,
    ) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            GeneratorError::Parse(format!(
                "Failed to read Discovery file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        Self::from_json(&content, service_name, api_version)
    }

    /// Parse Discovery document from JSON string
    pub fn from_json(json: &str, service_name: &str, api_version: &str) -> Result<Self> {
        let doc: DiscoveryDoc = serde_json::from_str(json)
            .map_err(|e| GeneratorError::Parse(format!("Failed to parse Discovery JSON: {}", e)))?;

        Ok(Self {
            doc,
            service_name: service_name.to_string(),
            api_version: api_version.to_string(),
        })
    }

    /// Parse Discovery document into ServiceDefinition IR
    pub fn parse(&self) -> Result<ServiceDefinition> {
        // Use the converter module to transform Discovery -> ServiceDefinition
        super::converter::convert_discovery_to_service_definition(
            &self.doc,
            &self.service_name,
            &self.api_version,
        )
    }

    /// Get reference to the underlying Discovery document
    pub fn doc(&self) -> &DiscoveryDoc {
        &self.doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_discovery() {
        let discovery_json = r##"{
            "discoveryVersion": "v1",
            "name": "storage",
            "version": "v1",
            "title": "Cloud Storage JSON API",
            "rootUrl": "https://storage.googleapis.com/",
            "servicePath": "storage/v1/"
        }"##;

        let parser = DiscoveryParser::from_json(discovery_json, "storage", "v1");
        assert!(parser.is_ok());

        let parser = parser.unwrap();
        assert_eq!(parser.doc.name, "storage");
        assert_eq!(parser.doc.version, "v1");
    }
}
