//! Protobuf/gRPC service parser
//!
//! Parses Protocol Buffer FileDescriptorSet to extract gRPC service definitions.
//!
//! ## Sources
//! - **Generated FileDescriptorSet**: Compiled from .proto files using protoc
//! - **gRPC Reflection**: Retrieved from live gRPC services
//!
//! ## Use Cases
//! - Google Cloud APIs (many expose gRPC interfaces)
//! - gRPC-first microservices
//! - Any service with .proto definitions
//!
//! ## Example
//! ```rust,ignore
//! use hemmer_provider_generator_parser::ProtobufParser;
//!
//! let parser = ProtobufParser::from_file_descriptor_set(
//!     include_bytes!("service.pb"),
//!     "my_service",
//!     "v1"
//! )?;
//! let service_def = parser.parse()?;
//! ```

mod converter;
mod parser;

pub use parser::ProtobufParser;
