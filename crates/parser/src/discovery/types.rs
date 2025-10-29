//! Google Discovery Document type definitions
//!
//! Based on JSON Schema Draft 3 with Google-specific extensions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Discovery Document root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryDoc {
    /// Discovery version (e.g., "v1")
    #[serde(rename = "discoveryVersion")]
    pub discovery_version: String,

    /// API name (e.g., "storage", "compute")
    pub name: String,

    /// API version (e.g., "v1")
    pub version: String,

    /// API title
    pub title: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Root URL (e.g., "<https://storage.googleapis.com/>")
    #[serde(rename = "rootUrl")]
    pub root_url: String,

    /// Service path (e.g., "storage/v1/")
    #[serde(rename = "servicePath")]
    pub service_path: String,

    /// Base path
    #[serde(rename = "basePath")]
    #[serde(default)]
    pub base_path: Option<String>,

    /// Parameters
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,

    /// Authentication scopes
    #[serde(default)]
    pub auth: Option<Auth>,

    /// Schemas (data types)
    #[serde(default)]
    pub schemas: HashMap<String, Schema>,

    /// Resources (collections of methods)
    #[serde(default)]
    pub resources: HashMap<String, Resource>,

    /// Methods (at root level, rare)
    #[serde(default)]
    pub methods: HashMap<String, Method>,
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    /// OAuth 2.0 scopes
    #[serde(default)]
    pub oauth2: Option<OAuth2>,
}

/// OAuth 2.0 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2 {
    /// Scopes
    #[serde(default)]
    pub scopes: HashMap<String, Scope>,
}

/// OAuth scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Scope description
    pub description: String,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter type (string, integer, boolean, etc.)
    #[serde(rename = "type")]
    #[serde(default)]
    pub param_type: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Default value
    #[serde(default)]
    pub default: Option<String>,

    /// Required flag
    #[serde(default)]
    pub required: bool,

    /// Location (query, path)
    #[serde(default)]
    pub location: Option<String>,

    /// Enum values
    #[serde(rename = "enum")]
    #[serde(default)]
    pub enum_values: Vec<String>,
}

/// Schema (data type) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema ID
    #[serde(default)]
    pub id: Option<String>,

    /// Type (string, object, array, etc.)
    #[serde(rename = "type")]
    #[serde(default)]
    pub schema_type: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Properties (for object type)
    #[serde(default)]
    pub properties: HashMap<String, Schema>,

    /// Additional properties
    #[serde(rename = "additionalProperties")]
    #[serde(default)]
    pub additional_properties: Option<Box<Schema>>,

    /// Items (for array type)
    #[serde(default)]
    pub items: Option<Box<Schema>>,

    /// Reference to another schema
    #[serde(rename = "$ref")]
    #[serde(default)]
    pub ref_schema: Option<String>,

    /// Format (e.g., "int32", "date-time")
    #[serde(default)]
    pub format: Option<String>,

    /// Enum values
    #[serde(rename = "enum")]
    #[serde(default)]
    pub enum_values: Vec<String>,

    /// Required properties
    #[serde(default)]
    pub required: Vec<String>,
}

/// Resource (collection of methods)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Methods for this resource
    #[serde(default)]
    pub methods: HashMap<String, Method>,

    /// Nested resources
    #[serde(default)]
    pub resources: HashMap<String, Resource>,
}

/// Method (API operation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    /// Method ID (e.g., "storage.buckets.insert")
    pub id: String,

    /// HTTP path
    pub path: String,

    /// HTTP method (GET, POST, PUT, DELETE, PATCH)
    #[serde(rename = "httpMethod")]
    pub http_method: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Parameters
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,

    /// Request body schema
    #[serde(default)]
    pub request: Option<MethodRequest>,

    /// Response schema
    #[serde(default)]
    pub response: Option<MethodResponse>,

    /// Scopes required
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Method request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodRequest {
    /// Reference to request schema
    #[serde(rename = "$ref")]
    pub ref_schema: String,
}

/// Method response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodResponse {
    /// Reference to response schema
    #[serde(rename = "$ref")]
    pub ref_schema: String,
}

impl DiscoveryDoc {
    /// Get a schema by reference
    /// e.g., "Bucket" -> returns Bucket schema
    pub fn resolve_schema_ref(&self, ref_name: &str) -> Option<&Schema> {
        self.schemas.get(ref_name)
    }

    /// Extract resource name from method ID
    /// e.g., "storage.buckets.insert" -> "Bucket"
    pub fn extract_resource_from_method_id(method_id: &str) -> Option<String> {
        let parts: Vec<&str> = method_id.split('.').collect();
        if parts.len() >= 2 {
            let resource = parts[parts.len() - 2];
            // Singularize (remove trailing 's')
            let mut singular = resource.to_string();
            if singular.ends_with('s') && singular.len() > 1 {
                singular.pop();
            }
            // Capitalize
            if let Some(first) = singular.chars().next() {
                singular = first.to_uppercase().to_string() + &singular[1..];
            }
            Some(singular)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resource_from_method_id() {
        assert_eq!(
            DiscoveryDoc::extract_resource_from_method_id("storage.buckets.insert"),
            Some("Bucket".to_string())
        );
        assert_eq!(
            DiscoveryDoc::extract_resource_from_method_id("compute.instances.insert"),
            Some("Instance".to_string())
        );
        assert_eq!(
            DiscoveryDoc::extract_resource_from_method_id("bigquery.datasets.get"),
            Some("Dataset".to_string())
        );
    }
}
