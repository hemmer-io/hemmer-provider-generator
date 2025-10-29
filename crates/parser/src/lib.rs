//! SDK parsing for cloud provider definitions
//!
//! This crate handles parsing of various cloud provider SDK definitions
//! (such as AWS Smithy models) into an intermediate representation.

use hemmer_provider_generator_common::{GeneratorError, Result};

/// Parse AWS SDK definitions
pub fn parse_aws_sdk(_path: &str) -> Result<()> {
    // TODO: Implement AWS SDK parsing (Phase 2)
    Err(GeneratorError::Parse(
        "AWS SDK parsing not yet implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_stub() {
        let result = parse_aws_sdk("dummy");
        assert!(result.is_err());
    }
}
