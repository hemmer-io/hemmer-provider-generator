//! OpenAPI 3.0 specification parser
//!
//! Parses OpenAPI 3.0 specs (Swagger) into ServiceDefinition IR.
//!
//! ## Supported Platforms
//! - **Kubernetes**: Parse OpenAPI specs from K8s API server
//! - **Azure**: Parse OpenAPI specs from azure-rest-api-specs repo
//! - **Generic**: Any OpenAPI 3.0 compliant specification
//!
//! ## OpenAPI Sources
//!
//! ### Kubernetes
//! - From cluster: `kubectl proxy && curl http://localhost:8001/openapi/v2`
//! - From GitHub: `https://github.com/kubernetes/kubernetes/blob/master/api/openapi-spec/swagger.json`
//!
//! ### Azure
//! - Repository: `https://github.com/Azure/azure-rest-api-specs`
//! - Example: `specification/compute/resource-manager/Microsoft.Compute/stable/2021-11-01/compute.json`
//!
//! ## Usage
//! ```rust,ignore
//! use hemmer_provider_generator_parser::openapi::OpenApiParser;
//!
//! let parser = OpenApiParser::from_file("k8s-openapi.json", "kubernetes", "1.27.0")?;
//! let service_def = parser.parse()?;
//! ```

mod converter;
mod parser;
mod types;

pub use parser::{OpenApiParser, ProviderHint};
pub use types::*;
