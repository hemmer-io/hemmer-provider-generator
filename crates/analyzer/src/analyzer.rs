//! Core SDK analysis orchestration

use crate::{
    client_detector, config_detector, confidence::ConfidenceReport, crate_pattern_detector,
    error_detector, output, workspace_detector::WorkspaceInfo, AnalyzerError, Result,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Analyzed provider metadata (simplified for analyzer output)
#[derive(Debug, Clone)]
pub struct AnalyzedMetadata {
    pub provider_name: String,
    pub sdk_crate_pattern: String,
    pub client_type_pattern: String,
    pub config_crate: Option<String>,
    pub async_client: bool,
    pub config_attrs: Vec<AnalyzedConfigAttr>,
    pub error_metadata_import: Option<String>,
    pub error_categorization: HashMap<String, Vec<String>>,
}

/// Simplified config attribute for analyzer output
#[derive(Debug, Clone)]
pub struct AnalyzedConfigAttr {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// SDK analyzer - main entry point
pub struct SdkAnalyzer {
    repo_path: PathBuf,
    provider_name: String,
    verbose: bool,
}

/// Complete analysis result
pub struct AnalysisResult {
    /// Generated provider metadata
    pub metadata: AnalyzedMetadata,
    /// Confidence scores for each field
    pub confidence: ConfidenceReport,
    /// Warnings that need manual review
    pub warnings: Vec<AnalysisWarning>,
}

/// Warning about analysis results
#[derive(Debug, Clone)]
pub enum AnalysisWarning {
    LowConfidence { field: String, score: f32 },
    NoPattern { field: String },
    RequiresReview { field: String, reason: String },
}

impl SdkAnalyzer {
    /// Create a new SDK analyzer
    pub fn new(repo_path: PathBuf, provider_name: String) -> Self {
        Self {
            repo_path,
            provider_name,
            verbose: false,
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Run the full analysis pipeline
    pub fn analyze(&self) -> Result<AnalysisResult> {
        self.log("Starting SDK analysis...");

        // Phase 1: Detect workspace structure
        self.log("Detecting workspace structure...");
        let workspace = WorkspaceInfo::detect(&self.repo_path)?;

        self.log(&format!(
            "Found {} packages (workspace: {})",
            workspace.packages.len(),
            workspace.is_workspace
        ));

        // Filter to SDK crates
        let sdk_crates = workspace.sdk_crates();
        if sdk_crates.is_empty() {
            return Err(AnalyzerError::NoSdkCrates);
        }

        self.log(&format!("Identified {} SDK crates", sdk_crates.len()));

        // Phase 2: Detect crate pattern
        self.log("Detecting crate pattern...");
        let crate_pattern = crate_pattern_detector::detect_pattern(&sdk_crates);
        self.log(&format!(
            "Pattern: {} (confidence: {:.2})",
            crate_pattern.pattern, crate_pattern.confidence
        ));

        // Phase 3: Detect client type pattern
        self.log("Detecting client type pattern...");
        let client_pattern =
            client_detector::detect_pattern(&self.repo_path, &sdk_crates).unwrap_or_else(|e| {
                self.log(&format!("Client detection warning: {e}"));
                // Fallback to inference
                client_detector::ClientPattern {
                    pattern: "unknown::Client".to_string(),
                    confidence: 0.3,
                    samples: vec![],
                    async_client: true,
                }
            });
        self.log(&format!(
            "Client pattern: {} (confidence: {:.2})",
            client_pattern.pattern, client_pattern.confidence
        ));

        // Phase 4: Detect config patterns
        self.log("Detecting configuration patterns...");
        let config_info = config_detector::detect_config(&self.repo_path, &workspace)?;
        if let Some(ref config_crate) = config_info.config_crate {
            self.log(&format!("Config crate: {config_crate}"));
        }
        self.log(&format!(
            "Found {} config attributes",
            config_info.attributes.len()
        ));

        // Phase 5: Detect error categorization
        self.log("Detecting error patterns...");
        let error_info = if let Some(first_crate) = sdk_crates.first() {
            error_detector::detect_errors(&self.repo_path, &first_crate.name)
                .unwrap_or_else(|e| {
                    self.log(&format!("Error detection warning: {e}"));
                    error_detector::ErrorInfo {
                        metadata_import: None,
                        categorization: HashMap::new(),
                        confidence: 0.3,
                    }
                })
        } else {
            error_detector::ErrorInfo {
                metadata_import: None,
                categorization: HashMap::new(),
                confidence: 0.3,
            }
        };
        self.log(&format!(
            "Categorized {} error types",
            error_info.categorization.len()
        ));

        // Convert config attributes
        let config_attrs: Vec<AnalyzedConfigAttr> = config_info
            .attributes
            .iter()
            .map(|attr| AnalyzedConfigAttr {
                name: attr.name.clone(),
                description: attr.description.clone(),
                required: attr.required,
            })
            .collect();

        // Build metadata
        let metadata = AnalyzedMetadata {
            provider_name: self.provider_name.clone(),
            sdk_crate_pattern: crate_pattern.pattern.clone(),
            client_type_pattern: client_pattern.pattern.clone(),
            config_crate: config_info.config_crate.clone(),
            async_client: config_info.async_client,
            config_attrs,
            error_metadata_import: error_info.metadata_import.clone(),
            error_categorization: error_info.categorization.clone(),
        };

        // Calculate overall confidence
        let confidence = ConfidenceReport::new(
            crate_pattern.confidence,
            client_pattern.confidence,
            config_info.config_crate_confidence,
            config_info.attributes_confidence,
            error_info.confidence,
        );

        self.log(&format!(
            "Overall confidence: {:.2} ({})",
            confidence.overall,
            confidence.level()
        ));

        // Generate warnings
        let warnings = self.generate_warnings(&confidence);

        Ok(AnalysisResult {
            metadata,
            confidence,
            warnings,
        })
    }

    /// Generate warnings for low confidence fields
    fn generate_warnings(&self, confidence: &ConfidenceReport) -> Vec<AnalysisWarning> {
        let mut warnings = Vec::new();

        let fields = [
            ("crate_pattern", confidence.crate_pattern),
            ("client_type", confidence.client_type),
            ("config_crate", confidence.config_crate),
            ("config_attrs", confidence.config_attrs),
            ("error_categorization", confidence.error_categorization),
        ];

        for (field, score) in fields {
            if score < 0.7 {
                warnings.push(AnalysisWarning::LowConfidence {
                    field: field.to_string(),
                    score,
                });
            }
        }

        // Always warn about error categorization
        warnings.push(AnalysisWarning::RequiresReview {
            field: "error_categorization".to_string(),
            reason: "Error categorization always requires manual review and testing".to_string(),
        });

        warnings
    }

    /// Log message if verbose
    fn log(&self, message: &str) {
        if self.verbose {
            eprintln!("{message}");
        }
    }
}

impl AnalysisResult {
    /// Write result to YAML file
    pub fn write_yaml(&self, output_path: &str) -> Result<()> {
        let yaml_content = output::generate_yaml(self)?;
        fs::write(output_path, yaml_content)?;
        Ok(())
    }

    /// Get YAML as string
    pub fn to_yaml(&self) -> Result<String> {
        output::generate_yaml(self)
    }

    /// Print warnings to stderr
    pub fn print_warnings(&self) {
        if self.warnings.is_empty() {
            return;
        }

        eprintln!("\nWarnings:");
        for warning in &self.warnings {
            match warning {
                AnalysisWarning::LowConfidence { field, score } => {
                    eprintln!("  ⚠ {field}: Low confidence ({score:.2}) - needs review");
                }
                AnalysisWarning::NoPattern { field } => {
                    eprintln!("  ⚠ {field}: No pattern detected");
                }
                AnalysisWarning::RequiresReview { field, reason } => {
                    eprintln!("  ⚠ {field}: {reason}");
                }
            }
        }
    }

    /// Print summary to stderr
    pub fn print_summary(&self) {
        eprintln!("\n✓ Analysis complete");
        eprintln!("  Overall confidence: {:.2} ({})",
            self.confidence.overall,
            self.confidence.level()
        );
        eprintln!("  Crate pattern: {} ({:.2})",
            self.metadata.sdk_crate_pattern,
            self.confidence.crate_pattern
        );
        eprintln!("  Client pattern: {} ({:.2})",
            self.metadata.client_type_pattern,
            self.confidence.client_type
        );

        if let Some(ref config) = self.metadata.config_crate {
            eprintln!("  Config crate: {config}");
        }

        let automation = self.calculate_automation_percentage();
        eprintln!("\n  Automation: {automation}% ({}/9 fields)",
            (automation as f32 / 100.0 * 9.0) as usize
        );
    }

    /// Calculate automation percentage (fields with >0.7 confidence)
    fn calculate_automation_percentage(&self) -> usize {
        let total_fields = 9.0; // Total important fields
        let mut automated = 0.0;

        if self.confidence.crate_pattern >= 0.7 {
            automated += 1.0;
        }
        if self.confidence.client_type >= 0.7 {
            automated += 1.0;
        }
        if self.confidence.config_crate >= 0.7 {
            automated += 1.0;
        }
        if self.confidence.config_attrs >= 0.7 {
            automated += 1.0;
        }
        // Error categorization gets partial credit
        if self.confidence.error_categorization >= 0.5 {
            automated += 0.5;
        }

        ((automated / total_fields) * 100.0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_calculation() {
        let result = AnalysisResult {
            metadata: AnalyzedMetadata {
                provider_name: "aws".to_string(),
                sdk_crate_pattern: "aws-sdk-{service}".to_string(),
                client_type_pattern: "aws_sdk_{service}::Client".to_string(),
                config_crate: Some("aws-config".to_string()),
                async_client: true,
                config_attrs: vec![],
                error_metadata_import: None,
                error_categorization: HashMap::new(),
            },
            confidence: ConfidenceReport::new(0.95, 0.90, 0.85, 0.60, 0.50),
            warnings: vec![],
        };

        // crate (0.95 ✓) + client (0.90 ✓) + config_crate (0.85 ✓) + error (0.50 ✓ partial)
        // = 3.5 / 9 = ~38%
        let automation = result.calculate_automation_percentage();
        assert!((35..=40).contains(&automation));
    }

    #[test]
    fn test_warning_generation() {
        let analyzer = SdkAnalyzer::new(PathBuf::from("."), "test".to_string());
        let confidence = ConfidenceReport::new(0.95, 0.65, 0.50, 0.40, 0.30);

        let warnings = analyzer.generate_warnings(&confidence);

        // Should have warnings for fields <0.7
        assert!(warnings.iter().any(|w| matches!(
            w,
            AnalysisWarning::LowConfidence { field, .. } if field == "client_type"
        )));
        assert!(warnings.iter().any(|w| matches!(
            w,
            AnalysisWarning::LowConfidence { field, .. } if field == "config_crate"
        )));
    }
}
