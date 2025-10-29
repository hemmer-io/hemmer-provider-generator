# Claude AI Context - Hemmer Provider Generator

This document provides comprehensive context for Claude AI when working on this project.

## Project Overview

**Hemmer Provider Generator** is a code generation tool that automatically creates Hemmer infrastructure providers from cloud provider SDKs (AWS, GCP, Azure). It eliminates the manual work of creating providers and ensures consistency across all Hemmer providers.

### Goals
- **Automation**: Generate 80%+ of provider code automatically
- **Consistency**: All providers follow the same patterns
- **Speed**: Create a new provider in minutes, not days
- **Maintainability**: Easy to update providers when SDKs change
- **Quality**: Generated code passes clippy and includes tests

## Architecture

The project follows a multi-phase approach with a clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                   Provider Generator                        │
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │   Parser     │───▶│  Generator   │───▶│   Output     │ │
│  │              │    │              │    │              │ │
│  │ - AWS SDK    │    │ - Templates  │    │ - provider.k │ │
│  │ - GCP SDK    │    │ - Mappings   │    │ - Rust code  │ │
│  │ - Azure SDK  │    │ - Transforms │    │ - Tests      │ │
│  └──────────────┘    └──────────────┘    └──────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Workspace Structure

This is a Cargo workspace with 4 crates:

```
hemmer-provider-generator/
├── Cargo.toml                    # Workspace configuration
├── .github/
│   └── workflows/
│       └── ci.yml               # GitHub Actions CI pipeline
├── .pre-commit-config.yaml      # Pre-commit hooks
├── crates/
│   ├── common/                  # Shared types and utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs          # GeneratorError, Provider, FieldType
│   ├── parser/                  # SDK parsing (Phase 2)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs          # parse_aws_sdk() - TODO
│   ├── generator/               # Code generation (Phase 3)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs          # generate_provider() - TODO
│   └── cli/                     # Command-line interface
│       ├── Cargo.toml
│       └── src/
│           └── main.rs          # CLI with clap
├── LICENSE                      # Apache 2.0
├── README.md                    # Project documentation
└── CONTRIBUTING.md              # Contribution guidelines
```

## Development Phases

### Phase 1: Foundation & Planning ✅ COMPLETE
**Status**: Completed (Issue #1)

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

### Phase 2: AWS SDK Parser Implementation 🚧 TODO
**Status**: Not started (Issue #2)

**Goal**: Build parser to extract resource definitions from AWS SDK (Smithy models)

**Key Tasks**:
- Research AWS SDK structure (aws-sdk-rust)
- Understand Smithy model format
- Implement Smithy model parser
- Map SDK operations to CRUD (Create, Read, Update, Delete)
- Type mapping (AWS → Rust → KCL)
- Extract documentation strings

**File**: `crates/parser/src/lib.rs`

### Phase 3: Generator Core Implementation 🚧 TODO
**Status**: Not started (Issue #3)

**Goal**: Transform parsed SDK definitions into provider artifacts

**Key Tasks**:
- Setup Tera template engine
- Create templates for provider.k (KCL manifest)
- Create templates for Rust code
- Generate tests
- Generate README and support files

**File**: `crates/generator/src/lib.rs`

### Phase 4: AWS Provider Generation (MVP) 🚧 TODO
**Status**: Not started (Issue #4)

**Goal**: Generate complete, working AWS provider with core resources

**Target Resources**:
- S3 Bucket
- VPC
- Subnet
- Security Group
- EC2 Instance (Basic)

### Phase 5: CLI Interface and Production Readiness 🚧 TODO
**Status**: Not started (Issue #5)

**Goal**: Build user-facing CLI and prepare for production use

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
mockall = "0.13"          # Testing (mocking)
```

## Code Standards

### Rust Style
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (enforced by CI)
- Address all `cargo clippy` warnings
- Add doc comments for public APIs
- Prefer explicit error types

### Commit Messages
Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat: implement AWS SDK Smithy parser
fix: correct type mapping for nested structures
docs: update README with usage examples
test: add integration tests for S3 resource generation
chore: update dependencies to latest versions
```

**Types**: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`, `perf`, `style`

### Testing
- Unit tests: In the same file (`#[cfg(test)] mod tests`)
- Integration tests: In `tests/` directory
- Doc tests: In documentation comments
- Target: >80% code coverage

## CI/CD Pipeline

### GitHub Actions Workflows
Located in `.github/workflows/ci.yml`:

1. **Format Check**: `cargo fmt --check`
2. **Clippy Linting**: `cargo clippy -- -D warnings`
3. **Tests**: Multi-platform (Ubuntu, macOS, Windows), Rust stable & beta
4. **Security Audit**: `cargo audit` (informational)
5. **Documentation**: `cargo doc --no-deps`

### Pre-commit Hooks
Located in `.pre-commit-config.yaml`:

- Trailing whitespace removal
- YAML/TOML validation
- `cargo fmt` auto-format
- `cargo clippy` linting
- Conventional commit message validation

## Type System

### Core Types (crates/common/src/lib.rs)

```rust
// Error type
pub enum GeneratorError {
    Parse(String),
    Generation(String),
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
}

// Provider types
pub enum Provider {
    Aws,
    Gcp,
    Azure,
}

// Field types in intermediate representation
pub enum FieldType {
    String,
    Integer,
    Boolean,
    List(Box<FieldType>),
    Map(Box<FieldType>, Box<FieldType>),
}
```

## Type Mapping Strategy (Phase 2)

### AWS SDK → Rust → KCL

| AWS SDK Type | Rust Type | KCL Type |
|--------------|-----------|----------|
| String       | String    | str      |
| Integer      | i64       | int      |
| Boolean      | bool      | bool     |
| List<T>      | Vec<T>    | [T]      |
| Map<K,V>     | HashMap<K,V> | {K:V} |

### Operation Mapping

| Pattern | CRUD Operation | Example |
|---------|----------------|---------|
| CreateX, PutX | Create | CreateBucket |
| DescribeX, GetX | Read | DescribeBucket |
| UpdateX, ModifyX | Update | UpdateBucket |
| DeleteX, TerminateX | Delete | DeleteBucket |

## CLI Usage (Planned - Phase 5)

```bash
# Generate AWS provider with specific services
hemmer-provider-generator generate aws \
  --services s3,ec2,vpc \
  --output ./provider-aws

# Generate all AWS services
hemmer-provider-generator generate aws --all

# Generate GCP provider
hemmer-provider-generator generate gcp \
  --services storage,compute \
  --output ./provider-gcp
```

## Generated Provider Structure (Target - Phase 4)

```
provider-aws/
├── Cargo.toml
├── README.md
├── provider.k              # Generated KCL manifest
├── src/
│   ├── lib.rs
│   ├── resources/
│   │   ├── s3_bucket.rs    # Generated resource
│   │   ├── vpc.rs
│   │   └── ...
│   └── client.rs           # AWS SDK client wrapper
├── tests/
│   └── integration_tests.rs
└── examples/
    └── basic.k
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

# Check formatting
cargo fmt --all --check

# Build documentation
cargo doc --workspace --all-features --no-deps --open

# Run the CLI
cargo run --bin hemmer-provider-generator -- --help
cargo run --bin hemmer-provider-generator -- generate aws --output ./test-output
```

### Pre-commit Hooks
```bash
# Install hooks
pip install pre-commit
pre-commit install
pre-commit install --hook-type commit-msg

# Run manually
pre-commit run --all-files
```

## Issue Tracking

All issues are tracked on GitHub and linked to the project board:

- **Issue #1**: Phase 1 - Project Setup and Foundation (✅ Complete)
- **Issue #2**: Phase 2 - AWS SDK Parser Implementation
- **Issue #3**: Phase 3 - Generator Core Implementation
- **Issue #4**: Phase 4 - AWS Provider Generation (MVP)
- **Issue #5**: Phase 5 - CLI Interface and Production Readiness

**Labels**:
- Phase labels: `phase-1`, `phase-2`, `phase-3`, `phase-4`, `phase-5`
- Type labels: `feat`, `chore`, `docs`, `testing`
- Component labels: `aws`, `parser`, `generator`

## Related Projects

- **Hemmer**: The main infrastructure-as-code tool ([hemmer-io/hemmer](https://github.com/hemmer-io/hemmer))
- **KCL**: Configuration language used for manifests

## Important Notes for Claude

1. **Current Status**: Phase 1 is complete. We have a working Rust workspace with CI/CD.
2. **Next Steps**: Start Phase 2 (AWS SDK Parser) when ready
3. **Code Quality**: All code must pass `cargo fmt`, `cargo clippy`, and tests
4. **Documentation**: Add doc comments for all public APIs
5. **Testing**: Write tests alongside implementation
6. **Conventional Commits**: Always use conventional commit format
7. **Branch Strategy**: Feature branches for each phase, PR to main

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [Smithy](https://smithy.io/2.0/) - AWS SDK modeling language
- [Tera Templates](https://keats.github.io/tera/)

## Contact & Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

---

**Last Updated**: 2025-10-28 (Phase 1 Complete)
