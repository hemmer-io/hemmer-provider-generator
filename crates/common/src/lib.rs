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
    Kubernetes,
}

/// Intermediate representation of a unified cloud provider with multiple services
///
/// This represents a complete provider (e.g., AWS) with all its services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDefinition {
    /// Cloud provider type
    pub provider: Provider,
    /// Provider name for code generation (e.g., "aws", "gcp")
    pub provider_name: String,
    /// SDK version
    pub sdk_version: String,
    /// All services in this provider
    pub services: Vec<ServiceDefinition>,
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

/// Metadata about an SDK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkMetadata {
    /// Cloud provider
    pub provider: Provider,
    /// SDK version
    pub sdk_version: String,
    /// SDK name (e.g., "aws-sdk-rust", "google-cloud-rust")
    pub sdk_name: String,
}

/// Trait for parsing SDK crates into ServiceDefinition IR
///
/// This trait enables a plugin-like architecture where:
/// - Built-in parsers are provided for AWS, GCP, Azure
/// - Custom parsers can be implemented for any SDK
///
/// # Example
///
/// ```rust
/// use hemmer_provider_generator_common::{SdkParser, ServiceDefinition, SdkMetadata, Provider, Result};
///
/// struct MyCustomParser {
///     service_name: String,
///     sdk_version: String,
/// }
///
/// impl SdkParser for MyCustomParser {
///     fn parse(&self) -> Result<ServiceDefinition> {
///         // Parse your SDK and return ServiceDefinition
///         todo!("Implement custom parsing logic")
///     }
///
///     fn supported_services(&self) -> Vec<String> {
///         vec!["my-service".to_string()]
///     }
///
///     fn metadata(&self) -> SdkMetadata {
///         SdkMetadata {
///             provider: Provider::Aws, // or your custom provider
///             sdk_version: self.sdk_version.clone(),
///             sdk_name: "my-custom-sdk".to_string(),
///         }
///     }
/// }
/// ```
pub trait SdkParser: Send + Sync {
    /// Parse the SDK and return service definition
    ///
    /// This method should:
    /// 1. Load SDK metadata (rustdoc JSON, OpenAPI spec, etc.)
    /// 2. Extract operations and types
    /// 3. Build ResourceDefinition instances
    /// 4. Return complete ServiceDefinition
    fn parse(&self) -> Result<ServiceDefinition>;

    /// List all services exposed by this SDK
    ///
    /// For AWS: ["s3", "ec2", "dynamodb", ...]
    /// For GCP: ["storage", "compute", ...]
    fn supported_services(&self) -> Vec<String>;

    /// Get metadata about the SDK
    ///
    /// Returns information about the SDK provider, version, and name
    fn metadata(&self) -> SdkMetadata;
}

/// Registry for managing SDK parsers
///
/// This registry allows:
/// - Registering built-in parsers (AWS, GCP, Azure)
/// - Registering custom user-provided parsers
/// - Retrieving parsers by provider name
///
/// # Example
///
/// ```rust
/// use hemmer_provider_generator_common::{ParserRegistry, Provider};
/// # use hemmer_provider_generator_common::{SdkParser, ServiceDefinition, SdkMetadata, Result};
/// #
/// # struct MyParser;
/// # impl SdkParser for MyParser {
/// #     fn parse(&self) -> Result<ServiceDefinition> { todo!() }
/// #     fn supported_services(&self) -> Vec<String> { vec![] }
/// #     fn metadata(&self) -> SdkMetadata {
/// #         SdkMetadata {
/// #             provider: Provider::Aws,
/// #             sdk_version: "1.0.0".to_string(),
/// #             sdk_name: "test".to_string(),
/// #         }
/// #     }
/// # }
///
/// let mut registry = ParserRegistry::new();
/// registry.register("aws", Box::new(MyParser));
///
/// let parser = registry.get("aws");
/// assert!(parser.is_some());
/// ```
pub struct ParserRegistry {
    parsers: HashMap<String, Box<dyn SdkParser>>,
}

impl ParserRegistry {
    /// Create a new empty parser registry
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    /// Register a parser with a given name
    ///
    /// # Arguments
    /// * `name` - Provider name (e.g., "aws", "gcp", "azure")
    /// * `parser` - Boxed parser implementing SdkParser trait
    ///
    /// # Example
    /// ```rust
    /// # use hemmer_provider_generator_common::{ParserRegistry, SdkParser, ServiceDefinition, SdkMetadata, Provider, Result};
    /// # struct MyParser;
    /// # impl SdkParser for MyParser {
    /// #     fn parse(&self) -> Result<ServiceDefinition> { todo!() }
    /// #     fn supported_services(&self) -> Vec<String> { vec![] }
    /// #     fn metadata(&self) -> SdkMetadata {
    /// #         SdkMetadata {
    /// #             provider: Provider::Aws,
    /// #             sdk_version: "1.0.0".to_string(),
    /// #             sdk_name: "test".to_string(),
    /// #         }
    /// #     }
    /// # }
    /// let mut registry = ParserRegistry::new();
    /// registry.register("my-provider", Box::new(MyParser));
    /// ```
    pub fn register(&mut self, name: &str, parser: Box<dyn SdkParser>) {
        self.parsers.insert(name.to_string(), parser);
    }

    /// Get a parser by provider name
    ///
    /// Returns `None` if no parser is registered with the given name.
    pub fn get(&self, name: &str) -> Option<&dyn SdkParser> {
        self.parsers.get(name).map(|p| p.as_ref())
    }

    /// List all registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.parsers.keys().cloned().collect()
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, name: &str) -> bool {
        self.parsers.contains_key(name)
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize a string to be a valid Rust identifier
///
/// This function ensures the result can be safely used as:
/// - Function names
/// - Variable names
/// - Module names
/// - Struct/enum names
///
/// ## Transformations
///
/// 1. Replaces special characters (`.`, `-`, `/`, etc.) with underscores
/// 2. Prefixes with `_` if starts with a digit
/// 3. Escapes Rust keywords with `r#` prefix
///
/// ## Examples
///
/// ```
/// use hemmer_provider_generator_common::sanitize_rust_identifier;
///
/// assert_eq!(sanitize_rust_identifier("rbac.authorization"), "rbac_authorization");
/// assert_eq!(sanitize_rust_identifier("type"), "r#type");
/// assert_eq!(sanitize_rust_identifier("acm-pca"), "acm_pca");
/// assert_eq!(sanitize_rust_identifier("123invalid"), "_123invalid");
/// assert_eq!(sanitize_rust_identifier("normal_name"), "normal_name");
/// ```
pub fn sanitize_rust_identifier(name: &str) -> String {
    // Replace special characters with underscores
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Clean up consecutive underscores
    let mut sanitized = sanitized;
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }

    // Remove leading/trailing underscores
    let sanitized = sanitized.trim_matches('_');

    // Ensure doesn't start with digit
    let sanitized = if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{}", sanitized)
    } else {
        sanitized.to_string()
    };

    // Escape Rust keywords with r# prefix
    const RUST_KEYWORDS: &[&str] = &[
        // Strict keywords (always reserved)
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", // Reserved keywords (reserved for future use)
        "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
        "unsized", "virtual", "yield",
        // Weak keywords (context-dependent, but safer to escape)
        "async", "await", "dyn", "try", "union",
    ];

    if RUST_KEYWORDS.contains(&sanitized.as_str()) {
        format!("r#{}", sanitized)
    } else {
        sanitized
    }
}

/// Sanitize a string to be part of a composite Rust identifier
///
/// Unlike `sanitize_rust_identifier`, this function appends `_` to keywords
/// instead of using the `r#` prefix, making it suitable for use in composite
/// names like function names (`plan_type_` instead of invalid `plan_r#type`).
///
/// This function ensures the result can be safely used in:
/// - Composite function names (e.g., `create_`, `plan_`, `read_`)
/// - Parts of larger identifiers
///
/// ## Transformations
///
/// 1. Replaces special characters (`.`, `-`, `/`, etc.) with underscores
/// 2. Prefixes with `_` if starts with a digit
/// 3. Appends `_` suffix to Rust keywords (instead of `r#` prefix)
///
/// ## Examples
///
/// ```
/// use hemmer_provider_generator_common::sanitize_identifier_part;
///
/// assert_eq!(sanitize_identifier_part("rbac.authorization"), "rbac_authorization");
/// assert_eq!(sanitize_identifier_part("type"), "type_");  // Suffix instead of r#
/// assert_eq!(sanitize_identifier_part("acm-pca"), "acm_pca");
/// assert_eq!(sanitize_identifier_part("123invalid"), "_123invalid");
/// assert_eq!(sanitize_identifier_part("normal_name"), "normal_name");
/// ```
pub fn sanitize_identifier_part(name: &str) -> String {
    // Replace special characters with underscores
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Clean up consecutive underscores
    let mut sanitized = sanitized;
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }

    // Remove leading/trailing underscores
    let sanitized = sanitized.trim_matches('_');

    // Ensure doesn't start with digit
    let sanitized = if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{}", sanitized)
    } else {
        sanitized.to_string()
    };

    // Append _ to Rust keywords (instead of r# prefix for composite names)
    const RUST_KEYWORDS: &[&str] = &[
        // Strict keywords (always reserved)
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", // Reserved keywords (reserved for future use)
        "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
        "unsized", "virtual", "yield",
        // Weak keywords (context-dependent, but safer to escape)
        "async", "await", "dyn", "try", "union",
    ];

    if RUST_KEYWORDS.contains(&sanitized.as_str()) {
        format!("{}_", sanitized)
    } else {
        sanitized
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

    // Mock parser for testing
    struct MockParser {
        service_name: String,
        sdk_version: String,
    }

    impl SdkParser for MockParser {
        fn parse(&self) -> Result<ServiceDefinition> {
            Ok(ServiceDefinition {
                provider: Provider::Aws,
                name: self.service_name.clone(),
                sdk_version: self.sdk_version.clone(),
                resources: vec![],
            })
        }

        fn supported_services(&self) -> Vec<String> {
            vec![self.service_name.clone()]
        }

        fn metadata(&self) -> SdkMetadata {
            SdkMetadata {
                provider: Provider::Aws,
                sdk_version: self.sdk_version.clone(),
                sdk_name: "mock-sdk".to_string(),
            }
        }
    }

    #[test]
    fn test_parser_registry_new() {
        let registry = ParserRegistry::new();
        assert_eq!(registry.list_providers().len(), 0);
    }

    #[test]
    fn test_parser_registry_register() {
        let mut registry = ParserRegistry::new();
        let parser = MockParser {
            service_name: "s3".to_string(),
            sdk_version: "1.0.0".to_string(),
        };

        registry.register("aws", Box::new(parser));
        assert!(registry.has_provider("aws"));
        assert!(!registry.has_provider("gcp"));
    }

    #[test]
    fn test_parser_registry_get() {
        let mut registry = ParserRegistry::new();
        let parser = MockParser {
            service_name: "s3".to_string(),
            sdk_version: "1.0.0".to_string(),
        };

        registry.register("aws", Box::new(parser));

        let retrieved = registry.get("aws");
        assert!(retrieved.is_some());

        let metadata = retrieved.unwrap().metadata();
        assert_eq!(metadata.provider, Provider::Aws);
        assert_eq!(metadata.sdk_name, "mock-sdk");
    }

    #[test]
    fn test_parser_registry_list_providers() {
        let mut registry = ParserRegistry::new();

        registry.register(
            "aws",
            Box::new(MockParser {
                service_name: "s3".to_string(),
                sdk_version: "1.0.0".to_string(),
            }),
        );

        registry.register(
            "gcp",
            Box::new(MockParser {
                service_name: "storage".to_string(),
                sdk_version: "2.0.0".to_string(),
            }),
        );

        let providers = registry.list_providers();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&"aws".to_string()));
        assert!(providers.contains(&"gcp".to_string()));
    }

    #[test]
    fn test_parser_registry_default() {
        let registry = ParserRegistry::default();
        assert_eq!(registry.list_providers().len(), 0);
    }

    #[test]
    fn test_sanitize_rust_identifier_dots() {
        assert_eq!(
            sanitize_rust_identifier("rbac.authorization"),
            "rbac_authorization"
        );
        assert_eq!(
            sanitize_rust_identifier("apis.internal.k8s.io"),
            "apis_internal_k8s_io"
        );
    }

    #[test]
    fn test_sanitize_rust_identifier_hyphens() {
        assert_eq!(sanitize_rust_identifier("acm-pca"), "acm_pca");
        assert_eq!(sanitize_rust_identifier("eks-fargate"), "eks_fargate");
    }

    #[test]
    fn test_sanitize_rust_identifier_keywords() {
        assert_eq!(sanitize_rust_identifier("type"), "r#type");
        assert_eq!(sanitize_rust_identifier("async"), "r#async");
        assert_eq!(sanitize_rust_identifier("await"), "r#await");
        assert_eq!(sanitize_rust_identifier("match"), "r#match");
        assert_eq!(sanitize_rust_identifier("self"), "r#self");
        assert_eq!(sanitize_rust_identifier("Self"), "r#Self");
    }

    #[test]
    fn test_sanitize_rust_identifier_starts_with_digit() {
        assert_eq!(sanitize_rust_identifier("123invalid"), "_123invalid");
        assert_eq!(sanitize_rust_identifier("2fa"), "_2fa");
    }

    #[test]
    fn test_sanitize_rust_identifier_special_characters() {
        assert_eq!(sanitize_rust_identifier("foo/bar"), "foo_bar");
        assert_eq!(sanitize_rust_identifier("foo@bar"), "foo_bar");
        assert_eq!(sanitize_rust_identifier("foo bar"), "foo_bar");
    }

    #[test]
    fn test_sanitize_rust_identifier_consecutive_underscores() {
        assert_eq!(sanitize_rust_identifier("foo__bar"), "foo_bar");
        assert_eq!(sanitize_rust_identifier("a...b"), "a_b");
    }

    #[test]
    fn test_sanitize_rust_identifier_unchanged() {
        assert_eq!(sanitize_rust_identifier("normal_name"), "normal_name");
        assert_eq!(sanitize_rust_identifier("ValidRustName"), "ValidRustName");
        assert_eq!(sanitize_rust_identifier("name123"), "name123");
    }

    #[test]
    fn test_sanitize_rust_identifier_edge_cases() {
        // Empty string behavior
        assert_eq!(sanitize_rust_identifier(""), "");
        // Only special characters
        assert_eq!(sanitize_rust_identifier("..."), "");
        // Leading/trailing underscores removed
        assert_eq!(sanitize_rust_identifier("_test_"), "test");
    }

    #[test]
    fn test_sanitize_identifier_part_dots() {
        assert_eq!(
            sanitize_identifier_part("rbac.authorization"),
            "rbac_authorization"
        );
    }

    #[test]
    fn test_sanitize_identifier_part_keywords() {
        // Keywords get _ suffix instead of r# prefix
        assert_eq!(sanitize_identifier_part("type"), "type_");
        assert_eq!(sanitize_identifier_part("async"), "async_");
        assert_eq!(sanitize_identifier_part("await"), "await_");
        assert_eq!(sanitize_identifier_part("match"), "match_");
    }

    #[test]
    fn test_sanitize_identifier_part_unchanged() {
        assert_eq!(sanitize_identifier_part("normal_name"), "normal_name");
        assert_eq!(sanitize_identifier_part("ValidRustName"), "ValidRustName");
    }
}
