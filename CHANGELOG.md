# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-11-01

### Changed

- **Dependency Updates** (#48)
  - Updated prost ecosystem to resolve type compatibility issues
  - `prost`: 0.13 → 0.14
  - `prost-types`: 0.13 → 0.14
  - `prost-reflect`: 0.14 → 0.16

### Added

- **Dependabot Configuration** (#41, #49)
  - Configured automated weekly dependency updates for Cargo and GitHub Actions
  - Added prost ecosystem grouping to bundle related dependencies together
  - Prevents future version conflicts by updating prost packages synchronously
  - Auto-assigns PRs to @hemmer-io/engineering team

## [0.3.0] - 2025-11-01

### Added

- **ProviderExecutor Trait Implementation** (#32, #38)
  - Generated providers now implement the complete `ProviderExecutor` trait from `hemmer-provider`
  - Added `configure()`, `plan()`, `create()`, `read()`, `update()`, `delete()` methods
  - Resource dispatcher routes operations based on "service.resource" format
  - Added `create_provider()` FFI factory function for dynamic library loading
  - Configured `crate-type = ["cdylib", "rlib"]` for dual binary/library compilation
  - Generated providers are now fully functional with Hemmer CLI

- **Service Module Code Generation** (#32, #38)
  - New `unified_service.rs.tera` template for complete service modules
  - Each service has dedicated CRUD dispatchers for all resources
  - Individual resource methods with placeholder SDK implementations
  - Proper async handling using provider's Tokio runtime via `block_on()`
  - Complete error handling with `hemmer_core::HemmerError`

- **Cross-Platform Binary Build Automation** (#34, #39)
  - Automated GitHub Actions workflow for building release binaries
  - Builds for 5 platforms: darwin-amd64, darwin-arm64, linux-amd64, linux-arm64, windows-amd64
  - Cross-compilation setup for Linux ARM64 with proper toolchains
  - Binary naming follows ProviderRegistry convention: `hemmer-provider-{name}-{platform}{ext}`
  - Automatic SHA256 checksum generation in `checksums.txt`
  - GitHub release creation with all artifacts (binaries, checksums, provider.k)
  - Triggered automatically on version tags (v*)

- **Enhanced Documentation** (#34, #39)
  - Added Installation section to README with Hemmer CLI and manual instructions
  - Added Building from Source section with cargo commands
  - Added Creating a Release section with complete tag-push workflow
  - Release assets documentation listing all generated files
  - Contributing section with provider regeneration instructions

- **Tokio Runtime Integration** (#33, #37)
  - Added `runtime: tokio::runtime::Runtime` field to generated providers
  - Changed `new()` from async to sync, using `runtime.block_on()` internally
  - Wrapped AWS config loading and client initialization with `block_on()`
  - Fixes critical issue where AWS SDK operations fail when loaded as dynamic libraries (cdylib)
  - Runtime context now properly crosses FFI boundary

- **Service Name Normalization** (#31, #36)
  - Added `remove_date_suffix()` function to strip AWS SDK version dates
  - Handles patterns: `_YYYY_MM_DD` and `-YYYY-MM-DD`
  - Examples: `s3_2006_03_01` → `s3`, `ec2_2016_11_15` → `ec2`
  - Cleaner generated code with consistent service names across AWS services

### Fixed

- **Auto-Tag Workflow** (#35)
  - Updated `.github/workflows/auto-tag.yml` to use `AUTO_TAG_PAT` instead of `GITHUB_TOKEN`
  - Fixes issue where tag creation didn't trigger release workflow
  - GitHub workflows triggered by `GITHUB_TOKEN` don't trigger other workflows (security feature)
  - Using PAT allows proper workflow chaining

### Changed

- **Dependencies**
  - Added `hemmer-provider` and `hemmer-core` dependencies to generated providers
  - Added `async-trait = "0.1"` for trait implementations
  - All generated providers now have Hemmer runtime dependencies

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
