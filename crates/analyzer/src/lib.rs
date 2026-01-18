//! SDK Analyzer for Hemmer Provider Generator
//!
//! Automatically generates provider metadata YAML files by analyzing SDK repositories.
//!
//! # Examples
//!
//! ```no_run
//! use hemmer_provider_generator_analyzer::SdkAnalyzer;
//! use std::path::PathBuf;
//!
//! let analyzer = SdkAnalyzer::new(
//!     PathBuf::from("./aws-sdk-rust"),
//!     "aws".to_string()
//! );
//! let result = analyzer.analyze().expect("Analysis failed");
//!
//! println!("Overall confidence: {:.2}", result.confidence.overall);
//! result.write_yaml("providers/aws.sdk-metadata.yaml").expect("Write failed");
//! ```

mod analyzer;
mod client_detector;
mod confidence;
mod config_detector;
mod crate_pattern_detector;
mod error_detector;
pub mod git_cloner;
mod output;
mod workspace_detector;

pub use analyzer::{
    AnalysisResult, AnalysisWarning, AnalyzedConfigAttr, AnalyzedMetadata, SdkAnalyzer,
};
pub use confidence::ConfidenceReport;
pub use git_cloner::ClonedRepo;

use thiserror::Error;

/// Errors that can occur during SDK analysis
#[derive(Debug, Error)]
pub enum AnalyzerError {
    #[error("Failed to read SDK directory: {0}")]
    DirectoryRead(#[from] std::io::Error),

    #[error("Failed to parse Cargo metadata: {0}")]
    CargoMetadata(String),

    #[error("Failed to parse Rust source: {0}")]
    SynParse(String),

    #[error("No SDK crates found in workspace")]
    NoSdkCrates,

    #[error("Failed to detect pattern: {0}")]
    PatternDetection(String),

    #[error("Failed to generate YAML: {0}")]
    YamlGeneration(#[from] serde_yaml::Error),

    #[error("Failed to format output: {0}")]
    FormatError(#[from] std::fmt::Error),

    #[error("Analysis error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AnalyzerError>;
