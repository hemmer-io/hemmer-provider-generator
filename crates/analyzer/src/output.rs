//! YAML output generation with confidence annotations

use crate::{
    analyzer::AnalysisResult,
    confidence::ConfidenceReport,
    Result,
};
use chrono::Utc;
use std::fmt::Write as FmtWrite;

/// Generate annotated YAML metadata file
pub fn generate_yaml(result: &AnalysisResult) -> Result<String> {
    let mut output = String::new();

    // Header with analysis metadata
    write_header(&mut output, &result.confidence)?;

    // Version
    writeln!(output, "version: 1")?;
    writeln!(output)?;

    // Provider section
    write_provider_section(&mut output, result)?;

    // SDK section
    write_sdk_section(&mut output, result)?;

    // Config section
    write_config_section(&mut output, result)?;

    // Errors section
    write_errors_section(&mut output, result)?;

    Ok(output)
}

fn write_header(output: &mut String, confidence: &ConfidenceReport) -> Result<()> {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let level = confidence.level();

    writeln!(output, "# SDK Analysis Result")?;
    writeln!(output, "# Generated: {timestamp}")?;
    writeln!(output, "# Overall Confidence: {:.2} ({level})", confidence.overall)?;
    writeln!(output, "# Analyzer Version: 0.3.5")?;
    writeln!(output)?;

    Ok(())
}

fn write_provider_section(output: &mut String, result: &AnalysisResult) -> Result<()> {
    writeln!(output, "provider:")?;
    writeln!(output, "  name: {}", result.metadata.provider_name)?;
    writeln!(output, "  display_name: {}  # TODO: Update display name",
        capitalize(&result.metadata.provider_name))?;
    writeln!(output)?;

    Ok(())
}

fn write_sdk_section(output: &mut String, result: &AnalysisResult) -> Result<()> {
    writeln!(output, "sdk:")?;

    // Crate pattern
    let crate_conf = result.confidence.crate_pattern;
    writeln!(
        output,
        "  # Confidence: {:.2} ({})",
        crate_conf,
        confidence_label(crate_conf)
    )?;
    writeln!(
        output,
        "  crate_pattern: \"{}\"",
        result.metadata.sdk_crate_pattern
    )?;
    writeln!(output)?;

    // Client type
    let client_conf = result.confidence.client_type;
    writeln!(
        output,
        "  # Confidence: {:.2} ({})",
        client_conf,
        confidence_label(client_conf)
    )?;
    writeln!(
        output,
        "  client_type_pattern: \"{}\"",
        result.metadata.client_type_pattern
    )?;
    writeln!(output)?;

    // Config crate
    if let Some(ref config_crate) = result.metadata.config_crate {
        let config_conf = result.confidence.config_crate;
        writeln!(
            output,
            "  # Confidence: {:.2} ({})",
            config_conf,
            confidence_label(config_conf)
        )?;
        writeln!(output, "  config_crate: {config_crate}")?;
        writeln!(output)?;
    }

    // Async client
    writeln!(output, "  async_client: {}", result.metadata.async_client)?;
    writeln!(output)?;

    // Shared client for monolithic SDKs
    if result.metadata.is_monolithic {
        writeln!(output, "  # Monolithic SDK detected - uses single client for all resources")?;
        writeln!(output, "  uses_shared_client: true")?;
        writeln!(output)?;
    }

    // Dependencies (placeholder)
    writeln!(output, "  # TODO: Add SDK dependencies")?;
    writeln!(output, "  dependencies:")?;
    if let Some(ref config_crate) = result.metadata.config_crate {
        writeln!(output, "    - \"{config_crate} = \\\"1\\\"\"")?;
    }
    writeln!(output)?;

    Ok(())
}

fn write_config_section(output: &mut String, result: &AnalysisResult) -> Result<()> {
    writeln!(output, "config:")?;

    let attrs_conf = result.confidence.config_attrs;
    writeln!(
        output,
        "  # Confidence: {:.2} ({}) - Review and customize",
        attrs_conf,
        confidence_label(attrs_conf)
    )?;

    // Initialization
    writeln!(output, "  initialization:")?;
    if let Some(ref config_crate) = result.metadata.config_crate {
        let init_pattern = config_crate.replace("-config", "");
        writeln!(output, "    snippet: \"{init_pattern}_config::from_env()\"")?;
    } else {
        writeln!(output, "    snippet: \"Config::from_env()\"  # TODO: Update")?;
    }
    writeln!(output, "    var_name: config_loader")?;
    writeln!(output)?;

    // Load
    writeln!(output, "  load:")?;
    writeln!(output, "    snippet: \"config_loader.load().await\"")?;
    writeln!(output, "    var_name: sdk_config")?;
    writeln!(output)?;

    // Client from config
    writeln!(output, "  client_from_config:")?;
    writeln!(output, "    snippet: \"{{client_type}}::new(&sdk_config)\"")?;
    writeln!(output, "    var_name: client")?;
    writeln!(output)?;

    // Attributes
    if !result.metadata.config_attrs.is_empty() {
        writeln!(output, "  # Detected attributes - verify and customize")?;
        writeln!(output, "  attributes:")?;
        for attr in &result.metadata.config_attrs {
            writeln!(output, "    - name: {}", attr.name)?;
            writeln!(output, "      description: \"{}\"", attr.description)?;
            writeln!(output, "      required: {}", attr.required)?;
            writeln!(output, "      # TODO: Add setter and extractor")?;
        }
    } else {
        writeln!(output, "  # TODO: Add config attributes")?;
        writeln!(output, "  # attributes:")?;
        writeln!(output, "  #   - name: region")?;
        writeln!(output, "  #     description: \"Region for requests\"")?;
        writeln!(output, "  #     required: false")?;
        writeln!(output, "  #     setter: \"config_loader = config_loader.region({{value}})\"")?;
        writeln!(output, "  #     extractor: \"as_str()\"")?;
    }
    writeln!(output)?;

    Ok(())
}

fn write_errors_section(output: &mut String, result: &AnalysisResult) -> Result<()> {
    let error_conf = result.confidence.error_categorization;

    writeln!(output, "errors:")?;
    writeln!(
        output,
        "  # Confidence: {:.2} ({}) - Manual review required",
        error_conf,
        confidence_label(error_conf)
    )?;

    // Metadata import
    if let Some(ref import) = result.metadata.error_metadata_import {
        writeln!(output, "  metadata_import: \"{import}\"")?;
    } else {
        writeln!(output, "  # TODO: Add error metadata import if applicable")?;
        writeln!(output, "  # metadata_import: \"provider_types::error::metadata::ProvideErrorMetadata\"")?;
    }
    writeln!(output)?;

    // Categorization
    if !result.metadata.error_categorization.is_empty() {
        writeln!(output, "  # Detected error patterns - review and extend")?;
        writeln!(output, "  categorization:")?;

        for (category, errors) in &result.metadata.error_categorization {
            writeln!(output, "    {category}:")?;
            for error in errors {
                writeln!(output, "      - \"{error}\"")?;
            }
        }
    } else {
        writeln!(output, "  # TODO: Add error categorization")?;
        writeln!(output, "  # categorization:")?;
        writeln!(output, "  #   not_found:")?;
        writeln!(output, "  #     - \"NotFound\"")?;
    }

    Ok(())
}

fn confidence_label(score: f32) -> &'static str {
    match score {
        s if s >= 0.8 => "HIGH",
        s if s >= 0.6 => "MEDIUM",
        _ => "LOW",
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{AnalysisResult, AnalyzedMetadata};
    use crate::confidence::ConfidenceReport;
    use std::collections::HashMap;

    #[test]
    fn test_generate_yaml_basic() {
        let result = AnalysisResult {
            metadata: AnalyzedMetadata {
                provider_name: "aws".to_string(),
                sdk_crate_pattern: "aws-sdk-{service}".to_string(),
                client_type_pattern: "aws_sdk_{service}::Client".to_string(),
                config_crate: Some("aws-config".to_string()),
                async_client: true,
                is_monolithic: false,
                config_attrs: vec![],
                error_metadata_import: None,
                error_categorization: HashMap::new(),
            },
            confidence: ConfidenceReport::new(0.95, 0.90, 0.85, 0.0, 0.3),
            warnings: vec![],
        };

        let yaml = generate_yaml(&result).unwrap();

        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("name: aws"));
        assert!(yaml.contains("crate_pattern: \"aws-sdk-{service}\""));
        assert!(yaml.contains("client_type_pattern: \"aws_sdk_{service}::Client\""));
        assert!(yaml.contains("config_crate: aws-config"));
        assert!(yaml.contains("async_client: true"));
    }

    #[test]
    fn test_confidence_label() {
        assert_eq!(confidence_label(0.9), "HIGH");
        assert_eq!(confidence_label(0.7), "MEDIUM");
        assert_eq!(confidence_label(0.5), "LOW");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("aws"), "Aws");
        assert_eq!(capitalize("gcp"), "Gcp");
        assert_eq!(capitalize(""), "");
    }
}
