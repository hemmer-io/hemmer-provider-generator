//! Integration test for recursive nested blocks code generation
//!
//! This test validates that the generator properly creates SDK builder code
//! for nested block structures (2 levels deep).

use hemmer_provider_generator_common::{
    BlockDefinition, FieldDefinition, FieldType, NestingMode, OperationMapping, Operations,
    Provider, ResourceDefinition, ServiceDefinition,
};
use hemmer_provider_generator_generator::ProviderGenerator;
use tempfile::TempDir;

#[test]
fn test_generate_provider_with_recursive_nested_blocks() {
    // Create a service definition with 2-level nested blocks:
    // bucket_lifecycle -> lifecycle_configuration (Single) -> rules (List)
    let service_def = ServiceDefinition {
        provider: Provider::Aws,
        name: "s3".to_string(),
        sdk_version: "1.0.0".to_string(),
        data_sources: vec![],
        resources: vec![ResourceDefinition {
            name: "bucket_lifecycle".to_string(),
            description: Some("S3 Bucket Lifecycle Configuration".to_string()),
            fields: vec![FieldDefinition {
                name: "bucket".to_string(),
                field_type: FieldType::String,
                required: true,
                sensitive: false,
                immutable: false,
                description: Some("Bucket name".to_string()),
                response_accessor: None,
            }],
            outputs: vec![],
            blocks: vec![BlockDefinition {
                name: "lifecycle_configuration".to_string(),
                description: Some("Lifecycle configuration".to_string()),
                attributes: vec![],
                blocks: vec![BlockDefinition {
                    name: "rules".to_string(),
                    description: Some("Lifecycle rules".to_string()),
                    attributes: vec![
                        FieldDefinition {
                            name: "id".to_string(),
                            field_type: FieldType::String,
                            required: true,
                            sensitive: false,
                            immutable: false,
                            description: Some("Rule ID".to_string()),
                            response_accessor: None,
                        },
                        FieldDefinition {
                            name: "status".to_string(),
                            field_type: FieldType::String,
                            required: false,
                            sensitive: false,
                            immutable: false,
                            description: Some("Rule status".to_string()),
                            response_accessor: None,
                        },
                    ],
                    blocks: vec![],
                    nesting_mode: NestingMode::List,
                    min_items: 0,
                    max_items: 0,
                    sdk_type_name: Some("LifecycleRule".to_string()),
                    sdk_accessor_method: Some("set_rules".to_string()),
                }],
                nesting_mode: NestingMode::Single,
                min_items: 1,
                max_items: 1,
                sdk_type_name: Some("BucketLifecycleConfiguration".to_string()),
                sdk_accessor_method: Some("set_lifecycle_configuration".to_string()),
            }],
            id_field: None,
            operations: Operations {
                create: Some(OperationMapping {
                    sdk_operation: "put_bucket_lifecycle_configuration".to_string(),
                    additional_operations: vec![],
                }),
                read: Some(OperationMapping {
                    sdk_operation: "get_bucket_lifecycle_configuration".to_string(),
                    additional_operations: vec![],
                }),
                update: Some(OperationMapping {
                    sdk_operation: "put_bucket_lifecycle_configuration".to_string(),
                    additional_operations: vec![],
                }),
                delete: Some(OperationMapping {
                    sdk_operation: "delete_bucket_lifecycle".to_string(),
                    additional_operations: vec![],
                }),
                import: None,
            },
        }],
    };

    // Generate provider to temp directory
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    // Also save to a persistent location for debugging
    let debug_dir = std::path::PathBuf::from("/tmp/test-recursive-nested-blocks-integration");
    let _ = std::fs::remove_dir_all(&debug_dir);
    std::fs::create_dir_all(&debug_dir).unwrap();

    let generator = ProviderGenerator::new(service_def.clone()).unwrap();
    let result = generator.generate_to_directory(output_path);
    assert!(result.is_ok(), "Generation failed: {:?}", result);

    // Also generate to debug location
    let generator2 = ProviderGenerator::new(service_def).unwrap();
    let _ = generator2.generate_to_directory(&debug_dir);

    // Read generated lib.rs
    let lib_rs = std::fs::read_to_string(output_path.join("src/lib.rs")).unwrap();

    // Verify lifecycle_configuration block handling (Level 1 - Single)
    assert!(
        lib_rs.contains("lifecycle_configuration"),
        "Should handle lifecycle_configuration block"
    );
    assert!(
        lib_rs.contains("BucketLifecycleConfiguration::builder"),
        "Should create BucketLifecycleConfiguration builder"
    );
    assert!(
        lib_rs.contains("set_lifecycle_configuration"),
        "Should use set_lifecycle_configuration accessor"
    );

    // Verify rules block handling (Level 2 - List within Single)
    assert!(
        lib_rs.contains("// Handle nested blocks within lifecycle_configuration"),
        "Should have comment for nested blocks in lifecycle_configuration"
    );
    assert!(
        lib_rs.contains("if let Some(nested_data) = block_data.get(\"rules\")"),
        "Should extract rules from lifecycle_configuration"
    );
    assert!(
        lib_rs.contains("LifecycleRule::builder"),
        "Should create LifecycleRule builder"
    );
    assert!(
        lib_rs.contains("set_rules"),
        "Should use set_rules accessor"
    );

    // Verify rules attributes are handled
    assert!(
        lib_rs.contains("\"id\""),
        "Should handle id attribute in rules"
    );
    assert!(
        lib_rs.contains("\"status\""),
        "Should handle status attribute in rules"
    );

    // Verify read operation extracts nested blocks
    assert!(
        lib_rs.contains("// Extract nested blocks from response"),
        "Read operation should extract nested blocks"
    );

    // Count nesting levels in create operation
    let create_section = lib_rs
        .split("async fn create")
        .nth(1)
        .expect("Should have create function");

    let lifecycle_config_count = create_section.matches("BucketLifecycleConfiguration").count();
    let lifecycle_rule_count = create_section.matches("LifecycleRule").count();

    assert!(
        lifecycle_config_count > 0,
        "Create should build BucketLifecycleConfiguration"
    );
    assert!(
        lifecycle_rule_count > 0,
        "Create should build LifecycleRule"
    );

    println!("✅ Generated provider with 2-level nested blocks");
    println!("   Level 1: lifecycle_configuration (Single) - BucketLifecycleConfiguration");
    println!("   Level 2: rules (List) - LifecycleRule");
    println!("   Generated code properly handles all nesting levels!");
}
