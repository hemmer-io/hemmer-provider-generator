//! Google Discovery Document parser
//!
//! Parses Google Cloud Discovery Documents into ServiceDefinition IR.
//!
//! ## Discovery Document Format
//!
//! Google Cloud APIs publish "Discovery Documents" that describe REST APIs.
//! Format is based on JSON Schema Draft 3 with Google-specific extensions.
//!
//! ## Discovery Sources
//!
//! - **List all APIs**: `GET https://www.googleapis.com/discovery/v1/apis`
//! - **Get specific API**: `GET https://{service}.googleapis.com/$discovery/rest?version={version}`
//!
//! Examples:
//! - Cloud Storage: `https://storage.googleapis.com/$discovery/rest?version=v1`
//! - Compute Engine: `https://compute.googleapis.com/$discovery/rest?version=v1`
//! - BigQuery: `https://bigquery.googleapis.com/$discovery/rest?version=v2`
//!
//! ## Usage
//! ```rust,ignore
//! use hemmer_provider_generator_parser::discovery::DiscoveryParser;
//!
//! let parser = DiscoveryParser::from_file("storage-v1.json", "storage", "v1")?;
//! let service_def = parser.parse()?;
//! ```

mod converter;
mod parser;
mod types;

pub use parser::DiscoveryParser;
pub use types::*;
