//! Integration test for provider generation

use hemmer_provider_generator_common::{
    FieldDefinition, FieldType, OperationMapping, Operations, Provider, ResourceDefinition,
    ServiceDefinition,
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
            },
        }],
    };

    // Generate provider to temp directory
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path();

    let generator = ProviderGenerator::new(service_def).unwrap();
    let result = generator.generate_to_directory(output_path);

    assert!(result.is_ok(), "Generation failed: {:?}", result);

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
        cargo_toml.contains("hemmer-provider-sdk = \"0.2\""),
        "Should have Hemmer SDK 0.2+ dependency for protocol versioning"
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

    println!("✅ Provider generated successfully to: {:?}", output_path);
}
