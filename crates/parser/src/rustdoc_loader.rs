//! Rustdoc JSON loader
//!
//! Loads and parses rustdoc JSON output from AWS SDK crates.

use hemmer_provider_generator_common::{FieldDefinition, GeneratorError, Result};
use rustdoc_types::{Crate, Id, ItemEnum, Type};
use std::path::Path;

/// Loads rustdoc JSON from a file path
///
/// The JSON file should be generated with:
/// ```bash
/// cargo +nightly rustdoc --package aws-sdk-s3 -- -Z unstable-options --output-format json
/// ```
pub struct RustdocLoader;

impl RustdocLoader {
    /// Load rustdoc JSON from a file
    pub fn load_from_file(path: &Path) -> Result<Crate> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            GeneratorError::Parse(format!("Failed to read rustdoc JSON file: {}", e))
        })?;

        let crate_data: Crate = serde_json::from_str(&content)
            .map_err(|e| GeneratorError::Parse(format!("Failed to parse rustdoc JSON: {}", e)))?;

        Ok(crate_data)
    }

    /// Extract operation module names from the crate
    ///
    /// In AWS SDK crates, operations are in the `operation` module
    pub fn find_operation_modules(crate_data: &Crate) -> Vec<String> {
        let mut operations = Vec::new();

        // Find the root module
        let root_id = &crate_data.root;
        if let Some(root_item) = crate_data.index.get(root_id) {
            // Look for the "operation" module
            if let rustdoc_types::ItemEnum::Module(module) = &root_item.inner {
                for item_id in &module.items {
                    if let Some(item) = crate_data.index.get(item_id) {
                        if let Some(name) = &item.name {
                            if name == "operation" {
                                // Found operation module, extract its submodules
                                if let rustdoc_types::ItemEnum::Module(op_module) = &item.inner {
                                    for op_id in &op_module.items {
                                        if let Some(op_item) = crate_data.index.get(op_id) {
                                            if let Some(op_name) = &op_item.name {
                                                operations.push(op_name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        operations
    }

    /// Extract type names from the types module
    pub fn find_type_modules(crate_data: &Crate) -> Vec<String> {
        let mut types = Vec::new();

        let root_id = &crate_data.root;
        if let Some(root_item) = crate_data.index.get(root_id) {
            if let rustdoc_types::ItemEnum::Module(module) = &root_item.inner {
                for item_id in &module.items {
                    if let Some(item) = crate_data.index.get(item_id) {
                        if let Some(name) = &item.name {
                            if name == "types" {
                                if let rustdoc_types::ItemEnum::Module(types_module) = &item.inner {
                                    for type_id in &types_module.items {
                                        if let Some(type_item) = crate_data.index.get(type_id) {
                                            if let Some(type_name) = &type_item.name {
                                                types.push(type_name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        types
    }

    /// Extract fields from a struct by name
    ///
    /// Searches for a struct with the given name and extracts its field definitions.
    /// Returns empty vector if struct not found or has no fields.
    pub fn extract_struct_fields(crate_data: &Crate, struct_name: &str) -> Vec<FieldDefinition> {
        // Find the struct item
        let struct_item = crate_data
            .index
            .values()
            .find(|item| item.name.as_deref() == Some(struct_name));

        let struct_item = match struct_item {
            Some(item) => item,
            None => return vec![],
        };

        // Extract fields from the struct
        if let ItemEnum::Struct(struct_data) = &struct_item.inner {
            match &struct_data.kind {
                rustdoc_types::StructKind::Plain { fields, .. } => {
                    return fields
                        .iter()
                        .filter_map(|field_id| Self::extract_field_definition(crate_data, field_id))
                        .collect();
                }
                _ => return vec![],
            }
        }

        vec![]
    }

    /// Extract a single field definition from a field ID
    fn extract_field_definition(crate_data: &Crate, field_id: &Id) -> Option<FieldDefinition> {
        let field_item = crate_data.index.get(field_id)?;
        let field_name = field_item.name.as_ref()?.clone();

        // Extract field type
        if let ItemEnum::StructField(field_type) = &field_item.inner {
            let (field_type_mapped, required) =
                Self::map_rustdoc_type_to_field_type(crate_data, field_type);

            Some(FieldDefinition {
                name: field_name.clone(),
                field_type: field_type_mapped,
                required,
                sensitive: crate::TypeMapper::is_sensitive(&field_name),
                immutable: crate::TypeMapper::is_immutable(&field_name),
                description: field_item.docs.clone(),
            })
        } else {
            None
        }
    }

    /// Map rustdoc Type to our FieldType
    ///
    /// Returns (FieldType, required: bool)
    #[allow(clippy::only_used_in_recursion)]
    fn map_rustdoc_type_to_field_type(
        crate_data: &Crate,
        rustdoc_type: &Type,
    ) -> (hemmer_provider_generator_common::FieldType, bool) {
        use hemmer_provider_generator_common::FieldType;

        match rustdoc_type {
            Type::ResolvedPath(path) => {
                // Extract the last segment of the path as the type name
                let type_name = path.path.rsplit("::").next().unwrap_or(&path.path);

                // Check if it's Option<T>
                if type_name == "Option" {
                    if let Some(generic_args) = &path.args {
                        if let rustdoc_types::GenericArgs::AngleBracketed { args, .. } =
                            generic_args.as_ref()
                        {
                            if let Some(rustdoc_types::GenericArg::Type(inner_type)) = args.first()
                            {
                                let (inner_field_type, _) =
                                    Self::map_rustdoc_type_to_field_type(crate_data, inner_type);
                                return (inner_field_type, false); // Option means not required
                            }
                        }
                    }
                    return (FieldType::String, false);
                }

                // Check if it's Vec<T>
                if type_name == "Vec" {
                    if let Some(generic_args) = &path.args {
                        if let rustdoc_types::GenericArgs::AngleBracketed { args, .. } =
                            generic_args.as_ref()
                        {
                            if let Some(rustdoc_types::GenericArg::Type(inner_type)) = args.first()
                            {
                                let (inner_field_type, _) =
                                    Self::map_rustdoc_type_to_field_type(crate_data, inner_type);
                                return (FieldType::List(Box::new(inner_field_type)), true);
                            }
                        }
                    }
                    return (FieldType::List(Box::new(FieldType::String)), true);
                }

                // Check if it's HashMap<K, V>
                if type_name == "HashMap" {
                    if let Some(generic_args) = &path.args {
                        if let rustdoc_types::GenericArgs::AngleBracketed { args, .. } =
                            generic_args.as_ref()
                        {
                            if args.len() >= 2 {
                                let key_type = if let Some(rustdoc_types::GenericArg::Type(k)) =
                                    args.first()
                                {
                                    let (kt, _) =
                                        Self::map_rustdoc_type_to_field_type(crate_data, k);
                                    kt
                                } else {
                                    FieldType::String
                                };

                                let value_type =
                                    if let Some(rustdoc_types::GenericArg::Type(v)) = args.get(1) {
                                        let (vt, _) =
                                            Self::map_rustdoc_type_to_field_type(crate_data, v);
                                        vt
                                    } else {
                                        FieldType::String
                                    };

                                return (
                                    FieldType::Map(Box::new(key_type), Box::new(value_type)),
                                    true,
                                );
                            }
                        }
                    }
                    return (
                        FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::String)),
                        true,
                    );
                }

                // Map basic types
                (crate::TypeMapper::map_type(type_name), true)
            }
            Type::Primitive(prim) => (crate::TypeMapper::map_type(prim), true),
            _ => (FieldType::String, true), // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rustdoc_loader_api() {
        // This is a placeholder test - real tests would need actual rustdoc JSON files
        // We'll test with integration tests that generate rustdoc JSON
    }
}
