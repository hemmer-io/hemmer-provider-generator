# hemmer-provider-generator-common

[![crates.io](https://img.shields.io/crates/v/hemmer-provider-generator-common.svg)](https://crates.io/crates/hemmer-provider-generator-common)
[![Documentation](https://docs.rs/hemmer-provider-generator-common/badge.svg)](https://docs.rs/hemmer-provider-generator-common)

Shared types and data structures for the Hemmer Provider Generator.

## Overview

This crate provides the core intermediate representation (IR) used across all components of the Hemmer Provider Generator. It defines cloud-agnostic types that represent service definitions, resources, fields, and operations.

## Key Types

- `ServiceDefinition` - Complete service specification with resources and operations
- `ResourceDefinition` - Individual resource with fields, outputs, and CRUD operations
- `FieldDefinition` - Resource field with name, type, and optionality
- `FieldType` - Universal type system (String, Integer, Float, Boolean, DateTime, List, Map)
- `Provider` - Enum representing cloud providers (AWS, GCP, Azure, Kubernetes)
- `Operations` - CRUD operation mappings (Create, Read, Update, Delete)

## Usage

This crate is typically used as a dependency by parser and generator crates:

```rust
use hemmer_provider_generator_common::{
    ServiceDefinition, ResourceDefinition, FieldDefinition, FieldType, Provider
};

// Create a service definition
let service = ServiceDefinition {
    provider: Provider::Aws,
    name: "s3".to_string(),
    sdk_version: "1.0.0".to_string(),
    resources: vec![
        ResourceDefinition {
            name: "Bucket".to_string(),
            description: Some("S3 bucket resource".to_string()),
            fields: vec![
                FieldDefinition {
                    name: "name".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    description: Some("Bucket name".to_string()),
                }
            ],
            outputs: vec![],
            operations: Default::default(),
        }
    ],
};
```

## Features

- Cloud-agnostic intermediate representation
- Comprehensive error types with `thiserror`
- Serialization support with `serde`
- Zero dependencies beyond error handling and serialization

## Documentation

For detailed API documentation, see [docs.rs/hemmer-provider-generator-common](https://docs.rs/hemmer-provider-generator-common).

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.
