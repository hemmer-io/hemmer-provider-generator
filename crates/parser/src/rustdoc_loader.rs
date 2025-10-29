//! Rustdoc JSON loader
//!
//! Loads and parses rustdoc JSON output from AWS SDK crates.

use hemmer_provider_generator_common::{GeneratorError, Result};
use rustdoc_types::Crate;
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rustdoc_loader_api() {
        // This is a placeholder test - real tests would need actual rustdoc JSON files
        // We'll test with integration tests that generate rustdoc JSON
    }
}
