//! AWS SDK parser implementation
//!
//! Parses AWS SDK structure into ServiceDefinition IR.
//!
//! ## Usage Modes
//!
//! 1. **Hardcoded mode**: Returns predefined resource definitions (S3 bucket example)
//! 2. **Rustdoc JSON mode**: Parses rustdoc JSON output for automated discovery
//!
//! To generate rustdoc JSON:
//! ```bash
//! cargo +nightly rustdoc --package aws-sdk-s3 -- -Z unstable-options --output-format json
//! ```

use hemmer_provider_generator_common::{
    FieldDefinition, GeneratorError, OperationMapping, Operations, Provider, ResourceDefinition,
    Result, ServiceDefinition,
};

use crate::{CrudOperation, OperationClassifier, RustdocLoader};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// AWS SDK parser
pub struct AwsParser {
    service_name: String,
    sdk_version: String,
    rustdoc_json_path: Option<PathBuf>,
}

impl AwsParser {
    /// Create a new AWS parser in hardcoded mode
    pub fn new(service_name: &str, sdk_version: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            sdk_version: sdk_version.to_string(),
            rustdoc_json_path: None,
        }
    }

    /// Create a new AWS parser with rustdoc JSON path for automated parsing
    pub fn with_rustdoc_json(
        service_name: &str,
        sdk_version: &str,
        rustdoc_json_path: PathBuf,
    ) -> Self {
        Self {
            service_name: service_name.to_string(),
            sdk_version: sdk_version.to_string(),
            rustdoc_json_path: Some(rustdoc_json_path),
        }
    }

    /// Parse the AWS service into ServiceDefinition
    ///
    /// # Modes
    /// - If rustdoc_json_path is provided: Parse from rustdoc JSON (automated)
    /// - Otherwise: Use hardcoded resource definitions (S3 bucket example)
    pub fn parse(&self) -> Result<ServiceDefinition> {
        if let Some(json_path) = &self.rustdoc_json_path {
            self.parse_from_rustdoc(json_path)
        } else {
            self.parse_hardcoded()
        }
    }

    /// Parse from rustdoc JSON (automated mode)
    fn parse_from_rustdoc(&self, json_path: &Path) -> Result<ServiceDefinition> {
        // Load rustdoc JSON
        let crate_data = RustdocLoader::load_from_file(json_path)?;

        // Extract operations
        let operations = RustdocLoader::find_operation_modules(&crate_data);

        // Group operations by resource
        let grouped = self.group_operations_by_resource(operations);

        // Build resources from grouped operations
        let resources = grouped
            .into_iter()
            .map(|(resource_name, ops)| {
                self.build_resource_from_operations(&crate_data, &resource_name, ops)
            })
            .collect();

        Ok(ServiceDefinition {
            provider: Provider::Aws,
            name: self.service_name.clone(),
            sdk_version: self.sdk_version.clone(),
            resources,
            data_sources: vec![],  // Will implement data source detection later
        })
    }

    /// Parse using hardcoded resource definitions
    fn parse_hardcoded(&self) -> Result<ServiceDefinition> {
        if self.service_name == "s3" {
            Ok(self.parse_s3_service())
        } else {
            Err(GeneratorError::Parse(format!(
                "Service '{}' not yet supported in hardcoded mode. Use with_rustdoc_json() for automated parsing.",
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
            data_sources: vec![],  // Will implement data source detection later
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
                    response_accessor: None,
                },
                FieldDefinition {
                    name: "acl".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::String,
                    required: false,
                    sensitive: false,
                    immutable: false,
                    description: Some("Canned ACL to apply to the bucket".to_string()),
                    response_accessor: None,
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
                    response_accessor: None,
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
                    response_accessor: Some("location".to_string()),
                },
                FieldDefinition {
                    name: "arn".to_string(),
                    field_type: hemmer_provider_generator_common::FieldType::String,
                    required: true,
                    sensitive: false,
                    immutable: true,
                    description: Some("Amazon Resource Name (ARN) of the bucket".to_string()),
                    response_accessor: Some("arn".to_string()),
                },
            ],
            id_field: None, // Will implement ID detection later
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
                import: None, // Will implement later
            },
        }
    }

    /// Group operations by resource name
    ///
    /// Automatically discovers resources from operation names.
    fn group_operations_by_resource(
        &self,
        operations: Vec<String>,
    ) -> HashMap<String, Vec<(String, CrudOperation)>> {
        let mut grouped: HashMap<String, Vec<(String, CrudOperation)>> = HashMap::new();

        for op in operations {
            if let Some(crud) = OperationClassifier::classify(&op) {
                let resource = OperationClassifier::extract_resource(&op);
                grouped.entry(resource).or_default().push((op, crud));
            }
        }

        grouped
    }

    /// Build a resource definition from discovered operations
    ///
    /// Extracts field definitions from Input/Output types in rustdoc JSON.
    fn build_resource_from_operations(
        &self,
        crate_data: &rustdoc_types::Crate,
        resource_name: &str,
        operations: Vec<(String, CrudOperation)>,
    ) -> ResourceDefinition {
        let mut ops = Operations {
            create: None,
            read: None,
            update: None,
            delete: None,
            import: None, // Will implement later
        };

        let mut create_input_struct = None;
        let mut read_output_struct = None;

        // Map operations to CRUD
        for (op_name, crud_type) in operations {
            let mapping = OperationMapping {
                sdk_operation: op_name.clone(),
                additional_operations: vec![],
            };

            match crud_type {
                CrudOperation::Create => {
                    if ops.create.is_none() {
                        // Track the Input struct name for field extraction
                        // AWS SDK convention: {Operation}Input
                        let input_name = self.to_pascal_case(&op_name) + "Input";
                        create_input_struct = Some(input_name);
                        ops.create = Some(mapping);
                    }
                }
                CrudOperation::Read => {
                    if ops.read.is_none() {
                        // Track the Output struct name for output extraction
                        let output_name = self.to_pascal_case(&op_name) + "Output";
                        read_output_struct = Some(output_name);
                        ops.read = Some(mapping);
                    }
                }
                CrudOperation::Update => {
                    if ops.update.is_none() {
                        ops.update = Some(mapping);
                    }
                }
                CrudOperation::Delete => {
                    if ops.delete.is_none() {
                        ops.delete = Some(mapping);
                    }
                }
            }
        }

        // Extract fields from Create operation's Input struct
        let fields = if let Some(input_struct) = create_input_struct {
            RustdocLoader::extract_struct_fields(crate_data, &input_struct)
        } else {
            vec![]
        };

        // Extract outputs from Read operation's Output struct
        let outputs = if let Some(output_struct) = read_output_struct {
            RustdocLoader::extract_struct_fields(crate_data, &output_struct)
        } else {
            vec![]
        };

        ResourceDefinition {
            name: resource_name.to_string(),
            description: Some(format!(
                "{} resource (auto-discovered from SDK)",
                resource_name
            )),
            fields,
            outputs,
            id_field: None, // Will implement ID detection later
            operations: ops,
        }
    }

    /// Convert snake_case to PascalCase
    ///
    /// AWS SDK operations are in snake_case (create_bucket)
    /// but struct names are in PascalCase (CreateBucketInput)
    fn to_pascal_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

/// Implementation of SdkParser trait for AWS SDK
impl hemmer_provider_generator_common::SdkParser for AwsParser {
    fn parse(&self) -> Result<ServiceDefinition> {
        // Delegate to existing parse method
        Self::parse(self)
    }

    fn supported_services(&self) -> Vec<String> {
        // In hardcoded mode, only S3 is supported
        // In rustdoc JSON mode, any service can be parsed
        if self.rustdoc_json_path.is_some() {
            // With rustdoc JSON, any AWS service is supported
            vec![self.service_name.clone()]
        } else {
            // Hardcoded mode only supports S3
            vec!["s3".to_string()]
        }
    }

    fn metadata(&self) -> hemmer_provider_generator_common::SdkMetadata {
        hemmer_provider_generator_common::SdkMetadata {
            provider: Provider::Aws,
            sdk_version: self.sdk_version.clone(),
            sdk_name: "aws-sdk-rust".to_string(),
        }
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

    #[test]
    fn test_sdk_parser_trait() {
        use hemmer_provider_generator_common::SdkParser;

        let parser = AwsParser::new("s3", "1.0.0");

        // Test parse method through trait
        let result = SdkParser::parse(&parser);
        assert!(result.is_ok());

        // Test supported_services
        let services = parser.supported_services();
        assert_eq!(services, vec!["s3"]);

        // Test metadata
        let metadata = parser.metadata();
        assert_eq!(metadata.provider, Provider::Aws);
        assert_eq!(metadata.sdk_version, "1.0.0");
        assert_eq!(metadata.sdk_name, "aws-sdk-rust");
    }

    #[test]
    fn test_sdk_parser_trait_with_rustdoc() {
        use hemmer_provider_generator_common::SdkParser;
        use std::path::PathBuf;

        let parser =
            AwsParser::with_rustdoc_json("s3", "1.0.0", PathBuf::from("/path/to/rustdoc.json"));

        // Test supported_services with rustdoc JSON path
        let services = parser.supported_services();
        assert_eq!(services, vec!["s3"]);

        // Test metadata
        let metadata = parser.metadata();
        assert_eq!(metadata.provider, Provider::Aws);
        assert_eq!(metadata.sdk_name, "aws-sdk-rust");
    }
}
