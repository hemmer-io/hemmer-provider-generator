//! Code and manifest generation for Hemmer providers
//!
//! This crate transforms parsed SDK definitions into provider artifacts
//! including KCL manifests, Rust code, and tests.

mod templates;

use hemmer_provider_generator_common::{GeneratorError, Result, ServiceDefinition};
use std::fs;
use std::path::Path;
use tera::Tera;

/// Provider generator
///
/// Transforms ServiceDefinition IR into complete provider package:
/// - provider.k (KCL manifest)
/// - Rust source code
/// - Tests
/// - Cargo.toml
/// - README.md
pub struct ProviderGenerator {
    service_def: ServiceDefinition,
    tera: Tera,
}

impl ProviderGenerator {
    /// Create a new provider generator from ServiceDefinition
    pub fn new(service_def: ServiceDefinition) -> Result<Self> {
        let tera = templates::load_templates()?;
        Ok(Self { service_def, tera })
    }

    /// Generate all provider artifacts to a directory
    pub fn generate_to_directory(&self, output_dir: &Path) -> Result<()> {
        // Create output directory structure
        fs::create_dir_all(output_dir).map_err(|e| {
            GeneratorError::Generation(format!("Failed to create output directory: {}", e))
        })?;

        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir).map_err(|e| {
            GeneratorError::Generation(format!("Failed to create src directory: {}", e))
        })?;

        let resources_dir = src_dir.join("resources");
        fs::create_dir_all(&resources_dir).map_err(|e| {
            GeneratorError::Generation(format!("Failed to create resources directory: {}", e))
        })?;

        // Generate all artifacts
        self.generate_provider_k(output_dir)?;
        self.generate_cargo_toml(output_dir)?;
        self.generate_lib_rs(&src_dir)?;
        self.generate_resources(&resources_dir)?;
        self.generate_readme(output_dir)?;

        Ok(())
    }

    /// Generate provider.k (KCL manifest)
    fn generate_provider_k(&self, output_dir: &Path) -> Result<()> {
        let context = self.create_context();
        let rendered = self
            .tera
            .render("provider.k", &context)
            .map_err(|e| GeneratorError::Generation(format!("Template error: {:?}", e)))?;

        let output_path = output_dir.join("provider.k");
        fs::write(output_path, rendered).map_err(|e| {
            GeneratorError::Generation(format!("Failed to write provider.k: {}", e))
        })?;

        Ok(())
    }

    /// Generate Cargo.toml
    fn generate_cargo_toml(&self, output_dir: &Path) -> Result<()> {
        let context = self.create_context();
        let rendered = self
            .tera
            .render("Cargo.toml", &context)
            .map_err(|e| GeneratorError::Generation(format!("Template error: {}", e)))?;

        let output_path = output_dir.join("Cargo.toml");
        fs::write(output_path, rendered).map_err(|e| {
            GeneratorError::Generation(format!("Failed to write Cargo.toml: {}", e))
        })?;

        Ok(())
    }

    /// Generate lib.rs
    fn generate_lib_rs(&self, src_dir: &Path) -> Result<()> {
        let context = self.create_context();
        let rendered = self
            .tera
            .render("lib.rs", &context)
            .map_err(|e| GeneratorError::Generation(format!("Template error: {}", e)))?;

        let output_path = src_dir.join("lib.rs");
        fs::write(output_path, rendered)
            .map_err(|e| GeneratorError::Generation(format!("Failed to write lib.rs: {}", e)))?;

        Ok(())
    }

    /// Generate resource modules
    fn generate_resources(&self, resources_dir: &Path) -> Result<()> {
        for resource in &self.service_def.resources {
            let mut context = self.create_context();
            context.insert("resource", resource);

            let rendered = self
                .tera
                .render("resource.rs", &context)
                .map_err(|e| GeneratorError::Generation(format!("Template error: {}", e)))?;

            let output_path = resources_dir.join(format!("{}.rs", resource.name));
            fs::write(output_path, rendered).map_err(|e| {
                GeneratorError::Generation(format!(
                    "Failed to write resource {}.rs: {}",
                    resource.name, e
                ))
            })?;
        }

        // Generate mod.rs for resources
        let mut context = self.create_context();
        let resource_names: Vec<&str> = self
            .service_def
            .resources
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        context.insert("resource_names", &resource_names);

        let rendered = self
            .tera
            .render("resources_mod.rs", &context)
            .map_err(|e| GeneratorError::Generation(format!("Template error: {}", e)))?;

        let output_path = resources_dir.join("mod.rs");
        fs::write(output_path, rendered).map_err(|e| {
            GeneratorError::Generation(format!("Failed to write resources/mod.rs: {}", e))
        })?;

        Ok(())
    }

    /// Generate README.md
    fn generate_readme(&self, output_dir: &Path) -> Result<()> {
        let context = self.create_context();
        let rendered = self
            .tera
            .render("README.md", &context)
            .map_err(|e| GeneratorError::Generation(format!("Template error: {}", e)))?;

        let output_path = output_dir.join("README.md");
        fs::write(output_path, rendered)
            .map_err(|e| GeneratorError::Generation(format!("Failed to write README.md: {}", e)))?;

        Ok(())
    }

    /// Create template context from ServiceDefinition
    fn create_context(&self) -> tera::Context {
        let mut context = tera::Context::new();
        context.insert("service", &self.service_def);
        context.insert("provider", &format!("{:?}", self.service_def.provider));
        context.insert("service_name", &self.service_def.name);
        context.insert("sdk_version", &self.service_def.sdk_version);
        context.insert("resources", &self.service_def.resources);
        context
    }
}

/// Generate provider artifacts (convenience function)
pub fn generate_provider(service_def: ServiceDefinition, output_path: &str) -> Result<()> {
    let generator = ProviderGenerator::new(service_def)?;
    generator.generate_to_directory(Path::new(output_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemmer_provider_generator_common::Provider;

    #[test]
    fn test_generator_creation() {
        let service_def = ServiceDefinition {
            provider: Provider::Aws,
            name: "s3".to_string(),
            sdk_version: "1.0.0".to_string(),
            resources: vec![],
        };

        let result = ProviderGenerator::new(service_def);
        assert!(result.is_ok());
    }
}
