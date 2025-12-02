//! Converts Discovery document to ServiceDefinition IR

use super::types::{DiscoveryDoc, Method, Schema};
use hemmer_provider_generator_common::{
    BlockDefinition, FieldDefinition, FieldType, NestingMode, OperationMapping, Operations,
    Provider, ResourceDefinition, Result, ServiceDefinition,
};
use std::collections::HashMap;

/// Convert Discovery document to ServiceDefinition
pub fn convert_discovery_to_service_definition(
    doc: &DiscoveryDoc,
    service_name: &str,
    api_version: &str,
) -> Result<ServiceDefinition> {
    // Extract resources from methods
    let resources = extract_resources_from_doc(doc)?;

    Ok(ServiceDefinition {
        provider: Provider::Gcp,
        name: service_name.to_string(),
        sdk_version: api_version.to_string(),
        resources,
        data_sources: vec![], // Will implement data source detection later
    })
}

/// Extract resources from Discovery document
fn extract_resources_from_doc(doc: &DiscoveryDoc) -> Result<Vec<ResourceDefinition>> {
    let mut resource_map: HashMap<String, ResourceMethods> = HashMap::new();

    // Collect all methods from resources
    collect_methods_from_resources(&doc.resources, &mut resource_map);

    // Also check root-level methods (rare but possible)
    for (method_name, method) in &doc.methods {
        if let Some(resource_name) = DiscoveryDoc::extract_resource_from_method_id(&method.id) {
            let entry = resource_map
                .entry(resource_name.clone())
                .or_insert_with(|| ResourceMethods::new(resource_name));

            classify_method(method_name, method, entry);
        }
    }

    // Convert to ResourceDefinitions
    let mut resources = Vec::new();
    for (_name, methods) in resource_map {
        if let Some(resource_def) = build_resource_from_methods(doc, methods)? {
            resources.push(resource_def);
        }
    }

    Ok(resources)
}

/// Recursively collect methods from resources
fn collect_methods_from_resources(
    resources: &HashMap<String, super::types::Resource>,
    resource_map: &mut HashMap<String, ResourceMethods>,
) {
    for resource in resources.values() {
        for (method_name, method) in &resource.methods {
            if let Some(resource_name) = DiscoveryDoc::extract_resource_from_method_id(&method.id) {
                let entry = resource_map
                    .entry(resource_name.clone())
                    .or_insert_with(|| ResourceMethods::new(resource_name));

                classify_method(method_name, method, entry);
            }
        }

        // Recursively process nested resources
        collect_methods_from_resources(&resource.resources, resource_map);
    }
}

/// Temporary structure to collect methods for a resource
#[derive(Debug, Clone)]
struct ResourceMethods {
    name: String,
    create: Option<Method>,
    read: Option<Method>,
    update: Option<Method>,
    delete: Option<Method>,
}

impl ResourceMethods {
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

/// Classify method into CRUD operation
fn classify_method(method_name: &str, method: &Method, methods: &mut ResourceMethods) {
    // Discovery methods are typically named: insert, get, update, patch, delete
    match method_name {
        "insert" | "create" => methods.create = Some(method.clone()),
        "get" | "read" => methods.read = Some(method.clone()),
        "update" | "patch" => methods.update = Some(method.clone()),
        "delete" => methods.delete = Some(method.clone()),
        _ => {
            // Try to infer from HTTP method
            match method.http_method.as_str() {
                "POST" => methods.create = methods.create.clone().or_else(|| Some(method.clone())),
                "GET" => methods.read = methods.read.clone().or_else(|| Some(method.clone())),
                "PUT" | "PATCH" => {
                    methods.update = methods.update.clone().or_else(|| Some(method.clone()))
                }
                "DELETE" => {
                    methods.delete = methods.delete.clone().or_else(|| Some(method.clone()))
                }
                _ => {}
            }
        }
    }
}

/// Build ResourceDefinition from methods
fn build_resource_from_methods(
    doc: &DiscoveryDoc,
    methods: ResourceMethods,
) -> Result<Option<ResourceDefinition>> {
    // Need at least one method
    if methods.create.is_none()
        && methods.read.is_none()
        && methods.update.is_none()
        && methods.delete.is_none()
    {
        return Ok(None);
    }

    // Extract fields from create/update method
    let fields = if let Some(ref create_method) = methods.create {
        extract_fields_from_method(doc, create_method)?
    } else if let Some(ref update_method) = methods.update {
        extract_fields_from_method(doc, update_method)?
    } else {
        Vec::new()
    };

    // Extract outputs from read method
    let outputs = if let Some(ref read_method) = methods.read {
        extract_outputs_from_method(doc, read_method)?
    } else {
        Vec::new()
    };

    // Detect nested blocks from create/update method
    let blocks =
        if let Some(ref create_method) = methods.create.as_ref().or(methods.update.as_ref()) {
            extract_blocks_from_method(doc, create_method)?
        } else {
            Vec::new()
        };

    // Get description
    let description = methods
        .create
        .as_ref()
        .and_then(|m| m.description.clone())
        .or_else(|| methods.read.as_ref().and_then(|m| m.description.clone()));

    Ok(Some(ResourceDefinition {
        name: to_snake_case(&methods.name),
        description,
        fields,
        outputs,
        blocks,
        id_field: None, // Will implement ID detection later
        operations: Operations {
            create: methods.create.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.id.split('.').next_back().unwrap_or(&m.id)),
                additional_operations: vec![],
            }),
            read: methods.read.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.id.split('.').next_back().unwrap_or(&m.id)),
                additional_operations: vec![],
            }),
            update: methods.update.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.id.split('.').next_back().unwrap_or(&m.id)),
                additional_operations: vec![],
            }),
            delete: methods.delete.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.id.split('.').next_back().unwrap_or(&m.id)),
                additional_operations: vec![],
            }),
            import: None, // Will implement later
        },
    }))
}

/// Extract fields from method request
fn extract_fields_from_method(doc: &DiscoveryDoc, method: &Method) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    // Extract from request schema
    if let Some(ref request) = method.request {
        if let Some(schema) = doc.resolve_schema_ref(&request.ref_schema) {
            fields.extend(extract_fields_from_schema(doc, schema)?);
        }
    }

    // Extract from parameters
    for (param_name, param) in &method.parameters {
        if param.location.as_deref() == Some("path") {
            // Path parameters are typically identifiers
            let field_type = match param.param_type.as_deref() {
                Some("string") => FieldType::String,
                Some("integer") => FieldType::Integer,
                Some("boolean") => FieldType::Boolean,
                _ => FieldType::String,
            };

            fields.push(FieldDefinition {
                name: to_snake_case(param_name),
                field_type,
                required: param.required,
                sensitive: false,
                immutable: true, // Path params are usually immutable identifiers
                description: param.description.clone(),
                response_accessor: None, // Input fields don't have response accessors
            });
        }
    }

    Ok(fields)
}

/// Extract outputs from method response
fn extract_outputs_from_method(
    doc: &DiscoveryDoc,
    method: &Method,
) -> Result<Vec<FieldDefinition>> {
    let mut outputs = Vec::new();

    if let Some(ref response) = method.response {
        if let Some(schema) = doc.resolve_schema_ref(&response.ref_schema) {
            outputs = extract_fields_from_schema(doc, schema)?;
        }
    }

    Ok(outputs)
}

/// Extract fields from schema (used for response/output fields)
fn extract_fields_from_schema(doc: &DiscoveryDoc, schema: &Schema) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    for (field_name, field_schema) in &schema.properties {
        let field_type = convert_schema_to_field_type(doc, field_schema)?;
        let required = schema.required.contains(field_name);
        let accessor_name = to_snake_case(field_name);

        fields.push(FieldDefinition {
            name: accessor_name.clone(),
            field_type,
            required,
            sensitive: false,
            immutable: false,
            description: field_schema.description.clone(),
            // Response fields have accessors for extracting values from SDK responses
            response_accessor: Some(accessor_name),
        });
    }

    Ok(fields)
}

/// Extract nested blocks from method request
fn extract_blocks_from_method(doc: &DiscoveryDoc, method: &Method) -> Result<Vec<BlockDefinition>> {
    let mut blocks = Vec::new();

    if let Some(ref request) = method.request {
        if let Some(schema) = doc.resolve_schema_ref(&request.ref_schema) {
            blocks = detect_nested_blocks_from_schema(doc, schema)?;
        }
    }

    Ok(blocks)
}

/// Detect nested blocks from schema properties
fn detect_nested_blocks_from_schema(
    doc: &DiscoveryDoc,
    schema: &Schema,
) -> Result<Vec<BlockDefinition>> {
    let mut blocks = Vec::new();

    // Check each property for potential blocks
    for (prop_name, prop_schema) in &schema.properties {
        if let Some(block) = try_extract_block_from_property(doc, prop_name, prop_schema)? {
            blocks.push(block);
        }
    }

    Ok(blocks)
}

/// Try to extract a BlockDefinition from a schema property
fn try_extract_block_from_property(
    doc: &DiscoveryDoc,
    prop_name: &str,
    schema: &Schema,
) -> Result<Option<BlockDefinition>> {
    // Resolve reference if needed
    let resolved_schema = if let Some(ref ref_name) = schema.ref_schema {
        if let Some(s) = doc.resolve_schema_ref(ref_name) {
            s
        } else {
            return Ok(None);
        }
    } else {
        schema
    };

    match resolved_schema.schema_type.as_deref() {
        // Array of objects → List block
        Some("array") => {
            if let Some(ref items_schema) = resolved_schema.items {
                // Resolve item schema if it's a reference
                let items = if let Some(ref ref_name) = items_schema.ref_schema {
                    if let Some(s) = doc.resolve_schema_ref(ref_name) {
                        s
                    } else {
                        return Ok(None);
                    }
                } else {
                    items_schema.as_ref()
                };

                // Check if items are objects (not primitive types)
                if items.schema_type.as_deref() == Some("object") || items.ref_schema.is_some() {
                    // This is an array of objects - perfect for a block!
                    let attributes = extract_fields_from_schema_for_block(doc, items)?;
                    let nested_blocks = detect_nested_blocks_from_schema(doc, items)?;

                    return Ok(Some(BlockDefinition {
                        name: to_snake_case(prop_name),
                        description: resolved_schema.description.clone(),
                        attributes,
                        blocks: nested_blocks,
                        nesting_mode: NestingMode::List,
                        min_items: 0,
                        max_items: 0, // 0 = unlimited
                        sdk_type_name: None, // TODO: Extract from GCP Discovery schema
                        sdk_accessor_method: None, // TODO: Extract for GCP
                    }));
                }
            }
        }
        // Single object → Single block
        Some("object") => {
            // Skip if this is a simple map (has additional_properties but no properties)
            if resolved_schema.additional_properties.is_some()
                && resolved_schema.properties.is_empty()
            {
                return Ok(None);
            }

            // Only treat as block if it's complex enough (3+ properties or has nested structures)
            if is_complex_schema(doc, resolved_schema) {
                let attributes = extract_fields_from_schema_for_block(doc, resolved_schema)?;
                let nested_blocks = detect_nested_blocks_from_schema(doc, resolved_schema)?;

                return Ok(Some(BlockDefinition {
                    name: to_snake_case(prop_name),
                    description: resolved_schema.description.clone(),
                    attributes,
                    blocks: nested_blocks,
                    nesting_mode: NestingMode::Single,
                    min_items: 1,
                    max_items: 1,
                    sdk_type_name: None, // TODO: Extract from GCP Discovery schema
                    sdk_accessor_method: None, // TODO: Extract for GCP
                }));
            }
        }
        _ => {}
    }

    Ok(None)
}

/// Check if a schema is complex enough to be a block
fn is_complex_schema(doc: &DiscoveryDoc, schema: &Schema) -> bool {
    if schema.properties.len() >= 3 {
        return true;
    }

    // Check if any property is itself an object or array
    for prop_schema in schema.properties.values() {
        let resolved = if let Some(ref ref_name) = prop_schema.ref_schema {
            if let Some(s) = doc.resolve_schema_ref(ref_name) {
                s
            } else {
                continue;
            }
        } else {
            prop_schema
        };

        match resolved.schema_type.as_deref() {
            Some("object") | Some("array") => return true,
            _ => {}
        }
    }

    false
}

/// Extract fields from schema for block attributes (skip nested structures)
fn extract_fields_from_schema_for_block(
    doc: &DiscoveryDoc,
    schema: &Schema,
) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    for (field_name, field_schema) in &schema.properties {
        // Resolve reference if needed
        let resolved = if let Some(ref ref_name) = field_schema.ref_schema {
            if let Some(s) = doc.resolve_schema_ref(ref_name) {
                s
            } else {
                continue;
            }
        } else {
            field_schema
        };

        // Skip complex nested structures (those will be blocks)
        if is_potential_block_property(doc, resolved) {
            continue;
        }

        let field_type = convert_schema_to_field_type(doc, resolved)?;
        let accessor_name = to_snake_case(field_name);

        fields.push(FieldDefinition {
            name: accessor_name,
            field_type,
            required: false, // Discovery format doesn't have clear required markers in the same way
            sensitive: false,
            immutable: false,
            description: resolved.description.clone(),
            response_accessor: None,
        });
    }

    Ok(fields)
}

/// Check if a property should be treated as a block rather than a field
fn is_potential_block_property(doc: &DiscoveryDoc, schema: &Schema) -> bool {
    // Resolve reference if needed
    let resolved = if let Some(ref ref_name) = schema.ref_schema {
        if let Some(s) = doc.resolve_schema_ref(ref_name) {
            s
        } else {
            return false;
        }
    } else {
        schema
    };

    match resolved.schema_type.as_deref() {
        // Array of objects
        Some("array") => {
            if let Some(ref items) = resolved.items {
                let items_resolved = if let Some(ref ref_name) = items.ref_schema {
                    if let Some(s) = doc.resolve_schema_ref(ref_name) {
                        s
                    } else {
                        return false;
                    }
                } else {
                    items.as_ref()
                };
                items_resolved.schema_type.as_deref() == Some("object")
                    || items_resolved.ref_schema.is_some()
            } else {
                false
            }
        }
        // Complex objects (not simple maps)
        Some("object") => {
            if resolved.additional_properties.is_some() && resolved.properties.is_empty() {
                false // This is a map
            } else {
                is_complex_schema(doc, resolved)
            }
        }
        _ => false,
    }
}

/// Convert Discovery schema to FieldType
fn convert_schema_to_field_type(doc: &DiscoveryDoc, schema: &Schema) -> Result<FieldType> {
    // Handle reference
    if let Some(ref ref_name) = schema.ref_schema {
        if let Some(resolved) = doc.resolve_schema_ref(ref_name) {
            return convert_schema_to_field_type(doc, resolved);
        }
    }

    // Handle type
    match schema.schema_type.as_deref() {
        Some("string") => match schema.format.as_deref() {
            Some("date-time") | Some("date") => Ok(FieldType::DateTime),
            _ => Ok(FieldType::String),
        },
        Some("integer") => Ok(FieldType::Integer),
        Some("number") => Ok(FieldType::Float),
        Some("boolean") => Ok(FieldType::Boolean),
        Some("array") => {
            if let Some(ref items) = schema.items {
                let item_type = convert_schema_to_field_type(doc, items)?;
                Ok(FieldType::List(Box::new(item_type)))
            } else {
                Ok(FieldType::List(Box::new(FieldType::String)))
            }
        }
        Some("object") => {
            if let Some(ref additional_props) = schema.additional_properties {
                // This is a map
                let value_type = convert_schema_to_field_type(doc, additional_props)?;
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
        assert_eq!(to_snake_case("BucketName"), "bucket_name");
        assert_eq!(
            to_snake_case("storage.buckets.insert"),
            "storage.buckets.insert"
        );
        assert_eq!(to_snake_case("CloudStorage"), "cloud_storage");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
        assert_eq!(to_snake_case("__test__"), "test");
    }
}
