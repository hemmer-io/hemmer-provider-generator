//! Integration test for Discovery parser

use hemmer_provider_generator_parser::discovery::DiscoveryParser;

#[test]
fn test_parse_gcs_discovery() {
    // Simplified Google Cloud Storage Discovery document
    let discovery_json = r##"{
        "discoveryVersion": "v1",
        "name": "storage",
        "version": "v1",
        "title": "Cloud Storage JSON API",
        "description": "Stores and retrieves potentially large, immutable data objects.",
        "rootUrl": "https://storage.googleapis.com/",
        "servicePath": "storage/v1/",
        "parameters": {
            "alt": {
                "type": "string",
                "description": "Data format for the response.",
                "default": "json",
                "location": "query"
            }
        },
        "schemas": {
            "Bucket": {
                "id": "Bucket",
                "type": "object",
                "description": "A bucket resource",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "The ID of the bucket"
                    },
                    "name": {
                        "type": "string",
                        "description": "The name of the bucket"
                    },
                    "location": {
                        "type": "string",
                        "description": "The location of the bucket"
                    },
                    "storageClass": {
                        "type": "string",
                        "description": "The storage class of the bucket"
                    }
                },
                "required": ["name"]
            }
        },
        "resources": {
            "buckets": {
                "methods": {
                    "insert": {
                        "id": "storage.buckets.insert",
                        "path": "b",
                        "httpMethod": "POST",
                        "description": "Creates a new bucket",
                        "parameters": {
                            "project": {
                                "type": "string",
                                "description": "A valid API project identifier",
                                "required": true,
                                "location": "query"
                            }
                        },
                        "request": {
                            "$ref": "Bucket"
                        },
                        "response": {
                            "$ref": "Bucket"
                        },
                        "scopes": [
                            "https://www.googleapis.com/auth/devstorage.full_control"
                        ]
                    },
                    "get": {
                        "id": "storage.buckets.get",
                        "path": "b/{bucket}",
                        "httpMethod": "GET",
                        "description": "Returns metadata for the specified bucket",
                        "parameters": {
                            "bucket": {
                                "type": "string",
                                "description": "Name of a bucket",
                                "required": true,
                                "location": "path"
                            }
                        },
                        "response": {
                            "$ref": "Bucket"
                        },
                        "scopes": [
                            "https://www.googleapis.com/auth/devstorage.full_control"
                        ]
                    },
                    "delete": {
                        "id": "storage.buckets.delete",
                        "path": "b/{bucket}",
                        "httpMethod": "DELETE",
                        "description": "Permanently deletes an empty bucket",
                        "parameters": {
                            "bucket": {
                                "type": "string",
                                "description": "Name of a bucket",
                                "required": true,
                                "location": "path"
                            }
                        },
                        "scopes": [
                            "https://www.googleapis.com/auth/devstorage.full_control"
                        ]
                    },
                    "patch": {
                        "id": "storage.buckets.patch",
                        "path": "b/{bucket}",
                        "httpMethod": "PATCH",
                        "description": "Updates a bucket",
                        "parameters": {
                            "bucket": {
                                "type": "string",
                                "description": "Name of a bucket",
                                "required": true,
                                "location": "path"
                            }
                        },
                        "request": {
                            "$ref": "Bucket"
                        },
                        "response": {
                            "$ref": "Bucket"
                        },
                        "scopes": [
                            "https://www.googleapis.com/auth/devstorage.full_control"
                        ]
                    }
                }
            }
        }
    }"##;

    // Parse the Discovery document
    let parser = DiscoveryParser::from_json(discovery_json, "storage", "v1").unwrap();
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
        bucket.operations.update.is_some(),
        "Should have update operation"
    );
    assert!(
        bucket.operations.delete.is_some(),
        "Should have delete operation"
    );

    // Verify operation mappings
    if let Some(ref create_op) = bucket.operations.create {
        assert_eq!(create_op.sdk_operation, "insert");
    }

    if let Some(ref read_op) = bucket.operations.read {
        assert_eq!(read_op.sdk_operation, "get");
    }

    // Verify we extracted fields from the Bucket schema
    assert!(!bucket.fields.is_empty(), "Should have fields");

    println!("✅ Successfully parsed Discovery document!");
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
