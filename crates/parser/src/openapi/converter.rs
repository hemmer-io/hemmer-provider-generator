//! Converts OpenAPI spec to ServiceDefinition IR

use super::parser::ProviderHint;
use super::types::{OpenApiSpec, Operation, Schema, SchemaOrRef};
use hemmer_provider_generator_common::{
    FieldDefinition, FieldType, OperationMapping, Operations, Provider, ResourceDefinition, Result,
    ServiceDefinition,
};
use std::collections::HashMap;

/// Convert OpenAPI spec to ServiceDefinition
pub fn convert_openapi_to_service_definition(
    spec: &OpenApiSpec,
    service_name: &str,
    api_version: &str,
    provider_hint: Option<ProviderHint>,
) -> Result<ServiceDefinition> {
    // Determine provider from hint or default
    let provider = match provider_hint {
        Some(ProviderHint::Kubernetes) => Provider::Kubernetes,
        Some(ProviderHint::Azure) => Provider::Azure,
        _ => Provider::Kubernetes, // Default to Kubernetes for generic OpenAPI
    };

    // Extract resources from paths
    let resources = extract_resources_from_paths(spec)?;

    Ok(ServiceDefinition {
        provider,
        name: service_name.to_string(),
        sdk_version: api_version.to_string(),
        resources,
        data_sources: vec![], // Will implement data source detection later
    })
}

/// Extract resources from OpenAPI paths
fn extract_resources_from_paths(spec: &OpenApiSpec) -> Result<Vec<ResourceDefinition>> {
    let mut resource_map: HashMap<String, ResourceOperations> = HashMap::new();

    // Group operations by resource
    for (path, path_item) in &spec.paths {
        if let Some(resource_name) = OpenApiSpec::extract_resource_from_path(path) {
            let entry = resource_map
                .entry(resource_name.clone())
                .or_insert_with(|| ResourceOperations::new(resource_name));

            // Map HTTP methods to CRUD operations
            if let Some(ref op) = path_item.post {
                entry.create = Some(op.clone());
            }
            if let Some(ref op) = path_item.get {
                // GET can be read (single resource) or list (collection)
                if path.contains('{') && path.matches('{').count() >= 2 {
                    // Path like /pods/{name} - this is a read
                    entry.read = Some(op.clone());
                } else {
                    // Path like /pods - this might be list, but we'll use it as read
                    entry.read = entry.read.clone().or_else(|| Some(op.clone()));
                }
            }
            if let Some(ref op) = path_item.put {
                entry.update = Some(op.clone());
            }
            if let Some(ref op) = path_item.patch {
                entry.update = entry.update.clone().or_else(|| Some(op.clone()));
            }
            if let Some(ref op) = path_item.delete {
                entry.delete = Some(op.clone());
            }
        }
    }

    // Convert to ResourceDefinitions
    let mut resources = Vec::new();
    for (_name, ops) in resource_map {
        if let Some(resource_def) = build_resource_from_operations(spec, ops)? {
            resources.push(resource_def);
        }
    }

    Ok(resources)
}

/// Temporary structure to collect operations for a resource
#[derive(Debug, Clone)]
struct ResourceOperations {
    name: String,
    create: Option<Operation>,
    read: Option<Operation>,
    update: Option<Operation>,
    delete: Option<Operation>,
}

impl ResourceOperations {
    fn new(name: String) -> Self {
        Self {
            name,
            create: None,
            read: None,
            update: None,
            delete: None,
        }
    }
}

/// Build ResourceDefinition from operations
fn build_resource_from_operations(
    spec: &OpenApiSpec,
    ops: ResourceOperations,
) -> Result<Option<ResourceDefinition>> {
    // Need at least one operation
    if ops.create.is_none() && ops.read.is_none() && ops.update.is_none() && ops.delete.is_none() {
        return Ok(None);
    }

    // Extract fields from create/update operation
    let fields = if let Some(ref create_op) = ops.create {
        extract_fields_from_operation(spec, create_op)?
    } else if let Some(ref update_op) = ops.update {
        extract_fields_from_operation(spec, update_op)?
    } else {
        Vec::new()
    };

    // Extract outputs from read operation
    let outputs = if let Some(ref read_op) = ops.read {
        extract_outputs_from_operation(spec, read_op)?
    } else {
        Vec::new()
    };

    // Get description
    let description = ops
        .create
        .as_ref()
        .and_then(|op| op.description.clone())
        .or_else(|| ops.read.as_ref().and_then(|op| op.description.clone()));

    Ok(Some(ResourceDefinition {
        name: to_snake_case(&ops.name),
        description,
        fields,
        outputs,
        id_field: None, // Will implement ID detection later
        operations: Operations {
            create: ops.create.and_then(|op| {
                op.operation_id.map(|id| OperationMapping {
                    sdk_operation: to_snake_case(&id),
                    additional_operations: vec![],
                })
            }),
            read: ops.read.and_then(|op| {
                op.operation_id.map(|id| OperationMapping {
                    sdk_operation: to_snake_case(&id),
                    additional_operations: vec![],
                })
            }),
            update: ops.update.and_then(|op| {
                op.operation_id.map(|id| OperationMapping {
                    sdk_operation: to_snake_case(&id),
                    additional_operations: vec![],
                })
            }),
            delete: ops.delete.and_then(|op| {
                op.operation_id.map(|id| OperationMapping {
                    sdk_operation: to_snake_case(&id),
                    additional_operations: vec![],
                })
            }),
            import: None, // Will implement later
        },
    }))
}

/// Extract fields from operation request body
fn extract_fields_from_operation(
    spec: &OpenApiSpec,
    operation: &Operation,
) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    if let Some(ref request_body) = operation.request_body {
        // Get the schema from the first content type (usually application/json)
        if let Some(media_type) = request_body.content.values().next() {
            if let Some(ref schema_or_ref) = media_type.schema {
                fields = extract_fields_from_schema(spec, schema_or_ref, false)?;
            }
        }
    }

    // Also check parameters
    for param in &operation.parameters {
        if let Some(ref schema) = param.schema {
            let field_type = convert_schema_to_field_type(spec, schema)?;
            fields.push(FieldDefinition {
                name: to_snake_case(&param.name),
                field_type,
                required: param.required,
                sensitive: false,
                immutable: param.location == "path", // Path params are usually immutable identifiers
                description: param.description.clone(),
                response_accessor: None, // Input fields don't have response accessors
            });
        }
    }

    Ok(fields)
}

/// Extract outputs from operation responses
fn extract_outputs_from_operation(
    spec: &OpenApiSpec,
    operation: &Operation,
) -> Result<Vec<FieldDefinition>> {
    let mut outputs = Vec::new();

    // Look for 200/201 responses
    for status in &["200", "201"] {
        if let Some(response) = operation.responses.get(*status) {
            if let Some(media_type) = response.content.values().next() {
                if let Some(ref schema_or_ref) = media_type.schema {
                    outputs = extract_fields_from_schema(spec, schema_or_ref, true)?;
                    break;
                }
            }
        }
    }

    Ok(outputs)
}

/// Extract fields from schema
///
/// `is_response` - if true, fields are from a response and should have response accessors
fn extract_fields_from_schema(
    spec: &OpenApiSpec,
    schema_or_ref: &SchemaOrRef,
    is_response: bool,
) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    let schema = match schema_or_ref {
        SchemaOrRef::Schema(s) => s.as_ref(),
        SchemaOrRef::Reference { ref_path } => {
            if let Some(s) = spec.resolve_schema_ref(ref_path) {
                s
            } else {
                return Ok(fields);
            }
        }
    };

    // Extract properties
    for (field_name, field_schema_or_ref) in &schema.properties {
        let field_schema = match field_schema_or_ref {
            SchemaOrRef::Schema(s) => s.as_ref(),
            SchemaOrRef::Reference { ref_path } => {
                if let Some(s) = spec.resolve_schema_ref(ref_path) {
                    s
                } else {
                    continue;
                }
            }
        };

        let field_type = convert_schema_to_field_type(spec, field_schema)?;
        let required = schema.required.contains(field_name);
        let accessor_name = to_snake_case(field_name);

        fields.push(FieldDefinition {
            name: accessor_name.clone(),
            field_type,
            required,
            sensitive: false,
            immutable: false,
            description: field_schema.description.clone(),
            // Only response fields have accessors
            response_accessor: if is_response {
                Some(accessor_name)
            } else {
                None
            },
        });
    }

    Ok(fields)
}

/// Convert OpenAPI schema to FieldType
fn convert_schema_to_field_type(spec: &OpenApiSpec, schema: &Schema) -> Result<FieldType> {
    // Handle reference
    if let Some(ref ref_path) = schema.ref_path {
        if let Some(resolved) = spec.resolve_schema_ref(ref_path) {
            return convert_schema_to_field_type(spec, resolved);
        }
    }

    // Handle type
    match schema.schema_type.as_deref() {
        Some("string") => match schema.format.as_deref() {
            Some("date-time") => Ok(FieldType::DateTime),
            _ => Ok(FieldType::String),
        },
        Some("integer") => Ok(FieldType::Integer),
        Some("number") => Ok(FieldType::Float),
        Some("boolean") => Ok(FieldType::Boolean),
        Some("array") => {
            if let Some(ref items) = schema.items {
                let item_type = match items.as_ref() {
                    SchemaOrRef::Schema(s) => convert_schema_to_field_type(spec, s.as_ref())?,
                    SchemaOrRef::Reference { ref_path } => {
                        if let Some(s) = spec.resolve_schema_ref(ref_path) {
                            convert_schema_to_field_type(spec, s)?
                        } else {
                            FieldType::String
                        }
                    }
                };
                Ok(FieldType::List(Box::new(item_type)))
            } else {
                Ok(FieldType::List(Box::new(FieldType::String)))
            }
        }
        Some("object") => {
            if let Some(ref additional_props) = schema.additional_properties {
                // This is a map
                let value_type = match additional_props.as_ref() {
                    SchemaOrRef::Schema(s) => convert_schema_to_field_type(spec, s.as_ref())?,
                    SchemaOrRef::Reference { ref_path } => {
                        if let Some(s) = spec.resolve_schema_ref(ref_path) {
                            convert_schema_to_field_type(spec, s)?
                        } else {
                            FieldType::String
                        }
                    }
                };
                Ok(FieldType::Map(
                    Box::new(FieldType::String),
                    Box::new(value_type),
                ))
            } else {
                Ok(FieldType::String) // Complex object, default to string
            }
        }
        _ => Ok(FieldType::String), // Default fallback
    }
}

/// Convert PascalCase or camelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            // Add underscore before uppercase if:
            // 1. Not at the start
            // 2. Previous char is lowercase or digit
            // 3. OR next char is lowercase (handles HTTPServer -> http_server)
            let should_add_underscore = i > 0
                && (chars[i - 1].is_lowercase()
                    || chars[i - 1].is_ascii_digit()
                    || (i + 1 < chars.len() && chars[i + 1].is_lowercase()));

            if should_add_underscore && !result.ends_with('_') {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == ' ' {
            // Replace hyphens and spaces with underscores
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
        } else {
            result.push(ch);
        }
    }

    // Clean up multiple consecutive underscores
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    // Strip leading and trailing underscores
    result.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("PodName"), "pod_name");
        assert_eq!(
            to_snake_case("createNamespacedPod"),
            "create_namespaced_pod"
        );
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
        assert_eq!(to_snake_case("IOError"), "io_error");
        assert_eq!(to_snake_case("v1_api"), "v1_api"); // Already snake_case
        assert_eq!(to_snake_case("__test__"), "test"); // Strip extra underscores
        assert_eq!(to_snake_case("some-resource"), "some_resource"); // Hyphens
    }
}
