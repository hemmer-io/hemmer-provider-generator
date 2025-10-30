# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2025-10-29

### Fixed

- **Service Name Sanitization** (#29)
  - Enhanced `infer_service_name()` to remove domain suffixes (`.k8s.io`, `.apiserver`, `.googleapis.com`, `.azure.com`)
  - Removes spec file suffixes (`_openapi`, `-openapi`, `_discovery`, `-discovery`)
  - Removes version suffixes (`__v1`, `-v1`)
  - Added `sanitize_name()` function to convert invalid characters to valid Rust identifiers
  - Converts dots to underscores
  - Cleans up consecutive underscores
  - Strips leading/trailing underscores
  - Fixes K8s service names: `apis__internal` → `apis_internal`

- **Resource Name Sanitization** (#29)
  - Enhanced `extract_resource_from_path()` in OpenAPI parser to strip domain suffixes
  - Removes `.k8s.io`, `.googleapis.com`, `.apiserver`, `.azure.com` from resource names
  - Fixes K8s resource names: `Storage.k8s.io` → `Storage`, `Internal.apiserver.k8s.io` → `Internal`
  - All generated resource names are now valid Rust identifiers

- **Unified README Formatting** (#29)
  - Fixed CRUD operation formatting in `unified_README.md.tera`
  - Added proper spacing before operation brackets
  - Changed: `**ResourceName**[CRUD]` → `**ResourceName** [CRUD]`
  - Only shows brackets when operations exist
  - Improved readability of generated documentation

## [0.2.1] - 2025-10-29

### Fixed

- **Snake Case Conversion** (#25, #26)
  - Improved `to_snake_case` function in all parser converters (Smithy, OpenAPI, Discovery, Protobuf)
  - Properly handles consecutive capitals (e.g., `HTTPServer` → `http_server` instead of `httpserver`)
  - Cleans up multiple consecutive underscores (e.g., `__test__` → `test`)
  - Strips leading and trailing underscores
  - Handles hyphens and spaces by converting to underscores
  - Fixes K8s resource naming showing `__api` and other formatting issues

- **README Operations Formatting** (#25, #26)
  - Fixed CRUD operation list items in both `README.md.tera` and `unified_README.md.tera`
  - Operations now display on separate lines instead of running together
  - Improved readability of generated provider documentation

### Added

- **Test Directory Filtering** (#26)
  - Spec discovery now skips directories containing `__test__` in their name
  - Prevents processing test fixtures during unified provider generation
  - Reduces noise and improves generation speed for large SDK directories

- **Graceful Parse Error Handling** (#26)
  - Parse errors no longer fail the entire unified generation process
  - Problematic specs are skipped with a warning message
  - Generation continues with successfully parsed specs
  - Displays count of skipped specs at the end of processing
  - Enables successful generation even when some specs have issues (e.g., GCP identity service)

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
