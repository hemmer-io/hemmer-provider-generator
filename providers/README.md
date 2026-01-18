# Provider SDK Metadata Files

This directory contains YAML metadata files that define provider-specific SDK configuration for code generation.

## Purpose

These metadata files externalize provider configuration from Rust code, making it easier to:
- Add new cloud providers without modifying Rust code
- Update SDK patterns and dependencies
- Customize error handling per provider
- Maintain provider configurations independently

## File Naming Convention

Metadata files follow the pattern: `{provider-name}.sdk-metadata.yaml`

Examples:
- `aws.sdk-metadata.yaml`
- `gcp.sdk-metadata.yaml`
- `azure.sdk-metadata.yaml`
- `kubernetes.sdk-metadata.yaml`

## Schema Structure

### Root Structure

```yaml
version: 1              # Metadata format version

provider:
  name: string          # Provider identifier (e.g., "aws")
  display_name: string  # Human-readable name (e.g., "Amazon Web Services")

sdk:
  crate_pattern: string          # SDK crate naming pattern
  client_type_pattern: string    # Client type pattern
  config_crate: string?          # Optional config crate name
  async_client: bool             # Whether SDK uses async clients
  region_attr: string?           # Optional region attribute name
  dependencies: string[]         # Additional dependencies

config:
  initialization:
    snippet: string     # Code to initialize config loader
    var_name: string    # Variable name for config loader

  load:
    snippet: string     # Code to load/finalize config
    var_name: string    # Variable name for loaded config

  client_from_config:
    snippet: string     # Code to create client from config
    var_name: string    # Variable name for client

  attributes:
    - name: string           # Attribute name
      description: string    # Human-readable description
      required: bool         # Whether required
      setter: string?        # Optional setter snippet
      extractor: string?     # Optional value extractor

errors:
  metadata_import: string?              # Optional error metadata trait
  categorization:
    category_name: string[]  # Error code patterns
```

## Field Descriptions

### provider

- **name**: Provider identifier used in code and CLI (lowercase, e.g., "aws", "gcp")
- **display_name**: Human-readable provider name for documentation

### sdk

- **crate_pattern**: Pattern for SDK crate names with `{service}` placeholder
  - Example: `"aws-sdk-{service}"` → `aws-sdk-s3`, `aws-sdk-ec2`

- **client_type_pattern**: Pattern for client types with `{service}` placeholder
  - Example: `"aws_sdk_{service}::Client"` → `aws_sdk_s3::Client`

- **config_crate**: Optional config crate name
  - Example: `"aws-config"`, `"azure_identity"`

- **async_client**: Boolean indicating if SDK clients are async

- **region_attr**: Optional region/location attribute name
  - Example: `"region"` for AWS, `"location"` for GCP/Azure

- **dependencies**: Additional Cargo dependencies beyond service SDK crate
  - Format: `["crate-name = \"version\""]`
  - Example: `["aws-config = \"1\"", "aws-smithy-types = \"1\""]`

### config

Configuration code generation patterns use these placeholders:
- `{service}`: Replaced with service name
- `{value}`: Replaced with extracted config value
- `{config}`: Replaced with config variable name

#### initialization

Code to initialize the config loader/builder.

- **snippet**: Rust code expression
  - Example (AWS): `"aws_config::from_env()"`
  - Example (GCP): `"ClientConfig::default()"`
- **var_name**: Variable name for the loader
  - Example: `"config_loader"`, `"config_builder"`

#### load

Code to finalize and load the configuration.

- **snippet**: Rust code expression
  - Example (AWS): `"config_loader.load().await"`
  - Example (K8s): `"Config::from_kubeconfig(&kubeconfig_data).await"`
- **var_name**: Variable name for loaded config
  - Example: `"sdk_config"`, `"config"`

#### client_from_config

Code to create SDK client from loaded config.

- **snippet**: Rust code expression with `{client_type}` placeholder
  - Example (AWS): `"{client_type}::new(&sdk_config)"`
  - Example (K8s): `"Client::try_from({config})"`
- **var_name**: Not currently used for client creation

#### attributes

List of provider-specific configuration attributes.

- **name**: Attribute identifier (snake_case)
  - Example: `"region"`, `"profile"`, `"project_id"`

- **description**: Human-readable description
  - Example: `"AWS region to use"`

- **required**: Boolean indicating if attribute is required

- **setter**: Optional code snippet to set this config value
  - Uses `{value}` placeholder for extracted JSON value
  - Example: `"config_loader = config_loader.region(aws_config::Region::new({value}.to_string()))"`

- **extractor**: Optional value extraction expression
  - Example: `"as_str()"`, `"as_i64()"`

### errors

Error handling configuration.

#### metadata_import

Optional import path for error metadata trait.

- Example: `"aws_smithy_types::error::metadata::ProvideErrorMetadata"`

#### categorization

Map of ProviderError category names to error code patterns.

Supported category names:
- `not_found`
- `already_exists`
- `permission_denied`
- `validation`
- `failed_precondition`
- `resource_exhausted`
- `unavailable`
- `deadline_exceeded`
- `unimplemented`

Pattern types:
- **Exact match**: `"NotFound"` → matches exactly "NotFound"
- **Prefix wildcard**: `"NoSuch*"` → matches codes starting with "NoSuch"
- **Suffix wildcard**: `"*InUse"` → matches codes ending with "InUse"
- **Contains wildcard**: `"*Limit*"` → matches codes containing "Limit"

Example:
```yaml
errors:
  categorization:
    not_found:
      - "NotFound"
      - "NoSuch*"
      - "ResourceNotFoundException"
    permission_denied:
      - "AccessDenied"
      - "Unauthorized"
```

## Complete Example: AWS

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
    - aws-config = "1"
    - aws-smithy-types = "1"
    - aws-smithy-runtime-api = "1"

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
      setter: "config_loader = config_loader.region(aws_config::Region::new({value}.to_string()))"
      extractor: "as_str()"

    - name: profile
      description: AWS profile to use
      required: false
      setter: "config_loader = config_loader.profile_name({value})"
      extractor: "as_str()"

errors:
  metadata_import: "aws_smithy_types::error::metadata::ProvideErrorMetadata"
  categorization:
    not_found:
      - "NotFound"
      - "NoSuch*"
      - "ResourceNotFoundException"
    already_exists:
      - "AlreadyExists"
      - "AlreadyOwned"
      - "EntityAlreadyExists"
      - "DuplicateRequest"
    permission_denied:
      - "AccessDenied"
      - "Unauthorized"
      - "InvalidAccessKeyId"
      - "SignatureDoesNotMatch"
      - "ExpiredToken"
      - "InvalidToken"
    validation:
      - "Invalid*"
      - "Malformed*"
      - "ValidationException"
      - "ValidationError"
      - "*ParameterValue"
    failed_precondition:
      - "*NotEmpty"
      - "*InUse"
      - "*Conflict"
      - "OperationAborted"
      - "PreconditionFailed"
      - "ConditionalCheckFailedException"
    resource_exhausted:
      - "*LimitExceeded"
      - "*TooMany"
      - "SlowDown"
      - "Throttling"
      - "ThrottlingException"
      - "ProvisionedThroughputExceededException"
      - "RequestLimitExceeded"
    unavailable:
      - "ServiceUnavailable"
      - "InternalError"
      - "InternalFailure"
      - "*ServiceException"
    deadline_exceeded:
      - "RequestTimeout"
      - "RequestExpired"
      - "*Timeout"
    unimplemented:
      - "NotImplemented"
      - "UnsupportedOperation"
```

## Adding a New Provider

To add support for a new cloud provider:

1. Create a new file: `{provider-name}.sdk-metadata.yaml`
2. Define the provider information (name, display_name)
3. Specify SDK crate and client type patterns
4. Define configuration code generation snippets
5. Add provider-specific config attributes
6. (Optional) Configure error categorization
7. The provider will automatically be available via `Provider::from_name("{provider-name}")`

## Validation

The schema is validated at load time by the `ProviderSdkMetadata::load()` function in `crates/common/src/sdk_metadata.rs`.

Common errors:
- **Missing required fields**: Ensure all non-optional fields are present
- **Invalid YAML syntax**: Check for proper indentation and quoting
- **Invalid patterns**: Verify `{service}` and `{value}` placeholders are used correctly
- **Unknown categories**: Use only supported ProviderError category names

## Testing

Metadata files are tested by:
1. Unit tests in `crates/common/src/sdk_metadata.rs`
2. Integration tests that load and validate all provider files
3. Code generation tests that verify generated output matches expected patterns

## Schema Versioning

The `version` field at the root allows for schema evolution:
- **version 1**: Initial schema (current)
- Future versions may add fields while maintaining backwards compatibility
