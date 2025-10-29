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

#### Option 1: Install from crates.io (Recommended)

**All Platforms:**
```bash
cargo install hemmer-provider-generator
```

#### Option 2: Quick Install Script

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/hemmer-io/hemmer-provider-generator/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/hemmer-io/hemmer-provider-generator/main/install.ps1 | iex
```

#### Option 3: Install from GitHub

**All Platforms:**
```bash
cargo install --git https://github.com/hemmer-io/hemmer-provider-generator.git
```

#### Option 4: Build from source

**Linux/macOS:**
```bash
# Clone the repository
git clone https://github.com/hemmer-io/hemmer-provider-generator.git
cd hemmer-provider-generator

# Build and install
cargo install --path crates/cli

# Or just build
cargo build --release
./target/release/hemmer-provider-generator --help
```

**Windows (PowerShell):**
```powershell
# Clone the repository
git clone https://github.com/hemmer-io/hemmer-provider-generator.git
cd hemmer-provider-generator

# Build and install
cargo install --path crates/cli

# Or just build
cargo build --release
.\target\release\hemmer-provider-generator.exe --help
```

### Usage

#### 1. Parse a Spec File (Inspect Without Generating)

```bash
# Auto-detect format
hemmer-provider-generator parse --spec storage-v1.json -v

# Explicit format
hemmer-provider-generator parse \
  --spec service.pb \
  --format protobuf \
  --service myservice
```

**Output**: Service definition summary with resource count and CRUD operations

#### 2. Generate a Single Service Provider

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

**Output**: Complete provider package with provider.k, Cargo.toml, and Rust code

#### 3. Generate Unified Multi-Service Provider (New!)

Generate a single provider package with multiple cloud services:

**Option A: Explicit Spec List**
```bash
hemmer-provider-generator generate-unified \
  --provider aws \
  --specs s3-model.json,dynamodb-model.json,lambda-model.json \
  --service-names s3,dynamodb,lambda \
  --output ./provider-aws
```

**Option B: Directory Scanning (Recommended)**
```bash
# Recursively scans directory for all spec files
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir /path/to/aws-sdk-models/models/ \
  --filter s3,dynamodb,lambda \
  --output ./provider-aws \
  -v
```

**Key Features:**
- 🔍 **Recursive Discovery**: Automatically finds all spec files in directory tree
- 🎯 **Smart Filtering**: `--filter` flag to select specific services by name
- 📦 **Unified Output**: Single provider with multiple services
- 🚀 **Large-Scale**: Tested with 400+ AWS services, 18 GCP services, K8s specs

**Real-World Examples:**

```bash
# Generate complete AWS provider (all services)
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir ~/aws-sdk-models/models/ \
  --output ./provider-aws

# Generate filtered GCP provider (compute + storage + bigquery)
hemmer-provider-generator generate-unified \
  --provider gcp \
  --spec-dir ~/google-api-go-client/ \
  --filter compute,storage,bigquery \
  --output ./provider-gcp

# Generate Kubernetes provider (apps + core APIs)
hemmer-provider-generator generate-unified \
  --provider kubernetes \
  --spec-dir ~/kubernetes/api/openapi-spec/v3/ \
  --filter apps,core \
  --output ./provider-k8s
```

## 📋 Supported Spec Formats

| Format | Cloud Provider(s) | Source Repositories | Status |
|--------|------------------|---------------------|--------|
| **Smithy** | AWS | [aws/api-models-aws](https://github.com/aws/api-models-aws) (406 services) | ✅ Tested |
| **OpenAPI 3.0** | Kubernetes, Azure | [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | ✅ Tested |
| **Discovery** | Google Cloud | [googleapis/google-api-go-client](https://github.com/googleapis/google-api-go-client) (436 resources) | ✅ Tested |
| **Protobuf** | gRPC Services | Compiled .proto files (FileDescriptorSet) | ✅ Supported |

### Getting Spec Files

**AWS (Smithy)**
```bash
git clone https://github.com/aws/api-models-aws.git
cd api-models-aws/models
# Contains 406 service specs (S3, DynamoDB, Lambda, etc.)
```

**Google Cloud (Discovery)**
```bash
git clone https://github.com/googleapis/google-api-go-client.git
cd google-api-go-client
# Contains 436 resources across all GCP services
```

**Kubernetes (OpenAPI)**
```bash
git clone https://github.com/kubernetes/kubernetes.git
cd kubernetes/api/openapi-spec/v3
# Contains OpenAPI specs for all K8s APIs
```

**Protobuf (gRPC)**
```bash
# Compile .proto files to FileDescriptorSet
protoc --descriptor_set_out=service.pb \
  --include_imports \
  service.proto
```

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

## 🎯 Real-World Examples

### Example 1: Complete AWS Provider (406 Services)

```bash
# Clone AWS SDK specs
git clone --depth 1 https://github.com/aws/api-models-aws.git /tmp/aws-sdk

# Generate unified AWS provider with specific services
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir /tmp/aws-sdk/models/ \
  --filter s3,dynamodb,lambda,ec2,rds,sqs,sns \
  --output ./provider-aws \
  -v

# Result: Single provider-aws package with 7 services
```

### Example 2: Google Cloud Provider (Storage, Compute, BigQuery)

```bash
# Clone Google API specs
git clone --depth 1 https://github.com/googleapis/google-api-go-client.git /tmp/gcp-api

# Generate unified GCP provider
hemmer-provider-generator generate-unified \
  --provider gcp \
  --spec-dir /tmp/gcp-api/ \
  --filter storage,compute,bigquery \
  --output ./provider-gcp \
  -v

# Result: 18 services matched, 436 resources parsed
```

### Example 3: Kubernetes Provider (Core + Apps APIs)

```bash
# Clone Kubernetes specs
git clone --depth 1 https://github.com/kubernetes/kubernetes.git /tmp/k8s

# Generate unified Kubernetes provider
hemmer-provider-generator generate-unified \
  --provider kubernetes \
  --spec-dir /tmp/k8s/api/openapi-spec/v3/ \
  --filter apps,core \
  --output ./provider-k8s \
  -v

# Result: 2 services, 9 resources
```

### Example 4: Single Service Provider (AWS S3 Only)

```bash
# Download single Smithy spec
curl -o s3.json https://raw.githubusercontent.com/aws/api-models-aws/main/models/s3.json

# Generate single-service provider
hemmer-provider-generator generate \
  --spec s3.json \
  --format smithy \
  --service s3 \
  --output ./providers/aws-s3

# Result: provider-s3 package with 38 resources
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

### Single Service Generation

1. **Parse**: Load and parse spec file using appropriate parser
2. **Transform**: Convert to cloud-agnostic ServiceDefinition IR
3. **Generate**: Apply Tera templates to create provider files
4. **Output**: Complete provider package ready to use

### Multi-Service Generation (Unified Provider)

1. **Discover**: Recursively scan directory for `.json` and `.pb` files
2. **Filter**: Match service names against `--filter` patterns
3. **Parse**: Parse all discovered specs into ServiceDefinitions
4. **Aggregate**: Combine services into single ProviderDefinition
5. **Generate**: Create unified provider package with all services (🚧 in progress)

### Auto-Detection Logic

The CLI automatically detects spec format from:
- **File extension**: `.pb` → Protobuf, `.json` → Parse content
- **Filename patterns**: `smithy-model.json`, `storage-discovery.json`, `*openapi*.json`
- **Content markers**:
  - `"smithy"` + `"shapes"` → Smithy
  - `"openapi"` + `"paths"` → OpenAPI
  - `"discoveryVersion"` + `"resources"` → Discovery

### Service Name Filtering

When using `--filter`, service names are matched against spec filenames:
- `--filter s3` matches: `s3.json`, `s3-2006-03-01.json`, `s3control.json`
- `--filter storage` matches: `storage.json`, `storage-v1-api.json`, `storagetransfer.json`
- Multiple filters: `--filter s3,dynamodb,lambda` matches any of the three

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
- 🚧 Phase 6: Unified Multi-Service Providers (in progress - [#16](https://github.com/hemmer-io/hemmer-provider-generator/issues/16))

### Feature Status

| Feature | Status | Notes |
|---------|--------|-------|
| Single-service generation | ✅ Complete | Fully tested |
| Smithy parser | ✅ Complete | 406 AWS services |
| OpenAPI parser | ✅ Complete | Kubernetes, Azure |
| Discovery parser | ✅ Complete | 436 GCP resources |
| Protobuf parser | ✅ Complete | gRPC services |
| Directory scanning | ✅ Complete | Recursive discovery |
| Service filtering | ✅ Complete | Pattern matching |
| Unified generation | 🚧 In Progress | Preview mode ([PR #19](https://github.com/hemmer-io/hemmer-provider-generator/pull/19)) |

## 📊 Test Results

**Tested with real SDK repositories:**
- **AWS**: 406 services scanned, 91 resources parsed (S3, DynamoDB, Lambda)
- **GCP**: 18 services matched, 436 resources parsed (Storage, Compute, BigQuery)
- **Kubernetes**: 2 services, 9 resources parsed (Apps, Core APIs)

---

**Version**: 0.1.1
**Last Updated**: 2025-10-29
