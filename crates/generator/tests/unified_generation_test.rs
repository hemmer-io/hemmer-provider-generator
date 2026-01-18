//! Integration test for unified multi-service provider generation

use hemmer_provider_generator_common::{
    BlockDefinition, FieldDefinition, FieldType, NestingMode, OperationMapping, Operations,
    Provider, ProviderDefinition, ResourceDefinition, ServiceDefinition,
};
use hemmer_provider_generator_generator::UnifiedProviderGenerator;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_generate_unified_aws_provider() {
    // Create a test ProviderDefinition with two services
    let s3_service = ServiceDefinition {
        provider: Provider::Aws,
        name: "s3".to_string(),
        sdk_version: "1.0.0".to_string(),
        data_sources: vec![], // Will implement data source detection later
        resources: vec![ResourceDefinition {
            name: "bucket".to_string(),
            description: Some("S3 bucket resource".to_string()),
            fields: vec![
                FieldDefinition {
                    name: "bucket_name".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    sensitive: false,
                    immutable: true,
                    description: Some("The name of the bucket".to_string()),
                    response_accessor: None,
                },
                FieldDefinition {
                    name: "region".to_string(),
                    field_type: FieldType::String,
                    required: false,
                    sensitive: false,
                    immutable: true,
                    description: Some("The AWS region".to_string()),
                    response_accessor: None,
                },
            ],
            outputs: vec![FieldDefinition {
                name: "arn".to_string(),
                field_type: FieldType::String,
                required: false,
                sensitive: false,
                immutable: false,
                description: Some("The ARN of the bucket".to_string()),
                response_accessor: Some("arn".to_string()),
            }],
            blocks: vec![],
            id_field: None, // Will implement ID detection later
            operations: Operations {
                create: Some(OperationMapping {
                    sdk_operation: "create_bucket".to_string(),
                    additional_operations: vec![],
                }),
                read: Some(OperationMapping {
                    sdk_operation: "head_bucket".to_string(),
                    additional_operations: vec![],
                }),
                update: None,
                delete: Some(OperationMapping {
                    sdk_operation: "delete_bucket".to_string(),
                    additional_operations: vec![],
                }),
                import: None, // Will implement later
            },
        }],
    };

    let dynamodb_service = ServiceDefinition {
        provider: Provider::Aws,
        name: "dynamodb".to_string(),
        sdk_version: "1.0.0".to_string(),
        data_sources: vec![], // Will implement data source detection later
        resources: vec![ResourceDefinition {
            name: "table".to_string(),
            description: Some("DynamoDB table resource".to_string()),
            fields: vec![
                FieldDefinition {
                    name: "table_name".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    sensitive: false,
                    immutable: true,
                    description: Some("The name of the table".to_string()),
                    response_accessor: None,
                },
                FieldDefinition {
                    name: "read_capacity".to_string(),
                    field_type: FieldType::Integer,
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Read capacity units".to_string()),
                    response_accessor: None,
                },
            ],
            outputs: vec![FieldDefinition {
                name: "table_arn".to_string(),
                field_type: FieldType::String,
                required: false,
                sensitive: false,
                immutable: false,
                description: Some("The ARN of the table".to_string()),
                response_accessor: Some("table_arn".to_string()),
            }],
            blocks: vec![BlockDefinition {
                name: "global_secondary_index".to_string(),
                description: Some("Global secondary index configuration".to_string()),
                attributes: vec![
                    FieldDefinition {
                        name: "index_name".to_string(),
                        field_type: FieldType::String,
                        required: true,
                        sensitive: false,
                        immutable: false,
                        description: Some("Name of the index".to_string()),
                        response_accessor: None,
                    },
                    FieldDefinition {
                        name: "hash_key".to_string(),
                        field_type: FieldType::String,
                        required: true,
                        sensitive: false,
                        immutable: false,
                        description: Some("Hash key attribute name".to_string()),
                        response_accessor: None,
                    },
                    FieldDefinition {
                        name: "range_key".to_string(),
                        field_type: FieldType::String,
                        required: false,
                        sensitive: false,
                        immutable: false,
                        description: Some("Range key attribute name".to_string()),
                        response_accessor: None,
                    },
                ],
                blocks: vec![], // Could have nested projection blocks
                nesting_mode: NestingMode::List,
                min_items: 0,
                max_items: 20, // DynamoDB limit
                sdk_type_name: Some("GlobalSecondaryIndex".to_string()),
                sdk_accessor_method: Some("set_global_secondary_indexes".to_string()),
            }],
            id_field: None, // Will implement ID detection later
            operations: Operations {
                create: Some(OperationMapping {
                    sdk_operation: "create_table".to_string(),
                    additional_operations: vec![],
                }),
                read: Some(OperationMapping {
                    sdk_operation: "describe_table".to_string(),
                    additional_operations: vec![],
                }),
                update: Some(OperationMapping {
                    sdk_operation: "update_table".to_string(),
                    additional_operations: vec![],
                }),
                delete: Some(OperationMapping {
                    sdk_operation: "delete_table".to_string(),
                    additional_operations: vec![],
                }),
                import: None, // Will implement later
            },
        }],
    };

    let provider_def = ProviderDefinition {
        provider: Provider::Aws,
        provider_name: "aws".to_string(),
        sdk_version: "1.0.0".to_string(),
        services: vec![s3_service, dynamodb_service],
    };

    // Create generator
    let generator =
        UnifiedProviderGenerator::new(provider_def).expect("Failed to create generator");

    // Generate to temp directory
    let output_dir = PathBuf::from("/tmp/hemmer-test-unified-provider");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).expect("Failed to clean up test directory");
    }

    generator
        .generate_to_directory(&output_dir)
        .expect("Failed to generate provider");

    // Verify top-level files exist
    assert!(output_dir.join("provider.jcf").exists());
    assert!(output_dir.join("Cargo.toml").exists());
    assert!(output_dir.join("README.md").exists());
    assert!(output_dir.join("src/main.rs").exists());
    assert!(output_dir.join("src/lib.rs").exists());

    // Verify docs directory and files exist
    assert!(output_dir.join("docs").exists());
    assert!(output_dir.join("docs/installation.md").exists());
    assert!(output_dir.join("docs/getting-started.md").exists());
    assert!(output_dir.join("docs/services").exists());
    assert!(output_dir.join("docs/services/s3.md").exists());
    assert!(output_dir.join("docs/services/dynamodb.md").exists());

    // Verify service directories exist
    assert!(output_dir.join("src/s3/mod.rs").exists());
    assert!(output_dir.join("src/s3/resources/mod.rs").exists());
    assert!(output_dir.join("src/s3/resources/bucket.rs").exists());

    assert!(output_dir.join("src/dynamodb/mod.rs").exists());
    assert!(output_dir.join("src/dynamodb/resources/mod.rs").exists());
    assert!(output_dir.join("src/dynamodb/resources/table.rs").exists());

    // Verify content of provider.jcf
    let provider_jcf_content =
        fs::read_to_string(output_dir.join("provider.jcf")).expect("Failed to read provider.jcf");
    assert!(provider_jcf_content.contains("name: \"aws\""));
    assert!(provider_jcf_content.contains("protocol: \"grpc\""));
    assert!(provider_jcf_content.contains("s3:"));
    assert!(provider_jcf_content.contains("dynamodb:"));
    assert!(provider_jcf_content.contains("bucket:"));
    assert!(provider_jcf_content.contains("table:"));

    // Verify content of Cargo.toml
    let cargo_toml_content =
        fs::read_to_string(output_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");
    assert!(cargo_toml_content.contains("hemmer-aws-provider"));
    assert!(cargo_toml_content.contains("aws-sdk-s3"));
    assert!(cargo_toml_content.contains("aws-sdk-dynamodb"));
    assert!(cargo_toml_content.contains("hemmer-provider-sdk = \"0.3.1\""));
    assert!(cargo_toml_content.contains("[[bin]]"));

    // Verify content of main.rs
    let main_rs_content =
        fs::read_to_string(output_dir.join("src/main.rs")).expect("Failed to read main.rs");
    assert!(main_rs_content.contains("hemmer_provider_sdk::serve"));
    assert!(main_rs_content.contains("AwsProvider"));

    // Verify content of lib.rs
    let lib_rs_content =
        fs::read_to_string(output_dir.join("src/lib.rs")).expect("Failed to read lib.rs");
    assert!(lib_rs_content.contains("pub mod s3;"));
    assert!(lib_rs_content.contains("pub mod dynamodb;"));
    assert!(lib_rs_content.contains("pub struct AwsProvider"));
    assert!(lib_rs_content.contains("impl ProviderService for AwsProvider"));
    assert!(lib_rs_content.contains("fn schema(&self)"));
    assert!(lib_rs_content.contains("PROTOCOL_VERSION"));
    assert!(lib_rs_content.contains("SDK_PROTOCOL_VERSION"));

    // Note: Full compilation testing requires realistic service definitions
    // See issue #91 for comprehensive integration testing
    println!("✅ Generated unified code structure verified (full compilation testing in #91)");

    // Clean up
    fs::remove_dir_all(&output_dir).expect("Failed to clean up test directory");
}

#[test]
fn test_generate_unified_provider_with_empty_services() {
    let provider_def = ProviderDefinition {
        provider: Provider::Gcp,
        provider_name: "gcp".to_string(),
        sdk_version: "1.0.0".to_string(),
        services: vec![],
    };

    let generator =
        UnifiedProviderGenerator::new(provider_def).expect("Failed to create generator");

    let output_dir = PathBuf::from("/tmp/hemmer-test-empty-unified-provider");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).expect("Failed to clean up test directory");
    }

    generator
        .generate_to_directory(&output_dir)
        .expect("Failed to generate provider");

    // Verify top-level files exist even with no services
    assert!(output_dir.join("provider.jcf").exists());
    assert!(output_dir.join("Cargo.toml").exists());
    assert!(output_dir.join("README.md").exists());
    assert!(output_dir.join("src/main.rs").exists());
    assert!(output_dir.join("src/lib.rs").exists());

    // Clean up
    fs::remove_dir_all(&output_dir).expect("Failed to clean up test directory");
}
