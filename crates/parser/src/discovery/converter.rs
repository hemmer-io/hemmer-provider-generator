//! Converts Discovery document to ServiceDefinition IR

use super::types::{DiscoveryDoc, Method, Schema};
use hemmer_provider_generator_common::{
    FieldDefinition, FieldType, OperationMapping, Operations, Provider, ResourceDefinition, Result,
    ServiceDefinition,
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
        // Nested blocks will be detected in future parser enhancements
        blocks: vec![],
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
