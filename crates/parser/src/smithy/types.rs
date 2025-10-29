//! Smithy JSON AST type definitions
//!
//! These types represent the structure of Smithy JSON files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root Smithy model document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmithyModel {
    /// Smithy version (e.g., "2.0")
    pub smithy: String,

    /// Shape definitions (operations, structures, services, etc.)
    #[serde(default)]
    pub shapes: HashMap<String, Shape>,

    /// Metadata about the model
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A Smithy shape (can be service, operation, structure, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Shape {
    /// Service definition
    Service {
        /// API version
        #[serde(default)]
        version: Option<String>,

        /// Operations exposed by this service
        #[serde(default)]
        operations: Vec<ShapeReference>,

        /// Resources managed by this service
        #[serde(default)]
        resources: Vec<ShapeReference>,

        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Operation definition
    Operation {
        /// Input shape
        #[serde(default)]
        input: Option<ShapeReference>,

        /// Output shape
        #[serde(default)]
        output: Option<ShapeReference>,

        /// Error shapes
        #[serde(default)]
        errors: Vec<ShapeReference>,

        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Structure definition (input/output types)
    Structure {
        /// Member fields
        #[serde(default)]
        members: HashMap<String, Member>,

        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// String type
    String {
        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Integer type
    Integer {
        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Long type
    Long {
        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Boolean type
    Boolean {
        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Double/Float type
    Double {
        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Timestamp type
    Timestamp {
        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// List type
    List {
        /// Member type
        member: ShapeReference,

        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Map type
    Map {
        /// Key type
        key: ShapeReference,

        /// Value type
        value: ShapeReference,

        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Resource definition (e.g., S3 Bucket)
    Resource {
        /// Identifiers for this resource
        #[serde(default)]
        identifiers: HashMap<String, ShapeReference>,

        /// Create operation
        #[serde(default)]
        create: Option<ShapeReference>,

        /// Read operation
        #[serde(default)]
        read: Option<ShapeReference>,

        /// Update operation
        #[serde(default)]
        update: Option<ShapeReference>,

        /// Delete operation
        #[serde(default)]
        delete: Option<ShapeReference>,

        /// List operation
        #[serde(default)]
        list: Option<ShapeReference>,

        /// Put operation
        #[serde(default)]
        put: Option<ShapeReference>,

        /// Traits (metadata)
        #[serde(default)]
        traits: HashMap<String, serde_json::Value>,
    },

    /// Fallback for other shape types
    #[serde(other)]
    Other,
}

/// Reference to another shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeReference {
    /// Target shape ID (e.g., "com.amazonaws.s3#Bucket")
    pub target: String,
}

/// Structure member definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    /// Target shape for this member
    pub target: String,

    /// Traits (metadata)
    #[serde(default)]
    pub traits: HashMap<String, serde_json::Value>,
}

impl SmithyModel {
    /// Find the service shape in the model
    pub fn find_service(&self) -> Option<(&String, &Shape)> {
        self.shapes
            .iter()
            .find(|(_, shape)| matches!(shape, Shape::Service { .. }))
    }

    /// Get a shape by its ID
    pub fn get_shape(&self, shape_id: &str) -> Option<&Shape> {
        self.shapes.get(shape_id)
    }

    /// Extract service name from shape ID
    /// e.g., "com.amazonaws.s3#S3" -> "s3"
    pub fn extract_service_name(shape_id: &str) -> String {
        if let Some(hash_pos) = shape_id.rfind('#') {
            let after_hash = &shape_id[hash_pos + 1..];
            after_hash.to_lowercase()
        } else {
            shape_id.to_lowercase()
        }
    }
}

/// Common Smithy trait names
pub mod traits {
    pub const DOCUMENTATION: &str = "smithy.api#documentation";
    pub const REQUIRED: &str = "smithy.api#required";
    pub const READONLY: &str = "smithy.api#readonly";
    pub const SENSITIVE: &str = "smithy.api#sensitive";
    pub const HTTP: &str = "smithy.api#http";
    pub const HTTP_PAYLOAD: &str = "smithy.api#httpPayload";
    pub const IDEMPOTENT: &str = "smithy.api#idempotent";
    pub const PAGINATED: &str = "smithy.api#paginated";
}
