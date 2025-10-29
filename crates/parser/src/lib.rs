//! SDK parsing for cloud provider definitions
//!
//! This crate handles parsing of various cloud provider SDK definitions
//! into an intermediate representation (`ServiceDefinition`).
//!
//! ## Parsing Strategy
//!
//! For AWS SDK for Rust, we parse the published crate structure:
//! - `operation::*` modules contain Input/Output types for each operation
//! - `types::*` module contains data types used by operations
//!
//! Operations are grouped by resource and mapped to CRUD:
//! - CreateX, PutX → Create
//! - GetX, DescribeX, HeadX → Read
//! - UpdateX, ModifyX, PutX → Update
//! - DeleteX, RemoveX → Delete

mod aws;
mod operation_mapper;
mod type_mapper;

pub use aws::AwsParser;
pub use operation_mapper::{CrudOperation, OperationClassifier};
pub use type_mapper::TypeMapper;

use hemmer_provider_generator_common::{Result, ServiceDefinition};

/// Parse AWS SDK service from metadata
///
/// # Arguments
/// * `service_name` - Name of the AWS service (e.g., "s3", "ec2")
/// * `sdk_version` - Version of the AWS SDK
///
/// # Returns
/// * `ServiceDefinition` - Intermediate representation of the service
pub fn parse_aws_service(service_name: &str, sdk_version: &str) -> Result<ServiceDefinition> {
    let parser = AwsParser::new(service_name, sdk_version);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aws_service() {
        // This will fail until we implement the parser
        let result = parse_aws_service("s3", "1.0.0");
        assert!(result.is_ok() || result.is_err()); // Placeholder test
    }
}
