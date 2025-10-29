//! Common types and utilities for the Hemmer Provider Generator
//!
//! This crate contains shared data structures, error types, and utilities
//! used across the parser, generator, and CLI components.

use serde::{Deserialize, Serialize};
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

/// Represents a field type in the intermediate representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Integer,
    Boolean,
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_creation() {
        let ft = FieldType::String;
        assert_eq!(ft, FieldType::String);
    }
}
