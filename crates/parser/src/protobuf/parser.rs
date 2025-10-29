//! Protobuf FileDescriptorSet parser

use hemmer_provider_generator_common::{GeneratorError, Result, ServiceDefinition};
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_types::FileDescriptorSet;
use std::fs;
use std::path::Path;

/// Protobuf/gRPC service parser
///
/// Parses a FileDescriptorSet (compiled .proto files) to extract
/// service definitions, RPC methods, and message types.
pub struct ProtobufParser {
    /// Descriptor pool for reflection
    pool: DescriptorPool,

    /// Service name (e.g., "storage", "compute")
    service_name: String,

    /// API version (e.g., "v1", "v2")
    api_version: String,
}

impl ProtobufParser {
    /// Load FileDescriptorSet from binary file
    ///
    /// # Example
    /// ```rust,ignore
    /// let parser = ProtobufParser::from_file(
    ///     "service.pb",
    ///     "storage",
    ///     "v1"
    /// )?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        service_name: &str,
        api_version: &str,
    ) -> Result<Self> {
        let bytes = fs::read(path.as_ref()).map_err(|e| {
            GeneratorError::Parse(format!(
                "Failed to read FileDescriptorSet file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        Self::from_file_descriptor_set(&bytes, service_name, api_version)
    }

    /// Parse FileDescriptorSet from bytes
    ///
    /// # Example
    /// ```rust,ignore
    /// let bytes = include_bytes!("service.pb");
    /// let parser = ProtobufParser::from_file_descriptor_set(
    ///     bytes,
    ///     "storage",
    ///     "v1"
    /// )?;
    /// ```
    pub fn from_file_descriptor_set(
        bytes: &[u8],
        service_name: &str,
        api_version: &str,
    ) -> Result<Self> {
        // Decode FileDescriptorSet
        let file_descriptor_set = FileDescriptorSet::decode(bytes).map_err(|e| {
            GeneratorError::Parse(format!("Failed to decode FileDescriptorSet: {}", e))
        })?;

        // Create descriptor pool
        let pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set).map_err(|e| {
            GeneratorError::Parse(format!("Failed to create DescriptorPool: {}", e))
        })?;

        Ok(Self {
            pool,
            service_name: service_name.to_string(),
            api_version: api_version.to_string(),
        })
    }

    /// Parse FileDescriptorSet into ServiceDefinition IR
    pub fn parse(&self) -> Result<ServiceDefinition> {
        // Use the converter module to transform protobuf -> ServiceDefinition
        super::converter::convert_protobuf_to_service_definition(
            &self.pool,
            &self.service_name,
            &self.api_version,
        )
    }

    /// Get reference to the underlying descriptor pool
    pub fn pool(&self) -> &DescriptorPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_descriptor_set() {
        // Create minimal FileDescriptorSet
        let file_descriptor_set = FileDescriptorSet { file: vec![] };
        let bytes = file_descriptor_set.encode_to_vec();

        let parser = ProtobufParser::from_file_descriptor_set(&bytes, "test", "v1");
        assert!(parser.is_ok());
    }
}
