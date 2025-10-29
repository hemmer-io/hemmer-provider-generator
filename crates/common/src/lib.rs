//! Common types and utilities for the Hemmer Provider Generator
//!
//! This crate contains shared data structures, error types, and utilities
//! used across the parser, generator, and CLI components.
//!
//! ## Architecture
//!
//! The generator follows this data flow:
//! 1. **Parser**: SDK crates → ServiceDefinition (intermediate representation)
//! 2. **Generator**: ServiceDefinition → Generated code (provider.k + Rust)
//! 3. **Output**: Generated provider implementing ProviderExecutor trait

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during provider generation
#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for generator operations
pub type Result<T> = std::result::Result<T, GeneratorError>;

/// Represents a cloud provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Aws,
    Gcp,
    Azure,
}

/// Intermediate representation of a cloud service (e.g., aws-sdk-s3)
///
/// This is the output of the parser phase and input to the generator phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Provider this service belongs to
    pub provider: Provider,
    /// Service name (e.g., "s3", "ec2")
    pub name: String,
    /// SDK version this was parsed from
    pub sdk_version: String,
    /// Resources discovered in this service
    pub resources: Vec<ResourceDefinition>,
}

/// Definition of a single resource type (e.g., S3 Bucket, EC2 Instance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    /// Resource type name (e.g., "bucket", "instance")
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Input fields for creating/updating the resource
    pub fields: Vec<FieldDefinition>,
    /// Output fields returned after operations
    pub outputs: Vec<FieldDefinition>,
    /// CRUD operations available for this resource
    pub operations: Operations,
}

/// CRUD operations mapped from SDK operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operations {
    /// Create operation (e.g., CreateBucket)
    pub create: Option<OperationMapping>,
    /// Read operation (e.g., HeadBucket, GetBucket)
    pub read: Option<OperationMapping>,
    /// Update operation (e.g., PutBucketAcl)
    pub update: Option<OperationMapping>,
    /// Delete operation (e.g., DeleteBucket)
    pub delete: Option<OperationMapping>,
}

/// Mapping of a CRUD operation to SDK operation(s)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMapping {
    /// SDK operation name (e.g., "create_bucket")
    pub sdk_operation: String,
    /// Additional operations that might be needed (e.g., for composite updates)
    pub additional_operations: Vec<String>,
}

/// Definition of a field in a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Field name (snake_case)
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Whether this field is required
    pub required: bool,
    /// Whether this field is sensitive (passwords, keys)
    pub sensitive: bool,
    /// Whether this field is immutable (requires replacement if changed)
    pub immutable: bool,
    /// Human-readable description
    pub description: Option<String>,
}

/// Represents a field type in the intermediate representation
///
/// Maps SDK types → IR types → KCL types → Generated Rust types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    /// String type
    String,
    /// Integer type (i64)
    Integer,
    /// Boolean type
    Boolean,
    /// Float type (f64)
    Float,
    /// List/Array of items
    List(Box<FieldType>),
    /// Map/Dictionary
    Map(Box<FieldType>, Box<FieldType>),
    /// Custom enum (represented as string variants)
    Enum(Vec<String>),
    /// DateTime (ISO 8601 string)
    DateTime,
    /// Nested object (represented as map)
    Object(HashMap<String, Box<FieldType>>),
}

impl FieldType {
    /// Convert to KCL type string for manifest generation
    pub fn to_kcl_type(&self) -> String {
        match self {
            FieldType::String => "String".to_string(),
            FieldType::Integer => "Integer".to_string(),
            FieldType::Boolean => "Boolean".to_string(),
            FieldType::Float => "Float".to_string(),
            FieldType::List(inner) => format!("List<{}>", inner.to_kcl_type()),
            FieldType::Map(k, v) => format!("Map<{},{}>", k.to_kcl_type(), v.to_kcl_type()),
            FieldType::Enum(_) => "String".to_string(), // Enums become strings
            FieldType::DateTime => "String".to_string(), // ISO 8601
            FieldType::Object(_) => "Map<String,Any>".to_string(),
        }
    }

    /// Convert to Rust type string for code generation
    pub fn to_rust_type(&self) -> String {
        match self {
            FieldType::String => "String".to_string(),
            FieldType::Integer => "i64".to_string(),
            FieldType::Boolean => "bool".to_string(),
            FieldType::Float => "f64".to_string(),
            FieldType::List(inner) => format!("Vec<{}>", inner.to_rust_type()),
            FieldType::Map(k, v) => {
                format!("HashMap<{}, {}>", k.to_rust_type(), v.to_rust_type())
            }
            FieldType::Enum(_) => "String".to_string(),
            FieldType::DateTime => "String".to_string(),
            FieldType::Object(_) => "HashMap<String, serde_json::Value>".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_creation() {
        let ft = FieldType::String;
        assert_eq!(ft, FieldType::String);
    }

    #[test]
    fn test_field_type_to_kcl() {
        assert_eq!(FieldType::String.to_kcl_type(), "String");
        assert_eq!(FieldType::Integer.to_kcl_type(), "Integer");
        assert_eq!(
            FieldType::List(Box::new(FieldType::String)).to_kcl_type(),
            "List<String>"
        );
        assert_eq!(
            FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::Integer)).to_kcl_type(),
            "Map<String,Integer>"
        );
    }

    #[test]
    fn test_field_type_to_rust() {
        assert_eq!(FieldType::String.to_rust_type(), "String");
        assert_eq!(FieldType::Integer.to_rust_type(), "i64");
        assert_eq!(
            FieldType::List(Box::new(FieldType::String)).to_rust_type(),
            "Vec<String>"
        );
        assert_eq!(
            FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::Integer))
                .to_rust_type(),
            "HashMap<String, i64>"
        );
    }

    #[test]
    fn test_service_definition() {
        let service = ServiceDefinition {
            provider: Provider::Aws,
            name: "s3".to_string(),
            sdk_version: "1.0.0".to_string(),
            resources: vec![],
        };

        assert_eq!(service.provider, Provider::Aws);
        assert_eq!(service.name, "s3");
    }
}
