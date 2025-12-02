//! Converts Smithy model to ServiceDefinition IR

use super::types::{Shape, SmithyModel};
use hemmer_provider_generator_common::{
    BlockDefinition, FieldDefinition, FieldType, GeneratorError, NestingMode, OperationMapping,
    Operations, Provider, ResourceDefinition, Result, ServiceDefinition,
};
use std::collections::HashMap;

/// Convert Smithy model to ServiceDefinition
pub fn convert_smithy_to_service_definition(
    model: &SmithyModel,
    service_name: &str,
    sdk_version: &str,
) -> Result<ServiceDefinition> {
    // Find service shape
    let (_service_id, service_shape) = model.find_service().ok_or_else(|| {
        GeneratorError::Parse("No service shape found in Smithy model".to_string())
    })?;

    // Extract resources from the model
    let resources = extract_resources(model, service_shape)?;

    Ok(ServiceDefinition {
        provider: Provider::Aws,
        name: service_name.to_string(),
        sdk_version: sdk_version.to_string(),
        resources,
        data_sources: vec![], // Will implement data source detection later
    })
}

/// Extract resources from Smithy service
fn extract_resources(
    model: &SmithyModel,
    service_shape: &Shape,
) -> Result<Vec<ResourceDefinition>> {
    let mut resources = Vec::new();

    // Get operations from service
    let operations = match service_shape {
        Shape::Service { operations, .. } => operations,
        _ => return Ok(resources),
    };

    // Group operations by resource
    let grouped = group_operations_by_resource(model, operations)?;

    // Convert each group to a ResourceDefinition
    for (resource_name, ops) in grouped {
        if let Some(resource) = build_resource_from_operations(model, &resource_name, ops)? {
            resources.push(resource);
        }
    }

    Ok(resources)
}

/// Group operations by resource name
/// e.g., "CreateBucket", "DeleteBucket" -> "Bucket"
fn group_operations_by_resource(
    _model: &SmithyModel,
    operations: &[super::types::ShapeReference],
) -> Result<HashMap<String, Vec<String>>> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

    for op_ref in operations {
        let op_name = extract_operation_name(&op_ref.target);

        // Extract resource name from operation
        // e.g., "CreateBucket" -> "Bucket"
        if let Some(resource_name) = extract_resource_from_operation(&op_name) {
            grouped
                .entry(resource_name.to_string())
                .or_default()
                .push(op_name);
        }
    }

    Ok(grouped)
}

/// Extract resource name from operation name
/// Examples:
/// - "CreateBucket" -> Some("Bucket")
/// - "PutObject" -> Some("Object")
/// - "ListBuckets" -> Some("Bucket")
fn extract_resource_from_operation(operation: &str) -> Option<String> {
    // Common CRUD prefixes
    let prefixes = [
        "Create", "Put", "Delete", "Get", "List", "Describe", "Update", "Head",
    ];

    for prefix in &prefixes {
        if let Some(resource) = operation.strip_prefix(prefix) {
            if !resource.is_empty() {
                // Remove trailing 's' for List operations
                let resource = if prefix == &"List" && resource.ends_with('s') {
                    &resource[..resource.len() - 1]
                } else {
                    resource
                };
                return Some(resource.to_string());
            }
        }
    }

    None
}

/// Build ResourceDefinition from grouped operations
fn build_resource_from_operations(
    model: &SmithyModel,
    resource_name: &str,
    operations: Vec<String>,
) -> Result<Option<ResourceDefinition>> {
    // Classify operations into CRUD
    let mut create_op = None;
    let mut read_op = None;
    let mut update_op = None;
    let mut delete_op = None;

    for op_name in &operations {
        if op_name.starts_with("Create") || op_name.starts_with("Put") {
            create_op = Some(op_name.clone());
        } else if op_name.starts_with("Get")
            || op_name.starts_with("Describe")
            || op_name.starts_with("Head")
        {
            read_op = Some(op_name.clone());
        } else if op_name.starts_with("Update") || op_name.starts_with("Modify") {
            update_op = Some(op_name.clone());
        } else if op_name.starts_with("Delete") {
            delete_op = Some(op_name.clone());
        }
    }

    // Need at least one operation to create a resource
    if create_op.is_none() && read_op.is_none() && update_op.is_none() && delete_op.is_none() {
        return Ok(None);
    }

    // Extract fields from create/update operation inputs
    let fields = if let Some(ref op) = create_op {
        extract_fields_from_operation(model, op)?
    } else if let Some(ref op) = update_op {
        extract_fields_from_operation(model, op)?
    } else {
        Vec::new()
    };

    // Extract outputs from read operation
    let outputs = if let Some(ref op) = read_op {
        extract_outputs_from_operation(model, op)?
    } else {
        Vec::new()
    };

    // Detect nested blocks from create/update operation inputs
    let blocks = if let Some(ref op) = create_op.as_ref().or(update_op.as_ref()) {
        extract_blocks_from_operation(model, op)?
    } else {
        Vec::new()
    };

    Ok(Some(ResourceDefinition {
        name: to_snake_case(resource_name),
        description: Some(format!("{} resource", resource_name)),
        fields,
        outputs,
        blocks,
        id_field: None, // Will implement ID detection later
        operations: Operations {
            create: create_op.map(|op| OperationMapping {
                sdk_operation: to_snake_case(&op),
                additional_operations: vec![],
            }),
            read: read_op.map(|op| OperationMapping {
                sdk_operation: to_snake_case(&op),
                additional_operations: vec![],
            }),
            update: update_op.map(|op| OperationMapping {
                sdk_operation: to_snake_case(&op),
                additional_operations: vec![],
            }),
            delete: delete_op.map(|op| OperationMapping {
                sdk_operation: to_snake_case(&op),
                additional_operations: vec![],
            }),
            import: None, // Will implement later
        },
    }))
}

/// Extract fields from operation input
fn extract_fields_from_operation(
    model: &SmithyModel,
    op_name: &str,
) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    // Find operation shape
    let op_shape = find_shape_by_name(model, op_name);

    if let Some(Shape::Operation {
        input: Some(input_ref),
        ..
    }) = op_shape
    {
        // Get input structure
        if let Some(Shape::Structure { members, traits: _ }) = model.get_shape(&input_ref.target) {
            for (field_name, member) in members {
                let field_type = convert_smithy_type_to_field_type(model, &member.target)?;
                let required = member.traits.contains_key(super::types::traits::REQUIRED);
                let sensitive = member.traits.contains_key(super::types::traits::SENSITIVE);
                let description = extract_documentation(&member.traits);

                fields.push(FieldDefinition {
                    name: to_snake_case(field_name),
                    field_type,
                    required,
                    sensitive,
                    immutable: false, // TODO: determine from traits
                    description,
                    response_accessor: None, // Input fields don't have response accessors
                });
            }
        }
    }

    Ok(fields)
}

/// Extract nested blocks from operation input
fn extract_blocks_from_operation(
    model: &SmithyModel,
    op_name: &str,
) -> Result<Vec<BlockDefinition>> {
    // Find operation shape
    let op_shape = find_shape_by_name(model, op_name);

    if let Some(Shape::Operation {
        input: Some(input_ref),
        ..
    }) = op_shape
    {
        // Get input structure
        if let Some(input_shape) = model.get_shape(&input_ref.target) {
            return detect_nested_blocks_from_structure(model, input_shape);
        }
    }

    Ok(Vec::new())
}

/// Extract outputs from operation output
fn extract_outputs_from_operation(
    model: &SmithyModel,
    op_name: &str,
) -> Result<Vec<FieldDefinition>> {
    let mut outputs = Vec::new();

    // Find operation shape
    let op_shape = find_shape_by_name(model, op_name);

    if let Some(Shape::Operation {
        output: Some(output_ref),
        ..
    }) = op_shape
    {
        // Get output structure
        if let Some(Shape::Structure { members, .. }) = model.get_shape(&output_ref.target) {
            for (field_name, member) in members {
                let field_type = convert_smithy_type_to_field_type(model, &member.target)?;
                let description = extract_documentation(&member.traits);

                // The SDK accessor method name is the snake_case version of the member name
                let accessor_name = to_snake_case(field_name);
                outputs.push(FieldDefinition {
                    name: accessor_name.clone(),
                    field_type,
                    required: false,
                    sensitive: member.traits.contains_key(super::types::traits::SENSITIVE),
                    immutable: true,
                    description,
                    // For AWS SDK, the response accessor is the same as the field name
                    response_accessor: Some(accessor_name),
                });
            }
        }
    }

    Ok(outputs)
}

/// Detect nested blocks from structure members
/// A member is considered a block if it's a List or Structure that contains complex data
fn detect_nested_blocks_from_structure(
    model: &SmithyModel,
    structure_shape: &Shape,
) -> Result<Vec<BlockDefinition>> {
    let mut blocks = Vec::new();

    if let Shape::Structure { members, .. } = structure_shape {
        for (member_name, member) in members {
            if let Some(block) = try_extract_block_from_member(model, member_name, member)? {
                blocks.push(block);
            }
        }
    }

    Ok(blocks)
}

/// Try to extract a BlockDefinition from a structure member
fn try_extract_block_from_member(
    model: &SmithyModel,
    member_name: &str,
    member: &super::types::Member,
) -> Result<Option<BlockDefinition>> {
    let target_shape = model.get_shape(&member.target);

    match target_shape {
        // List of structures → List block
        Some(Shape::List {
            member: list_member,
            ..
        }) => {
            if let Some(item_shape) = model.get_shape(&list_member.target) {
                if let Shape::Structure {
                    members: item_members,
                    ..
                } = item_shape
                {
                    // This is a list of structures - perfect for a block!
                    let attributes = extract_fields_from_structure_members(model, item_members)?;
                    let nested_blocks = detect_nested_blocks_from_structure(model, item_shape)?;

                    return Ok(Some(BlockDefinition {
                        name: to_snake_case(member_name),
                        description: extract_documentation(&member.traits),
                        attributes,
                        blocks: nested_blocks,
                        nesting_mode: NestingMode::List,
                        min_items: 0,
                        max_items: 0, // 0 = unlimited
                    }));
                }
            }
        }
        // Single structure → Single block
        Some(Shape::Structure {
            members: nested_members,
            ..
        }) => {
            // Skip if this looks like a simple wrapper (only 1-2 primitive fields)
            if is_complex_structure(model, nested_members) {
                let attributes = extract_fields_from_structure_members(model, nested_members)?;
                let nested_blocks =
                    detect_nested_blocks_from_structure(model, target_shape.unwrap())?;

                return Ok(Some(BlockDefinition {
                    name: to_snake_case(member_name),
                    description: extract_documentation(&member.traits),
                    attributes,
                    blocks: nested_blocks,
                    nesting_mode: NestingMode::Single,
                    min_items: 1,
                    max_items: 1,
                }));
            }
        }
        _ => {}
    }

    Ok(None)
}

/// Check if a structure is complex enough to be a block
/// Returns true if it has 3+ members or contains nested structures
fn is_complex_structure(
    model: &SmithyModel,
    members: &HashMap<String, super::types::Member>,
) -> bool {
    if members.len() >= 3 {
        return true;
    }

    // Check if any member is itself a structure or list
    for member in members.values() {
        if let Some(shape) = model.get_shape(&member.target) {
            match shape {
                Shape::Structure { .. } | Shape::List { .. } | Shape::Map { .. } => return true,
                _ => {}
            }
        }
    }

    false
}

/// Extract fields from structure members (helper for block attributes)
fn extract_fields_from_structure_members(
    model: &SmithyModel,
    members: &HashMap<String, super::types::Member>,
) -> Result<Vec<FieldDefinition>> {
    let mut fields = Vec::new();

    for (field_name, member) in members {
        // Only extract primitive fields as attributes (skip nested structures - those become blocks)
        let field_type = convert_smithy_type_to_field_type(model, &member.target)?;

        // Skip if this would be a nested block (we handle those separately)
        if is_potential_block_member(model, member) {
            continue;
        }

        let required = member.traits.contains_key(super::types::traits::REQUIRED);
        let sensitive = member.traits.contains_key(super::types::traits::SENSITIVE);
        let description = extract_documentation(&member.traits);

        fields.push(FieldDefinition {
            name: to_snake_case(field_name),
            field_type,
            required,
            sensitive,
            immutable: false,
            description,
            response_accessor: None,
        });
    }

    Ok(fields)
}

/// Check if a member should be treated as a block rather than a field
fn is_potential_block_member(model: &SmithyModel, member: &super::types::Member) -> bool {
    if let Some(shape) = model.get_shape(&member.target) {
        match shape {
            // Lists of structures are blocks
            Shape::List {
                member: list_member,
                ..
            } => {
                if let Some(item_shape) = model.get_shape(&list_member.target) {
                    matches!(item_shape, Shape::Structure { .. })
                } else {
                    false
                }
            }
            // Complex structures are blocks
            Shape::Structure { members, .. } => is_complex_structure(model, members),
            _ => false,
        }
    } else {
        false
    }
}

/// Convert Smithy type to FieldType
fn convert_smithy_type_to_field_type(model: &SmithyModel, shape_id: &str) -> Result<FieldType> {
    // Handle primitives by checking the type name
    if shape_id.contains("#String") || shape_id.ends_with("String") {
        return Ok(FieldType::String);
    }
    if shape_id.contains("#Integer") || shape_id.contains("#Long") {
        return Ok(FieldType::Integer);
    }
    if shape_id.contains("#Boolean") {
        return Ok(FieldType::Boolean);
    }
    if shape_id.contains("#Double") || shape_id.contains("#Float") {
        return Ok(FieldType::Float);
    }
    if shape_id.contains("#Timestamp") {
        return Ok(FieldType::DateTime);
    }

    // Look up the shape
    if let Some(shape) = model.get_shape(shape_id) {
        match shape {
            Shape::String { .. } => Ok(FieldType::String),
            Shape::Integer { .. } | Shape::Long { .. } => Ok(FieldType::Integer),
            Shape::Boolean { .. } => Ok(FieldType::Boolean),
            Shape::Double { .. } => Ok(FieldType::Float),
            Shape::Timestamp { .. } => Ok(FieldType::DateTime),
            Shape::List { member, .. } => {
                let inner_type = convert_smithy_type_to_field_type(model, &member.target)?;
                Ok(FieldType::List(Box::new(inner_type)))
            }
            Shape::Map { key, value, .. } => {
                let key_type = convert_smithy_type_to_field_type(model, &key.target)?;
                let value_type = convert_smithy_type_to_field_type(model, &value.target)?;
                Ok(FieldType::Map(Box::new(key_type), Box::new(value_type)))
            }
            _ => Ok(FieldType::String), // Default fallback
        }
    } else {
        Ok(FieldType::String) // Default fallback
    }
}

/// Find shape by operation name (handles various ID formats)
fn find_shape_by_name<'a>(model: &'a SmithyModel, name: &str) -> Option<&'a Shape> {
    // Try exact match first
    if let Some(shape) = model.get_shape(name) {
        return Some(shape);
    }

    // Try finding by suffix (e.g., "CreateBucket" in "com.amazonaws.s3#CreateBucket")
    model
        .shapes
        .iter()
        .find(|(id, _)| id.ends_with(&format!("#{}", name)))
        .map(|(_, shape)| shape)
}

/// Extract operation name from shape ID
/// e.g., "com.amazonaws.s3#CreateBucket" -> "CreateBucket"
fn extract_operation_name(shape_id: &str) -> String {
    shape_id
        .split('#')
        .next_back()
        .unwrap_or(shape_id)
        .to_string()
}

/// Extract documentation from traits
fn extract_documentation(traits: &HashMap<String, serde_json::Value>) -> Option<String> {
    traits
        .get(super::types::traits::DOCUMENTATION)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
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
    fn test_extract_resource_from_operation() {
        assert_eq!(
            extract_resource_from_operation("CreateBucket"),
            Some("Bucket".to_string())
        );
        assert_eq!(
            extract_resource_from_operation("DeleteObject"),
            Some("Object".to_string())
        );
        assert_eq!(
            extract_resource_from_operation("ListBuckets"),
            Some("Bucket".to_string())
        );
        assert_eq!(
            extract_resource_from_operation("GetBucketAcl"),
            Some("BucketAcl".to_string())
        );
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("CreateBucket"), "create_bucket");
        assert_eq!(to_snake_case("PutObject"), "put_object");
        assert_eq!(to_snake_case("S3Bucket"), "s3_bucket");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("XMLParser"), "xml_parser");
        assert_eq!(to_snake_case("__test__"), "test");
    }

    #[test]
    fn test_extract_operation_name() {
        assert_eq!(
            extract_operation_name("com.amazonaws.s3#CreateBucket"),
            "CreateBucket"
        );
        assert_eq!(extract_operation_name("CreateBucket"), "CreateBucket");
    }
}
