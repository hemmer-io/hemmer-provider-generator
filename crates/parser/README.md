# hemmer-provider-generator-parser

[![crates.io](https://img.shields.io/crates/v/hemmer-provider-generator-parser.svg)](https://crates.io/crates/hemmer-provider-generator-parser)
[![Documentation](https://docs.rs/hemmer-provider-generator-parser/badge.svg)](https://docs.rs/hemmer-provider-generator-parser)

Multi-format parsers for cloud SDK specifications (Smithy, OpenAPI, Discovery, Protobuf).

## Overview

This crate provides parsers for four different cloud SDK specification formats, converting them into a unified intermediate representation (`ServiceDefinition`). Each parser extracts resources, operations, and type information from the respective spec format.

## Supported Formats

| Format | Cloud Provider(s) | Parser Module |
|--------|------------------|---------------|
| **Smithy** | AWS | `smithy` |
| **OpenAPI 3.0** | Kubernetes, Azure | `openapi` |
| **Discovery** | Google Cloud | `discovery` |
| **Protobuf** | gRPC Services | `protobuf` |

## Usage

### Parse a Smithy Spec (AWS)

```rust
use hemmer_provider_generator_parser::smithy::SmithyParser;
use std::fs;

let spec_content = fs::read_to_string("s3-model.json")?;
let parser = SmithyParser::new(&spec_content)?;
let service_def = parser.parse("s3")?;

println!("Service: {}", service_def.name);
println!("Resources: {}", service_def.resources.len());
```

### Parse an OpenAPI Spec (Kubernetes)

```rust
use hemmer_provider_generator_parser::openapi::OpenApiParser;
use std::fs;

let spec_content = fs::read_to_string("kubernetes-api.json")?;
let parser = OpenApiParser::new(&spec_content)?;
let service_def = parser.parse("kubernetes")?;
```

### Parse a Discovery Spec (GCP)

```rust
use hemmer_provider_generator_parser::discovery::DiscoveryParser;
use std::fs;

let spec_content = fs::read_to_string("storage-v1.json")?;
let parser = DiscoveryParser::new(&spec_content)?;
let service_def = parser.parse("storage")?;
```

### Parse a Protobuf FileDescriptorSet (gRPC)

```rust
use hemmer_provider_generator_parser::protobuf::ProtobufParser;
use std::fs;

let spec_bytes = fs::read("service.pb")?;
let parser = ProtobufParser::new(&spec_bytes)?;
let service_def = parser.parse("storage")?;
```

## Features

- **Universal IR**: All parsers output the same `ServiceDefinition` type
- **Resource Discovery**: Automatically identifies resources from operations/methods
- **CRUD Mapping**: Maps operations to Create, Read, Update, Delete
- **Type Conversion**: Converts spec-specific types to universal `FieldType`
- **Error Handling**: Comprehensive error types for parsing failures

## Architecture

```
Spec File → Parser → ServiceDefinition (IR) → Generator
```

Each parser implements the same transformation:
1. Parse spec format into native structures
2. Identify resources and operations
3. Map types to universal `FieldType`
4. Build `ServiceDefinition` IR

## Documentation

For detailed API documentation, see [docs.rs/hemmer-provider-generator-parser](https://docs.rs/hemmer-provider-generator-parser).

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.
