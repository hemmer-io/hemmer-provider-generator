//! Integration test for OpenAPI parser

use hemmer_provider_generator_parser::openapi::{OpenApiParser, ProviderHint};

#[test]
fn test_parse_kubernetes_style_openapi() {
    // Simplified Kubernetes-style OpenAPI spec for Pod resource
    let openapi_json = r##"{
        "openapi": "3.0.0",
        "info": {
            "title": "Kubernetes",
            "version": "v1.27.0"
        },
        "paths": {
            "/api/v1/namespaces/{namespace}/pods": {
                "post": {
                    "operationId": "createNamespacedPod",
                    "description": "Create a Pod",
                    "parameters": [
                        {
                            "name": "namespace",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/Pod"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Created",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/Pod"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/v1/namespaces/{namespace}/pods/{name}": {
                "get": {
                    "operationId": "readNamespacedPod",
                    "description": "Read a Pod",
                    "parameters": [
                        {
                            "name": "namespace",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "OK",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/Pod"
                                    }
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "operationId": "deleteNamespacedPod",
                    "description": "Delete a Pod",
                    "parameters": [
                        {
                            "name": "namespace",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "OK"
                        }
                    }
                },
                "patch": {
                    "operationId": "patchNamespacedPod",
                    "description": "Patch a Pod",
                    "parameters": [
                        {
                            "name": "namespace",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "name",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "OK"
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "Pod": {
                    "type": "object",
                    "properties": {
                        "apiVersion": {
                            "type": "string",
                            "description": "API version"
                        },
                        "kind": {
                            "type": "string",
                            "description": "Resource kind"
                        },
                        "metadata": {
                            "$ref": "#/components/schemas/ObjectMeta"
                        },
                        "spec": {
                            "$ref": "#/components/schemas/PodSpec"
                        },
                        "status": {
                            "$ref": "#/components/schemas/PodStatus"
                        }
                    },
                    "required": ["apiVersion", "kind"]
                },
                "ObjectMeta": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the resource"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Namespace"
                        },
                        "labels": {
                            "type": "object",
                            "additionalProperties": {
                                "type": "string"
                            }
                        }
                    }
                },
                "PodSpec": {
                    "type": "object",
                    "properties": {
                        "containers": {
                            "type": "array",
                            "items": {
                                "$ref": "#/components/schemas/Container"
                            }
                        }
                    },
                    "required": ["containers"]
                },
                "Container": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string"
                        },
                        "image": {
                            "type": "string"
                        }
                    },
                    "required": ["name", "image"]
                },
                "PodStatus": {
                    "type": "object",
                    "properties": {
                        "phase": {
                            "type": "string"
                        },
                        "podIP": {
                            "type": "string"
                        }
                    }
                }
            }
        }
    }"##;

    // Parse the OpenAPI spec
    let parser = OpenApiParser::from_json(openapi_json, "kubernetes", "1.27.0")
        .unwrap()
        .with_provider_hint(ProviderHint::Kubernetes);

    let service_def = parser.parse().unwrap();

    // Verify service metadata
    assert_eq!(service_def.name, "kubernetes");
    assert_eq!(service_def.sdk_version, "1.27.0");

    // Should have extracted Pod resource
    assert!(
        !service_def.resources.is_empty(),
        "Should have extracted resources"
    );

    // Find the pod resource
    let pod = service_def
        .resources
        .iter()
        .find(|r| r.name == "pod")
        .expect("Should have pod resource");

    // Verify operations
    assert!(
        pod.operations.create.is_some(),
        "Should have create operation"
    );
    assert!(pod.operations.read.is_some(), "Should have read operation");
    assert!(
        pod.operations.update.is_some(),
        "Should have update operation"
    );
    assert!(
        pod.operations.delete.is_some(),
        "Should have delete operation"
    );

    // Verify operation IDs
    if let Some(ref create_op) = pod.operations.create {
        assert_eq!(create_op.sdk_operation, "create_namespaced_pod");
    }

    println!("✅ Successfully parsed OpenAPI spec!");
    println!("   Service: {}", service_def.name);
    println!("   Resources: {}", service_def.resources.len());
    println!(
        "   Pod operations: create={}, read={}, update={}, delete={}",
        pod.operations.create.is_some(),
        pod.operations.read.is_some(),
        pod.operations.update.is_some(),
        pod.operations.delete.is_some()
    );
}
