//! Operation classification and CRUD mapping
//!
//! Maps AWS SDK operation names to CRUD operations.

/// CRUD operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudOperation {
    Create,
    Read,
    Update,
    Delete,
}

/// Classifies SDK operations into CRUD operations
pub struct OperationClassifier;

impl OperationClassifier {
    /// Classify an operation name into a CRUD operation
    ///
    /// # Examples
    /// ```
    /// use hemmer_provider_generator_parser::OperationClassifier;
    /// use hemmer_provider_generator_parser::CrudOperation;
    ///
    /// assert_eq!(
    ///     OperationClassifier::classify("create_bucket"),
    ///     Some(CrudOperation::Create)
    /// );
    /// assert_eq!(
    ///     OperationClassifier::classify("get_bucket_location"),
    ///     Some(CrudOperation::Read)
    /// );
    /// ```
    pub fn classify(operation_name: &str) -> Option<CrudOperation> {
        let lower = operation_name.to_lowercase();

        // Create operations
        if lower.starts_with("create") || lower.starts_with("put") {
            // PutObject is create, but PutBucketAcl might be update
            // For now, treat all put_* as potentially create
            return Some(CrudOperation::Create);
        }

        // Read operations
        if lower.starts_with("get")
            || lower.starts_with("describe")
            || lower.starts_with("head")
            || lower.starts_with("list")
        {
            return Some(CrudOperation::Read);
        }

        // Update operations
        if lower.starts_with("update") || lower.starts_with("modify") {
            return Some(CrudOperation::Update);
        }

        // Delete operations
        if lower.starts_with("delete") || lower.starts_with("remove") {
            return Some(CrudOperation::Delete);
        }

        None
    }

    /// Extract resource name from operation name
    ///
    /// # Examples
    /// ```
    /// use hemmer_provider_generator_parser::OperationClassifier;
    ///
    /// assert_eq!(
    ///     OperationClassifier::extract_resource("create_bucket"),
    ///     "bucket"
    /// );
    /// assert_eq!(
    ///     OperationClassifier::extract_resource("put_bucket_acl"),
    ///     "bucket"
    /// );
    /// ```
    pub fn extract_resource(operation_name: &str) -> String {
        let lower = operation_name.to_lowercase();

        // Remove common operation prefixes
        let without_prefix = lower
            .strip_prefix("create_")
            .or_else(|| lower.strip_prefix("get_"))
            .or_else(|| lower.strip_prefix("put_"))
            .or_else(|| lower.strip_prefix("describe_"))
            .or_else(|| lower.strip_prefix("head_"))
            .or_else(|| lower.strip_prefix("list_"))
            .or_else(|| lower.strip_prefix("update_"))
            .or_else(|| lower.strip_prefix("modify_"))
            .or_else(|| lower.strip_prefix("delete_"))
            .or_else(|| lower.strip_prefix("remove_"))
            .unwrap_or(&lower);

        // Take first word as resource name (e.g., "bucket" from "bucket_acl")
        without_prefix
            .split('_')
            .next()
            .unwrap_or(without_prefix)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_create() {
        assert_eq!(
            OperationClassifier::classify("create_bucket"),
            Some(CrudOperation::Create)
        );
        assert_eq!(
            OperationClassifier::classify("put_object"),
            Some(CrudOperation::Create)
        );
    }

    #[test]
    fn test_classify_read() {
        assert_eq!(
            OperationClassifier::classify("get_bucket_location"),
            Some(CrudOperation::Read)
        );
        assert_eq!(
            OperationClassifier::classify("describe_instances"),
            Some(CrudOperation::Read)
        );
        assert_eq!(
            OperationClassifier::classify("head_bucket"),
            Some(CrudOperation::Read)
        );
        assert_eq!(
            OperationClassifier::classify("list_buckets"),
            Some(CrudOperation::Read)
        );
    }

    #[test]
    fn test_classify_update() {
        assert_eq!(
            OperationClassifier::classify("update_bucket"),
            Some(CrudOperation::Update)
        );
        assert_eq!(
            OperationClassifier::classify("modify_instance_attribute"),
            Some(CrudOperation::Update)
        );
    }

    #[test]
    fn test_classify_delete() {
        assert_eq!(
            OperationClassifier::classify("delete_bucket"),
            Some(CrudOperation::Delete)
        );
        assert_eq!(
            OperationClassifier::classify("remove_tags"),
            Some(CrudOperation::Delete)
        );
    }

    #[test]
    fn test_extract_resource() {
        assert_eq!(
            OperationClassifier::extract_resource("create_bucket"),
            "bucket"
        );
        assert_eq!(
            OperationClassifier::extract_resource("get_bucket_location"),
            "bucket"
        );
        assert_eq!(
            OperationClassifier::extract_resource("put_object"),
            "object"
        );
        assert_eq!(
            OperationClassifier::extract_resource("delete_bucket"),
            "bucket"
        );
    }
}
