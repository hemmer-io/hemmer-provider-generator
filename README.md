# Hemmer Provider Generator

[![CI](https://github.com/hemmer-io/hemmer-provider-generator/actions/workflows/ci.yml/badge.svg)](https://github.com/hemmer-io/hemmer-provider-generator/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Automatically generate Hemmer infrastructure providers from cloud SDK specifications.**

Transform any cloud provider's official SDK specification into a complete, working Hemmer provider package—no manual coding required.

## ✨ Features

- **Universal Spec Support**: Parse Smithy, OpenAPI, Discovery, and Protobuf specifications
- **Multi-Cloud**: Support for AWS, GCP, Azure, Kubernetes, and gRPC services
- **Auto-Detection**: Automatically detects spec format from file extension and content
- **Complete Generation**: Generates provider.k manifest, Rust code, tests, and documentation
- **Production Ready**: Fully tested (55 tests), clippy-clean, formatted code
- **Zero Manual Coding**: End-to-end automation from spec to provider package

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/hemmer-io/hemmer-provider-generator.git
cd hemmer-provider-generator

# Build the tool
cargo build --release

# Run it
./target/release/hemmer-provider-generator --help
```

### Usage

#### Parse a Spec File (Inspect Without Generating)

```bash
# Auto-detect format
hemmer-provider-generator parse --spec storage-v1.json -v

# Explicit format
hemmer-provider-generator parse \
  --spec service.pb \
  --format protobuf \
  --service myservice
```

#### Generate a Provider Package

```bash
# Generate from GCP Discovery document
hemmer-provider-generator generate \
  --spec storage-v1.json \
  --service storage \
  --output ./providers/gcp-storage

# Generate from AWS Smithy spec
hemmer-provider-generator generate \
  --spec s3-model.json \
  --format smithy \
  --service s3 \
  --output ./providers/aws-s3

# Generate from Kubernetes OpenAPI spec
hemmer-provider-generator generate \
  --spec kubernetes-api.json \
  --service kubernetes \
  --output ./providers/k8s
```

## 📋 Supported Spec Formats

| Format | Cloud Provider(s) | Example Source |
|--------|------------------|----------------|
| **Smithy** | AWS | [aws/api-models-aws](https://github.com/aws/api-models-aws) |
| **OpenAPI 3.0** | Kubernetes, Azure | Kubernetes API, Azure REST specs |
| **Discovery** | Google Cloud | [googleapis.com](https://www.googleapis.com/discovery/v1/apis) |
| **Protobuf** | gRPC Services | Compiled .proto files (FileDescriptorSet) |

## 🏗️ Architecture

```
Spec File → Auto-Detect Format → Parse → ServiceDefinition IR → Generate → Provider Package
```

**Cloud-Agnostic Design**: All parsers output the same intermediate representation (ServiceDefinition), making the generator completely cloud-agnostic.

## 📦 Generated Provider Structure

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

### Example Generated Output

**provider.k**:
```kcl
schema StorageProvider:
    schema Bucket:
        name: str
        location: str?
        storage_class: str?
```

**src/resources/bucket.rs**:
```rust
pub struct Bucket<'a> {
    provider: &'a crate::StorageProvider,
}

impl<'a> Bucket<'a> {
    pub async fn create(&self, name: String, ...) -> Result<String> { }
    pub async fn read(&self, id: &str) -> Result<()> { }
    pub async fn update(&self, id: &str, ...) -> Result<()> { }
    pub async fn delete(&self, id: &str) -> Result<()> { }
}
```

## 🎯 Use Cases

### Generate AWS S3 Provider

```bash
# Download Smithy spec from aws/api-models-aws
curl -O https://raw.githubusercontent.com/aws/api-models-aws/main/s3/2006-03-01/s3-2006-03-01.json

# Generate provider
hemmer-provider-generator generate \
  --spec s3-2006-03-01.json \
  --format smithy \
  --service s3 \
  --output ./providers/aws-s3
```

### Generate GCP Storage Provider

```bash
# Download Discovery document
curl -O https://storage.googleapis.com/$discovery/rest?version=v1

# Generate provider
hemmer-provider-generator generate \
  --spec rest\?version\=v1 \
  --format discovery \
  --service storage \
  --output ./providers/gcp-storage
```

### Generate Kubernetes Provider

```bash
# Download OpenAPI spec from your cluster
kubectl get --raw /openapi/v2 > kubernetes-api.json

# Generate provider
hemmer-provider-generator generate \
  --spec kubernetes-api.json \
  --service kubernetes \
  --output ./providers/k8s
```

## 🔧 Development

### Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace --all-features

# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific parser
cargo test --test smithy_parser_test
cargo test --test openapi_parser_test
cargo test --test discovery_parser_test
cargo test --test protobuf_parser_test

# With output
cargo test -- --nocapture
```

## 📚 Project Structure

This is a Cargo workspace with 4 crates:

- **`common/`** - Shared types (ServiceDefinition IR, FieldType, errors)
- **`parser/`** - Spec format parsers (Smithy, OpenAPI, Discovery, Protobuf)
- **`generator/`** - Code generation engine (Tera templates)
- **`cli/`** - Command-line interface

## 🎓 How It Works

1. **Parse**: Load and parse spec file using appropriate parser
2. **Transform**: Convert to cloud-agnostic ServiceDefinition IR
3. **Generate**: Apply Tera templates to create provider files
4. **Output**: Complete provider package ready to use

### Auto-Detection Logic

The CLI automatically detects spec format from:
- **File extension**: `.pb` → Protobuf
- **Filename patterns**: `smithy-model.json`, `storage-discovery.json`
- **Content markers**:
  - `"smithy"` + `"shapes"` → Smithy
  - `"openapi"` + `"paths"` → OpenAPI
  - `"discoveryVersion"` + `"resources"` → Discovery

## 🧪 Testing

- **55 total tests** across workspace
- **4 integration tests** (one per spec format)
- **Multi-platform CI** (Ubuntu, macOS, Windows)
- **All tests passing** ✅

## 🔗 Related Projects

- **[Hemmer](https://github.com/hemmer-io/hemmer)** - Infrastructure-as-Code tool that uses these providers
- **[KCL](https://www.kcl-lang.io/)** - Configuration language used for provider manifests

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

For development context and architecture details, see [CLAUDE.md](CLAUDE.md).

## 🎯 Status

**Production Ready** ✅

All planned phases (1-5) are complete:
- ✅ Phase 1: Foundation & Planning
- ✅ Phase 2: AWS SDK Parser (Smithy)
- ✅ Phase 3: Generator Core
- ✅ Phase 4: Multi-Cloud Parsers (OpenAPI, Discovery, Protobuf)
- ✅ Phase 5: CLI Interface & Production Readiness

---

**Version**: 0.1.0
**Last Updated**: 2025-10-29
