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

    // Verify that output fields have response accessors
    // GetBucketOutput has BucketName and CreationDate fields
    let bucket_name_output = bucket.outputs.iter().find(|o| o.name == "bucket_name");
    assert!(
        bucket_name_output.is_some(),
        "Should have bucket_name output from GetBucketOutput"
    );
    assert!(
        bucket_name_output.unwrap().response_accessor.is_some(),
        "Output field should have response_accessor"
    );
    assert_eq!(
        bucket_name_output.unwrap().response_accessor.as_deref(),
        Some("bucket_name"),
        "response_accessor should match field name"
    );

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

#[test]
fn test_parse_smithy_with_nested_blocks() {
    // Smithy model with nested blocks (lifecycle rules)
    let smithy_json = r#"{
        "smithy": "2.0",
        "shapes": {
            "com.example.storage#StorageService": {
                "type": "service",
                "version": "2023-01-01",
                "operations": [
                    { "target": "com.example.storage#PutBucketLifecycle" },
                    { "target": "com.example.storage#GetBucketLifecycle" }
                ]
            },
            "com.example.storage#PutBucketLifecycle": {
                "type": "operation",
                "input": {
                    "target": "com.example.storage#PutBucketLifecycleInput"
                },
                "output": {
                    "target": "com.example.storage#PutBucketLifecycleOutput"
                }
            },
            "com.example.storage#PutBucketLifecycleInput": {
                "type": "structure",
                "members": {
                    "Bucket": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#required": {}
                        }
                    },
                    "LifecycleRules": {
                        "target": "com.example.storage#LifecycleRuleList",
                        "traits": {
                            "smithy.api#documentation": "List of lifecycle rules"
                        }
                    }
                }
            },
            "com.example.storage#PutBucketLifecycleOutput": {
                "type": "structure",
                "members": {}
            },
            "com.example.storage#GetBucketLifecycle": {
                "type": "operation",
                "input": {
                    "target": "com.example.storage#GetBucketLifecycleInput"
                },
                "output": {
                    "target": "com.example.storage#GetBucketLifecycleOutput"
                }
            },
            "com.example.storage#GetBucketLifecycleInput": {
                "type": "structure",
                "members": {
                    "Bucket": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#required": {}
                        }
                    }
                }
            },
            "com.example.storage#GetBucketLifecycleOutput": {
                "type": "structure",
                "members": {
                    "LifecycleRules": {
                        "target": "com.example.storage#LifecycleRuleList"
                    }
                }
            },
            "com.example.storage#LifecycleRuleList": {
                "type": "list",
                "member": {
                    "target": "com.example.storage#LifecycleRule"
                }
            },
            "com.example.storage#LifecycleRule": {
                "type": "structure",
                "members": {
                    "Id": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#required": {},
                            "smithy.api#documentation": "Rule identifier"
                        }
                    },
                    "Status": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#documentation": "Rule status (Enabled or Disabled)"
                        }
                    },
                    "Prefix": {
                        "target": "smithy.api#String",
                        "traits": {
                            "smithy.api#documentation": "Object key prefix"
                        }
                    },
                    "ExpirationDays": {
                        "target": "smithy.api#Integer",
                        "traits": {
                            "smithy.api#documentation": "Days until expiration"
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

    // Find the bucket_lifecycle resource
    let bucket_lifecycle = service_def
        .resources
        .iter()
        .find(|r| r.name == "bucket_lifecycle")
        .expect("Should have bucket_lifecycle resource");

    // Verify the resource has blocks
    assert!(
        !bucket_lifecycle.blocks.is_empty(),
        "Should have detected nested blocks"
    );

    // Find the lifecycle_rules block
    let lifecycle_rules_block = bucket_lifecycle
        .blocks
        .iter()
        .find(|b| b.name == "lifecycle_rules")
        .expect("Should have lifecycle_rules block");

    // Verify SDK metadata extraction
    assert!(
        lifecycle_rules_block.sdk_type_name.is_some(),
        "Should have extracted SDK type name"
    );
    assert_eq!(
        lifecycle_rules_block.sdk_type_name.as_deref(),
        Some("LifecycleRule"),
        "SDK type name should be LifecycleRule"
    );

    assert!(
        lifecycle_rules_block.sdk_accessor_method.is_some(),
        "Should have extracted SDK accessor method"
    );
    assert_eq!(
        lifecycle_rules_block.sdk_accessor_method.as_deref(),
        Some("set_lifecycle_rules"),
        "SDK accessor method should be set_lifecycle_rules"
    );

    // Verify block attributes
    assert!(
        !lifecycle_rules_block.attributes.is_empty(),
        "Block should have attributes"
    );

    // Check for specific attributes
    let id_attr = lifecycle_rules_block
        .attributes
        .iter()
        .find(|a| a.name == "id");
    assert!(id_attr.is_some(), "Should have id attribute");
    assert!(id_attr.unwrap().required, "id should be required");

    let expiration_days_attr = lifecycle_rules_block
        .attributes
        .iter()
        .find(|a| a.name == "expiration_days");
    assert!(
        expiration_days_attr.is_some(),
        "Should have expiration_days attribute"
    );

    println!("✅ Successfully parsed Smithy model with nested blocks!");
    println!("   Service: {}", service_def.name);
    println!("   Resources: {}", service_def.resources.len());
    println!("   Lifecycle rules block:");
    println!(
        "     - SDK type: {:?}",
        lifecycle_rules_block.sdk_type_name
    );
    println!(
        "     - SDK accessor: {:?}",
        lifecycle_rules_block.sdk_accessor_method
    );
    println!(
        "     - Attributes: {}",
        lifecycle_rules_block.attributes.len()
    );
}
