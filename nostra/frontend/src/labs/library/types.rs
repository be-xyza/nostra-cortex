use serde::{Deserialize, Serialize};

use super::urn::{VersionSpec, VersionedRef};

#[derive(Clone, Debug, PartialEq)]
pub enum LibraryView {
    Bookshelf,
    Reader(String), // Book ID
    ManifestEditor(String), // dPub ID
    Loom,           // New Ingestion Mode
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Note {
    pub id: String,
    pub book_id: String,
    pub chapter_id: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LabConfig {
    pub sync_target: SyncTarget,
    pub encryption_level: EncryptionLevel,
    pub cortex_overlay: CortexOverlayMode,
    pub telemetry_enabled: bool,
    pub agent_access: AgentAccess,
    #[serde(default)]
    pub current_space_did: Option<String>,
    #[serde(default)]
    pub treaty_token: Option<String>,
    #[serde(default)]
    pub enforce_treaty: bool,
}

impl Default for LabConfig {
    fn default() -> Self {
        Self {
            sync_target: SyncTarget::Local,
            encryption_level: EncryptionLevel::Standard,
            cortex_overlay: CortexOverlayMode::Passive,
            telemetry_enabled: true,
            agent_access: AgentAccess::ReadOnly,
            current_space_did: None,
            treaty_token: None,
            enforce_treaty: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SyncTarget {
    Local,
    Canister,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EncryptionLevel {
    Standard,
    High,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CortexOverlayMode {
    Hidden,
    Passive,
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AgentAccess {
    ReadOnly,
    ReadWrite,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SortOption {
    Title,
    Author,
    Date,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Book {
    #[serde(rename = "@context")]
    pub context: serde_json::Value,
    pub meta: BookMeta,
    #[serde(default)]
    pub manifest: Option<DPubManifest>,
    #[serde(skip)]
    pub cover_color: String,
    pub content: Vec<BookContent>,
    #[serde(default)]
    pub editions: Vec<EditionSummary>,
    #[serde(default)]
    pub latest_edition: Option<String>,
    #[serde(default)]
    pub knowledge_graph: Option<KnowledgeGraph>,
    #[serde(default)]
    pub hypothesis: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BookMeta {
    pub id: String,
    pub title: String,
    pub version: Option<String>,
    pub phase: Option<String>,
    pub license: Option<String>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DPubManifest {
    pub chapters: Vec<ManifestNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ManifestNode {
    pub id: String,
    pub title_cache: String,
    #[serde(default)]
    pub reference: Option<VersionedRef>,
    #[serde(default)]
    pub children: Vec<ManifestNode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditionSummary {
    pub edition_id: String,
    pub version: String,
    pub published_at: String,
    pub content_root: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditionManifest {
    pub edition_id: String,
    pub dpub_id: String,
    pub version: String,
    #[serde(default)]
    pub name: Option<String>,
    pub content_root: String,
    pub chapters: Vec<ChapterManifest>,
    pub published_at: String,
    pub publisher: String,
    #[serde(default)]
    pub previous_edition: Option<String>,
    pub metadata: EditionMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditionMetadata {
    pub license: String,
    #[serde(default)]
    pub contributors: Vec<Attribution>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Attribution {
    pub actor_id: String,
    pub role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChapterManifest {
    pub index: u32,
    pub contribution_ref: ContributionVersionRef,
    pub content_hash: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContributionVersionRef {
    pub contribution_id: String,
    pub version_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Provenance {
    pub author_did: String,
    pub space_did: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BookContent {
    pub id: String,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: String,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Block {
    #[serde(rename = "Block::Heading")]
    Heading { level: u8, content: String },
    #[serde(rename = "Block::Paragraph")]
    Paragraph { content: ContentValue },
    #[serde(rename = "Block::Reference")]
    Reference {
        ref_id: String,
        display_text: String,
    },
    #[serde(rename = "Block::VersionedReference")]
    VersionedReference {
        urn: String,
        display_text: String,
        #[serde(default)]
        version: Option<VersionSpec>,
    },
    #[serde(untagged)]
    LegacyHtml { content: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ContentValue {
    String(String),
    Rich(Vec<RichTextSpan>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum RichTextSpan {
    Text { value: String },
    Bold { value: String },
    Italic { value: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeGraph {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub properties: std::collections::HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BookCollection {
    pub id: CollectionId,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub book_ids: Vec<String>,
    pub color_theme: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CollectionId {
    /// The Nine Constitutional Documents - all core governance
    Constitutional,
    /// Culture & Philosophy: Labs Constitution, UI/UX Manifesto
    Culture,
    /// Structure & Roles: Spaces, Stewardship, Contribution Lifecycle
    Structure,
    /// Authority & Process: Agent Charter, Governance Escalation
    Authority,
    /// Knowledge & Security: Knowledge Integrity, Security Privacy
    Security,
    /// Regulatory Compliance: GDPR, HIPAA, SOC2
    Compliance,
    /// System/Legacy: Internal docs, system references
    System,
    /// User Favorites
    Favorites,
    /// User Reading List
    ReadingList,
}

impl ToString for CollectionId {
    fn to_string(&self) -> String {
        match self {
            CollectionId::Constitutional => "Constitutional".to_string(),
            CollectionId::Culture => "Culture".to_string(),
            CollectionId::Structure => "Structure".to_string(),
            CollectionId::Authority => "Authority".to_string(),
            CollectionId::Security => "Security".to_string(),
            CollectionId::Compliance => "Compliance".to_string(),
            CollectionId::System => "System".to_string(),
            CollectionId::Favorites => "Favorites".to_string(),
            CollectionId::ReadingList => "Reading List".to_string(),
        }
    }
}
