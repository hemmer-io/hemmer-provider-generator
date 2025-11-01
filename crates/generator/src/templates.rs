//! Template loading and management

use hemmer_provider_generator_common::{GeneratorError, Result};
use std::collections::HashMap;
use tera::{Tera, Value};

/// Load all templates
pub fn load_templates() -> Result<Tera> {
    let mut tera = Tera::default();

    // Register custom filters
    tera.register_filter("kcl_type", kcl_type_filter);
    tera.register_filter("rust_type", rust_type_filter);
    tera.register_filter("capitalize", capitalize_filter);

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

    Ok(tera)
}

/// Load templates for unified multi-service provider generation
pub fn load_unified_templates() -> Result<Tera> {
    let mut tera = Tera::default();

    // Register custom filters
    tera.register_filter("kcl_type", kcl_type_filter);
    tera.register_filter("rust_type", rust_type_filter);
    tera.register_filter("capitalize", capitalize_filter);
    tera.register_filter("lower", lower_filter);
    tera.register_filter("sdk_dependency", sdk_dependency_filter);
    tera.register_filter("sanitize_identifier", sanitize_identifier_filter);

    // Add unified templates
    tera.add_raw_template(
        "unified_provider.k",
        include_str!("../templates/unified_provider.k.tera"),
    )
    .map_err(|e| {
        GeneratorError::Generation(format!("Failed to load unified_provider.k template: {}", e))
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
        "Aws" => format!("aws-sdk-{}", service_name),
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
fn sanitize_identifier_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    use hemmer_provider_generator_common::sanitize_rust_identifier;

    let s = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("sanitize_identifier filter expects a string"))?;

    Ok(Value::String(sanitize_rust_identifier(s)))
}
