//! Converts protobuf descriptors to ServiceDefinition IR

use hemmer_provider_generator_common::{
    FieldDefinition, FieldType, OperationMapping, Operations, Provider, ResourceDefinition, Result,
    ServiceDefinition,
};
use prost_reflect::{DescriptorPool, Kind, MethodDescriptor, ServiceDescriptor};
use std::collections::HashMap;

/// Convert protobuf DescriptorPool to ServiceDefinition
pub fn convert_protobuf_to_service_definition(
    pool: &DescriptorPool,
    service_name: &str,
    api_version: &str,
) -> Result<ServiceDefinition> {
    let mut resources = Vec::new();

    // Iterate through all services in the pool
    for service in pool.services() {
        // Extract resources from each service
        let service_resources = extract_resources_from_service(pool, &service)?;
        resources.extend(service_resources);
    }

    Ok(ServiceDefinition {
        provider: Provider::Gcp, // Most gRPC APIs are GCP, but could be configurable
        name: service_name.to_string(),
        sdk_version: api_version.to_string(),
        resources,
        data_sources: vec![],  // Will implement data source detection later
    })
}

/// Extract resources from a gRPC service
fn extract_resources_from_service(
    pool: &DescriptorPool,
    service: &ServiceDescriptor,
) -> Result<Vec<ResourceDefinition>> {
    // Group methods by resource
    let resource_map = group_methods_by_resource(service);

    let mut resources = Vec::new();

    for (resource_name, methods) in resource_map {
        if let Some(resource_def) = build_resource_from_methods(pool, &resource_name, methods)? {
            resources.push(resource_def);
        }
    }

    Ok(resources)
}

/// Group gRPC methods by resource name
///
/// Examples:
/// - CreateBucket, GetBucket, DeleteBucket -> "Bucket"
/// - CreateInstance, UpdateInstance -> "Instance"
fn group_methods_by_resource(
    service: &ServiceDescriptor,
) -> HashMap<String, Vec<MethodDescriptor>> {
    let mut resource_map: HashMap<String, Vec<MethodDescriptor>> = HashMap::new();

    for method in service.methods() {
        if let Some(resource_name) = extract_resource_from_method_name(method.name()) {
            resource_map
                .entry(resource_name)
                .or_default()
                .push(method.clone());
        }
    }

    resource_map
}

/// Extract resource name from gRPC method name
///
/// Examples:
/// - "CreateBucket" -> Some("Bucket")
/// - "GetInstance" -> Some("Instance")
/// - "ListBuckets" -> Some("Bucket")
/// - "UpdateDatabase" -> Some("Database")
fn extract_resource_from_method_name(method_name: &str) -> Option<String> {
    let prefixes = [
        "Create", "Get", "Update", "Delete", "Put", "Patch", "List", "Describe", "Insert",
    ];

    for prefix in &prefixes {
        if method_name.starts_with(prefix) && method_name.len() > prefix.len() {
            let resource = &method_name[prefix.len()..];
            // Singularize if ends with 's' (e.g., "ListBuckets" -> "Bucket")
            let mut singular = resource.to_string();
            if singular.ends_with('s') && singular.len() > 1 {
                singular.pop();
            }
            return Some(singular);
        }
    }

    None
}

/// Build ResourceDefinition from gRPC methods
fn build_resource_from_methods(
    pool: &DescriptorPool,
    resource_name: &str,
    methods: Vec<MethodDescriptor>,
) -> Result<Option<ResourceDefinition>> {
    // Classify methods into CRUD operations
    let mut create_method = None;
    let mut read_method = None;
    let mut update_method = None;
    let mut delete_method = None;

    for method in &methods {
        let name = method.name();
        if name.starts_with("Create") || name.starts_with("Insert") {
            create_method = Some(method);
        } else if name.starts_with("Get") || name.starts_with("Describe") {
            read_method = Some(method);
        } else if name.starts_with("Update") || name.starts_with("Patch") || name.starts_with("Put")
        {
            update_method = Some(method);
        } else if name.starts_with("Delete") {
            delete_method = Some(method);
        }
    }

    // Need at least one CRUD operation
    if create_method.is_none()
        && read_method.is_none()
        && update_method.is_none()
        && delete_method.is_none()
    {
        return Ok(None);
    }

    // Extract fields from create/update request (input fields, no response accessors)
    let fields = if let Some(method) = create_method {
        extract_fields_from_message(pool, method.input(), false)?
    } else if let Some(method) = update_method {
        extract_fields_from_message(pool, method.input(), false)?
    } else {
        Vec::new()
    };

    // Extract outputs from read response (output fields, have response accessors)
    let outputs = if let Some(method) = read_method {
        extract_fields_from_message(pool, method.output(), true)?
    } else {
        Vec::new()
    };

    Ok(Some(ResourceDefinition {
        name: to_snake_case(resource_name),
        description: None, // Could extract from proto comments in future
        fields,
        outputs,
        id_field: None, // Will implement ID detection later
        operations: Operations {
            create: create_method.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.name()),
                additional_operations: vec![],
            }),
            read: read_method.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.name()),
                additional_operations: vec![],
            }),
            update: update_method.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.name()),
                additional_operations: vec![],
            }),
            delete: delete_method.map(|m| OperationMapping {
                sdk_operation: to_snake_case(m.name()),
                additional_operations: vec![],
            }),
            import: None, // Will implement later
        },
    }))
}

/// Extract fields from protobuf message descriptor
///
/// `is_response` - if true, fields are from a response and should have response accessors
fn extract_fields_from_message(
    pool: &DescriptorPool,
    message: prost_reflect::MessageDescriptor,
    is_response: bool,
) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    for field in message.fields() {
        let field_type = convert_protobuf_kind_to_field_type(pool, &field.kind())?;
        let accessor_name = to_snake_case(field.name());

        fields.push(FieldDefinition {
            name: accessor_name.clone(),
            field_type,
            required: !field.is_list() && !field.is_map(),
            sensitive: false,
            immutable: false,
            description: None, // Could extract from proto comments
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

/// Convert protobuf Kind to FieldType
fn convert_protobuf_kind_to_field_type(_pool: &DescriptorPool, kind: &Kind) -> Result<FieldType> {
    Ok(match kind {
        Kind::Double | Kind::Float => FieldType::Float,
        Kind::Int32
        | Kind::Int64
        | Kind::Uint32
        | Kind::Uint64
        | Kind::Sint32
        | Kind::Sint64
        | Kind::Fixed32
        | Kind::Fixed64
        | Kind::Sfixed32
        | Kind::Sfixed64 => FieldType::Integer,
        Kind::Bool => FieldType::Boolean,
        Kind::String | Kind::Bytes => FieldType::String,
        Kind::Message(msg_desc) => {
            // Check if it's a well-known type
            let full_name = msg_desc.full_name();
            match full_name {
                "google.protobuf.Timestamp" => FieldType::DateTime,
                _ => {
                    // Complex message - could recursively extract fields
                    // For now, treat as string (JSON representation)
                    FieldType::String
                }
            }
        }
        Kind::Enum(_) => FieldType::String, // Enums as strings
    })
}

/// Convert PascalCase to snake_case
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
    fn test_extract_resource_from_method_name() {
        assert_eq!(
            extract_resource_from_method_name("CreateBucket"),
            Some("Bucket".to_string())
        );
        assert_eq!(
            extract_resource_from_method_name("GetInstance"),
            Some("Instance".to_string())
        );
        assert_eq!(
            extract_resource_from_method_name("ListBuckets"),
            Some("Bucket".to_string())
        );
        assert_eq!(
            extract_resource_from_method_name("UpdateDatabase"),
            Some("Database".to_string())
        );
        assert_eq!(
            extract_resource_from_method_name("DeleteObject"),
            Some("Object".to_string())
        );
        assert_eq!(extract_resource_from_method_name("InvalidMethod"), None);
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("CreateBucket"), "create_bucket");
        assert_eq!(to_snake_case("GetInstance"), "get_instance");
        assert_eq!(to_snake_case("BucketName"), "bucket_name");
    }
}
