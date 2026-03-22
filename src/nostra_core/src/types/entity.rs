use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub attributes: serde_json::Value,
    // Add other fields as needed for the VectorStore
}
