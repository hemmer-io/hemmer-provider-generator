//! Integration test for Protobuf parser

use hemmer_provider_generator_parser::ProtobufParser;
use prost::Message;
use prost_types::{
    field_descriptor_proto, DescriptorProto, FieldDescriptorProto, FileDescriptorProto,
    FileDescriptorSet, MethodDescriptorProto, ServiceDescriptorProto,
};

/// Create a sample FileDescriptorSet with a Storage service
fn create_sample_storage_service() -> FileDescriptorSet {
    // Define Bucket message
    let bucket_message = DescriptorProto {
        name: Some("Bucket".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("name".to_string()),
                number: Some(1),
                label: Some(field_descriptor_proto::Label::Required as i32),
                r#type: Some(field_descriptor_proto::Type::String as i32),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("location".to_string()),
                number: Some(2),
                label: Some(field_descriptor_proto::Label::Optional as i32),
                r#type: Some(field_descriptor_proto::Type::String as i32),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("storage_class".to_string()),
                number: Some(3),
                label: Some(field_descriptor_proto::Label::Optional as i32),
                r#type: Some(field_descriptor_proto::Type::String as i32),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // Define CreateBucketRequest message
    let create_bucket_request = DescriptorProto {
        name: Some("CreateBucketRequest".to_string()),
        field: vec![
            FieldDescriptorProto {
                name: Some("project".to_string()),
                number: Some(1),
                label: Some(field_descriptor_proto::Label::Required as i32),
                r#type: Some(field_descriptor_proto::Type::String as i32),
                ..Default::default()
            },
            FieldDescriptorProto {
                name: Some("bucket".to_string()),
                number: Some(2),
                label: Some(field_descriptor_proto::Label::Required as i32),
                r#type: Some(field_descriptor_proto::Type::Message as i32),
                type_name: Some(".storage.Bucket".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // Define GetBucketRequest message
    let get_bucket_request = DescriptorProto {
        name: Some("GetBucketRequest".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("name".to_string()),
            number: Some(1),
            label: Some(field_descriptor_proto::Label::Required as i32),
            r#type: Some(field_descriptor_proto::Type::String as i32),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Define DeleteBucketRequest message
    let delete_bucket_request = DescriptorProto {
        name: Some("DeleteBucketRequest".to_string()),
        field: vec![FieldDescriptorProto {
            name: Some("name".to_string()),
            number: Some(1),
            label: Some(field_descriptor_proto::Label::Required as i32),
            r#type: Some(field_descriptor_proto::Type::String as i32),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Define Empty message (for delete response)
    let empty_message = DescriptorProto {
        name: Some("Empty".to_string()),
        field: vec![],
        ..Default::default()
    };

    // Define Storage service
    let storage_service = ServiceDescriptorProto {
        name: Some("Storage".to_string()),
        method: vec![
            MethodDescriptorProto {
                name: Some("CreateBucket".to_string()),
                input_type: Some(".storage.CreateBucketRequest".to_string()),
                output_type: Some(".storage.Bucket".to_string()),
                ..Default::default()
            },
            MethodDescriptorProto {
                name: Some("GetBucket".to_string()),
                input_type: Some(".storage.GetBucketRequest".to_string()),
                output_type: Some(".storage.Bucket".to_string()),
                ..Default::default()
            },
            MethodDescriptorProto {
                name: Some("DeleteBucket".to_string()),
                input_type: Some(".storage.DeleteBucketRequest".to_string()),
                output_type: Some(".storage.Empty".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    // Create file descriptor
    let file_descriptor = FileDescriptorProto {
        name: Some("storage.proto".to_string()),
        package: Some("storage".to_string()),
        message_type: vec![
            bucket_message,
            create_bucket_request,
            get_bucket_request,
            delete_bucket_request,
            empty_message,
        ],
        service: vec![storage_service],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    };

    FileDescriptorSet {
        file: vec![file_descriptor],
    }
}

#[test]
fn test_parse_protobuf_storage_service() {
    // Create sample FileDescriptorSet
    let file_descriptor_set = create_sample_storage_service();
    let bytes = file_descriptor_set.encode_to_vec();

    // Parse the FileDescriptorSet
    let parser = ProtobufParser::from_file_descriptor_set(&bytes, "storage", "v1").unwrap();
    let service_def = parser.parse().unwrap();

    // Verify service metadata
    assert_eq!(service_def.name, "storage");
    assert_eq!(service_def.sdk_version, "v1");

    // Should have extracted Bucket resource
    assert!(
        !service_def.resources.is_empty(),
        "Should have extracted resources"
    );

    // Find the bucket resource
    let bucket = service_def
        .resources
        .iter()
        .find(|r| r.name == "bucket")
        .expect("Should have bucket resource");

    // Verify operations
    assert!(
        bucket.operations.create.is_some(),
        "Should have create operation"
    );
    assert!(
        bucket.operations.read.is_some(),
        "Should have read operation"
    );
    assert!(
        bucket.operations.delete.is_some(),
        "Should have delete operation"
    );

    // Verify operation mappings
    if let Some(ref create_op) = bucket.operations.create {
        assert_eq!(create_op.sdk_operation, "create_bucket");
    }

    if let Some(ref read_op) = bucket.operations.read {
        assert_eq!(read_op.sdk_operation, "get_bucket");
    }

    if let Some(ref delete_op) = bucket.operations.delete {
        assert_eq!(delete_op.sdk_operation, "delete_bucket");
    }

    // Verify we extracted fields from the Bucket message
    assert!(!bucket.fields.is_empty(), "Should have fields");

    // Check for specific fields
    let field_names: Vec<&str> = bucket.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(
        field_names.contains(&"project"),
        "Should have project field"
    );

    println!("✅ Successfully parsed Protobuf FileDescriptorSet!");
    println!("   Service: {}", service_def.name);
    println!("   Resources: {}", service_def.resources.len());
    println!(
        "   Bucket operations: create={}, read={}, update={}, delete={}",
        bucket.operations.create.is_some(),
        bucket.operations.read.is_some(),
        bucket.operations.update.is_some(),
        bucket.operations.delete.is_some()
    );
}
