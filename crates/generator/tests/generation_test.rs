//! Integration test for provider generation

use hemmer_provider_generator_common::{
    BlockDefinition, FieldDefinition, FieldType, NestingMode, OperationMapping, Operations,
    Provider, ResourceDefinition, ServiceDefinition,
};
use hemmer_provider_generator_generator::ProviderGenerator;
use tempfile::TempDir;

#[test]
fn test_generate_s3_provider() {
    // Create a simple S3 service definition
    let service_def = ServiceDefinition {
        provider: Provider::Aws,
        name: "s3".to_string(),
        sdk_version: "1.0.0".to_string(),
        data_sources: vec![], // Will implement data source detection later
        resources: vec![ResourceDefinition {
            name: "bucket".to_string(),
            description: Some("S3 Bucket for object storage".to_string()),
            fields: vec![
                FieldDefinition {
                    name: "bucket".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    sensitive: false,
                    immutable: true,
                    description: Some("Bucket name".to_string()),
                    response_accessor: None,
                },
                FieldDefinition {
                    name: "acl".to_string(),
                    field_type: FieldType::String,
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Access control list".to_string()),
                    response_accessor: None,
                },
            ],
            outputs: vec![FieldDefinition {
                name: "arn".to_string(),
                field_type: FieldType::String,
                required: true,
                sensitive: false,
                immutable: true,
                description: Some("Amazon Resource Name".to_string()),
                response_accessor: Some("arn".to_string()),
            }],
            blocks: vec![BlockDefinition {
                name: "lifecycle_rule".to_string(),
                description: Some("Lifecycle configuration rule".to_string()),
                attributes: vec![
                    FieldDefinition {
                        name: "id".to_string(),
                        field_type: FieldType::String,
                        required: true,
                        sensitive: false,
                        immutable: false,
                        description: Some("Unique identifier for the rule".to_string()),
                        response_accessor: None,
                    },
                    FieldDefinition {
                        name: "enabled".to_string(),
                        field_type: FieldType::Boolean,
                        required: false,
                        sensitive: false,
                        immutable: false,
                        description: Some("Whether the rule is enabled".to_string()),
                        response_accessor: None,
                    },
                    FieldDefinition {
                        name: "prefix".to_string(),
                        field_type: FieldType::String,
                        required: false,
                        sensitive: false,
                        immutable: false,
                        description: Some("Object key prefix filter".to_string()),
                        response_accessor: None,
                    },
                    FieldDefinition {
                        name: "expiration_days".to_string(),
                        field_type: FieldType::Integer,
                        required: false,
                        sensitive: false,
                        immutable: false,
                        description: Some("Number of days until objects expire".to_string()),
                        response_accessor: None,
                    },
                ],
                blocks: vec![BlockDefinition {
                    name: "transition".to_string(),
                    description: Some("Transition actions".to_string()),
                    attributes: vec![
                        FieldDefinition {
                            name: "days".to_string(),
                            field_type: FieldType::Integer,
                            required: false,
                            sensitive: false,
                            immutable: false,
                            description: Some("Days until transition".to_string()),
                            response_accessor: None,
                        },
                        FieldDefinition {
                            name: "storage_class".to_string(),
                            field_type: FieldType::String,
                            required: true,
                            sensitive: false,
                            immutable: false,
                            description: Some("Target storage class".to_string()),
                            response_accessor: None,
                        },
                    ],
                    blocks: vec![], // Could nest even further
                    nesting_mode: NestingMode::List,
                    min_items: 0,
                    max_items: 0,
                    sdk_type_name: Some("Transition".to_string()),
                    sdk_accessor_method: Some("set_transitions".to_string()),
                }],
                nesting_mode: NestingMode::List,
                min_items: 0,
                max_items: 0, // 0 = unlimited
                sdk_type_name: Some("LifecycleRule".to_string()),
                sdk_accessor_method: Some("set_lifecycle_rules".to_string()),
            }],
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
                update: Some(OperationMapping {
                    sdk_operation: "put_bucket_acl".to_string(),
                    additional_operations: vec![],
                }),
                delete: Some(OperationMapping {
                    sdk_operation: "delete_bucket".to_string(),
                    additional_operations: vec![],
                }),
                import: None, // Will implement later
            },
        }],
    };

    // Generate provider to temp directory
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    // Also generate to a permanent location for inspection
    let permanent_dir = std::path::PathBuf::from("/tmp/test-recursive-blocks");
    let _ = std::fs::remove_dir_all(&permanent_dir);
    std::fs::create_dir_all(&permanent_dir).unwrap();

    let generator = ProviderGenerator::new(service_def.clone()).unwrap();
    let result = generator.generate_to_directory(output_path);
    assert!(result.is_ok(), "Generation failed: {:?}", result);

    // Also generate to permanent directory for inspection
    let generator2 = ProviderGenerator::new(service_def).unwrap();
    let _ = generator2.generate_to_directory(&permanent_dir);

    // Check that files were created
    assert!(
        output_path.join("provider.jcf").exists(),
        "provider.jcf should exist"
    );
    assert!(
        output_path.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        output_path.join("README.md").exists(),
        "README.md should exist"
    );
    assert!(
        output_path.join("src/main.rs").exists(),
        "src/main.rs should exist"
    );
    assert!(
        output_path.join("src/lib.rs").exists(),
        "src/lib.rs should exist"
    );
    assert!(
        output_path.join("src/resources/mod.rs").exists(),
        "src/resources/mod.rs should exist"
    );
    assert!(
        output_path.join("src/resources/bucket.rs").exists(),
        "src/resources/bucket.rs should exist"
    );

    // Check provider.jcf content
    let provider_jcf = std::fs::read_to_string(output_path.join("provider.jcf")).unwrap();
    assert!(
        provider_jcf.contains("name: \"s3\""),
        "Should contain provider name"
    );
    assert!(
        provider_jcf.contains("protocol: \"grpc\""),
        "Should specify gRPC protocol"
    );
    assert!(
        provider_jcf.contains("bucket:"),
        "Should contain bucket resource"
    );

    // Check Cargo.toml content
    let cargo_toml = std::fs::read_to_string(output_path.join("Cargo.toml")).unwrap();
    assert!(
        cargo_toml.contains("hemmer-s3-provider"),
        "Should have correct package name"
    );
    assert!(
        cargo_toml.contains("aws-sdk-s3"),
        "Should have AWS SDK dependency"
    );
    assert!(
        cargo_toml.contains("hemmer-provider-sdk = \"0.3.1\""),
        "Should have Hemmer SDK 0.3.1+ dependency for updated ProviderService signatures"
    );
    assert!(
        cargo_toml.contains("[[bin]]"),
        "Should define binary target"
    );

    // Check main.rs content
    let main_rs = std::fs::read_to_string(output_path.join("src/main.rs")).unwrap();
    assert!(
        main_rs.contains("hemmer_provider_sdk::serve"),
        "Should use SDK serve function"
    );
    assert!(
        main_rs.contains("S3Provider"),
        "Should reference provider struct"
    );

    // Check lib.rs content
    let lib_rs = std::fs::read_to_string(output_path.join("src/lib.rs")).unwrap();
    assert!(
        lib_rs.contains("impl ProviderService for"),
        "Should implement ProviderService trait"
    );
    assert!(
        lib_rs.contains("fn schema(&self)"),
        "Should implement schema method"
    );
    assert!(
        lib_rs.contains("async fn create"),
        "Should implement create method"
    );
    assert!(
        lib_rs.contains("PROTOCOL_VERSION"),
        "Should import protocol version constants"
    );
    assert!(
        lib_rs.contains("SDK_PROTOCOL_VERSION"),
        "Should re-export protocol version for consumers"
    );

    // Check nested block generation
    assert!(
        lib_rs.contains("BlockType"),
        "Should import BlockType for nested blocks"
    );
    assert!(
        lib_rs.contains("NestingMode"),
        "Should import NestingMode for nested blocks"
    );
    assert!(
        lib_rs.contains("lifecycle_rule"),
        "Should include lifecycle_rule block"
    );
    assert!(
        lib_rs.contains("NestingMode::List"),
        "Should specify List nesting mode for lifecycle_rule"
    );
    assert!(
        lib_rs.contains("expiration_days"),
        "Should include lifecycle_rule attributes"
    );

    println!("✅ Provider generated successfully to: {:?}", output_path);
}
