//! Nostra Book V1 Types
//!
//! Rust representations of the `nostra-book-v1.schema.json` schema.
//! These types allow for strongly-typed deserialization of compiled knowledge artifacts.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// The root "Book" artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostraBook {
    /// JSON-LD context (array of strings or objects)
    #[serde(rename = "@context")]
    pub context: Vec<JsonValue>,

    /// Book metadata (provenance, phase, version)
    pub meta: BookMeta,

    /// Optional configuration for memory/resilience
    #[serde(default)]
    pub config: Option<BookConfig>,

    /// Table of Contents structure
    pub structure: BookStructure,

    /// The actual chapter content
    pub content: Vec<Chapter>,

    /// Optional embedded knowledge graph
    #[serde(default)]
    pub knowledge_graph: Option<KnowledgeGraph>,
}

/// Metadata for a Book artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMeta {
    /// URN identifier (e.g., "urn:nostra:book:security-privacy")
    pub id: String,

    /// Type constant ("Contribution::Book")
    #[serde(rename = "type")]
    pub book_type: String,

    /// SHA256 hash of the content version
    pub version_hash: String,

    /// Contribution phase (Exploratory, Deliberative, Decisive, Executable, Archival)
    pub phase: ContributionPhase,

    /// Access policy (optional)
    #[serde(default)]
    pub access_policy: Option<AccessPolicy>,

    /// Provenance information
    pub provenance: Provenance,
}

/// Contribution lifecycle phases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContributionPhase {
    Exploratory,
    Deliberative,
    Decisive,
    Executable,
    Archival,
}

/// Access control policy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessPolicy {
    #[serde(rename = "Space::Public")]
    Public,
    #[serde(rename = "Space::Private")]
    Private,
    #[serde(rename = "Space::Gated")]
    Gated,
}

/// Provenance (authorship) information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// DID of the author
    pub author_did: String,

    /// DID of the owning Space
    pub space_did: String,

    /// ISO 8601 timestamp
    pub created_at: String,
}

/// Book configuration (memory, resilience).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookConfig {
    #[serde(default)]
    pub memory_allocation: Option<MemoryAllocation>,

    #[serde(default)]
    pub resilience: Option<ResilienceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    #[serde(default)]
    pub primary_storage: Option<String>,
    #[serde(default)]
    pub cache_layer: Option<String>,
    #[serde(default)]
    pub vector_index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    #[serde(default)]
    pub offline_mode: Option<String>,
    #[serde(default)]
    pub sync_priority: Option<String>,
}

/// Book structure (Table of Contents).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStructure {
    pub toc: Vec<TocEntry>,
}

/// A single entry in the Table of Contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    pub id: String,
    pub title: String,
    pub level: u8,
    pub link: String,
}

/// A chapter within the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,

    #[serde(rename = "type")]
    pub chapter_type: String,

    #[serde(default)]
    pub version: Option<String>,

    pub blocks: Vec<Block>,
}

/// A content block within a chapter.
/// This is a semi-structured type to handle the polymorphic `content` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block type (e.g., "Block::Heading", "Block::Paragraph", "Block::List")
    #[serde(rename = "type")]
    pub block_type: String,

    /// Block content (can be string or array of objects)
    #[serde(default)]
    pub content: Option<JsonValue>,

    /// Heading level (for headings)
    #[serde(default)]
    pub level: Option<u8>,

    /// Reference ID (for citations, links)
    #[serde(default)]
    pub ref_id: Option<String>,

    /// Display text (for links)
    #[serde(default)]
    pub display_text: Option<String>,
}

/// Embedded knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGraph {
    #[serde(default)]
    pub entities: Vec<Entity>,

    #[serde(default)]
    pub relations: Vec<Relation>,
}

/// An entity in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,

    #[serde(rename = "type")]
    pub entity_type: String,

    pub properties: JsonValue,
}

/// A relation between entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_book() {
        let json = r#"{
            "@context": ["https://nostra.network/ns/book"],
            "meta": {
                "id": "urn:nostra:book:test",
                "type": "Contribution::Book",
                "version_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                "phase": "Exploratory",
                "provenance": {
                    "author_did": "did:nostra:user:test",
                    "space_did": "did:nostra:space:core",
                    "created_at": "2026-01-01T00:00:00Z"
                }
            },
            "structure": {
                "toc": []
            },
            "content": []
        }"#;

        let book: NostraBook = serde_json::from_str(json).expect("Failed to parse minimal book");
        assert_eq!(book.meta.id, "urn:nostra:book:test");
        assert_eq!(book.meta.phase, ContributionPhase::Exploratory);
        assert!(book.content.is_empty());
    }
}
