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

/// Filter to convert FieldType to KCL type
fn kcl_type_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let field_type_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("kcl_type filter expects a string"))?;

    // Parse the field_type string (this is a simplified version)
    // In a real implementation, we'd deserialize the FieldType enum
    let kcl_type = match field_type_str {
        "String" => "str",
        "Integer" => "int",
        "Boolean" => "bool",
        "Float" => "float",
        "DateTime" => "str",
        _ if field_type_str.starts_with("List") => "[str]", // Simplified
        _ if field_type_str.starts_with("Map") => "{str: str}", // Simplified
        _ => "str",                                         // Default fallback
    };

    Ok(Value::String(kcl_type.to_string()))
}

/// Filter to convert FieldType to Rust type
fn rust_type_filter(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let field_type_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("rust_type filter expects a string"))?;

    // Parse the field_type string (simplified version)
    let rust_type = match field_type_str {
        "String" => "String",
        "Integer" => "i64",
        "Boolean" => "bool",
        "Float" => "f64",
        "DateTime" => "String",
        _ if field_type_str.starts_with("List") => "Vec<String>", // Simplified
        _ if field_type_str.starts_with("Map") => "HashMap<String, String>", // Simplified
        _ => "String",                                            // Default fallback
    };

    Ok(Value::String(rust_type.to_string()))
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
