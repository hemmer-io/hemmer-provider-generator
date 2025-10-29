# Hemmer Provider Generator

Code generator for automatically creating Hemmer providers from cloud provider SDKs (AWS, GCP, Azure).

## Overview

The Hemmer Provider Generator automates the creation of infrastructure providers by:

1. Parsing cloud provider SDK definitions
2. Generating provider manifests (KCL)
3. Generating provider binary code (Rust)
4. Creating proper release artifacts

This tool eliminates the manual work of creating providers and ensures consistency across all Hemmer providers.

## Goals

- **Automation**: Generate 80%+ of provider code automatically
- **Consistency**: All providers follow the same patterns
- **Speed**: Create a new provider in minutes, not days
- **Maintainability**: Easy to update providers when SDKs change
- **Quality**: Generated code passes clippy and includes tests

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Provider Generator                        │
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │   Parser     │───▶│  Generator   │───▶│   Emitter    │ │
│  │              │    │              │    │              │ │
│  │ - AWS SDK    │    │ - Templates  │    │ - provider.k │ │
│  │ - GCP SDK    │    │ - Mappings   │    │ - Rust code  │ │
│  │ - Azure SDK  │    │ - Transforms │    │ - Tests      │ │
│  └──────────────┘    └──────────────┘    └──────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │  Provider Package │
                    │                   │
                    │  - Manifest       │
                    │  - Binary         │
                    │  - Checksums      │
                    │  - README         │
                    └───────────────────┘
```

## Project Phases

### Phase 1: Foundation & Planning

**Goal**: Establish project structure and design

- [ ] Project setup (Rust workspace, CI/CD)
- [ ] Architecture design document
- [ ] Data model definitions
- [ ] Template system design
- [ ] AWS SDK research (service definitions)

### Phase 2: Parser Implementation

**Goal**: Parse AWS SDK definitions into intermediate representation

- [ ] AWS SDK parser (Smithy models)
- [ ] Service definition loader
- [ ] Type mapping (AWS → Rust)
- [ ] Field extraction
- [ ] Operation discovery (CRUD mapping)
- [ ] Documentation extraction

### Phase 3: Generator Core

**Goal**: Transform parsed data into provider artifacts

- [ ] Template engine integration (Tera/Handlebars)
- [ ] Provider manifest generator (KCL)
- [ ] Rust code generator
- [ ] Test generator
- [ ] README generator
- [ ] Field validation logic

### Phase 4: AWS Provider Support

**Goal**: Generate working AWS provider

- [ ] S3 bucket resource
- [ ] VPC resource
- [ ] Subnet resource
- [ ] Security group resource
- [ ] EC2 instance resource
- [ ] Integration testing
- [ ] Example configurations

### Phase 5: Multi-Cloud & Polish

**Goal**: Support GCP, Azure, and production readiness

- [ ] GCP SDK support
- [ ] Azure SDK support
- [ ] CLI interface
- [ ] Documentation
- [ ] Release automation
- [ ] Performance optimization

## Technology Stack

- **Language**: Rust
- **Template Engine**: Tera or Handlebars
- **Parser**: Custom (Smithy, OpenAPI parsers)
- **CLI**: clap
- **Testing**: Built-in Rust test framework
- **CI/CD**: GitHub Actions

## Generated Provider Structure

```bash

provider-aws/
├── Cargo.toml
├── README.md
├── provider.k              # Generated manifest
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

## Usage (Planned)

```bash
# Generate AWS provider
hemmer-provider-generator aws \
  --services s3,ec2,vpc \
  --output ./provider-aws

# Generate all supported AWS services
hemmer-provider-generator aws --all

# Generate GCP provider
hemmer-provider-generator gcp \
  --services storage,compute \
  --output ./provider-gcp
```

## Key Design Decisions

### 1. SDK Parsing Strategy

- Use SDK service definitions (Smithy for AWS)
- Map operations to CRUD (Create, Read, Update, Delete)
- Handle pagination, waiters, and retries

### 2. Type Mapping

| SDK Type | Rust Type | KCL Type |
|----------|-----------|----------|
| String | String | str |
| Integer | i64 | int |
| Boolean | bool | bool |
| List | Vec<T> | [T] |
| Map | HashMap<K,V> | {K:V} |

### 3. Operation Mapping

- **Create**: Operations that create resources (CreateX, PutX)
- **Read**: Operations that describe/get resources (DescribeX, GetX)
- **Update**: Operations that modify resources (UpdateX, ModifyX)
- **Delete**: Operations that remove resources (DeleteX, TerminateX)

### 4. Code Generation Approach

- Template-based generation (Tera templates)
- AST manipulation for complex logic
- Format with rustfmt after generation

## Milestones

### Milestone 1: MVP (Weeks 1-2)

- Basic AWS S3 bucket generation
- Proof of concept end-to-end
- Manual testing

### Milestone 2: AWS Core (Weeks 3-4)

- 5 AWS resources (S3, VPC, Subnet, SG, EC2)
- Automated tests
- CLI interface

### Milestone 3: Production (Weeks 5-6)

- All common AWS resources
- Documentation
- CI/CD pipeline
- First release

### Milestone 4: Multi-Cloud (Weeks 7-8)

- GCP support
- Azure support
- Provider templates

## Contributing

This project follows the same contribution guidelines as the main Hemmer repository.

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

---

**Status**: 🚧 Planning Phase

**Next Steps**: See issues and project board for current work.
