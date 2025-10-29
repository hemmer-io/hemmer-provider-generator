//! Integration test for Smithy parser

use hemmer_provider_generator_parser::smithy::SmithyParser;

#[test]
fn test_parse_simple_smithy_model() {
    // Minimal Smithy model with a service and simple operation
    let smithy_json = r#"{
        "smithy": "2.0",
        "shapes": {
            "com.example.storage#StorageService": {
                "type": "service",
                "version": "2023-01-01",
                "operations": [
                    { "target": "com.example.storage#CreateBucket" },
                    { "target": "com.example.storage#GetBucket" },
                    { "target": "com.example.storage#DeleteBucket" }
                ],
                "traits": {
                    "smithy.api#documentation": "Simple storage service"
                }
            },
            "com.example.storage#CreateBucket": {
                "type": "operation",
                "input": {
                    "target": "com.example.storage#CreateBucketInput"
                },
                "output": {
                    "target": "com.example.storage#CreateBucketOutput"
                },
                "traits": {
                    "smithy.api#documentation": "Creates a new bucket"
                }
            },
            "com.example.storage#CreateBucketInput": {
                "type": "structure",
                "members": {
                    "BucketName": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#required": {},
                            "smithy.api#documentation": "Name of the bucket"
                        }
                    },
                    "Region": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#documentation": "AWS region"
                        }
                    }
                }
            },
            "com.example.storage#CreateBucketOutput": {
                "type": "structure",
                "members": {
                    "Location": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#documentation": "Bucket location"
                        }
                    }
                }
            },
            "com.example.storage#GetBucket": {
                "type": "operation",
                "input": {
                    "target": "com.example.storage#GetBucketInput"
                },
                "output": {
                    "target": "com.example.storage#GetBucketOutput"
                }
            },
            "com.example.storage#GetBucketInput": {
                "type": "structure",
                "members": {
                    "BucketName": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#required": {}
                        }
                    }
                }
            },
            "com.example.storage#GetBucketOutput": {
                "type": "structure",
                "members": {
                    "BucketName": {
                        "target": "smithy.api#String"
                    },
                    "CreationDate": {
                        "target": "smithy.api#Timestamp"
                    }
                }
            },
            "com.example.storage#DeleteBucket": {
                "type": "operation",
                "input": {
                    "target": "com.example.storage#DeleteBucketInput"
                }
            },
            "com.example.storage#DeleteBucketInput": {
                "type": "structure",
                "members": {
                    "BucketName": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#required": {}
                        }
                    }
                }
            }
        }
    }"#;

    // Parse the Smithy model
    let parser = SmithyParser::from_json(smithy_json, "storage", "2023-01-01").unwrap();
    let service_def = parser.parse().unwrap();

    // Verify service metadata
    assert_eq!(service_def.name, "storage");
    assert_eq!(service_def.sdk_version, "2023-01-01");

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

    // Verify fields from CreateBucket input
    assert!(!bucket.fields.is_empty(), "Should have fields");
    let bucket_name_field = bucket.fields.iter().find(|f| f.name == "bucket_name");
    assert!(bucket_name_field.is_some(), "Should have bucket_name field");
    assert!(
        bucket_name_field.unwrap().required,
        "bucket_name should be required"
    );

    // Verify outputs from GetBucket output
    assert!(!bucket.outputs.is_empty(), "Should have outputs");

    println!("✅ Successfully parsed Smithy model!");
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
