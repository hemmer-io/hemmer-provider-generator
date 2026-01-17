//! Type mapping from SDK types to intermediate representation
//!
//! Maps AWS SDK Rust types to our `FieldType` IR.

use hemmer_provider_generator_common::FieldType;

/// Maps SDK type names to FieldType
pub struct TypeMapper;

impl TypeMapper {
    /// Map a Rust type string to FieldType
    ///
    /// # Examples
    /// ```
    /// use hemmer_provider_generator_parser::TypeMapper;
    /// use hemmer_provider_generator_common::FieldType;
    ///
    /// assert_eq!(TypeMapper::map_type("String"), FieldType::String);
    /// assert_eq!(TypeMapper::map_type("i64"), FieldType::Integer);
    /// assert_eq!(TypeMapper::map_type("bool"), FieldType::Boolean);
    /// ```
    pub fn map_type(rust_type: &str) -> FieldType {
        // Remove Option wrapper
        let type_str = rust_type
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(rust_type);

        match type_str {
            "String" | "&str" | "str" => FieldType::String,
            "i32" | "i64" | "u32" | "u64" => FieldType::Integer,
            "f32" | "f64" => FieldType::Float,
            "bool" => FieldType::Boolean,
            s if s.starts_with("Vec<") => {
                let inner = s
                    .strip_prefix("Vec<")
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or("String");
                FieldType::List(Box::new(Self::map_type(inner)))
            },
            s if s.starts_with("HashMap<") => {
                // Parse HashMap<K, V>
                let inner = s
                    .strip_prefix("HashMap<")
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or("String, String");
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    FieldType::Map(
                        Box::new(Self::map_type(parts[0])),
                        Box::new(Self::map_type(parts[1])),
                    )
                } else {
                    // Default to String -> String map
                    FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::String))
                }
            },
            "DateTime" => FieldType::DateTime,
            _ => {
                // Unknown types default to String (enum variants, custom types)
                FieldType::String
            },
        }
    }

    /// Check if a type is optional (wrapped in Option<>)
    pub fn is_optional(rust_type: &str) -> bool {
        rust_type.starts_with("Option<")
    }

    /// Check if a field name suggests it's sensitive
    pub fn is_sensitive(field_name: &str) -> bool {
        let lower = field_name.to_lowercase();
        lower.contains("password")
            || lower.contains("secret")
            || lower.contains("key")
            || lower.contains("token")
            || lower.contains("credential")
    }

    /// Check if a field name suggests it's immutable
    pub fn is_immutable(field_name: &str) -> bool {
        let lower = field_name.to_lowercase();
        // Common immutable fields in AWS
        lower == "name"
            || lower == "id"
            || lower == "arn"
            || lower.ends_with("_name")
            || lower.ends_with("_id")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_basic_types() {
        assert_eq!(TypeMapper::map_type("String"), FieldType::String);
        assert_eq!(TypeMapper::map_type("i64"), FieldType::Integer);
        assert_eq!(TypeMapper::map_type("bool"), FieldType::Boolean);
        assert_eq!(TypeMapper::map_type("f64"), FieldType::Float);
        assert_eq!(TypeMapper::map_type("DateTime"), FieldType::DateTime);
    }

    #[test]
    fn test_map_optional() {
        assert_eq!(TypeMapper::map_type("Option<String>"), FieldType::String);
        assert_eq!(TypeMapper::map_type("Option<i64>"), FieldType::Integer);
    }

    #[test]
    fn test_map_collections() {
        assert_eq!(
            TypeMapper::map_type("Vec<String>"),
            FieldType::List(Box::new(FieldType::String))
        );
        assert_eq!(
            TypeMapper::map_type("HashMap<String, i64>"),
            FieldType::Map(Box::new(FieldType::String), Box::new(FieldType::Integer))
        );
    }

    #[test]
    fn test_is_optional() {
        assert!(TypeMapper::is_optional("Option<String>"));
        assert!(!TypeMapper::is_optional("String"));
    }

    #[test]
    fn test_is_sensitive() {
        assert!(TypeMapper::is_sensitive("password"));
        assert!(TypeMapper::is_sensitive("secret_key"));
        assert!(TypeMapper::is_sensitive("access_token"));
        assert!(!TypeMapper::is_sensitive("bucket_name"));
    }

    #[test]
    fn test_is_immutable() {
        assert!(TypeMapper::is_immutable("name"));
        assert!(TypeMapper::is_immutable("bucket_name"));
        assert!(TypeMapper::is_immutable("id"));
        assert!(!TypeMapper::is_immutable("acl"));
    }
}
