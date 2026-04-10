use candid::Deserialize;
use ic_cdk::management_canister::raw_rand;
use ic_cdk_timers::TimerId;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use nostra_workflow_core::{
    Action, Engine, Step, Transition, WorkflowDefinition, WorkflowId, WorkflowInstance,
    WorkflowStatus,
};
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub mod a2ui_types;
pub use a2ui_types::*;

pub mod a2ui_adapter;
pub mod vfs;
pub mod flow_graph;
use flow_graph::{FlowGraph, FlowLayout, FlowLayoutInput};

use sha2::{Digest, Sha256};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const WORKFLOW_MAGIC: &[u8; 4] = b"NWF1";
const WORKFLOW_DEF_MAGIC: &[u8; 4] = b"NWD1";

#[derive(Deserialize)]
struct StorableWorkflow(WorkflowInstance);

impl Storable for StorableWorkflow {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(WORKFLOW_MAGIC.len() + payload.len());
        bytes.extend_from_slice(WORKFLOW_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        if !bytes.starts_with(WORKFLOW_MAGIC) {
            panic!("Legacy workflow storage format detected; reinstall required.");
        }
        StorableWorkflow(postcard::from_bytes(&bytes[WORKFLOW_MAGIC.len()..]).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableWorkflowDefinition(WorkflowDefinition);

impl Storable for StorableWorkflowDefinition {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(WORKFLOW_DEF_MAGIC.len() + payload.len());
        bytes.extend_from_slice(WORKFLOW_DEF_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        if !bytes.starts_with(WORKFLOW_DEF_MAGIC) {
            panic!("Legacy workflow definition storage format detected; reinstall required.");
        }
        StorableWorkflowDefinition(postcard::from_bytes(&bytes[WORKFLOW_DEF_MAGIC.len()..]).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableWorkflowId(String);

impl Storable for StorableWorkflowId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableWorkflowId(String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 64,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableWorkflowDefinitionId(String);

impl Storable for StorableWorkflowDefinitionId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableWorkflowDefinitionId(String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableMutationId(String);

impl Storable for StorableMutationId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableMutationId(String::from_utf8(bytes.to_vec()).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static WORKFLOWS: RefCell<StableBTreeMap<StorableWorkflowId, StorableWorkflow, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static WORKFLOW_DEFINITIONS: RefCell<StableBTreeMap<StorableWorkflowDefinitionId, StorableWorkflowDefinition, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static OFFLINE_CONFLICT_INDEX: RefCell<StableBTreeMap<StorableMutationId, StorableWorkflowId, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

// --------------------------------------------------------------------------------
// Public API

// --------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictMutation {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub idempotency_key: String,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub kip_command: String,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_attempt_at: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictEvent {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub mutation: OfflineConflictMutation,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictDecision {
    #[serde(default)]
    pub mutation_id: String,
    #[serde(default)]
    pub decision: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictSummary {
    pub mutation_id: String,
    pub workflow_id: String,
    pub kind: String,
    pub error: String,
    pub source: Option<String>,
    pub status: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubFile {
    #[serde(rename = "@context", default)]
    pub context: serde_json::Value,
    pub meta: DPubMeta,
    #[serde(default)]
    pub manifest: Option<DPubManifest>,
    pub content: Vec<DPubChapter>,
    #[serde(default)]
    pub editions: Vec<EditionSummary>,
    #[serde(default)]
    pub latest_edition: Option<String>,
    #[serde(default)]
    pub hypothesis: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubMeta {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    pub provenance: DPubProvenance,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubProvenance {
    pub author_did: String,
    pub space_did: String,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubManifest {
    pub chapters: Vec<ManifestNode>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ManifestNode {
    pub id: String,
    pub title_cache: String,
    #[serde(default)]
    pub children: Vec<ManifestNode>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubChapter {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: String,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Block {
    #[serde(rename = "Block::Heading")]
    Heading { level: u8, content: String },
    #[serde(rename = "Block::Paragraph")]
    Paragraph { content: ContentValue },
    #[serde(rename = "Block::Reference")]
    Reference { ref_id: String, display_text: String },
    #[serde(rename = "Block::VersionedReference")]
    VersionedReference {
        urn: String,
        display_text: String,
        #[serde(default)]
        version: Option<serde_json::Value>,
    },
    #[serde(untagged)]
    LegacyHtml { content: String },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ContentValue {
    String(String),
    Rich(Vec<RichTextSpan>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RichTextSpan {
    Text { value: String },
    Bold { value: String },
    Italic { value: String },
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EditionSummary {
    pub edition_id: String,
    pub version: String,
    pub published_at: String,
    pub content_root: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
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

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EditionMetadata {
    pub license: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ChapterManifest {
    pub index: u32,
    pub contribution_ref: ContributionVersionRef,
    pub content_hash: String,
    pub title: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ContributionVersionRef {
    pub contribution_id: String,
    pub version_hash: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn merkle_root_hex(leaf_hashes_hex: &[String]) -> String {
    if leaf_hashes_hex.is_empty() {
        return sha256_hex(b"");
    }
    if leaf_hashes_hex.len() == 1 {
        return leaf_hashes_hex[0].clone();
    }

    let mut level: Vec<String> = leaf_hashes_hex.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        for chunk in level.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                chunk[0].clone()
            };
            next.push(sha256_hex(combined.as_bytes()));
        }
        level = next;
    }
    level[0].clone()
}

fn ordered_chapters_from_manifest(dpub: &DPubFile) -> Vec<DPubChapter> {
    let mut ordered: Vec<DPubChapter> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(manifest) = dpub.manifest.as_ref() {
        for node in manifest.chapters.iter() {
            if let Some(ch) = dpub.content.iter().find(|c| c.id == node.id) {
                ordered.push(ch.clone());
                seen.insert(ch.id.clone());
            }
        }
    }

    for ch in dpub.content.iter() {
        if !seen.contains(&ch.id) {
            ordered.push(ch.clone());
        }
    }

    ordered
}

fn workflow_definition_from_template(template: &str) -> Option<WorkflowDefinition> {
    match template {
        "offline_conflict" => Some(build_offline_conflict_definition()),
        "approval" => Some(build_simple_template(
            "approval",
            "Approval Request",
            "Review and approve the request.",
        )),
        "governance" => Some(build_simple_template(
            "governance",
            "Governance Review",
            "Review and decide on the governance proposal.",
        )),
        _ => None,
    }
}

fn build_simple_template(
    id: &str,
    step_title: &str,
    step_description: &str,
) -> WorkflowDefinition {
    let mut steps = HashMap::new();

    let start = Step::new("start", step_title)
        .with_action(Action::UserTask {
            description: step_description.to_string(),
            candidate_roles: vec![],
            candidate_users: vec![],
            a2ui_schema: None,
        })
        .with_transition(Transition::to("done"));

    let done = Step::new("done", "Completed").with_action(Action::None);

    steps.insert("start".to_string(), start);
    steps.insert("done".to_string(), done);

    WorkflowDefinition {
        id: id.to_string(),
        steps,
        start_step_id: "start".to_string(),
    }
}

fn build_offline_conflict_definition() -> WorkflowDefinition {
    let mut steps = HashMap::new();

    let review = Step::new("review", "Resolve Offline Conflict")
        .with_action(Action::UserTask {
            description: "Resolve offline replay conflict.".to_string(),
            candidate_roles: vec![],
            candidate_users: vec![],
            a2ui_schema: Some(offline_conflict_a2ui_schema()),
        })
        .with_transition(Transition::to("resolved"));

    let resolved = Step::new("resolved", "Conflict resolved").with_action(Action::None);

    steps.insert("review".to_string(), review);
    steps.insert("resolved".to_string(), resolved);

    WorkflowDefinition {
        id: "offline_conflict".to_string(),
        steps,
        start_step_id: "review".to_string(),
    }
}

fn offline_conflict_a2ui_schema() -> String {
    let mut components = Vec::new();
    let mut card_props = HashMap::new();
    card_props.insert(
        "title".to_string(),
        serde_json::Value::String("Offline Conflict".to_string()),
    );
    card_props.insert(
        "description".to_string(),
        serde_json::Value::String(
            "Resolve the offline replay conflict (retry / fork / discard).".to_string(),
        ),
    );

    components.push(Component {
        id: "root".to_string(),
        component_type: ComponentType::Card,
        props: card_props,
        a11y: None,
        children: vec![
            "summary".to_string(),
            "error".to_string(),
            "command".to_string(),
            "actions".to_string(),
        ],
        data_bind: None,
    });

    components.push(Component {
        id: "summary".to_string(),
        component_type: ComponentType::Text,
        props: HashMap::from([(
            "text".to_string(),
            serde_json::Value::String("Conflict details will appear here.".to_string()),
        )]),
        a11y: None,
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "error".to_string(),
        component_type: ComponentType::Text,
        props: HashMap::from([
            (
                "text".to_string(),
                serde_json::Value::String("Error details will appear here.".to_string()),
            ),
            (
                "tone".to_string(),
                serde_json::Value::String("error".to_string()),
            ),
        ]),
        a11y: None,
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "command".to_string(),
        component_type: ComponentType::CodeBlock,
        props: HashMap::from([(
            "code".to_string(),
            serde_json::Value::String("KIP command preview".to_string()),
        )]),
        a11y: None,
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "actions".to_string(),
        component_type: ComponentType::Row,
        props: HashMap::new(),
        a11y: None,
        children: vec![
            "conflict_retry".to_string(),
            "conflict_fork".to_string(),
            "conflict_discard".to_string(),
        ],
        data_bind: None,
    });

    components.push(Component {
        id: "conflict_retry".to_string(),
        component_type: ComponentType::Button,
        props: HashMap::from([
            (
                "label".to_string(),
                serde_json::Value::String("Retry".to_string()),
            ),
            (
                "action".to_string(),
                serde_json::Value::String("conflict_retry:{mutation_id}".to_string()),
            ),
        ]),
        a11y: Some(A11yProperties::with_label("Retry")),
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "conflict_fork".to_string(),
        component_type: ComponentType::Button,
        props: HashMap::from([
            (
                "label".to_string(),
                serde_json::Value::String("Fork".to_string()),
            ),
            (
                "action".to_string(),
                serde_json::Value::String("conflict_fork:{mutation_id}".to_string()),
            ),
        ]),
        a11y: Some(A11yProperties::with_label("Fork")),
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "conflict_discard".to_string(),
        component_type: ComponentType::Button,
        props: HashMap::from([
            (
                "label".to_string(),
                serde_json::Value::String("Discard".to_string()),
            ),
            (
                "action".to_string(),
                serde_json::Value::String("conflict_discard:{mutation_id}".to_string()),
            ),
        ]),
        a11y: Some(A11yProperties::with_label("Discard")),
        children: vec![],
        data_bind: None,
    });

    serde_json::to_string(&components).unwrap_or_else(|_| "[]".to_string())
}

async fn create_workflow_instance(definition: WorkflowDefinition) -> String {
    let rand_bytes = raw_rand().await.expect("Failed to generate randomness");
    let id = hex::encode(rand_bytes);

    let instance = WorkflowInstance::new(id.clone(), definition.clone());

    WORKFLOWS.with(|p| {
        p.borrow_mut()
            .insert(StorableWorkflowId(id.clone()), StorableWorkflow(instance));
    });

    WORKFLOW_DEFINITIONS.with(|p| {
        p.borrow_mut().insert(
            StorableWorkflowDefinitionId(definition.id.clone()),
            StorableWorkflowDefinition(definition),
        );
    });

    ic_cdk_timers::set_timer(Duration::from_secs(0), || ic_cdk::futures::spawn(tick()));

    id
}

#[ic_cdk::update]
async fn start_workflow(definition_json: String) -> String {
    let definition = match serde_json::from_str::<WorkflowDefinition>(&definition_json) {
        Ok(def) => def,
        Err(_) => match workflow_definition_from_template(definition_json.trim()) {
            Some(def) => def,
            None => {
                return format!(
                    "error: unknown workflow template or invalid definition ({})",
                    definition_json
                )
            }
        },
    };

    create_workflow_instance(definition).await
}

#[ic_cdk::query]
fn get_workflow(id: WorkflowId) -> Option<String> {
    WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(id))
            .map(|w| serde_json::to_string(&w.0).unwrap())
    })
}

// --------------------------------------------------------------------------------
// Flow Graph API (MVP)
// --------------------------------------------------------------------------------

#[ic_cdk::query]
fn get_flow_graph(workflow_id: String, version: Option<String>) -> Result<FlowGraph, String> {
    if workflow_id.trim().is_empty() {
        return Err("workflow_id is required".to_string());
    }
    let definition = WORKFLOW_DEFINITIONS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowDefinitionId(workflow_id.clone()))
            .map(|w| w.0.clone())
    });

    let definition = definition.ok_or_else(|| "workflow definition not found".to_string())?;
    Ok(flow_graph::derive_graph(&definition, version))
}

#[ic_cdk::query]
fn get_flow_layout(
    workflow_id: String,
    graph_version: Option<String>,
) -> Result<FlowLayout, String> {
    flow_graph::get_flow_layout(workflow_id, graph_version)
}

#[ic_cdk::update]
fn set_flow_layout(input: FlowLayoutInput) -> Result<FlowLayout, String> {
    flow_graph::set_flow_layout(input)
}

// --------------------------------------------------------------------------------
// VFS API (013)
// --------------------------------------------------------------------------------

#[ic_cdk::update]
fn write_file(path: String, content: Vec<u8>, mime_type: String) -> Result<(), String> {
    vfs::vfs_write(path, content, mime_type)
}

#[ic_cdk::query]
fn read_file(path: String) -> Result<Vec<u8>, String> {
    vfs::vfs_read(path)
}

#[ic_cdk::query]
fn list_files(prefix: String) -> Vec<(String, vfs::FileMetadata)> {
    vfs::vfs_list(prefix)
}

// --------------------------------------------------------------------------------
// dPub V1 API (Edition publication + feed)
// --------------------------------------------------------------------------------

#[ic_cdk::update]
async fn publish_dpub_edition(
    dpub_path: String,
    edition_version: String,
    edition_name: Option<String>,
    override_token: Option<String>,
) -> Result<EditionManifest, String> {
    let dpub_bytes = vfs::vfs_read(dpub_path.clone())?;
    let mut dpub: DPubFile = serde_json::from_slice(&dpub_bytes).map_err(|e| e.to_string())?;

    let license = dpub.meta.license.clone().unwrap_or_default();
    if license.trim().is_empty() {
        return Err("Missing license".to_string());
    }
    if license.to_lowercase().contains("arranged")
        && override_token
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return Err("License requires an explicit override token (Arranged)".to_string());
    }

    let published_at = ic_cdk::api::time().to_string();
    let edition_id = hex::encode(raw_rand().await.map_err(|e| format!("{:?}", e))?);
    let publisher = ic_cdk::api::msg_caller().to_text();
    let ordered_content = ordered_chapters_from_manifest(&dpub);

    let mut chapter_manifests: Vec<ChapterManifest> = Vec::new();
    let mut leaf_hashes: Vec<String> = Vec::new();
    for (i, ch) in ordered_content.iter().enumerate() {
        let bytes = serde_json::to_vec(&ch.blocks).map_err(|e| e.to_string())?;
        let content_hash = sha256_hex(&bytes);
        leaf_hashes.push(content_hash.clone());
        chapter_manifests.push(ChapterManifest {
            index: i as u32,
            contribution_ref: ContributionVersionRef {
                contribution_id: ch.id.clone(),
                version_hash: content_hash.clone(),
            },
            content_hash,
            title: ch.title.clone().unwrap_or_else(|| ch.id.clone()),
        });
    }

    let content_root = merkle_root_hex(&leaf_hashes);

    let manifest = EditionManifest {
        edition_id: edition_id.clone(),
        dpub_id: dpub.meta.id.clone(),
        version: edition_version.clone(),
        name: edition_name,
        content_root: content_root.clone(),
        chapters: chapter_manifests,
        published_at: published_at.clone(),
        publisher,
        previous_edition: dpub.latest_edition.clone(),
        metadata: EditionMetadata { license },
    };

    // Snapshot is the full dPub JSON at time of publication (immutable edition view).
    // (V1: we keep the same content, but pin meta.version/phase for clarity.)
    let mut snapshot = dpub.clone();
    snapshot.content = ordered_content;
    snapshot.meta.version = Some(edition_version.clone());
    snapshot.meta.phase = Some("Archival".to_string());

    let base_dir = dpub_path
        .rsplit_once('/')
        .map(|(dir, _)| dir.to_string())
        .unwrap_or_else(|| "lib/dpubs".to_string());
    let edition_dir = format!("{}/editions/{}", base_dir, edition_version);
    let manifest_path = format!("{}/edition_manifest.json", edition_dir);
    let snapshot_path = format!("{}/snapshot.json", edition_dir);

    vfs::vfs_write(
        manifest_path.clone(),
        serde_json::to_vec(&manifest).map_err(|e| e.to_string())?,
        "application/json".to_string(),
    )?;
    vfs::vfs_write(
        snapshot_path.clone(),
        serde_json::to_vec(&snapshot).map_err(|e| e.to_string())?,
        "application/json".to_string(),
    )?;

    // Update dPub index (latest edition + summary list)
    dpub.meta.version = Some(edition_version.clone());
    dpub.meta.phase = Some("Archival".to_string());
    dpub.latest_edition = Some(edition_version.clone());
    dpub.editions.push(EditionSummary {
        edition_id: edition_id.clone(),
        version: edition_version.clone(),
        published_at: published_at.clone(),
        content_root: content_root.clone(),
    });
    vfs::vfs_write(
        dpub_path.clone(),
        serde_json::to_vec(&dpub).map_err(|e| e.to_string())?,
        "application/json".to_string(),
    )?;

    // Native feed (V1)
    let feed_path = format!("{}/feed.json", base_dir);
    let feed = serde_json::json!({
        "type": "dpub.feed.v1",
        "dpub_id": dpub.meta.id,
        "latest_edition": dpub.latest_edition,
        "editions": dpub.editions,
    });
    let _ = vfs::vfs_write(
        feed_path,
        serde_json::to_vec(&feed).unwrap_or_default(),
        "application/json".to_string(),
    );

    // Chronicle append (simple JSONL stored via VFS)
    let chronicle_path = "/lib/chronicle/edition_published.jsonl".to_string();
    let mut existing = String::new();
    if let Ok(bytes) = vfs::vfs_read(chronicle_path.clone()) {
        existing = String::from_utf8_lossy(&bytes).to_string();
    }
    let event = serde_json::json!({
        "type": "edition.published",
        "dpub_id": manifest.dpub_id,
        "edition_id": edition_id,
        "version": edition_version,
        "content_root": content_root,
        "published_at": published_at,
    });
    existing.push_str(&serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()));
    existing.push('\n');
    let _ = vfs::vfs_write(
        chronicle_path,
        existing.into_bytes(),
        "application/jsonl".to_string(),
    );

    // Audit trace (glass box)
    let trace = serde_json::json!({
        "type": "audit_trace.v1",
        "action": "publish_edition",
        "dpub_id": dpub.meta.id,
        "edition_id": manifest.edition_id,
        "timestamp": ic_cdk::api::time().to_string(),
        "inputs": {
            "dpub_path": dpub_path.clone(),
            "edition_version": manifest.version,
            "override_token": override_token,
            "chapter_count": dpub.content.len(),
        },
        "outputs": {
            "content_root": manifest.content_root,
            "manifest_path": manifest_path,
            "snapshot_path": snapshot_path,
        }
    });
    let trace_path = format!("/lib/audit_traces/publish_edition_{}.json", edition_id);
    let _ = vfs::vfs_write(
        trace_path,
        serde_json::to_vec(&trace).unwrap_or_default(),
        "application/json".to_string(),
    );

    Ok(manifest)
}

fn is_valid_dpub_dir(base_dir: &str) -> bool {
    let trimmed = base_dir.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    let normalized = trimmed.trim_start_matches('/');
    normalized.starts_with("lib/dpubs/")
}

fn treaty_required(
    viewer_space: Option<&str>,
    dpub_space: &str,
    treaty_token: Option<&str>,
) -> bool {
    let viewer = viewer_space.map(|v| v.trim()).filter(|v| !v.is_empty());
    let Some(viewer) = viewer else {
        return false;
    };
    if viewer == dpub_space {
        return false;
    }
    treaty_token.unwrap_or_default().trim().is_empty()
}

fn build_dpub_feed(
    base_dir: &str,
    mut editions: Vec<serde_json::Value>,
    limit: usize,
) -> serde_json::Value {
    editions.reverse();
    editions.truncate(limit);
    serde_json::json!({
        "type": "dpub.feed.v1",
        "dpub_dir": base_dir,
        "items": editions,
    })
}

fn is_valid_vfs_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    let normalized = trimmed.trim_start_matches('/');
    normalized.starts_with("lib/")
}

fn dpub_base_dir_from_path(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_start_matches('/');
    let mut parts = trimmed.split('/');
    if parts.next()? != "lib" {
        return None;
    }
    if parts.next()? != "dpubs" {
        return None;
    }
    let slug = parts.next()?;
    if slug.trim().is_empty() {
        return None;
    }
    Some(format!("lib/dpubs/{}", slug))
}

fn enforce_non_dpub_guard(
    viewer_space_did: Option<&str>,
    treaty_token: Option<&str>,
) -> Result<(), String> {
    let viewer = viewer_space_did
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Viewer space required for guarded access".to_string())?;
    let token = treaty_token
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "Treaty token required for guarded access".to_string())?;
    let _ = (viewer, token);
    Ok(())
}

fn enforce_dpub_treaty(
    base_dir: &str,
    viewer_space_did: Option<&str>,
    treaty_token: Option<&str>,
) -> Result<(), String> {
    if !is_valid_dpub_dir(base_dir) {
        return Err("Invalid dPub dir".to_string());
    }
    let viewer = viewer_space_did
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Viewer space required for guarded access".to_string())?;
    let meta_path = format!("{}/dpub.json", base_dir);
    let dpub_bytes = vfs::vfs_read(meta_path)
        .map_err(|_| "Missing dPub metadata for treaty enforcement".to_string())?;
    let dpub: DPubFile = serde_json::from_slice(&dpub_bytes).map_err(|e| e.to_string())?;
    let dpub_space = dpub.meta.provenance.space_did;
    if treaty_required(Some(viewer), &dpub_space, treaty_token) {
        return Err("Treaty required for cross-space access".to_string());
    }
    Ok(())
}

fn enforce_vfs_guarded(
    path_or_prefix: &str,
    viewer_space_did: Option<&str>,
    treaty_token: Option<&str>,
) -> Result<(), String> {
    if !is_valid_vfs_path(path_or_prefix) {
        return Err("Invalid VFS path".to_string());
    }
    if let Some(base_dir) = dpub_base_dir_from_path(path_or_prefix) {
        return enforce_dpub_treaty(&base_dir, viewer_space_did, treaty_token);
    }
    enforce_non_dpub_guard(viewer_space_did, treaty_token)
}

#[ic_cdk::query]
fn get_dpub_feed(
    dpub_dir: String,
    limit: u32,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<String, String> {
    // Minimal feed: list edition manifests under dpub_dir/editions/*
    let base_dir = dpub_dir.trim_end_matches('/').to_string();
    if !is_valid_dpub_dir(&base_dir) {
        return Err("Invalid dPub dir".to_string());
    }

    if let Some(viewer) = viewer_space_did.as_deref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        let meta_path = format!("{}/dpub.json", base_dir);
        let dpub_bytes = vfs::vfs_read(meta_path)
            .map_err(|_| "Missing dPub metadata for treaty enforcement".to_string())?;
        let dpub: DPubFile = serde_json::from_slice(&dpub_bytes).map_err(|e| e.to_string())?;
        let dpub_space = dpub.meta.provenance.space_did;
        if treaty_required(Some(viewer), &dpub_space, treaty_token.as_deref()) {
            return Err("Treaty required for cross-space feed access".to_string());
        }
    }

    let prefix = format!("{}/editions/", base_dir);
    let mut entries = vfs::vfs_list(prefix.clone());
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut editions: Vec<serde_json::Value> = Vec::new();
    for (path, _) in entries {
        if !path.ends_with("edition_manifest.json") {
            continue;
        }
        if let Ok(bytes) = vfs::vfs_read(path) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                editions.push(v);
            }
        }
    }

    let feed = build_dpub_feed(&base_dir, editions, limit as usize);
    serde_json::to_string(&feed).map_err(|e| e.to_string())
}

#[ic_cdk::query]
fn read_dpub_file_guarded(
    path: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<u8>, String> {
    let base_dir = dpub_base_dir_from_path(&path).ok_or_else(|| "Invalid dPub path".to_string())?;
    enforce_dpub_treaty(
        &base_dir,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    vfs::vfs_read(path)
}

#[ic_cdk::query]
fn list_dpub_files_guarded(
    prefix: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<(String, vfs::FileMetadata)>, String> {
    let base_dir =
        dpub_base_dir_from_path(&prefix).ok_or_else(|| "Invalid dPub prefix".to_string())?;
    enforce_dpub_treaty(
        &base_dir,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    Ok(vfs::vfs_list(prefix))
}

#[ic_cdk::query]
fn read_vfs_guarded(
    path: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<u8>, String> {
    enforce_vfs_guarded(
        &path,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    vfs::vfs_read(path)
}

#[ic_cdk::query]
fn list_vfs_guarded(
    prefix: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<(String, vfs::FileMetadata)>, String> {
    enforce_vfs_guarded(
        &prefix,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    Ok(vfs::vfs_list(prefix))
}

#[ic_cdk::update]
async fn tick() {
    let keys: Vec<StorableWorkflowId> = WORKFLOWS.with(|p| {
        // Fix: Use generic iteration or handle tuple explicitly
        p.borrow().iter().map(|x| x.key().clone()).collect()
    });

    for key in keys {
        let mut workflow = WORKFLOWS.with(|p| p.borrow().get(&key).unwrap().0);

        Engine::step(&mut workflow);

        WORKFLOWS.with(|p| {
            p.borrow_mut().insert(key, StorableWorkflow(workflow));
        });
    }
}

// --------------------------------------------------------------------------------
// System Hooks
// --------------------------------------------------------------------------------

#[ic_cdk::init]
fn init() {
    let timer_id = ic_cdk_timers::set_timer_interval(Duration::from_secs(1), || {
        ic_cdk::futures::spawn(tick());
    });

    TIMER_ID.with(|t| *t.borrow_mut() = Some(timer_id));
}

fn index_conflict(mutation_id: &str, workflow_id: &str) {
    OFFLINE_CONFLICT_INDEX.with(|p| {
        p.borrow_mut().insert(
            StorableMutationId(mutation_id.to_string()),
            StorableWorkflowId(workflow_id.to_string()),
        );
    });
}

fn remove_conflict_index(mutation_id: &str) {
    OFFLINE_CONFLICT_INDEX.with(|p| {
        p.borrow_mut()
            .remove(&StorableMutationId(mutation_id.to_string()));
    });
}

fn lookup_conflict_workflow(mutation_id: &str) -> Option<String> {
    OFFLINE_CONFLICT_INDEX.with(|p| {
        p.borrow()
            .get(&StorableMutationId(mutation_id.to_string()))
            .map(|id| id.0.clone())
    })
}

fn attach_conflict_to_workflow(workflow_id: &str, event: &OfflineConflictEvent) {
    let workflow = WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(workflow_id.to_string()))
            .map(|w| w.0)
    });

    if let Some(mut workflow) = workflow {
        workflow
            .context
            .set("mutation_id", event.mutation.id.clone());
        workflow
            .context
            .set("conflict_kind", event.kind.clone());
        workflow
            .context
            .set("conflict_error", event.error.clone());
        workflow
            .context
            .set("source", event.source.clone().unwrap_or_default());
        workflow.context.log(format!(
            "Offline conflict received: {} ({})",
            event.mutation.id, event.kind
        ));

        Engine::step(&mut workflow);

        WORKFLOWS.with(|p| {
            p.borrow_mut().insert(
                StorableWorkflowId(workflow_id.to_string()),
                StorableWorkflow(workflow),
            );
        });
    }
}

fn workflow_status_label(status: &WorkflowStatus) -> String {
    match status {
        WorkflowStatus::Running => "Running".to_string(),
        WorkflowStatus::Paused => "Paused".to_string(),
        WorkflowStatus::Completed => "Completed".to_string(),
        WorkflowStatus::Failed(msg) => format!("Failed: {}", msg),
    }
}

fn parse_offline_conflict_value(value: &serde_json::Value) -> Option<OfflineConflictEvent> {
    if let Some(payload) = value.get("payload") {
        let payload_value = if let Some(s) = payload.as_str() {
            serde_json::from_str::<serde_json::Value>(s).ok()?
        } else {
            payload.clone()
        };
        let mut event = parse_offline_conflict_value(&payload_value)?;
        if event.workflow_id.is_none() {
            if let Some(wid) = value.get("workflow_id").and_then(|v| v.as_str()) {
                event.workflow_id = Some(wid.to_string());
            }
        }
        return Some(event);
    }

    let mut event: OfflineConflictEvent = serde_json::from_value(value.clone()).ok()?;
    if event.workflow_id.is_none() {
        if let Some(wid) = value.get("workflow_id").and_then(|v| v.as_str()) {
            event.workflow_id = Some(wid.to_string());
        }
    }
    if event.kind.is_empty() {
        event.kind = "Conflict".to_string();
    }
    if event.error.is_empty() {
        event.error = "Unknown conflict".to_string();
    }
    if event.mutation.id.is_empty() {
        return None;
    }
    Some(event)
}

fn render_offline_conflict_surface(
    event: &OfflineConflictEvent,
    workflow_id: &str,
) -> A2UIMessage {
    let mut props = HashMap::new();
    props.insert(
        "title".to_string(),
        serde_json::Value::String(format!(
            "Offline {} ({})",
            event.kind, workflow_id
        )),
    );
    props.insert(
        "context".to_string(),
        serde_json::Value::String("inbox".to_string()),
    );
    props.insert(
        "tone".to_string(),
        serde_json::Value::String("critical".to_string()),
    );
    props.insert(
        "priority".to_string(),
        serde_json::Value::String("p0".to_string()),
    );
    props.insert(
        "workflow_id".to_string(),
        serde_json::Value::String(workflow_id.to_string()),
    );
    props.insert(
        "source".to_string(),
        serde_json::Value::String(
            event
                .source
                .clone()
                .unwrap_or_else(|| "workflow-engine".to_string()),
        ),
    );
    props.insert(
        "mutation_id".to_string(),
        serde_json::Value::String(event.mutation.id.clone()),
    );

    let cmd = if event.mutation.kip_command.len() > 240 {
        format!("{}...", &event.mutation.kip_command[..240])
    } else {
        event.mutation.kip_command.clone()
    };

    let retry_id = format!("conflict_retry:{}", event.mutation.id);
    let fork_id = format!("conflict_fork:{}", event.mutation.id);
    let discard_id = format!("conflict_discard:{}", event.mutation.id);

    let components = vec![
        Component {
            id: "root".to_string(),
            component_type: ComponentType::Card,
            props,
            a11y: None,
            children: vec![
                "summary".to_string(),
                "error".to_string(),
                "command".to_string(),
                "actions".to_string(),
            ],
            data_bind: None,
        },
        Component {
            id: "summary".to_string(),
            component_type: ComponentType::Text,
            props: HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String(format!(
                        "Mutation ID: {}\nAttempts: {}",
                        event.mutation.id, event.mutation.attempts
                    )),
                ),
                (
                    "tone".to_string(),
                    serde_json::Value::String("muted".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "error".to_string(),
            component_type: ComponentType::Text,
            props: HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String(format!("Error: {}", event.error)),
                ),
                (
                    "tone".to_string(),
                    serde_json::Value::String("error".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "command".to_string(),
            component_type: ComponentType::CodeBlock,
            props: HashMap::from([(
                "code".to_string(),
                serde_json::Value::String(cmd),
            )]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "actions".to_string(),
            component_type: ComponentType::Row,
            props: HashMap::new(),
            a11y: None,
            children: vec![retry_id.clone(), fork_id.clone(), discard_id.clone()],
            data_bind: None,
        },
        Component {
            id: retry_id,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Retry".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!("conflict_retry:{}", event.mutation.id)),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Retry")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: fork_id,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Fork".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!("conflict_fork:{}", event.mutation.id)),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Fork")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: discard_id,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Discard".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!("conflict_discard:{}", event.mutation.id)),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Discard")),
            children: vec![],
            data_bind: None,
        },
    ];

    A2UIMessage::RenderSurface {
        surface_id: format!("offline_conflict_{}", workflow_id),
        title: "Offline Conflict".to_string(),
        root: None,
        components,
        meta: Some(A2UIMeta {
            theme: Some("cortex".to_string()),
            tone: Some("critical".to_string()),
            context: Some("inbox".to_string()),
            density: Some("compact".to_string()),
            priority: Some("p0".to_string()),
            intent: Some("primary".to_string()),
            severity: Some("critical".to_string()),
            workflow_id: Some(workflow_id.to_string()),
            mutation_id: Some(event.mutation.id.clone()),
            source: event.source.clone().or(Some("workflow-engine".to_string())),
            timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
        }),
    }
}

fn render_decision_ack(decision: &OfflineConflictDecision, workflow_id: &str) -> A2UIMessage {
    let mut props = HashMap::new();
    props.insert(
        "title".to_string(),
        serde_json::Value::String("Conflict Decision Recorded".to_string()),
    );
    props.insert(
        "description".to_string(),
        serde_json::Value::String(format!(
            "Workflow {} recorded decision '{}' for mutation {}.",
            workflow_id, decision.decision, decision.mutation_id
        )),
    );
    props.insert(
        "workflow_id".to_string(),
        serde_json::Value::String(workflow_id.to_string()),
    );
    props.insert(
        "mutation_id".to_string(),
        serde_json::Value::String(decision.mutation_id.clone()),
    );
    props.insert(
        "decision".to_string(),
        serde_json::Value::String(decision.decision.clone()),
    );
    props.insert(
        "source".to_string(),
        serde_json::Value::String(
            decision
                .source
                .clone()
                .unwrap_or_else(|| "workflow-engine".to_string()),
        ),
    );

    let components = vec![Component {
        id: "root".to_string(),
        component_type: ComponentType::Card,
        props,
        a11y: None,
        children: vec![],
        data_bind: None,
    }];

    A2UIMessage::RenderSurface {
        surface_id: format!("offline_conflict_decision_{}", workflow_id),
        title: "Decision Recorded".to_string(),
        root: None,
        components,
        meta: Some(A2UIMeta {
            theme: Some("cortex".to_string()),
            tone: Some("info".to_string()),
            context: Some("inbox".to_string()),
            density: Some("compact".to_string()),
            priority: Some("p1".to_string()),
            intent: Some("secondary".to_string()),
            severity: Some("info".to_string()),
            workflow_id: Some(workflow_id.to_string()),
            mutation_id: Some(decision.mutation_id.clone()),
            source: decision.source.clone().or(Some("workflow-engine".to_string())),
            timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
        }),
    }
}

async fn handle_offline_conflict(value: serde_json::Value) -> String {
    let mut event = match parse_offline_conflict_value(&value) {
        Some(event) => event,
        None => {
            let response = A2UIMessage::Error {
                message: "Invalid offline_conflict payload".to_string(),
            };
            return serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
        }
    };

    if event.workflow_id.is_none() {
        event.workflow_id = Some(create_workflow_instance(build_offline_conflict_definition()).await);
    }

    let workflow_id = event.workflow_id.clone().unwrap_or_default();
    index_conflict(&event.mutation.id, &workflow_id);
    attach_conflict_to_workflow(&workflow_id, &event);

    let response = render_offline_conflict_surface(&event, &workflow_id);
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

async fn handle_offline_conflict_decision(value: serde_json::Value) -> String {
    let decision: OfflineConflictDecision = serde_json::from_value(value).unwrap_or_default();
    let workflow_id = decision
        .workflow_id
        .clone()
        .or_else(|| lookup_conflict_workflow(&decision.mutation_id));

    let Some(workflow_id) = workflow_id else {
        let response = A2UIMessage::Error {
            message: "Unknown workflow for decision".to_string(),
        };
        return serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    };

    let workflow = WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(workflow_id.clone()))
            .map(|w| w.0)
    });

    if let Some(mut workflow) = workflow {
        let mut payload = HashMap::new();
        if !decision.decision.is_empty() {
            payload.insert("decision".to_string(), decision.decision.clone());
        }
        if !decision.mutation_id.is_empty() {
            payload.insert("mutation_id".to_string(), decision.mutation_id.clone());
        }
        if let Some(source) = decision.source.clone() {
            payload.insert("source".to_string(), source);
        }

        Engine::complete_user_task(&mut workflow, Some(payload));

        WORKFLOWS.with(|p| {
            p.borrow_mut().insert(
                StorableWorkflowId(workflow_id.clone()),
                StorableWorkflow(workflow),
            );
        });

        remove_conflict_index(&decision.mutation_id);

        let response = render_decision_ack(&decision, &workflow_id);
        return serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    }

    let response = A2UIMessage::Error {
        message: "Workflow instance not found".to_string(),
    };
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

#[ic_cdk::query]
fn list_offline_conflicts() -> Vec<OfflineConflictSummary> {
    OFFLINE_CONFLICT_INDEX.with(|index| {
        index
            .borrow()
            .iter()
            .map(|entry| {
                let mutation_id = entry.key().0.clone();
                let workflow_id = entry.value().0.clone();

                let workflow = WORKFLOWS.with(|p| {
                    p.borrow()
                        .get(&StorableWorkflowId(workflow_id.clone()))
                        .map(|w| w.0)
                });

                if let Some(wf) = workflow {
                    let ctx = &wf.context;
                    let mutation_id = ctx.get("mutation_id").cloned().unwrap_or(mutation_id);
                    let kind = ctx
                        .get("conflict_kind")
                        .cloned()
                        .unwrap_or_else(|| "Conflict".to_string());
                    let error = ctx
                        .get("conflict_error")
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string());
                    let source = ctx.get("source").cloned();
                    let status = workflow_status_label(&wf.status);

                    OfflineConflictSummary {
                        mutation_id,
                        workflow_id,
                        kind,
                        error,
                        source,
                        status,
                    }
                } else {
                    OfflineConflictSummary {
                        mutation_id,
                        workflow_id,
                        kind: "Conflict".to_string(),
                        error: "Unknown".to_string(),
                        source: None,
                        status: "Missing".to_string(),
                    }
                }
            })
            .collect()
    })
}

#[ic_cdk::update]
async fn process_message(message: String) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&message) {
        if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
            match msg_type {
                "offline_conflict" => return handle_offline_conflict(value).await,
                "offline_conflict_decision" => {
                    return handle_offline_conflict_decision(value).await
                }
                _ => {}
            }
        }
    }

    let mut props = HashMap::new();
    props.insert(
        "content".to_string(),
        serde_json::Value::String(format!("ECHO: {}", message)),
    );

    let components = vec![Component {
        id: "response_card".to_string(),
        component_type: ComponentType::Card,
        props,
        a11y: None,
        children: vec![],
        data_bind: None,
    }];

    let response = A2UIMessage::RenderSurface {
        surface_id: "chat_stream".to_string(),
        title: "Console Response".to_string(),
        root: None,
        components,
        meta: None,
    };

    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_valid_dpub_dir() {
        assert!(is_valid_dpub_dir("lib/dpubs/example"));
        assert!(is_valid_dpub_dir("/lib/dpubs/example"));
        assert!(!is_valid_dpub_dir("lib/books/example"));
        assert!(!is_valid_dpub_dir("../lib/dpubs/example"));
        assert!(!is_valid_dpub_dir(""));
    }

    #[test]
    fn test_treaty_required() {
        assert!(!treaty_required(None, "space:a", None));
        assert!(!treaty_required(Some("space:a"), "space:a", None));
        assert!(treaty_required(Some("space:b"), "space:a", None));
        assert!(!treaty_required(Some("space:b"), "space:a", Some("token")));
        assert!(!treaty_required(Some(""), "space:a", None));
    }

    #[test]
    fn test_build_dpub_feed_orders_and_limits() {
        let editions = vec![json!({"version": "1.0.0"}), json!({"version": "1.0.1"})];
        let feed = build_dpub_feed("lib/dpubs/x", editions, 1);
        let items = feed.get("items").and_then(|v| v.as_array()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].get("version").and_then(|v| v.as_str()), Some("1.0.1"));
    }

    #[test]
    fn test_dpub_base_dir_from_path() {
        assert_eq!(
            dpub_base_dir_from_path("lib/dpubs/my-dpub/editions/1.0.0"),
            Some("lib/dpubs/my-dpub".to_string())
        );
        assert_eq!(
            dpub_base_dir_from_path("/lib/dpubs/my-dpub/dpub.json"),
            Some("lib/dpubs/my-dpub".to_string())
        );
        assert_eq!(dpub_base_dir_from_path("lib/books/x"), None);
        assert_eq!(dpub_base_dir_from_path("lib/dpubs/"), None);
    }

    #[test]
    fn test_is_valid_vfs_path() {
        assert!(is_valid_vfs_path("lib/dpubs/x"));
        assert!(is_valid_vfs_path("/lib/chronicle/events.jsonl"));
        assert!(!is_valid_vfs_path("../lib/dpubs/x"));
        assert!(!is_valid_vfs_path(""));
    }

    #[test]
    fn test_enforce_non_dpub_guard_requires_token() {
        assert!(enforce_non_dpub_guard(Some("space:a"), None).is_err());
        assert!(enforce_non_dpub_guard(Some("space:a"), Some(" ")).is_err());
        assert!(enforce_non_dpub_guard(None, Some("token")).is_err());
        assert!(enforce_non_dpub_guard(Some("space:a"), Some("token")).is_ok());
    }
}

// Custom export for JSON interface
ic_cdk::export_candid!();
