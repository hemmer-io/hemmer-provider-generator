//! AWS SDK parser implementation
//!
//! Parses AWS SDK structure into ServiceDefinition IR.

use hemmer_provider_generator_common::{
    FieldDefinition, GeneratorError, OperationMapping, Operations, Provider, ResourceDefinition,
    Result, ServiceDefinition,
};

use crate::{CrudOperation, OperationClassifier};
use std::collections::HashMap;

/// AWS SDK parser
pub struct AwsParser {
    service_name: String,
    sdk_version: String,
}

impl AwsParser {
    /// Create a new AWS parser
    pub fn new(service_name: &str, sdk_version: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            sdk_version: sdk_version.to_string(),
        }
    }

    /// Parse the AWS service into ServiceDefinition
    ///
    /// For Phase 2 MVP, this demonstrates the parsing logic
    /// with hardcoded S3 bucket example.
    pub fn parse(&self) -> Result<ServiceDefinition> {
        // For now, return a hardcoded S3 bucket example
        // In a full implementation, this would:
        // 1. Load AWS SDK crate metadata (from docs.rs JSON or rustdoc)
        // 2. Parse operation modules
        // 3. Extract types and fields
        // 4. Group operations by resource
        // 5. Build ServiceDefinition

        if self.service_name == "s3" {
            Ok(self.parse_s3_service())
        } else {
            Err(GeneratorError::Parse(format!(
                "Service '{}' not yet supported. Currently only 's3' is implemented.",
                self.service_name
            )))
        }
    }

    /// Parse S3 service (hardcoded example for MVP)
    fn parse_s3_service(&self) -> ServiceDefinition {
        ServiceDefinition {
            provider: Provider::Aws,
            name: self.service_name.clone(),
            sdk_version: self.sdk_version.clone(),
            resources: vec![self.create_s3_bucket_resource()],
        }
    }

    /// Create S3 Bucket resource definition (example)
    fn create_s3_bucket_resource(&self) -> ResourceDefinition {
        ResourceDefinition {
            name: "bucket".to_string(),
            description: Some("S3 Bucket for object storage".to_string()),
            fields: vec![
                FieldDefinition {
                    name: "bucket".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::String,
                    required: true,
                    sensitive: false,
                    immutable: true, // Bucket name is immutable
                    description: Some("Bucket name (globally unique)".to_string()),
                },
                FieldDefinition {
                    name: "acl".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::String,
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Canned ACL to apply to the bucket".to_string()),
                },
                FieldDefinition {
                    name: "tags".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::Map(
                        Box::new(hemmer_provider_generator_common::FieldType::String),
                        Box::new(hemmer_provider_generator_common::FieldType::String),
                    ),
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Tags to apply to the bucket".to_string()),
                },
            ],
            outputs: vec![
                FieldDefinition {
                    name: "location".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::String,
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Bucket location/region".to_string()),
                },
                FieldDefinition {
                    name: "arn".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::String,
                    required: true,
                    sensitive: false,
                    immutable: true,
                    description: Some("Amazon Resource Name (ARN) of the bucket".to_string()),
                },
            ],
            operations: Operations {
                create: Some(OperationMapping {
                    sdk_operation: "create_bucket".to_string(),
                    additional_operations: vec![],
                }),
                read: Some(OperationMapping {
                    sdk_operation: "head_bucket".to_string(),
                    additional_operations: vec!["get_bucket_location".to_string()],
                }),
                update: Some(OperationMapping {
                    sdk_operation: "put_bucket_tagging".to_string(),
                    additional_operations: vec!["put_bucket_acl".to_string()],
                }),
                delete: Some(OperationMapping {
                    sdk_operation: "delete_bucket".to_string(),
                    additional_operations: vec![],
                }),
            },
        }
    }

    /// Group operations by resource name
    ///
    /// This would be used in a full implementation to automatically
    /// discover resources from operation names.
    #[allow(dead_code)]
    fn group_operations_by_resource(
        &self,
        operations: Vec<String>,
    ) -> HashMap<String, Vec<(String, CrudOperation)>> {
        let mut grouped: HashMap<String, Vec<(String, CrudOperation)>> = HashMap::new();

        for op in operations {
            if let Some(crud) = OperationClassifier::classify(&op) {
                let resource = OperationClassifier::extract_resource(&op);
                grouped
                    .entry(resource.to_string())
                    .or_default()
                    .push((op, crud));
            }
        }

        grouped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_s3_service() {
        let parser = AwsParser::new("s3", "1.0.0");
        let result = parser.parse();

        assert!(result.is_ok());
        let service = result.unwrap();

        assert_eq!(service.provider, Provider::Aws);
        assert_eq!(service.name, "s3");
        assert_eq!(service.sdk_version, "1.0.0");
        assert_eq!(service.resources.len(), 1);

        let bucket = &service.resources[0];
        assert_eq!(bucket.name, "bucket");
        assert_eq!(bucket.fields.len(), 3);
        assert_eq!(bucket.outputs.len(), 2);

        // Check operations are mapped
        assert!(bucket.operations.create.is_some());
        assert!(bucket.operations.read.is_some());
        assert!(bucket.operations.update.is_some());
        assert!(bucket.operations.delete.is_some());
    }

    #[test]
    fn test_parse_unsupported_service() {
        let parser = AwsParser::new("ec2", "1.0.0");
        let result = parser.parse();

        assert!(result.is_err());
    }

    #[test]
    fn test_group_operations() {
        let parser = AwsParser::new("s3", "1.0.0");
        let operations = vec![
            "create_bucket".to_string(),
            "delete_bucket".to_string(),
            "get_bucket_location".to_string(),
            "put_object".to_string(),
            "get_object".to_string(),
        ];

        let grouped = parser.group_operations_by_resource(operations);

        assert_eq!(grouped.len(), 2); // bucket and object
        assert!(grouped.contains_key("bucket"));
        assert!(grouped.contains_key("object"));
        assert_eq!(grouped["bucket"].len(), 3); // create, delete, get
        assert_eq!(grouped["object"].len(), 2); // put, get
    }
}
