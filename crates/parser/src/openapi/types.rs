//! OpenAPI 3.0 type definitions
//!
//! Simplified representation focusing on resource extraction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI document root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// OpenAPI version (e.g., "3.0.0")
    pub openapi: String,

    /// API metadata
    pub info: Info,

    /// API paths (endpoints)
    #[serde(default)]
    pub paths: HashMap<String, PathItem>,

    /// Reusable components
    #[serde(default)]
    pub components: Option<Components>,

    /// Servers
    #[serde(default)]
    pub servers: Vec<Server>,
}

/// API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    /// API title
    pub title: String,

    /// API version
    pub version: String,

    /// API description
    #[serde(default)]
    pub description: Option<String>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server URL
    pub url: String,

    /// Server description
    #[serde(default)]
    pub description: Option<String>,
}

/// Path item (operations for a path)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    /// GET operation
    #[serde(default)]
    pub get: Option<Operation>,

    /// POST operation
    #[serde(default)]
    pub post: Option<Operation>,

    /// PUT operation
    #[serde(default)]
    pub put: Option<Operation>,

    /// PATCH operation
    #[serde(default)]
    pub patch: Option<Operation>,

    /// DELETE operation
    #[serde(default)]
    pub delete: Option<Operation>,

    /// Path parameters
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

/// HTTP operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation ID (unique identifier)
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,

    /// Summary
    #[serde(default)]
    pub summary: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Parameters
    #[serde(default)]
    pub parameters: Vec<Parameter>,

    /// Request body
    #[serde(rename = "requestBody")]
    #[serde(default)]
    pub request_body: Option<RequestBody>,

    /// Responses
    #[serde(default)]
    pub responses: HashMap<String, Response>,

    /// Tags (for grouping)
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Location: query, header, path, cookie
    #[serde(rename = "in")]
    pub location: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Required flag
    #[serde(default)]
    pub required: bool,

    /// Schema
    #[serde(default)]
    pub schema: Option<Schema>,
}

/// Request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Content types
    pub content: HashMap<String, MediaType>,

    /// Required flag
    #[serde(default)]
    pub required: bool,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Description
    pub description: String,

    /// Content types
    #[serde(default)]
    pub content: HashMap<String, MediaType>,
}

/// Media type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    /// Schema
    #[serde(default)]
    pub schema: Option<SchemaOrRef>,
}

/// Schema or reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SchemaOrRef {
    /// Direct schema
    Schema(Box<Schema>),

    /// Reference to schema
    Reference {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
}

/// Schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Type: string, number, integer, boolean, array, object
    #[serde(rename = "type")]
    #[serde(default)]
    pub schema_type: Option<String>,

    /// Format (e.g., int32, int64, date-time)
    #[serde(default)]
    pub format: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Properties (for object type)
    #[serde(default)]
    pub properties: HashMap<String, SchemaOrRef>,

    /// Required properties
    #[serde(default)]
    pub required: Vec<String>,

    /// Items schema (for array type)
    #[serde(default)]
    pub items: Option<Box<SchemaOrRef>>,

    /// Additional properties
    #[serde(rename = "additionalProperties")]
    #[serde(default)]
    pub additional_properties: Option<Box<SchemaOrRef>>,

    /// Enum values
    #[serde(rename = "enum")]
    #[serde(default)]
    pub enum_values: Vec<serde_json::Value>,

    /// Reference
    #[serde(rename = "$ref")]
    #[serde(default)]
    pub ref_path: Option<String>,

    /// Extensions (x-kubernetes-*, etc.)
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// Reusable components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Schemas
    #[serde(default)]
    pub schemas: HashMap<String, Schema>,

    /// Parameters
    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,

    /// Request bodies
    #[serde(rename = "requestBodies")]
    #[serde(default)]
    pub request_bodies: HashMap<String, RequestBody>,

    /// Responses
    #[serde(default)]
    pub responses: HashMap<String, Response>,
}

impl OpenApiSpec {
    /// Get a schema by reference path
    /// e.g., "#/components/schemas/Pod" -> returns Pod schema
    pub fn resolve_schema_ref(&self, ref_path: &str) -> Option<&Schema> {
        if !ref_path.starts_with("#/components/schemas/") {
            return None;
        }

        let schema_name = ref_path.strip_prefix("#/components/schemas/")?;
        self.components
            .as_ref()
            .and_then(|c| c.schemas.get(schema_name))
    }

    /// Extract resource name from path
    /// e.g., "/api/v1/namespaces/{namespace}/pods/{name}" -> "Pod"
    pub fn extract_resource_from_path(path: &str) -> Option<String> {
        // Split path and find the resource segment
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Look for last non-parameter segment
        for segment in segments.iter().rev() {
            if !segment.starts_with('{') {
                // Capitalize and singularize
                let mut resource = segment.to_string();

                // Remove trailing 's' for plurals
                if resource.ends_with('s') && resource.len() > 1 {
                    resource.pop();
                }

                // Capitalize first letter
                if let Some(first) = resource.chars().next() {
                    resource = first.to_uppercase().to_string() + &resource[1..];
                }

                return Some(resource);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resource_from_path() {
        assert_eq!(
            OpenApiSpec::extract_resource_from_path("/api/v1/namespaces/{namespace}/pods"),
            Some("Pod".to_string())
        );
        assert_eq!(
            OpenApiSpec::extract_resource_from_path("/api/v1/namespaces/{namespace}/pods/{name}"),
            Some("Pod".to_string())
        );
        assert_eq!(
            OpenApiSpec::extract_resource_from_path("/apis/apps/v1/deployments"),
            Some("Deployment".to_string())
        );
    }
}
