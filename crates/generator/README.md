# hemmer-provider-generator-generator

[![crates.io](https://img.shields.io/crates/v/hemmer-provider-generator-generator.svg)](https://crates.io/crates/hemmer-provider-generator-generator)
[![Documentation](https://docs.rs/hemmer-provider-generator-generator/badge.svg)](https://docs.rs/hemmer-provider-generator-generator)

Code generation engine for Hemmer infrastructure providers.

## Overview

This crate transforms cloud-agnostic service definitions into complete, working Hemmer provider packages. It uses the Tera template engine to generate KCL schemas, Rust code, tests, and documentation.

## Features

- **Template-Based Generation**: Uses Tera (Jinja2-like) templates for flexible code generation
- **Complete Packages**: Generates provider.k, Cargo.toml, lib.rs, resource modules, and README
- **Type Mapping**: Converts universal `FieldType` to Rust and KCL types
- **Custom Filters**: Provides `kcl_type`, `rust_type`, and `capitalize` template filters
- **Production Ready**: Generated code is clippy-clean and properly formatted

## Usage

```rust
use hemmer_provider_generator_generator::ProviderGenerator;
use hemmer_provider_generator_common::{ServiceDefinition, Provider};
use std::path::PathBuf;

// Create a service definition (typically from a parser)
let service_def = ServiceDefinition {
    provider: Provider::Aws,
    name: "s3".to_string(),
    sdk_version: "1.0.0".to_string(),
    resources: vec![/* resources */],
};

// Generate provider package
let generator = ProviderGenerator::new();
let output_dir = PathBuf::from("./provider-s3");
generator.generate(&service_def, &output_dir)?;
```

## Generated Structure

```
provider-{service}/
├── Cargo.toml                    # Package manifest with SDK dependencies
├── README.md                     # Auto-generated documentation
├── provider.k                    # KCL manifest with resource schemas
└── src/
    ├── lib.rs                   # Provider struct and resource accessors
    └── resources/
        ├── mod.rs               # Resource exports
        └── {resource}.rs        # Individual resource implementations
```

## Templates

The crate includes 6 Tera templates:

- `provider.k.tera` - KCL schema definitions
- `Cargo.toml.tera` - Package manifest
- `lib.rs.tera` - Provider struct and accessors
- `resource.rs.tera` - Individual resource implementations
- `resources_mod.rs.tera` - Resource module exports
- `README.md.tera` - Provider documentation

## Type Mapping

| FieldType | Rust Type | KCL Type |
|-----------|-----------|----------|
| String | `String` | `str` |
| Integer | `i64` | `int` |
| Float | `f64` | `float` |
| Boolean | `bool` | `bool` |
| DateTime | `String` | `str` |
| List(T) | `Vec<T>` | `[T]` |
| Map(K,V) | `HashMap<K,V>` | `{K:V}` |

## Custom Filters

```rust
// In templates:
{{ field.field_type | kcl_type }}    // Convert to KCL type
{{ field.field_type | rust_type }}   // Convert to Rust type
{{ resource.name | capitalize }}     // Capitalize string
```

## Documentation

For detailed API documentation, see [docs.rs/hemmer-provider-generator-generator](https://docs.rs/hemmer-provider-generator-generator).

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.
