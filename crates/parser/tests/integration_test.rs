//! Integration tests for rustdoc JSON parsing

use hemmer_provider_generator_parser::RustdocLoader;
use std::path::PathBuf;

#[test]
fn test_load_rustdoc_json() {
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test_example.json");

    let result = RustdocLoader::load_from_file(&json_path);
    assert!(result.is_ok(), "Failed to load rustdoc JSON: {:?}", result);

    let crate_data = result.unwrap();
    assert_eq!(crate_data.crate_version, Some("0.1.0".to_string()));
}

#[test]
fn test_extract_struct_fields() {
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test_example.json");

    let crate_data = RustdocLoader::load_from_file(&json_path).unwrap();

    // Test extracting CreateBucketInput fields
    let input_fields = RustdocLoader::extract_struct_fields(&crate_data, "CreateBucketInput");

    assert_eq!(input_fields.len(), 2, "Expected 2 input fields");

    // Check bucket field
    let bucket_field = input_fields.iter().find(|f| f.name == "bucket");
    assert!(bucket_field.is_some(), "bucket field not found");
    let bucket_field = bucket_field.unwrap();
    assert_eq!(
        bucket_field.field_type,
        hemmer_provider_generator_common::FieldType::String
    );
    assert!(bucket_field.required, "bucket should be required");
    assert_eq!(
        bucket_field.description,
        Some("Name of the bucket".to_string())
    );

    // Check acl field
    let acl_field = input_fields.iter().find(|f| f.name == "acl");
    assert!(acl_field.is_some(), "acl field not found");
    let acl_field = acl_field.unwrap();
    assert_eq!(
        acl_field.field_type,
        hemmer_provider_generator_common::FieldType::String
    );
    assert!(!acl_field.required, "acl should be optional");
    assert_eq!(
        acl_field.description,
        Some("Access control list".to_string())
    );
}

#[test]
fn test_extract_output_fields() {
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test_example.json");

    let crate_data = RustdocLoader::load_from_file(&json_path).unwrap();

    // Test extracting CreateBucketOutput fields
    let output_fields = RustdocLoader::extract_struct_fields(&crate_data, "CreateBucketOutput");

    assert_eq!(output_fields.len(), 1, "Expected 1 output field");

    // Check location field
    let location_field = output_fields.iter().find(|f| f.name == "location");
    assert!(location_field.is_some(), "location field not found");
    let location_field = location_field.unwrap();
    assert_eq!(
        location_field.field_type,
        hemmer_provider_generator_common::FieldType::String
    );
    assert!(
        !location_field.required,
        "location should be optional (Option<String>)"
    );
    assert_eq!(
        location_field.description,
        Some("Location of the created bucket".to_string())
    );
}

#[test]
fn test_nonexistent_struct() {
    let json_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test_example.json");

    let crate_data = RustdocLoader::load_from_file(&json_path).unwrap();

    let fields = RustdocLoader::extract_struct_fields(&crate_data, "NonExistentStruct");
    assert_eq!(
        fields.len(),
        0,
        "Should return empty vec for non-existent struct"
    );
}
