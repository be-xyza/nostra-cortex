//! Nostra Book Parser
//!
//! Rust structs for deserializing `nostra-book-v1.schema.json` artifacts.
//! Part of the Cortex Knowledge Engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Top-Level Book Structure
// ============================================================================

/// A compiled knowledge artifact containing structured text, configuration,
/// and graph data. Matches `nostra-book-v1.schema.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NostraBook {
    /// JSON-LD context for semantic interoperability
    #[serde(rename = "@context")]
    pub context: Vec<ContextEntry>,

    /// Book metadata (required)
    pub meta: BookMeta,

    /// Optional configuration for memory and resilience
    #[serde(default)]
    pub config: Option<BookConfig>,

    /// Table of contents structure (required)
    pub structure: BookStructure,

    /// Content chapters (required)
    pub content: Vec<Chapter>,

    /// Optional embedded knowledge graph
    #[serde(default)]
    pub knowledge_graph: Option<KnowledgeGraph>,
}

/// Context can be a string URL or an object with prefix mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContextEntry {
    Url(String),
    Prefixes(HashMap<String, String>),
}

// ============================================================================
// Metadata
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMeta {
    /// URN identifier: `urn:nostra:book:<id>`
    pub id: String,

    /// Always "Contribution::Book"
    #[serde(rename = "type")]
    pub book_type: String,

    /// Content hash: `sha256:<hash>`
    pub version_hash: String,

    /// Contribution phase
    pub phase: ContributionPhase,

    /// Access policy (optional)
    #[serde(default)]
    pub access_policy: Option<AccessPolicy>,

    /// Provenance information
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionPhase {
    Exploratory,
    Deliberative,
    Decisive,
    Executable,
    Archival,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicy {
    #[serde(rename = "Space::Public")]
    Public,
    #[serde(rename = "Space::Private")]
    Private,
    #[serde(rename = "Space::Gated")]
    Gated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Author DID: `did:nostra:user:<id>`
    pub author_did: String,

    /// Space DID: `did:nostra:space:<id>`
    pub space_did: String,

    /// ISO 8601 timestamp
    pub created_at: String,
}

// ============================================================================
// Configuration
// ============================================================================

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
    pub primary_storage: Option<PrimaryStorage>,

    #[serde(default)]
    pub cache_layer: Option<CacheLayer>,

    #[serde(default)]
    pub vector_index: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimaryStorage {
    #[serde(rename = "Canister::StableMemory")]
    StableMemory,
    #[serde(rename = "Canister::Heap")]
    Heap,
    AssetCanister,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheLayer {
    #[serde(rename = "Client::WasmHeap")]
    WasmHeap,
    #[serde(rename = "Client::IndexedDB")]
    IndexedDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    #[serde(default)]
    pub offline_mode: Option<OfflineMode>,

    #[serde(default)]
    pub sync_priority: Option<SyncPriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OfflineMode {
    ReadOnly,
    ReadWrite,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncPriority {
    High,
    Medium,
    Low,
}

// ============================================================================
// Structure (Table of Contents)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStructure {
    pub toc: Vec<TocEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    pub id: String,
    pub title: String,
    pub level: u8,
    pub link: String,
}

// ============================================================================
// Content (Chapters and Blocks)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,

    #[serde(rename = "type")]
    pub chapter_type: String,

    #[serde(default)]
    pub version: Option<String>,

    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "type")]
    pub block_type: String,

    #[serde(default)]
    pub content: Option<BlockContent>,

    #[serde(default)]
    pub level: Option<u8>,

    #[serde(default)]
    pub ref_id: Option<String>,

    #[serde(default)]
    pub display_text: Option<String>,

    /// Flexible storage for expanded artifact types (media, resources, etc.)
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Block content can be a simple string or structured inline elements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockContent {
    Text(String),
    Inline(Vec<InlineElement>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineElement {
    #[serde(rename = "type")]
    pub element_type: String,

    #[serde(default)]
    pub value: Option<String>,
}

// ============================================================================
// Knowledge Graph
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGraph {
    #[serde(default)]
    pub entities: Vec<Entity>,

    #[serde(default)]
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,

    #[serde(rename = "type")]
    pub entity_type: String,

    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

// ============================================================================
// Parsing API
// ============================================================================

impl NostraBook {
    /// Parse a NostraBook from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parse a NostraBook from a file path
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&content)?)
    }

    /// Get the book's unique identifier
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get chapter by ID
    pub fn get_chapter(&self, id: &str) -> Option<&Chapter> {
        self.content.iter().find(|c| c.id == id)
    }

    /// Get all entity IDs from the knowledge graph
    pub fn entity_ids(&self) -> Vec<&str> {
        self.knowledge_graph
            .as_ref()
            .map(|kg| kg.entities.iter().map(|e| e.id.as_str()).collect())
            .unwrap_or_default()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SCHEMA_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../research/057-development-brain/SPECS/FULL_BOOK_SCHEMA.json"
    );

    #[test]
    fn test_parse_full_book_schema() {
        let path = std::path::Path::new(TEST_SCHEMA_PATH);

        // Skip test if file doesn't exist (CI environment)
        if !path.exists() {
            eprintln!("Skipping test: {} not found", TEST_SCHEMA_PATH);
            return;
        }

        let book = NostraBook::from_file(path).expect("Failed to parse FULL_BOOK_SCHEMA.json");

        // Verify meta
        assert_eq!(book.meta.id, "urn:nostra:book:example-full-spec");
        assert_eq!(book.meta.book_type, "Contribution::Book");
        assert!(matches!(book.meta.phase, ContributionPhase::Deliberative));

        // Verify structure
        assert_eq!(book.structure.toc.len(), 2);
        assert_eq!(book.structure.toc[0].title, "The Purpose");

        // Verify content
        assert!(!book.content.is_empty());
        let chapter = book.get_chapter("ch1").expect("Chapter ch1 not found");
        assert_eq!(chapter.blocks.len(), 3);

        // Verify knowledge graph
        let kg = book.knowledge_graph.as_ref().expect("KG missing");
        assert_eq!(kg.entities.len(), 1);
        assert_eq!(kg.relations.len(), 1);
        assert_eq!(kg.entities[0].entity_type, "Concept");
    }

    #[test]
    fn test_parse_minimal_book() {
        let json = r#"{
            "@context": ["https://nostra.network/ns/v2"],
            "meta": {
                "id": "urn:nostra:book:test",
                "type": "Contribution::Book",
                "version_hash": "sha256:abc123",
                "phase": "Exploratory",
                "provenance": {
                    "author_did": "did:nostra:user:test",
                    "space_did": "did:nostra:space:test",
                    "created_at": "2026-01-24T00:00:00Z"
                }
            },
            "structure": { "toc": [] },
            "content": []
        }"#;

        let book = NostraBook::from_json(json).expect("Failed to parse minimal book");
        assert_eq!(book.id(), "urn:nostra:book:test");
        assert!(book.content.is_empty());
    }

    #[test]
    fn test_parse_extended_properties() {
        let json = r#"{
            "@context": ["https://nostra.network/ns/v2"],
            "meta": {
                "id": "urn:nostra:book:media",
                "type": "Contribution::Book",
                "version_hash": "sha256:0",
                "phase": "Exploratory",
                "provenance": { "author_did": "", "space_did": "", "created_at": "" }
            },
            "structure": { "toc": [] },
            "content": [{
                "id": "c1", "type": "Contribution::Chapter",
                "blocks": [{
                    "type": "Block::Video",
                    "url": "https://example.com/video.mp4",
                    "duration": 120,
                    "resolution": { "w": 1920, "h": 1080 }
                }]
            }]
        }"#;

        let book = NostraBook::from_json(json).expect("Failed to parse extended book");
        let block = &book.content[0].blocks[0];

        assert_eq!(block.block_type, "Block::Video");

        // Verify dynamic properties
        let url = block
            .properties
            .get("url")
            .and_then(|v| v.as_str())
            .expect("Missing url");
        assert_eq!(url, "https://example.com/video.mp4");

        let duration = block
            .properties
            .get("duration")
            .and_then(|v| v.as_u64())
            .expect("Missing duration");
        assert_eq!(duration, 120);

        let resolution = block
            .properties
            .get("resolution")
            .expect("Missing resolution");
        assert_eq!(resolution["w"], 1920);
    }
}
