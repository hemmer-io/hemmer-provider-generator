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
                },
                FieldDefinition {
                    name: "acl".to_string(),
                    field_type: FieldType::String,
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Access control list".to_string()),
                },
            ],
            outputs: vec![FieldDefinition {
                name: "arn".to_string(),
                field_type: FieldType::String,
                required: true,
                sensitive: false,
                immutable: true,
                description: Some("Amazon Resource Name".to_string()),
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
    assert!(output_path.join("provider.k").exists());
    assert!(output_path.join("Cargo.toml").exists());
    assert!(output_path.join("README.md").exists());
    assert!(output_path.join("src/lib.rs").exists());
    assert!(output_path.join("src/resources/mod.rs").exists());
    assert!(output_path.join("src/resources/bucket.rs").exists());

    // Check provider.k content
    let provider_k = std::fs::read_to_string(output_path.join("provider.k")).unwrap();
    assert!(provider_k.contains("S3Provider"));
    assert!(provider_k.contains("Bucket"));
    assert!(provider_k.contains("bucket: str"));

    // Check Cargo.toml content
    let cargo_toml = std::fs::read_to_string(output_path.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("hemmer-s3-provider"));
    assert!(cargo_toml.contains("aws-sdk-s3"));

    println!("✅ Provider generated successfully to: {:?}", output_path);
}
