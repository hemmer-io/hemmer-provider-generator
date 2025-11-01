# Claude AI Context - Hemmer Provider Generator

This document provides comprehensive context for Claude AI when working on this project.

## Project Overview

**Hemmer Provider Generator** is a production-ready code generation tool that automatically creates Hemmer infrastructure providers from cloud provider SDK specifications. It eliminates manual provider development by parsing official SDK specs (Smithy, OpenAPI, Discovery, Protobuf) and generating complete provider packages.

### Goals ✅ ACHIEVED
- **Automation**: Generate 100% of provider boilerplate automatically
- **Consistency**: All providers follow identical patterns
- **Speed**: Create a new provider in seconds, not days
- **Maintainability**: Update providers by re-running generation
- **Quality**: Generated code passes clippy and includes comprehensive tests
- **Cloud-Agnostic**: Support any cloud provider with official SDK specifications

## Architecture

The project uses a spec-based approach with universal intermediate representation:

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Hemmer Provider Generator                        │
│                                                                      │
│  ┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐  │
│  │ Spec Parser │────▶│ ServiceDefinition│────▶│    Generator    │  │
│  │             │     │       (IR)       │     │                 │  │
│  │ - Smithy    │     │                  │     │  - Templates    │  │
│  │ - OpenAPI   │     │  Cloud-Agnostic  │     │  - Tera Engine  │  │
│  │ - Discovery │     │  Intermediate    │     │  - Type Maps    │  │
│  │ - Protobuf  │     │  Representation  │     │                 │  │
│  └─────────────┘     └──────────────────┘     └─────────────────┘  │
│                                                         │            │
└─────────────────────────────────────────────────────────┼────────────┘
                                                          ▼
                                                ┌──────────────────┐
                                                │ Provider Package │
                                                │                  │
                                                │  - provider.k    │
                                                │  - Cargo.toml    │
                                                │  - src/lib.rs    │
                                                │  - src/resources/│
                                                │  - README.md     │
                                                └──────────────────┘
```

## Workspace Structure

This is a Cargo workspace with 4 crates:

```
hemmer-provider-generator/
├── Cargo.toml                      # Workspace configuration
├── .github/
│   └── workflows/
│       └── ci.yml                 # GitHub Actions CI/CD
├── .pre-commit-config.yaml        # Pre-commit hooks
├── crates/
│   ├── common/                    # Shared types (IR, errors)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # ServiceDefinition, FieldType, Provider
│   ├── parser/                    # Spec format parsers
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Parser exports
│   │       ├── smithy/           # AWS Smithy parser
│   │       ├── openapi/          # Kubernetes/Azure OpenAPI parser
│   │       ├── discovery/        # GCP Discovery parser
│   │       ├── protobuf/         # gRPC Protobuf parser
│   │       └── tests/            # Integration tests (4)
│   ├── generator/                 # Code generation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs            # ProviderGenerator
│   │   │   └── templates.rs      # Tera setup & filters
│   │   ├── templates/             # Tera templates (6)
│   │   │   ├── provider.k.tera
│   │   │   ├── Cargo.toml.tera
│   │   │   ├── lib.rs.tera
│   │   │   ├── resource.rs.tera
│   │   │   ├── resources_mod.rs.tera
│   │   │   └── README.md.tera
│   │   └── tests/
│   │       └── generation_test.rs
│   └── cli/                       # CLI interface
│       ├── Cargo.toml
│       └── src/
│           └── main.rs           # Parse & generate commands
├── LICENSE                        # Apache 2.0
├── README.md                      # User-facing documentation
└── CONTRIBUTING.md                # Contribution guidelines
```

## Development Phases - ALL 6 PHASES COMPLETE ✅

### Phase 1: Foundation & Planning ✅ COMPLETE
**Status**: Completed (Issue #1, PR #1)

**Deliverables**:
- [x] Rust workspace structure (4 crates)
- [x] GitHub Actions CI pipeline
- [x] Pre-commit hooks configuration
- [x] Basic CLI structure
- [x] Error types and common utilities
- [x] Apache 2.0 license
- [x] Contributing guidelines

**Acceptance Criteria Met**:
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes
- ✅ `cargo clippy` has no warnings
- ✅ CI configuration ready

### Phase 2: AWS SDK Parser Implementation ✅ COMPLETE
**Status**: Completed (Issue #2, PR #8)

**Deliverables**:
- [x] AWS Smithy JSON AST parser
- [x] Resource discovery from operation names
- [x] CRUD operation classification
- [x] Type mapping (Smithy → FieldType)
- [x] Complete shape type system
- [x] Integration tests

**Implementation**: `crates/parser/src/smithy/` (950+ lines)

### Phase 3: Generator Core Implementation ✅ COMPLETE
**Status**: Completed (Issue #3, PR #10)

**Deliverables**:
- [x] Tera template engine integration
- [x] ServiceDefinition → Provider package transformation
- [x] provider.k generation
- [x] Rust code generation
- [x] Cargo.toml generation
- [x] README generation
- [x] Custom Tera filters (kcl_type, rust_type, capitalize)
- [x] Integration tests

**Implementation**: `crates/generator/` (690+ lines + 6 templates)

### Phase 4: Spec Format Parsers ✅ COMPLETE
**Status**: Completed (Issue #11, PR #11)

**Deliverables**:
- [x] Smithy parser (AWS)
- [x] OpenAPI parser (Kubernetes, Azure)
- [x] Discovery parser (GCP)
- [x] Protobuf parser (gRPC)
- [x] Format auto-detection
- [x] Provider::Kubernetes enum variant
- [x] 4 integration tests

**Implementation**: `crates/parser/src/{smithy,openapi,discovery,protobuf}/` (3,683+ lines)

### Phase 5: CLI Interface and Production Readiness ✅ COMPLETE
**Status**: Completed (Issue #5, PR #11)

**Deliverables**:
- [x] `parse` command - inspect specs without generating
- [x] `generate` command - full provider generation
- [x] Format auto-detection from file/content
- [x] Service name inference
- [x] Verbose mode
- [x] Comprehensive help and examples
- [x] End-to-end testing

**Implementation**: `crates/cli/src/main.rs` (388 lines)

### Phase 6: Multi-Service Unified Providers ✅ COMPLETE (v0.2.0)
**Status**: Completed (Issue #16, #22, PR #19, #23)

**Deliverables**:
- [x] ProviderDefinition IR for multi-service providers (PR #19)
- [x] `generate-unified` CLI command with parsing/aggregation (PR #19)
- [x] Recursive directory scanning (--spec-dir flag) (PR #19)
- [x] Service name filtering (--filter flag) (PR #19)
- [x] Auto-detection of spec format from file extensions (.json, .pb) (PR #19)
- [x] Multi-spec parsing and aggregation (PR #19)
- [x] Real-world testing with 400+ AWS services, 18 GCP services, K8s specs (PR #19)
- [x] Cross-platform installation (Linux/macOS/Windows) (PR #19)
- [x] Install scripts (install.sh, install.ps1) (PR #19)
- [x] UnifiedProviderGenerator implementation (PR #23)
- [x] Unified provider templates (6 new .tera files) (PR #23)
- [x] Complete code generation for multi-service providers (PR #23)
- [x] Integration tests for unified generation (PR #23)
- [x] Comprehensive documentation (PR #19, #23)

**Key Features**:
1. **Directory Scanning**: Recursively discovers all spec files in directory tree
2. **Smart Filtering**: Match services by name patterns (e.g., `--filter s3,dynamodb,lambda`)
3. **Large-Scale Support**: Tested with full AWS SDK (406 services), GCP (436 resources), K8s
4. **Cross-Platform**: Works on Linux, macOS, and Windows with platform-specific installers

**Implementation**:
- `crates/common/src/lib.rs` - ProviderDefinition struct
- `crates/cli/src/main.rs` - UnifiedConfig, generate_unified_command, discover_specs (700+ lines)
- `install.sh` - Linux/macOS installation script
- `install.ps1` - Windows PowerShell installation script

**Test Results**:
- **AWS**: 406 services scanned, 8 matched (s3,dynamodb,lambda), 91 resources parsed
- **GCP**: 18 services matched (storage,compute,bigquery), 436 resources parsed
- **Kubernetes**: 2 services (apps,core), 9 resources parsed

**Usage Examples**:
```bash
# Generate unified AWS provider from entire SDK directory
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir /path/to/aws-sdk-models/models/ \
  --filter s3,dynamodb,lambda \
  --output ./provider-aws

# Generate GCP provider with multiple services
hemmer-provider-generator generate-unified \
  --provider gcp \
  --spec-dir /path/to/google-api-go-client/ \
  --filter storage,compute,bigquery \
  --output ./provider-gcp
```

**Implementation Complete**: Full unified provider code generation is now production-ready (v0.2.0).

## Supported Spec Formats

| Format | Provider(s) | Parser Location | Status |
|--------|-------------|-----------------|--------|
| **Smithy** | AWS | `crates/parser/src/smithy/` | ✅ |
| **OpenAPI 3.0** | Kubernetes, Azure | `crates/parser/src/openapi/` | ✅ |
| **Discovery** | GCP (REST APIs) | `crates/parser/src/discovery/` | ✅ |
| **Protobuf** | gRPC services | `crates/parser/src/protobuf/` | ✅ |

## Key Dependencies

### Workspace Dependencies (Cargo.toml)
```toml
clap = "4.5"              # CLI argument parsing
serde = "1.0"             # Serialization
serde_json = "1.0"        # JSON support
tera = "1.20"             # Template engine (Jinja2-like)
anyhow = "1.0"            # Error handling
thiserror = "1.0"         # Error derive macros
colored = "2.1"           # Terminal colors
prost = "0.13"            # Protobuf support
prost-types = "0.13"      # Protobuf type definitions
prost-reflect = "0.14"    # Protobuf reflection
```

## CLI Usage

The CLI has 3 main commands:

1. **parse** - Inspect specs without generating code
2. **generate** - Generate single-service provider
3. **generate-unified** - Generate multi-service provider (NEW in Phase 6)

### Parse Command (Inspect Spec)

```bash
# Auto-detect format and inspect
hemmer-provider-generator parse --spec storage-v1.json -v

# Explicit format specification
hemmer-provider-generator parse \
  --spec service.pb \
  --format protobuf \
  --service myservice
```

**Output**: Service definition summary with resource count and CRUD operations

### Generate Command (Single Service Provider)

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

# Generate from Protobuf FileDescriptorSet
hemmer-provider-generator generate \
  --spec service.pb \
  --format protobuf \
  --service storage \
  --output ./providers/grpc-storage
```

### Generate-Unified Command (Multi-Service Provider)

Generate a single provider with multiple services:

**Option A: Explicit Spec List**
```bash
hemmer-provider-generator generate-unified \
  --provider aws \
  --specs s3.json,dynamodb.json,lambda.json \
  --service-names s3,dynamodb,lambda \
  --output ./provider-aws
```

**Option B: Directory Scanning (Recommended)**
```bash
# Recursively scan directory for all specs
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir /path/to/aws-sdk-models/models/ \
  --filter s3,dynamodb,lambda \
  --output ./provider-aws \
  -v
```

**Real-World Examples**:
```bash
# Complete AWS provider with filtered services
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir ~/aws-sdk-models/models/ \
  --filter s3,dynamodb,lambda,ec2,rds \
  --output ./provider-aws

# GCP provider with storage, compute, bigquery
hemmer-provider-generator generate-unified \
  --provider gcp \
  --spec-dir ~/google-api-go-client/ \
  --filter storage,compute,bigquery \
  --output ./provider-gcp

# Kubernetes provider with apps and core APIs
hemmer-provider-generator generate-unified \
  --provider kubernetes \
  --spec-dir ~/kubernetes/api/openapi-spec/v3/ \
  --filter apps,core \
  --output ./provider-k8s
```

## CLI Features

### Smart Auto-Detection
The CLI automatically detects spec format from:
- **File extension**: `.pb` → Protobuf
- **Filename patterns**: `smithy-model.json`, `storage-discovery.json`
- **Content markers**:
  - `"smithy"` + `"shapes"` → Smithy
  - `"openapi"` + `"paths"` → OpenAPI
  - `"discoveryVersion"` + `"resources"` → Discovery

### Service Name Inference
- Extracts from filename: `storage-v1.json` → `storage`
- Removes version suffixes automatically: `api-v2.json` → `api`
- Manual override with `--service` flag

### Verbose Mode
```bash
hemmer-provider-generator generate --spec storage-v1.json --service storage -v
```
Shows:
- Detected format
- Service name and version
- Resource count with CRUD operations
- Generated file paths
- Field and output counts per resource

## Generated Provider Structure

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

### Example Generated Files

**provider.k**:
```kcl
schema StorageProvider:
    schema Bucket:
        name: str
        location: str?
        storage_class: str?

        # Outputs
        id_output?: str
```

**src/lib.rs**:
```rust
pub struct StorageProvider {
    // Provider implementation
}

impl StorageProvider {
    pub fn bucket(&self) -> resources::Bucket {
        resources::Bucket::new(self)
    }
}
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

## Type System

### ServiceDefinition IR (Intermediate Representation)

```rust
pub struct ServiceDefinition {
    pub provider: Provider,
    pub name: String,
    pub sdk_version: String,
    pub resources: Vec<ResourceDefinition>,
}

pub struct ResourceDefinition {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<FieldDefinition>,
    pub outputs: Vec<FieldDefinition>,
    pub operations: Operations,
}

pub struct Operations {
    pub create: Option<OperationMapping>,
    pub read: Option<OperationMapping>,
    pub update: Option<OperationMapping>,
    pub delete: Option<OperationMapping>,
}

pub enum Provider {
    Aws,
    Gcp,
    Azure,
    Kubernetes,
}

pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
}
```

## Type Mapping

### Universal Type Mapping (All Parsers → FieldType)

| Spec Type | FieldType | Rust Type | KCL Type |
|-----------|-----------|-----------|----------|
| string | String | String | str |
| integer/int32/int64 | Integer | i64 | int |
| float/double | Float | f64 | float |
| boolean/bool | Boolean | bool | bool |
| timestamp/datetime | DateTime | String | str |
| array/list | List(T) | Vec<T> | [T] |
| object/map | Map(K,V) | HashMap<K,V> | {K:V} |

### Operation Mapping

Different parsers use different strategies to identify CRUD operations:

**Smithy** (Operation names):
- CreateBucket, PutBucket → Create
- GetBucket, DescribeBucket → Read
- UpdateBucket, ModifyBucket → Update
- DeleteBucket → Delete

**OpenAPI** (HTTP methods):
- POST → Create
- GET → Read
- PUT, PATCH → Update
- DELETE → Delete

**Discovery** (Method names):
- insert, create → Create
- get, read → Read
- update, patch → Update
- delete → Delete

**Protobuf** (RPC method names):
- CreateBucket, InsertBucket → Create
- GetBucket, DescribeBucket → Read
- UpdateBucket, PatchBucket → Update
- DeleteBucket → Delete

## Testing

### Test Coverage
- **57 total tests** across workspace
- **32 unit tests** in parser crate
- **4 integration tests** (one per spec format)
- **3 generator integration tests** (single-service + 2 unified tests)
- All tests passing ✅

### Running Tests
```bash
# Run all tests
cargo test --workspace --all-features

# Run parser tests only
cargo test --package hemmer-provider-generator-parser

# Run specific integration test
cargo test --test smithy_parser_test

# Run with output
cargo test -- --nocapture
```

## Code Standards

### Rust Style
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (enforced by CI)
- Address all `cargo clippy` warnings (CI runs with `-D warnings`)
- Add doc comments for all public APIs
- Prefer explicit error types with `thiserror`

### Commit Messages
Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat: add Protobuf parser for gRPC services
fix: correct type mapping for nested structures
docs: update README with CLI usage examples
test: add integration test for Discovery parser
chore: update dependencies to latest versions
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`, `perf`, `style`

## CI/CD Pipeline

### GitHub Actions Workflows
Located in `.github/workflows/ci.yml`:

1. **Format Check**: `cargo fmt --check`
2. **Clippy Linting**: `cargo clippy --workspace -- -D warnings`
3. **Tests**: Multi-platform (Ubuntu, macOS, Windows), Rust stable & beta
4. **Security Audit**: `cargo audit` (informational)
5. **Documentation**: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`

All checks must pass before merging PRs.

### Pre-commit Hooks
Located in `.pre-commit-config.yaml`:

- Trailing whitespace removal
- YAML/TOML validation
- `cargo fmt` auto-format
- `cargo clippy` linting
- Conventional commit message validation

```bash
# Install hooks
pip install pre-commit
pre-commit install
pre-commit install --hook-type commit-msg

# Run manually
pre-commit run --all-files
```

## Common Commands

### Development
```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace --all-features

# Run clippy
cargo clippy --workspace --all-features --all-targets -- -D warnings

# Format code
cargo fmt --all

# Build documentation
cargo doc --workspace --all-features --no-deps --open

# Run the CLI
cargo run --bin hemmer-provider-generator -- --help
cargo run --bin hemmer-provider-generator -- parse --spec test.json
cargo run --bin hemmer-provider-generator -- generate --spec test.json --service test --output ./output
```

### Testing Specific Parsers
```bash
# Test Smithy parser
cargo test --test smithy_parser_test -- --nocapture

# Test OpenAPI parser
cargo test --test openapi_parser_test -- --nocapture

# Test Discovery parser
cargo test --test discovery_parser_test -- --nocapture

# Test Protobuf parser
cargo test --test protobuf_parser_test -- --nocapture
```

## Issue Tracking

All issues are tracked on GitHub:

- **Issue #1**: Phase 1 - Project Setup and Foundation (✅ Complete)
- **Issue #2**: Phase 2 - AWS SDK Parser Implementation (✅ Complete)
- **Issue #3**: Phase 3 - Generator Core Implementation (✅ Complete)
- **Issue #4**: Phase 4 - AWS Provider Generation (MVP) (✅ Complete)
- **Issue #5**: Phase 5 - CLI Interface and Production Readiness (✅ Complete)
- **Issue #7**: Phase 2.5 - Parser Trait Abstraction (✅ Complete)
- **Issue #11**: Spec Format Parsers & CLI Implementation (✅ Complete)
- **Issue #16**: Phase 6 - Multi-Service Unified Providers (✅ Complete)

**Common Labels**:
- Phase labels: `phase-1`, `phase-2`, `phase-3`, `phase-4`, `phase-5`, `phase-6`
- Type labels: `feat`, `fix`, `docs`, `test`, `chore`
- Component labels: `aws`, `gcp`, `kubernetes`, `parser`, `generator`, `cli`, `multi-service`

## Related Projects

- **Hemmer**: The main infrastructure-as-code tool ([hemmer-io/hemmer](https://github.com/hemmer-io/hemmer))
- **KCL**: Configuration language used for provider manifests ([kcl-lang.io](https://www.kcl-lang.io/))

## Important Notes for Claude

1. **Current Status**: All 6 phases complete. Project is production-ready with multi-service support.
2. **Code Quality**: All code passes `cargo fmt`, `cargo clippy -D warnings`, and tests
3. **Documentation**: All public APIs have doc comments
4. **Testing**: 55 tests covering all parsers and generators
5. **Conventional Commits**: Always use conventional commit format
6. **Branch Strategy**: Feature branches for new work, PR to main
7. **Cross-Platform**: Supports Linux, macOS, and Windows with platform-specific installers

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [AWS Smithy](https://smithy.io/2.0/) - AWS SDK modeling language
- [OpenAPI Specification](https://spec.openapis.org/oas/v3.0.3) - REST API specs
- [Google Discovery Format](https://developers.google.com/discovery/v1/reference) - GCP API specs
- [Protocol Buffers](https://protobuf.dev/) - gRPC service definitions
- [Tera Templates](https://keats.github.io/tera/) - Template engine documentation

## What's Next

The core tool is complete with all 6 phases (v0.2.0). Potential future enhancements:

- Add more spec format parsers (Terraform schema, Pulumi schema)
- Enhanced template customization options
- Config file support (`.hemmergen.yaml`)
- Provider registry integration with hemmer
- Incremental generation (update existing providers)
- Documentation generation improvements
- Publish to crates.io for easy installation

## Contact & Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

---

**Last Updated**: 2025-11-01 (v0.3.2 - Bug Fixes for Invalid Generated Code)
