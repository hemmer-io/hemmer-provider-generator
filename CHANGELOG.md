# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-29

### Added

- **Unified Provider Code Generation** (#22, #23)
  - Complete implementation of multi-service provider generation
  - New `UnifiedProviderGenerator` struct that handles `ProviderDefinition` with multiple services
  - Six new Tera templates for unified provider structure:
    - `unified_provider.k.tera` - KCL schema with multiple services
    - `unified_Cargo.toml.tera` - Package manifest with all SDK dependencies
    - `unified_lib.rs.tera` - Main provider struct with service accessors
    - `service_mod.rs.tera` - Service-level module template
    - `unified_resource.rs.tera` - Resource implementations for unified structure
    - `unified_README.md.tera` - Multi-service provider documentation
  - `generate-unified` CLI command now generates actual code (previously showed TODO)
  - Generates complete multi-service directory structure with per-service modules
  - Integration tests for unified provider generation
  - README files for all library crates (common, parser, generator)

### Changed

- Updated `generate-unified` command to call `UnifiedProviderGenerator::generate_to_directory()`
- Improved CLI output messages for unified provider generation

### Fixed

- Resolved TODO placeholder in `generate_unified_command()`

## [0.1.2] - 2025-10-28

### Added

- Initial support for unified provider parsing and aggregation
- `generate-unified` CLI command with directory scanning
- Multi-spec parsing with service filtering
- `ProviderDefinition` IR structure for multi-service providers

## [0.1.1] - 2025-10-27

### Added

- Resource accessor methods with explicit lifetime markers

## [0.1.0] - 2025-10-26

### Added

- Initial release with core functionality
- AWS Smithy parser implementation
- OpenAPI parser (Kubernetes, Azure)
- Google Discovery format parser
- Protobuf parser for gRPC services
- Single-service provider code generation
- CLI with `parse` and `generate` commands
- Complete test suite (55+ tests)
- CI/CD pipeline with GitHub Actions
- Pre-commit hooks configuration

[0.2.0]: https://github.com/hemmer-io/hemmer-provider-generator/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/hemmer-io/hemmer-provider-generator/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/hemmer-io/hemmer-provider-generator/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hemmer-io/hemmer-provider-generator/releases/tag/v0.1.0
