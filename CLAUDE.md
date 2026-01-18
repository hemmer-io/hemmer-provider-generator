# Claude AI Context - Hemmer Provider Generator

This document provides comprehensive context for Claude AI when working on this project.

## Project Overview

**Hemmer Provider Generator** is a production-ready code generation tool that automatically creates Hemmer infrastructure providers from cloud provider SDK specifications. It eliminates manual provider development by parsing official SDK specs (Smithy, OpenAPI, Discovery, Protobuf) and generating complete gRPC provider packages.

### Goals
- **Automation**: Generate 100% of provider boilerplate automatically
- **Consistency**: All providers follow identical patterns
- **Speed**: Create a new provider in seconds, not days
- **Maintainability**: Update providers by re-running generation
- **Quality**: Generated code passes clippy and includes comprehensive tests
- **Cloud-Agnostic**: Support any cloud provider with official SDK specifications
- **gRPC Protocol**: All providers use hemmer-provider-sdk for gRPC communication

## GitHub Workflow Guidelines

### Issue Management
- **Labels**: Always apply appropriate labels when creating issues:
  - Type: `feat`, `fix`, `chore`, `docs`, `test`
  - Component: `parser`, `generator`, `cli`, `grpc`
  - Phase: `phase-1` through `phase-7` for phased work
- **Linking**: Reference related issues in descriptions

### Pull Request Workflow
- **Assignee**: Set the PR assignee to the person who requested the work (the user you're working with)
- **Labels**: Copy relevant labels from linked issue
- **Linked Issues**: Use "Closes #XX" in PR description to auto-close issues
- **Branch Naming**: Use conventional prefixes:
  - `feat/description` - New features
  - `fix/description` - Bug fixes
  - `docs/description` - Documentation
  - `chore/description` - Maintenance
- **Reviewer**: Automated via CODEOWNERS; no manual reviewer assignment needed

### Before Creating a PR
1. Ensure all tests pass: `cargo test --workspace --all-features`
2. Run clippy: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
3. Format code: `cargo fmt --all`
4. Update CLAUDE.md if architecture changes significantly

## Architecture

The project uses a spec-based approach with universal intermediate representation:

```
┌──────────────────────────────────────────────────────────────────────┐
│                    Hemmer Provider Generator                         │
│                                                                      │
│  ┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐   │
│  │ Spec Parser │────▶│ ServiceDefinition│────▶│    Generator    │   │
│  │             │     │       (IR)       │     │                 │   │
│  │ - Smithy    │     │                  │     │  - Templates    │   │
│  │ - OpenAPI   │     │  Cloud-Agnostic  │     │  - Tera Engine  │   │
│  │ - Discovery │     │  Intermediate    │     │  - Type Maps    │   │
│  │ - Protobuf  │     │  Representation  │     │  - SDK Filters  │   │
│  └─────────────┘     └──────────────────┘     └─────────────────┘   │
│                                                         │            │
└─────────────────────────────────────────────────────────┼────────────┘
                                                          ▼
                                              ┌────────────────────────┐
                                              │   gRPC Provider Pkg    │
                                              │                        │
                                              │  - provider.jcf        │
                                              │  - Cargo.toml          │
                                              │  - src/main.rs         │
                                              │  - src/lib.rs          │
                                              │  - src/{service}/      │
                                              │  - README.md           │
                                              │  - docs/               │
                                              └────────────────────────┘
```

## Workspace Structure

This is a Cargo workspace with 5 crates:

```
hemmer-provider-generator/
├── Cargo.toml                      # Workspace configuration
├── CLAUDE.md                       # This file - AI context
├── .github/
│   └── workflows/
│       └── ci.yml                  # GitHub Actions CI/CD
├── scripts/
│   ├── pre-commit                  # Pre-commit hook script
│   └── install-hooks.sh            # Hook installation
├── providers/                      # Provider SDK metadata (Phase 2)
│   ├── README.md                   # Schema documentation
│   ├── aws.sdk-metadata.yaml       # AWS configuration
│   ├── gcp.sdk-metadata.yaml       # GCP configuration
│   ├── azure.sdk-metadata.yaml     # Azure configuration
│   └── kubernetes.sdk-metadata.yaml # Kubernetes configuration
├── crates/
│   ├── common/                     # Shared types (IR, errors)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # ServiceDefinition, FieldType, Provider, ProviderSdkConfig
│   │       └── sdk_metadata.rs     # YAML metadata loader (Phase 2)
│   ├── parser/                     # Spec format parsers
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Parser exports
│   │       ├── smithy/             # AWS Smithy parser
│   │       ├── openapi/            # Kubernetes/Azure OpenAPI parser
│   │       ├── discovery/          # GCP Discovery parser
│   │       ├── protobuf/           # gRPC Protobuf parser
│   │       ├── rustdoc_loader.rs   # SDK rustdoc JSON loader
│   │       └── tests/              # Integration tests
│   ├── generator/                  # Code generation
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs              # ProviderGenerator, UnifiedProviderGenerator
│   │   │   └── templates.rs        # Tera setup & 26+ custom filters
│   │   ├── templates/              # Tera templates (21 files)
│   │   └── tests/
│   │       └── generation_test.rs
│   ├── analyzer/                   # SDK analyzer (Phase 3)
│   │   ├── Cargo.toml
│   │   ├── README.md               # Analyzer documentation
│   │   └── src/
│   │       ├── lib.rs              # SdkAnalyzer, AnalysisResult
│   │       ├── analyzer.rs         # Core orchestration
│   │       ├── workspace_detector.rs  # cargo_metadata analysis
│   │       ├── crate_pattern_detector.rs  # Crate naming patterns
│   │       ├── client_detector.rs  # Client type detection (syn)
│   │       ├── config_detector.rs  # Config pattern detection
│   │       ├── error_detector.rs   # Error categorization
│   │       ├── confidence.rs       # Confidence scoring
│   │       └── output.rs           # YAML generation
│   └── cli/                        # CLI interface
│       ├── Cargo.toml
│       └── src/
│           └── main.rs             # parse, generate, generate-unified, analyze-sdk commands
├── LICENSE                         # Apache 2.0
├── README.md                       # User-facing documentation
└── CONTRIBUTING.md                 # Contribution guidelines
```

## gRPC Provider Protocol

Generated providers implement the `ProviderService` trait from `hemmer-provider-sdk`:

### Required Trait Methods
- `schema()` - Return provider and resource schemas
- `configure()` - Initialize SDK clients from config
- `plan()` - Compute resource diffs (prior vs proposed state)
- `create()` / `read()` / `update()` / `delete()` - CRUD operations

### Protocol Versioning
- Providers export `PROTOCOL_VERSION` and `MIN_PROTOCOL_VERSION`
- Protocol negotiation via `check_protocol_version()`
- Current protocol version: 0.2

### JCL Manifest Format (provider.jcf)
```
provider = (
    name: "aws",
    version: "1.0.0",
    protocol: "grpc",

    config: (
        region: (type: "string", optional: true),
        profile: (type: "string", optional: true),
    ),

    services: (
        s3: (
            resources: (
                bucket: (
                    attributes: (...),
                    capabilities: (create: true, read: true, update: true, delete: true),
                ),
            ),
        ),
    ),
)
```

## Generated Provider Structure

### Single-Service Provider
```
provider-{service}/
├── Cargo.toml                    # Package manifest with hemmer-provider-sdk
├── provider.jcf                  # JCL manifest (protocol: "grpc")
├── README.md                     # Auto-generated documentation
└── src/
    ├── main.rs                   # Binary entry point with serve()
    ├── lib.rs                    # ProviderService implementation
    └── resources/
        ├── mod.rs                # Resource exports
        └── {resource}.rs         # CRUD implementations with SDK calls
```

### Unified Multi-Service Provider
```
provider-{name}/
├── Cargo.toml                    # Package manifest with all SDK dependencies
├── provider.jcf                  # Multi-service JCL manifest
├── README.md                     # Auto-generated documentation
├── docs/
│   ├── installation.md           # Installation guide
│   ├── getting-started.md        # Quick start guide
│   └── services/
│       └── {service}.md          # Per-service documentation
├── .github/workflows/
│   └── release.yml               # Cross-platform release workflow
└── src/
    ├── main.rs                   # Binary entry point
    ├── lib.rs                    # ProviderService with per-service clients
    └── {service}/                # Per-service modules
        ├── mod.rs                # Service dispatcher
        └── resources/
            └── {resource}.rs     # CRUD implementations
```

## Template Files

### Single-Service Templates (9 files)
| Template | Purpose |
|----------|---------|
| `main.rs.tera` | Binary entry point with `hemmer_provider_sdk::serve()` |
| `lib.rs.tera` | ProviderService trait implementation |
| `provider.jcf.tera` | JCL manifest with gRPC protocol |
| `provider.k.tera` | KCL schema (legacy, being phased out) |
| `Cargo.toml.tera` | Package manifest with SDK dependencies |
| `resource.rs.tera` | Resource CRUD handlers with SDK calls |
| `resources_mod.rs.tera` | Resource module exports |
| `service_mod.rs.tera` | Service module structure |
| `README.md.tera` | Provider documentation |

### Unified Multi-Service Templates (8 files)
| Template | Purpose |
|----------|---------|
| `unified_main.rs.tera` | Multi-service entry point |
| `unified_lib.rs.tera` | Multi-service ProviderService with per-service clients |
| `unified_provider.jcf.tera` | Multi-service JCL manifest |
| `unified_provider.k.tera` | Multi-service KCL schema (legacy) |
| `unified_Cargo.toml.tera` | Multi-service package manifest |
| `unified_service.rs.tera` | Service dispatcher module |
| `unified_resource.rs.tera` | Resource CRUD with SDK calls |
| `unified_README.md.tera` | Multi-service documentation |

### Documentation Templates (4 files)
| Template | Purpose |
|----------|---------|
| `docs_installation.md.tera` | Installation guide |
| `docs_getting_started.md.tera` | Quick start guide |
| `docs_service.md.tera` | Per-service documentation |
| `release.yml.tera` | GitHub Actions release workflow |

## Custom Tera Filters

### Type Conversion Filters
| Filter | Purpose | Example |
|--------|---------|---------|
| `rust_type` | FieldType → Rust type | `String` → `String` |
| `kcl_type` | FieldType → KCL syntax | `String` → `str` |
| `jcl_type` | FieldType → JCL syntax | `String` → `"string"` |
| `sdk_attr_type` | FieldType → SDK AttributeType | `String` → `AttributeType::String` |

### Provider-Agnostic SDK Filters
| Filter | Purpose |
|--------|---------|
| `client_type` | Generate SDK client type for service (e.g., `aws_sdk_s3::Client`) |
| `sdk_crate_module` | Generate SDK crate name (e.g., `aws-sdk-s3`) |
| `has_config_crate` | Check if provider has a config crate |
| `config_crate` | Get config crate name (e.g., `aws-config`) |
| `uses_shared_client` | Check if provider uses shared client (K8s) |

### String Manipulation Filters
| Filter | Purpose |
|--------|---------|
| `capitalize` | Capitalize first letter |
| `sanitize_identifier_part` | Make valid Rust identifier |
| `sanitize_rust_identifier` | Full Rust identifier sanitization |
| `to_camel_case` | Convert to CamelCase |
| `json_extractor` | Generate JSON value extraction code |

## Development Phases

### Phases 1-6: COMPLETE
- Phase 1: Foundation & Planning
- Phase 2: AWS SDK Parser Implementation
- Phase 3: Generator Core Implementation
- Phase 4: Spec Format Parsers (Smithy, OpenAPI, Discovery, Protobuf)
- Phase 5: CLI Interface and Production Readiness
- Phase 6: Multi-Service Unified Providers

### Phase 3 (New): SDK Analyzer Tool (✅ MVP Complete - 2026-01-18)

**Status**: MVP Complete

**Goal**: Automate 75% of provider metadata generation by analyzing SDK repositories.

**Implementation**: `crates/analyzer/`

The SDK Analyzer automatically generates Phase 2 metadata YAML files by analyzing provider SDK repositories. This eliminates manual provider configuration and reduces provider addition time from ~4 hours to ~30 minutes.

**Key Features**:
- **Workspace Detection**: Uses `cargo_metadata` to analyze Cargo workspace structure
- **Crate Pattern Detection**: Identifies SDK crate naming patterns (e.g., `aws-sdk-{service}`)
- **Client Type Detection**: Parses Rust AST with `syn` to find Client type patterns
- **Config Detection**: Discovers configuration crates and attributes
- **Error Categorization**: Maps error variants to standard categories using heuristics
- **Confidence Scoring**: Provides 0.0-1.0 confidence scores for each field
- **Annotated Output**: Generates YAML with TODO markers for low-confidence fields

**CLI Command**:
```bash
# Analyze SDK repository
hemmer-provider-generator analyze-sdk \
  --sdk-path ~/code/aws-sdk-rust \
  --name aws \
  --output providers/aws.sdk-metadata.yaml
```

**Architecture**:
```
crates/analyzer/
├── src/
│   ├── analyzer.rs               # Core orchestration
│   ├── workspace_detector.rs     # Cargo workspace analysis
│   ├── crate_pattern_detector.rs # Crate naming patterns
│   ├── client_detector.rs        # Client type detection (syn)
│   ├── config_detector.rs        # Config pattern detection
│   ├── error_detector.rs         # Error categorization heuristics
│   ├── confidence.rs             # Confidence scoring (weighted)
│   └── output.rs                 # Annotated YAML generation
```

**Confidence Scoring**:
- Overall = weighted average (crate: 30%, client: 30%, config_crate: 15%, config_attrs: 5%, errors: 20%)
- HIGH (0.8-1.0): Can use as-is
- MEDIUM (0.6-0.8): Verify before use
- LOW (<0.6): Needs manual review

**Test Coverage**: 25 unit tests

**Future Enhancements**:
- [ ] Git repository cloning support (issue #116)
- [ ] Interactive mode for ambiguous fields
- [ ] Incremental metadata updates
- [ ] ML-based pattern detection

### Phase 7: Comprehensive gRPC Providers (In Progress)

**Status**: In Progress

**Tracking Issues**:
- #77 - Generate actual SDK calls instead of placeholder TODOs (✅ Complete)
- #79 - Refactor provider generation to use gRPC protocol and JCL manifests (✅ Complete)
- #80 - Add protocol version support using SDK 0.2 (✅ Complete)
- #82 - Commit untracked gRPC templates
- #83 - Extract computed fields from SDK responses
- #84 - Implement import operation for existing resources
- #85 - Support data sources for read-only resource lookups
- #86 - Support nested block types in resource schemas
- #87 - Provider-agnostic SDK configuration from spec metadata
- #88 - Support streaming operations for watches and logs
- #89 - Generate proper plan diffs with attribute changes
- #90 - Map SDK errors to appropriate gRPC status codes
- #91 - End-to-end integration testing with hemmer-provider-sdk

**Goals**:
- Complete gRPC provider implementation
- Provider-agnostic code generation (eliminate hardcoded provider checks)
- Full SDK integration with proper response handling
- Comprehensive test coverage with hemmer-provider-sdk

## Supported Spec Formats

| Format | Provider(s) | Parser Location | Status |
|--------|-------------|-----------------|--------|
| **Smithy** | AWS | `crates/parser/src/smithy/` | ✅ |
| **OpenAPI 3.0** | Kubernetes, Azure | `crates/parser/src/openapi/` | ✅ |
| **Discovery** | GCP (REST APIs) | `crates/parser/src/discovery/` | ✅ |
| **Protobuf** | gRPC services | `crates/parser/src/protobuf/` | ✅ |

## Provider Configuration (Phase 2)

**Status**: ✅ Complete

Provider SDK configuration is now **externalized to YAML metadata files** in the `providers/` directory. This eliminates hardcoded provider logic in Rust and enables easy addition of new providers.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Provider Enum (lib.rs)                                      │
│ ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│ │   Aws    │  │   Gcp    │  │  Azure   │  │ Custom() │    │
│ └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘    │
│      │             │              │             │          │
└──────┼─────────────┼──────────────┼─────────────┼──────────┘
       │             │              │             │
       ▼             ▼              ▼             ▼
┌────────────────────────────────────────────────────────────┐
│ providers/ directory (YAML metadata files)                │
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────┐ │
│ │ aws.sdk-    │ │ gcp.sdk-    │ │ azure.sdk-  │ │ {name}│ │
│ │ metadata    │ │ metadata    │ │ metadata    │ │ .sdk- │ │
│ │ .yaml       │ │ .yaml       │ │ .yaml       │ │ meta  │ │
│ └─────────────┘ └─────────────┘ └─────────────┘ └──────┘ │
└────────────────────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────────────────────┐
│ ProviderSdkConfig (loaded dynamically)                    │
│ - SDK crate patterns                                       │
│ - Client type patterns                                     │
│ - Configuration code generation                            │
│ - Error categorization (auto-generated from YAML)          │
└────────────────────────────────────────────────────────────┘
```

### Key Components

1. **Provider Enum** (`crates/common/src/lib.rs`):
   - Built-in variants: `Aws`, `Gcp`, `Azure`, `Kubernetes`
   - Dynamic variant: `Custom(String)` for providers with metadata files
   - `Provider::from_name()` - Create provider from string name
   - `Provider::name()` - Get provider identifier
   - `Provider::sdk_config()` - Load configuration from YAML

2. **YAML Metadata Files** (`providers/*.sdk-metadata.yaml`):
   - **Provider info**: Name and display name
   - **SDK config**: Crate patterns, client types, dependencies
   - **Config codegen**: Initialization, loading, client creation snippets
   - **Config attributes**: Provider-specific settings (region, profile, etc.)
   - **Error handling**: Metadata imports and categorization rules

3. **Metadata Loader** (`crates/common/src/sdk_metadata.rs`):
   - `ProviderSdkMetadata::load()` - Parse YAML file
   - `to_provider_config()` - Convert to `ProviderSdkConfig`
   - `ErrorInfo::generate_categorization_function()` - Generate error handling code

### YAML Schema Example

```yaml
version: 1

provider:
  name: aws
  display_name: Amazon Web Services

sdk:
  crate_pattern: "aws-sdk-{service}"
  client_type_pattern: "aws_sdk_{service}::Client"
  config_crate: aws-config
  async_client: true
  region_attr: region
  dependencies:
    - "aws-config = \"1\""
    - "aws-smithy-types = \"1\""

config:
  initialization:
    snippet: "aws_config::from_env()"
    var_name: config_loader

  load:
    snippet: "config_loader.load().await"
    var_name: sdk_config

  client_from_config:
    snippet: "{client_type}::new(&sdk_config)"
    var_name: client

  attributes:
    - name: region
      description: AWS region to use
      required: false
      setter: "config_loader = config_loader.region(...)"
      extractor: "as_str()"

errors:
  metadata_import: "aws_smithy_types::error::metadata::ProvideErrorMetadata"
  categorization:
    not_found:
      - "NotFound"
      - "NoSuch*"        # Prefix wildcard
      - "ResourceNotFoundException"
    permission_denied:
      - "AccessDenied"
      - "*Unauthorized"  # Suffix wildcard
```

### Error Categorization

Error categorization rules in YAML are automatically converted to Rust code:

**YAML Pattern Types:**
- **Exact match**: `"NotFound"` → `c == "NotFound"`
- **Prefix wildcard**: `"NoSuch*"` → `c.starts_with("NoSuch")`
- **Suffix wildcard**: `"*InUse"` → `c.ends_with("InUse")`
- **Contains wildcard**: `"*Limit*"` → `c.contains("Limit")`

**Supported Categories:**
- `not_found` → `ProviderError::NotFound`
- `already_exists` → `ProviderError::AlreadyExists`
- `permission_denied` → `ProviderError::PermissionDenied`
- `validation` → `ProviderError::Validation`
- `failed_precondition` → `ProviderError::FailedPrecondition`
- `resource_exhausted` → `ProviderError::ResourceExhausted`
- `unavailable` → `ProviderError::Unavailable`
- `deadline_exceeded` → `ProviderError::DeadlineExceeded`
- `unimplemented` → `ProviderError::Unimplemented`

### Adding a New Provider

To add a new cloud provider:

1. Create `providers/{name}.sdk-metadata.yaml` following the schema
2. Define SDK patterns, configuration, and error handling
3. The provider becomes available via `Provider::from_name("{name}")`
4. No Rust code changes required!

Example:
```bash
# Create metadata file
cat > providers/digitalocean.sdk-metadata.yaml <<EOF
version: 1
provider:
  name: digitalocean
  display_name: DigitalOcean
sdk:
  crate_pattern: "digitalocean-sdk-{service}"
  ...
EOF

# Provider is now available
cargo run -- generate-unified --provider digitalocean ...
```

### Benefits

- **Zero Rust code changes** to add/modify providers
- **Easy maintenance** - update YAML files instead of Rust
- **Error handling generation** - automatic from categorization rules
- **Type safety** - validation at YAML load time
- **Documentation** - YAML files are self-documenting

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

pub struct ProviderSdkConfig {
    pub sdk_crate_pattern: String,      // e.g., "aws-sdk-{service}"
    pub client_type_pattern: String,    // e.g., "aws_sdk_{service}::Client"
    pub config_crate: Option<String>,   // e.g., Some("aws-config")
    pub async_client: bool,
    pub config_attrs: Vec<ProviderConfigAttr>,
}

pub enum Provider {
    Aws,
    Gcp,
    Azure,
    Kubernetes,
    Custom(String),  // Dynamic provider loaded from YAML metadata
}

pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
    Object,
}
```

## CLI Usage

The CLI has 3 main commands:

### parse - Inspect specs without generating
```bash
hemmer-provider-generator parse --spec storage-v1.json -v
```

### generate - Single-service provider
```bash
hemmer-provider-generator generate \
  --spec s3-model.json \
  --format smithy \
  --service s3 \
  --output ./providers/aws-s3
```

### generate-unified - Multi-service provider
```bash
hemmer-provider-generator generate-unified \
  --provider aws \
  --spec-dir /path/to/aws-sdk-models/models/ \
  --filter s3,dynamodb,lambda \
  --output ./provider-aws
```

## Testing

### Test Coverage
- **75 total tests** across workspace
- **32 unit tests** in parser crate
- **22 unit tests** in common crate
- **5 integration tests** (one per spec format + unified)
- All tests passing ✅

### Running Tests
```bash
# Run all tests
cargo test --workspace --all-features

# Run with output
cargo test -- --nocapture

# Run specific crate tests
cargo test --package hemmer-provider-generator-parser
cargo test --package hemmer-provider-generator-generator
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

## Common Commands

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
```

## Issue Labels

### Type Labels
- `feat` - New feature or capability
- `fix` - Bug fix
- `chore` - Maintenance tasks
- `docs` - Documentation changes
- `test` - Testing related

### Component Labels
- `parser` - Parser implementation
- `generator` - Code generation
- `cli` - CLI interface
- `grpc` - gRPC protocol related
- `sdk-analysis` - SDK metadata analysis

### Phase Labels
- `phase-1` through `phase-7` - Development phases

### Other Labels
- `streaming` - Streaming operations
- `error-handling` - Error handling improvements
- `integration` - Integration testing

## Related Projects

- **Hemmer**: The main infrastructure-as-code tool ([hemmer-io/hemmer](https://github.com/hemmer-io/hemmer))
- **hemmer-provider-sdk**: SDK for building Hemmer providers with gRPC

## Important Notes for Claude

1. **Current Focus**: Phase 7 - Comprehensive gRPC provider implementation
2. **Code Quality**: All code must pass `cargo fmt`, `cargo clippy -D warnings`, and tests
3. **Testing**: Currently 75 tests; add tests for new functionality
4. **Conventional Commits**: Always use conventional commit format
5. **PR Workflow**: Set PR assignee to the user who requested the work
6. **Labels**: Apply appropriate labels to all issues and PRs
7. **Provider-Agnostic**: Use Tera filters instead of hardcoded provider checks in templates

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [AWS Smithy](https://smithy.io/2.0/) - AWS SDK modeling language
- [OpenAPI Specification](https://spec.openapis.org/oas/v3.0.3) - REST API specs
- [Google Discovery Format](https://developers.google.com/discovery/v1/reference) - GCP API specs
- [Protocol Buffers](https://protobuf.dev/) - gRPC service definitions
- [Tera Templates](https://keats.github.io/tera/) - Template engine documentation
- [tonic](https://docs.rs/tonic) - gRPC for Rust

---

**Last Updated**: 2026-01-18 (Phase 2 - SDK Metadata Files Complete, Phase 3 - SDK Analyzer MVP Complete, Phase 7 In Progress)
