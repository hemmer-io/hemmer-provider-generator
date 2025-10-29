//! Smithy specification parser
//!
//! Parses AWS Smithy JSON AST format into ServiceDefinition IR.
//!
//! Smithy specs are available at: https://github.com/aws/api-models-aws
//!
//! ## Format
//! Smithy JSON AST contains:
//! - Service definitions with operations
//! - Shape definitions (structures, operations, primitives)
//! - Traits (metadata like documentation, HTTP bindings)
//!
//! ## Usage
//! ```rust,ignore
//! use hemmer_provider_generator_parser::smithy::SmithyParser;
//!
//! let parser = SmithyParser::from_file("api-models-aws/s3/2006-03-01/s3-2006-03-01.json")?;
//! let service_def = parser.parse()?;
//! ```

mod converter;
mod parser;
mod types;

pub use parser::SmithyParser;
pub use types::*;
