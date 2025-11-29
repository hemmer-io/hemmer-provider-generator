//! Template loading and management

use hemmer_provider_generator_common::{GeneratorError, Result};
use std::collections::HashMap;
use tera::{Tera, Value};

/// Load all templates
pub fn load_templates() -> Result<Tera> {
    let mut tera = Tera::default();

    // Register custom filters
    tera.register_filter("kcl_type", kcl_type_filter);
    tera.register_filter("jcl_type", jcl_type_filter);
    tera.register_filter("sdk_attr_type", sdk_attr_type_filter);
    tera.register_filter("rust_type", rust_type_filter);
    tera.register_filter("capitalize", capitalize_filter);
    tera.register_filter("lower", lower_filter);
    tera.register_filter("sdk_dependency", sdk_dependency_filter);
    tera.register_filter("client_type", client_type_filter);
    tera.register_filter("sdk_crate_module", sdk_crate_module_filter);
    tera.register_filter("has_config_crate", has_config_crate_filter);
    tera.register_filter("config_crate", config_crate_filter);
    tera.register_filter("uses_shared_client", uses_shared_client_filter);
    tera.register_filter("sanitize_identifier", sanitize_identifier_filter);
    tera.register_filter("sanitize_identifier_part", sanitize_identifier_part_filter);
    tera.register_filter("to_camel_case", to_camel_case_filter);
    tera.register_filter("json_extractor", json_extractor_filter);

    // Add templates inline for now (Phase 3 MVP)
    // In production, these could be loaded from files
    tera.add_raw_template("provider.k", include_str!("../templates/provider.k.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load provider.k template: {}", e))
        })?;

    tera.add_raw_template("Cargo.toml", include_str!("../templates/Cargo.toml.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load Cargo.toml template: {}", e))
        })?;

    tera.add_raw_template("lib.rs", include_str!("../templates/lib.rs.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load lib.rs template: {}", e))
        })?;

    tera.add_raw_template("resource.rs", include_str!("../templates/resource.rs.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load resource.rs template: {}", e))
        })?;

    tera.add_raw_template(
        "resources_mod.rs",
        include_str!("../templates/resources_mod.rs.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load resources_mod.rs template: {}", e))
    })?;

    tera.add_raw_template("README.md", include_str!("../templates/README.md.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load README.md template: {}", e))
        })?;

    tera.add_raw_template("main.rs", include_str!("../templates/main.rs.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load main.rs template: {}", e))
        })?;

    tera.add_raw_template(
        "provider.jcf",
        include_str!("../templates/provider.jcf.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load provider.jcf template: {}", e))
    })?;

    Ok(tera)
}

/// Load templates for unified multi-service provider generation
pub fn load_unified_templates() -> Result<Tera> {
    let mut tera = Tera::default();

    // Register custom filters
    tera.register_filter("kcl_type", kcl_type_filter);
    tera.register_filter("jcl_type", jcl_type_filter);
    tera.register_filter("sdk_attr_type", sdk_attr_type_filter);
    tera.register_filter("rust_type", rust_type_filter);
    tera.register_filter("capitalize", capitalize_filter);
    tera.register_filter("lower", lower_filter);
    tera.register_filter("sdk_dependency", sdk_dependency_filter);
    tera.register_filter("client_type", client_type_filter);
    tera.register_filter("sdk_crate_module", sdk_crate_module_filter);
    tera.register_filter("has_config_crate", has_config_crate_filter);
    tera.register_filter("config_crate", config_crate_filter);
    tera.register_filter("uses_shared_client", uses_shared_client_filter);
    tera.register_filter("sanitize_identifier", sanitize_identifier_filter);
    tera.register_filter("sanitize_identifier_part", sanitize_identifier_part_filter);
    tera.register_filter("to_camel_case", to_camel_case_filter);
    tera.register_filter("json_extractor", json_extractor_filter);

    // Add unified templates
    tera.add_raw_template(
        "unified_main.rs",
        include_str!("../templates/unified_main.rs.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load unified_main.rs template: {}", e))
    })?;

    tera.add_raw_template(
        "unified_provider.jcf",
        include_str!("../templates/unified_provider.jcf.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!(
            "Failed to load unified_provider.jcf template: {}",
            e
        ))
    })?;

    tera.add_raw_template(
        "unified_Cargo.toml",
        include_str!("../templates/unified_Cargo.toml.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load unified_Cargo.toml template: {}", e))
    })?;

    tera.add_raw_template(
        "unified_lib.rs",
        include_str!("../templates/unified_lib.rs.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load unified_lib.rs template: {}", e))
    })?;

    tera.add_raw_template(
        "unified_service.rs",
        include_str!("../templates/unified_service.rs.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load unified_service.rs template: {}", e))
    })?;

    tera.add_raw_template(
        "unified_resource.rs",
        include_str!("../templates/unified_resource.rs.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!(
            "Failed to load unified_resource.rs template: {}",
            e
        ))
    })?;

    tera.add_raw_template(
        "resources_mod.rs",
        include_str!("../templates/resources_mod.rs.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load resources_mod.rs template: {}", e))
    })?;

    tera.add_raw_template(
        "unified_README.md",
        include_str!("../templates/unified_README.md.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load unified_README.md template: {}", e))
    })?;

    tera.add_raw_template("release.yml", include_str!("../templates/release.yml.tera"))
        .map_err(|e| {
            GeneratorError::Generation(format!("Failed to load release.yml template: {}", e))
        })?;

    // Add docs templates
    tera.add_raw_template(
        "docs_installation.md",
        include_str!("../templates/docs_installation.md.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!(
            "Failed to load docs_installation.md template: {}",
            e
        ))
    })?;

    tera.add_raw_template(
        "docs_getting_started.md",
        include_str!("../templates/docs_getting_started.md.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!(
            "Failed to load docs_getting_started.md template: {}",
            e
        ))
    })?;

    tera.add_raw_template(
        "docs_service.md",
        include_str!("../templates/docs_service.md.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load docs_service.md template: {}", e))
    })?;

    Ok(tera)
}

/// Filter to convert FieldType to KCL type
fn kcl_type_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::FieldType;

    // Deserialize the FieldType from the Serde Value
    let field_type: FieldType = serde_json::from_value(value.clone())
        .map_err(|e| tera::Error::msg(format!("Failed to deserialize FieldType: {}", e)))?;

    // Use the built-in to_kcl_type method
    let kcl_type = field_type.to_kcl_type();

    // Convert KCL types to KCL syntax
    let kcl_syntax = match kcl_type.as_str() {
        "String" => "str".to_string(),
        "Integer" => "int".to_string(),
        "Boolean" => "bool".to_string(),
        "Float" => "float".to_string(),
        _ if kcl_type.starts_with("List<") => {
            // Extract inner type and convert: List<String> -> [str]
            let inner = kcl_type
                .strip_prefix("List<")
                .and_then(|s| s.strip_suffix(">"))
                .unwrap_or("str");
            format!(
                "[{}]",
                inner.replace("String", "str").replace("Integer", "int")
            )
        }
        _ if kcl_type.starts_with("Map<") => {
            // Convert Map<K,V> -> {k: v}
            "{str: str}".to_string() // Simplified for now
        }
        _ => "str".to_string(), // Default fallback
    };

    Ok(Value::String(kcl_syntax))
}

/// Filter to convert FieldType to Rust type
fn rust_type_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::FieldType;

    // Deserialize the FieldType from the Serde Value
    let field_type: FieldType = serde_json::from_value(value.clone())
        .map_err(|e| tera::Error::msg(format!("Failed to deserialize FieldType: {}", e)))?;

    // Use the built-in to_rust_type method
    let rust_type = field_type.to_rust_type();

    Ok(Value::String(rust_type))
}

/// Filter to capitalize first letter
fn capitalize_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("capitalize filter expects a string"))?;

    if s.is_empty() {
        return Ok(Value::String(s.to_string()));
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    let rest: String = chars.collect();

    Ok(Value::String(format!("{}{}", first, rest)))
}

/// Filter to convert string to lowercase
fn lower_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("lower filter expects a string"))?;

    Ok(Value::String(s.to_lowercase()))
}

/// Filter to generate SDK dependency name based on provider
/// Usage: {{ provider | sdk_dependency(service_name=service.name) }}
/// Examples:
///   - Aws + "s3" -> "aws-sdk-s3"
///   - Aws + "acm_pca" -> "aws-sdk-acmpca" (underscores removed)
///   - Aws + "configservice" -> "aws-sdk-config" (special mapping)
///   - Gcp + "storage" -> "google-storage"
///   - Azure + "compute" -> "azure-compute"
fn sdk_dependency_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let provider = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("sdk_dependency filter expects provider as a string"))?;

    let service_name = args
        .get("service_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("sdk_dependency filter requires service_name parameter"))?;

    let dependency = match provider {
        "Aws" => {
            // Special case mappings for AWS services that don't follow the standard pattern
            let normalized = match service_name {
                // Services that need renaming (5 total)
                "apigatewaymanagementapi" => "apigatewaymanagement",
                "configservice" => "config",
                "costandusagereportservice" => "costandusagereport",
                "lexmodels" => "lexmodelsv2",
                "pinpointsms" => "pinpointsmsvoice",

                // Services that don't exist or have 0.0.0 versions (return empty to skip)
                "chimesdk"
                | "lexmodelbuildingservice"
                | "lexruntimeservice"
                | "databasemigrationservice"
                | "elasticsearchservice"
                | "resourcegroupstaggingapi"
                | "marketplaceentitlementservice" => {
                    return Ok(Value::String(String::new()));
                }

                // Default: remove underscores from service name
                _ => service_name,
            };

            // Remove underscores for all AWS SDK crates
            format!("aws-sdk-{}", normalized.replace("_", ""))
        }
        "Gcp" => format!("google-{}", service_name),
        "Azure" => format!("azure-{}", service_name),
        _ => {
            return Err(tera::Error::msg(format!(
                "Unsupported provider for sdk_dependency: {}",
                provider
            )))
        }
    };

    Ok(Value::String(dependency))
}

/// Filter to sanitize strings to valid Rust identifiers
/// Usage: {{ resource.name | sanitize_identifier }}
/// Ensures the result is a valid Rust identifier:
/// - Replaces special characters with underscores
/// - Escapes Rust keywords with r# prefix
/// - Handles digits at start
///
/// Use for standalone identifiers (variable names, etc.)
fn sanitize_identifier_filter(
    value: &Value,
    _args: &HashMap<String, Value>,
) -> tera::Result<Value> {
    use hemmer_provider_generator_common::sanitize_rust_identifier;

    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("sanitize_identifier filter expects a string"))?;

    Ok(Value::String(sanitize_rust_identifier(s)))
}

/// Filter to sanitize strings for use in composite identifiers (function names)
/// Usage: {{ resource.name | sanitize_identifier_part }}
/// Similar to sanitize_identifier, but handles keywords differently:
/// - Replaces special characters with underscores
/// - Appends underscore suffix to keywords (not r# prefix)
/// - Handles digits at start
///
/// The r# prefix only works for complete identifiers, not parts of composite names.
/// For example: `plan_r#type()` is invalid, but `plan_type_()` is valid.
///
/// Use for composite identifiers (function name parts)
fn sanitize_identifier_part_filter(
    value: &Value,
    _args: &HashMap<String, Value>,
) -> tera::Result<Value> {
    use hemmer_provider_generator_common::sanitize_identifier_part;

    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("sanitize_identifier_part filter expects a string"))?;

    Ok(Value::String(sanitize_identifier_part(s)))
}

/// Filter to convert FieldType to JCL type string
/// Usage: {{ field.field_type | jcl_type }}
/// Examples:
///   - String -> "string"
///   - Integer -> "int"
///   - Float -> "float"
///   - Boolean -> "bool"
///   - List(String) -> "[string]"
///   - Map(String, Integer) -> "{string: int}"
fn jcl_type_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::FieldType;

    let field_type: FieldType = serde_json::from_value(value.clone())
        .map_err(|e| tera::Error::msg(format!("Failed to deserialize FieldType: {}", e)))?;

    let jcl_type = field_type_to_jcl(&field_type);
    Ok(Value::String(jcl_type))
}

/// Convert FieldType to JCL type string
fn field_type_to_jcl(field_type: &hemmer_provider_generator_common::FieldType) -> String {
    use hemmer_provider_generator_common::FieldType;

    match field_type {
        FieldType::String => "string".to_string(),
        FieldType::Integer => "int".to_string(),
        FieldType::Float => "float".to_string(),
        FieldType::Boolean => "bool".to_string(),
        FieldType::DateTime => "string".to_string(), // DateTime represented as string in JCL
        FieldType::List(inner) => format!("[{}]", field_type_to_jcl(inner)),
        FieldType::Map(key, value) => {
            format!(
                "{{{}: {}}}",
                field_type_to_jcl(key),
                field_type_to_jcl(value)
            )
        }
        FieldType::Enum(_) => "string".to_string(), // Enums represented as strings in JCL
        FieldType::Object(_) => "object".to_string(), // Complex objects
    }
}

/// Filter to convert FieldType to SDK AttributeType constructor
/// Usage: {{ field.field_type | sdk_attr_type }}
/// Examples:
///   - String -> AttributeType::String
///   - Integer -> AttributeType::Int64
///   - Float -> AttributeType::Float64
///   - Boolean -> AttributeType::Bool
///   - List(String) -> AttributeType::list(AttributeType::String)
///   - Map(String, Integer) -> AttributeType::map(AttributeType::Int64)
fn sdk_attr_type_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::FieldType;

    let field_type: FieldType = serde_json::from_value(value.clone())
        .map_err(|e| tera::Error::msg(format!("Failed to deserialize FieldType: {}", e)))?;

    let sdk_type = field_type_to_sdk_attr(&field_type);
    Ok(Value::String(sdk_type))
}

/// Convert FieldType to SDK AttributeType constructor expression
fn field_type_to_sdk_attr(field_type: &hemmer_provider_generator_common::FieldType) -> String {
    use hemmer_provider_generator_common::FieldType;

    match field_type {
        FieldType::String => "AttributeType::String".to_string(),
        FieldType::Integer => "AttributeType::Int64".to_string(),
        FieldType::Float => "AttributeType::Float64".to_string(),
        FieldType::Boolean => "AttributeType::Bool".to_string(),
        FieldType::DateTime => "AttributeType::String".to_string(), // DateTime as string
        FieldType::List(inner) => {
            format!("AttributeType::list({})", field_type_to_sdk_attr(inner))
        }
        FieldType::Map(_key, value) => {
            // SDK Map type only takes value type (keys are always strings)
            format!("AttributeType::map({})", field_type_to_sdk_attr(value))
        }
        FieldType::Enum(_) => "AttributeType::String".to_string(), // Enums as strings
        FieldType::Object(_) => "AttributeType::Dynamic".to_string(), // Complex objects as dynamic
    }
}

/// Filter to generate SDK client type for a provider and service
/// Usage: {{ provider | client_type(service_name=service.name) }}
/// Examples:
///   - Aws + "s3" -> "aws_sdk_s3::Client"
///   - Gcp + "storage" -> "google_cloud_storage::Client"
///   - Kubernetes -> "kube::Client"
fn client_type_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::Provider;

    let provider_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("client_type filter expects provider as a string"))?;

    let provider = match provider_str {
        "Aws" => Provider::Aws,
        "Gcp" => Provider::Gcp,
        "Azure" => Provider::Azure,
        "Kubernetes" => Provider::Kubernetes,
        _ => {
            return Err(tera::Error::msg(format!(
                "Unknown provider: {}",
                provider_str
            )))
        }
    };

    let service_name = args
        .get("service_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let client_type = provider.client_type_for_service(service_name);
    Ok(Value::String(client_type))
}

/// Filter to generate SDK crate name (Rust module style) for a provider and service
/// Usage: {{ provider | sdk_crate_module(service_name=service.name) }}
/// Examples:
///   - Aws + "s3" -> "aws_sdk_s3"
///   - Gcp + "storage" -> "google_cloud_storage"
fn sdk_crate_module_filter(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::Provider;

    let provider_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("sdk_crate_module filter expects provider as a string"))?;

    let provider = match provider_str {
        "Aws" => Provider::Aws,
        "Gcp" => Provider::Gcp,
        "Azure" => Provider::Azure,
        "Kubernetes" => Provider::Kubernetes,
        _ => {
            return Err(tera::Error::msg(format!(
                "Unknown provider: {}",
                provider_str
            )))
        }
    };

    let service_name = args
        .get("service_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Get the crate name and convert to module style (- to _)
    let crate_name = provider.sdk_crate_for_service(service_name);
    let module_name = crate_name.replace("-", "_");
    Ok(Value::String(module_name))
}

/// Filter to check if a provider has a config crate
/// Usage: {% if provider | has_config_crate %}
fn has_config_crate_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::Provider;

    let provider_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("has_config_crate filter expects provider as a string"))?;

    let provider = match provider_str {
        "Aws" => Provider::Aws,
        "Gcp" => Provider::Gcp,
        "Azure" => Provider::Azure,
        "Kubernetes" => Provider::Kubernetes,
        _ => {
            return Err(tera::Error::msg(format!(
                "Unknown provider: {}",
                provider_str
            )))
        }
    };

    let has_config = provider.sdk_config().config_crate.is_some();
    Ok(Value::Bool(has_config))
}

/// Filter to get the config crate name for a provider
/// Usage: {{ provider | config_crate }}
fn config_crate_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::Provider;

    let provider_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("config_crate filter expects provider as a string"))?;

    let provider = match provider_str {
        "Aws" => Provider::Aws,
        "Gcp" => Provider::Gcp,
        "Azure" => Provider::Azure,
        "Kubernetes" => Provider::Kubernetes,
        _ => {
            return Err(tera::Error::msg(format!(
                "Unknown provider: {}",
                provider_str
            )))
        }
    };

    let config_crate = provider.sdk_config().config_crate.unwrap_or_default();
    Ok(Value::String(config_crate))
}

/// Filter to check if a provider uses a shared client vs per-service clients
/// Usage: {% if provider | uses_shared_client %}
fn uses_shared_client_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::Provider;

    let provider_str = value.as_str().ok_or_else(|| {
        tera::Error::msg("uses_shared_client filter expects provider as a string")
    })?;

    let provider = match provider_str {
        "Aws" => Provider::Aws,
        "Gcp" => Provider::Gcp,
        "Azure" => Provider::Azure,
        "Kubernetes" => Provider::Kubernetes,
        _ => {
            return Err(tera::Error::msg(format!(
                "Unknown provider: {}",
                provider_str
            )))
        }
    };

    Ok(Value::Bool(provider.uses_shared_client()))
}

/// Filter to convert snake_case to camelCase for SDK accessor methods
/// Usage: {{ "bucket_name" | to_camel_case }} -> "bucketName"
fn to_camel_case_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("to_camel_case filter expects a string"))?;

    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    Ok(Value::String(result))
}

/// Filter to convert FieldType to JSON extraction method
/// Usage: {{ field.field_type | json_extractor }} -> "as_str()"
fn json_extractor_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::FieldType;

    let field_type: FieldType = serde_json::from_value(value.clone())
        .map_err(|e| tera::Error::msg(format!("Failed to deserialize FieldType: {}", e)))?;

    let extractor = match field_type {
        FieldType::String => "as_str().map(|s| s.to_string())",
        FieldType::Integer => "as_i64()",
        FieldType::Float => "as_f64()",
        FieldType::Boolean => "as_bool()",
        FieldType::DateTime => "as_str().map(|s| s.to_string())",
        FieldType::List(_) => "as_array().cloned()",
        FieldType::Map(_, _) => "as_object().cloned()",
        FieldType::Enum(_) => "as_str().map(|s| s.to_string())",
        FieldType::Object(_) => "clone()",
    };

    Ok(Value::String(extractor.to_string()))
}
