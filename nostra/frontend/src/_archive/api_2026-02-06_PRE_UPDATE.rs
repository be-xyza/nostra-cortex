#![allow(dead_code)]

use crate::types::*;
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;
use std::cell::RefCell;

thread_local! {
    static AGENT_CACHE: RefCell<Option<Agent>> = RefCell::new(None);
}

fn backend_canister_principal() -> Result<Principal, String> {
    let id = option_env!("BACKEND_CANISTER_ID")
        .ok_or_else(|| "Missing BACKEND_CANISTER_ID (build-time env)".to_string())?;
    Principal::from_text(id).map_err(|e| e.to_string())
}

fn workflow_engine_canister_principal() -> Result<Principal, String> {
    let id = option_env!("WORKFLOW_ENGINE_CANISTER_ID")
        .ok_or_else(|| "Missing WORKFLOW_ENGINE_CANISTER_ID (build-time env)".to_string())?;
    Principal::from_text(id).map_err(|e| e.to_string())
}

pub fn get_api_base_url() -> &'static str {
    "http://localhost:3003"
}

pub async fn create_agent() -> Agent {
    if let Some(agent) = AGENT_CACHE.with(|cell| cell.borrow().clone()) {
        return agent;
    }
    let origin = web_sys::window()
        .expect("no window")
        .location()
        .origin()
        .expect("no origin");
    let url = format!("{}/ic-api", origin);

    web_sys::console::log_1(&format!("Creating agent via proxy: {}", url).into());

    let agent = Agent::builder()
        .with_url(url)
        .build()
        .expect("failed to build agent");

    // Fetch root key for local development
    web_sys::console::log_1(&"Fetching root key...".into());

    if let Err(e) = agent.fetch_root_key().await {
        web_sys::console::log_1(&format!("FAILED to fetch root key: {}. This usually means the proxy or replica is unreachable.", e).into());
    } else {
        web_sys::console::log_1(&"Root key fetched successfully.".into());
    }
    AGENT_CACHE.with(|cell| {
        *cell.borrow_mut() = Some(agent.clone());
    });
    agent
}

pub async fn is_initialized(agent: &Agent) -> Result<bool, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "isInitialized")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, bool).map_err(|e| e.to_string())
}

pub async fn initialize(agent: &Agent) -> Result<bool, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "initialize")
        .with_arg(Encode!().unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, bool).map_err(|e| e.to_string())
}

pub async fn get_all_entities(agent: &Agent) -> Result<Vec<Entity>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getAllEntities")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<Entity>).map_err(|e| e.to_string())
}

pub async fn process_ai_query(agent: &Agent, input: String) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "processAIQuery")
        .with_arg(Encode!(&input).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, String).map_err(|e| e.to_string())
}

pub async fn get_chat_history(agent: &Agent) -> Result<Vec<ChatMessage>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getChatHistory")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<ChatMessage>).map_err(|e| e.to_string())
}

pub async fn workflow_start_workflow(agent: &Agent, template: String) -> Result<String, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .update(&canister_id, "start_workflow")
        .with_arg(Encode!(&template).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, String).map_err(|e| e.to_string())
}

pub async fn workflow_process_message(agent: &Agent, message: String) -> Result<String, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .update(&canister_id, "process_message")
        .with_arg(Encode!(&message).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, String).map_err(|e| e.to_string())
}

pub async fn workflow_list_offline_conflicts(
    agent: &Agent,
) -> Result<Vec<OfflineConflictSummary>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "list_offline_conflicts")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<OfflineConflictSummary>).map_err(|e| e.to_string())
}
pub async fn get_all_relationships(agent: &Agent) -> Result<Vec<Relationship>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getAllRelationships")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<Relationship>).map_err(|e| e.to_string())
}

pub async fn get_installed_libraries(agent: &Agent) -> Result<Vec<LibraryManifest>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getInstalledLibraries")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<LibraryManifest>).map_err(|e| e.to_string())
}

pub async fn get_enabled_library_ids(agent: &Agent) -> Result<Vec<String>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getEnabledLibraryIds")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<String>).map_err(|e| e.to_string())
}

pub async fn toggle_library(agent: &Agent, lib_id: String, enabled: bool) -> Result<(), String> {
    let canister_id = backend_canister_principal()?;
    agent
        .update(&canister_id, "toggleLibrary")
        .with_arg(Encode!(&lib_id, &enabled).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_all_workflow_instances(agent: &Agent) -> Result<Vec<WorkflowInstance>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getAllWorkflowInstances")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<WorkflowInstance>).map_err(|e| e.to_string())
}

pub async fn get_logs(agent: &Agent, limit: u64) -> Result<Vec<LogEntry>, String> {
    let canister_id = backend_canister_principal()?;
    // filters are Option, we'll pass None for now as per MVP
    let filter_source: Option<LogSource> = None;
    let filter_level: Option<LogLevel> = None;
    let limit_nat = candid::Nat::from(limit);

    let response = agent
        .query(&canister_id, "getLogs")
        .with_arg(Encode!(&filter_source, &filter_level, &limit_nat).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<LogEntry>).map_err(|e| e.to_string())
}

pub async fn get_chronicle_events(
    agent: &Agent,
    range: Option<(candid::Int, candid::Int)>, // (since, until)
    limit: u64,
) -> Result<Vec<ChronicleEvent>, String> {
    let canister_id = backend_canister_principal()?;
    let (since, until) = match range {
        Some((s, u)) => (Some(s), Some(u)),
        None => (None, None),
    };
    let limit_nat = candid::Nat::from(limit);

    let response = agent
        .query(&canister_id, "getChronicleEvents")
        .with_arg(Encode!(&since, &until, &limit_nat).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<ChronicleEvent>).map_err(|e| e.to_string())
}

pub async fn get_entity_events(
    agent: &Agent,
    entity_id: String,
    limit: u64,
) -> Result<Vec<ChronicleEvent>, String> {
    let canister_id = backend_canister_principal()?;
    let limit_nat = candid::Nat::from(limit);

    let response = agent
        .query(&canister_id, "getEntityEvents")
        .with_arg(Encode!(&entity_id, &limit_nat).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<ChronicleEvent>).map_err(|e| e.to_string())
}

pub async fn get_my_profile(agent: &Agent) -> Result<UserProfile, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "getMyProfile")
        .with_arg(Encode!().unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, UserProfile).map_err(|e| e.to_string())
}

pub async fn set_feature_flag(agent: &Agent, flag: String, enabled: bool) -> Result<(), String> {
    let canister_id = backend_canister_principal()?;
    agent
        .update(&canister_id, "setFeatureFlag")
        .with_arg(Encode!(&flag, &enabled).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn set_labs_opt_in(agent: &Agent, enabled: bool) -> Result<(), String> {
    let canister_id = backend_canister_principal()?;
    agent
        .update(&canister_id, "setLabsOptIn")
        .with_arg(Encode!(&enabled).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn save_ui(agent: &Agent, ui: SavedUI) -> Result<(), String> {
    let canister_id = backend_canister_principal()?;
    agent
        .update(&canister_id, "saveUI")
        .with_arg(Encode!(&ui).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn delete_ui(agent: &Agent, ui_id: String) -> Result<(), String> {
    let canister_id = backend_canister_principal()?;
    agent
        .update(&canister_id, "deleteUI")
        .with_arg(Encode!(&ui_id).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// -- Discussion API (067) --

pub async fn create_discussion(
    agent: &Agent,
    target_id: String,
    topic: String,
    policy: GatingPolicy,
) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "createDiscussion")
        .with_arg(Encode!(&target_id, &topic, &policy).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, String).map_err(|e| e.to_string())
}

pub async fn record_reflection(agent: &Agent, content: String) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "recordReflection")
        .with_arg(Encode!(&content).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, String).map_err(|e| e.to_string())
}

pub async fn post_message(
    agent: &Agent,
    disc_id: String,
    content: String,
    reflection_id: Option<String>,
) -> Result<Result<String, String>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "postMessage")
        .with_arg(Encode!(&disc_id, &content, &reflection_id).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    // The backend returns Result<Text, Text>, which maps to Result<String, String> in Rust
    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())
}

pub async fn get_discussion(agent: &Agent, id: String) -> Result<Option<Discussion>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getDiscussion")
        .with_arg(Encode!(&id).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Option<Discussion>).map_err(|e| e.to_string())
}

pub async fn execute_kip_mutation(agent: &Agent, command: String) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    // Return result string (JSON) or error
    let response = agent
        .update(&canister_id, "execute_kip_mutation")
        .with_arg(Encode!(&command).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    // Result is KipResult = #ok(Text) | #err(Text)
    // We need to decode the variant.
    // However, candid-rust decoding of variants can be tricky if we don't have the type defined.
    // Let's define a helper struct or use a custom Decode! if possible.
    // For now, assuming KipResult maps to Result<String, String> in Rust structure if defined,
    // or we can manually parse.
    // Creating a quick local type or using a generic Result might be easier if defined in types.rs.
    // Let's assume types.rs has KipResult or we decode as generic.
    // Actually, let's just decode as string first if possible, or `Variant`?
    // Let's look at types.rs next to be safe, but for now let's assume we can add it there or here.

    // TEMPORARY: Attempt to decode as a variant that maps to Result.
    // In Motoko: type KipResult = { #ok : Text; #err : Text };
    // In Rust Candid, this maps to Result<String, String>.
    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())?
}

pub async fn execute_kip_query(agent: &Agent, command: String) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "execute_kip")
        .with_arg(Encode!(&command).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())?
}

pub async fn get_system_status(agent: &Agent) -> Result<SystemStatus, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getSystemStatus")
        .with_arg(Encode!().unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, SystemStatus).map_err(|e| e.to_string())
}

// Vector Search API
pub async fn vector_search(agent: &Agent, query_text: String) -> Result<Vec<String>, String> {
    // 1. Get embedding from Worker (via Backend or direct?)
    // Creating embeddings in frontend via WASM is heavy (tract/candle).
    // For now, we will use a "Mock" search in the UI that blindly queries a known term
    // OR we ask the Backend to "embedded_search".
    //
    // To properly support this without backend changes, we'd need the Frontend
    // to call the Worker's "embed" endpoint if it existed.
    //
    // For this Prototype Phase 3:
    // We will assume the query is "Nostra" and just query the Vector DB directly with a dummy vector
    // to prove the UI flow.
    // In production, the backend `processSearch` should handle embedding.

    let canister_id = {
        let id = option_env!("VECTOR_DB_CANISTER_ID")
            .ok_or_else(|| "Missing VECTOR_DB_CANISTER_ID (build-time env)".to_string())?;
        Principal::from_text(id).map_err(|e| e.to_string())?
    };

    // Mock embedding - using explicit f32 type
    let query_vector: Vec<f32> = vec![0.1_f32, 0.2_f32, 0.3_f32, 0.4_f32];
    let limit: i32 = 5;

    let response = agent
        .query(&canister_id, "query")
        .with_arg(Encode!(&query_text, &query_vector, &limit).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    // Decode Result_2 = variant { Ok : vec text; Err : Error };
    #[derive(candid::CandidType, candid::Deserialize)]
    enum VectorResult {
        Ok(Vec<String>),
        Err(VectorError),
    }

    #[derive(candid::CandidType, candid::Deserialize)]
    enum VectorError {
        MemoryError,
        UniqueViolation,
        DimensionMismatch,
        NotFound,
        Unauthorized,
    }

    match Decode!(&response, VectorResult).map_err(|e| e.to_string())? {
        VectorResult::Ok(ids) => Ok(ids),
        VectorResult::Err(_) => Err("Vector DB error".to_string()),
    }
}

pub async fn verify_schedule(
    agent: &Agent,
    grid: Vec<Vec<candid::Int>>,
) -> Result<VerificationResult, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "verify_schedule")
        .with_arg(Encode!(&grid).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, VerificationResult).map_err(|e| e.to_string())
}

// -- VFS API (Phase 3) --

pub async fn write_file(
    agent: &Agent,
    path: String,
    content: Vec<u8>,
    mime_type: String,
) -> Result<(), String> {
    let canister_id = workflow_engine_canister_principal()?;
    agent
        .update(&canister_id, "write_file")
        .with_arg(Encode!(&path, &content, &mime_type).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn read_file(agent: &Agent, path: String) -> Result<Vec<u8>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "read_file")
        .with_arg(Encode!(&path).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<Vec<u8>, String>).map_err(|e| e.to_string())?
}

pub async fn list_files(
    agent: &Agent,
    prefix: String,
) -> Result<Vec<(String, FileMetadata)>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "list_files")
        .with_arg(Encode!(&prefix).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<(String, FileMetadata)>).map_err(|e| e.to_string())
}

pub async fn read_dpub_file_guarded(
    agent: &Agent,
    path: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<u8>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "read_dpub_file_guarded")
        .with_arg(Encode!(&path, &viewer_space_did, &treaty_token).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<Vec<u8>, String>).map_err(|e| e.to_string())?
}

pub async fn list_dpub_files_guarded(
    agent: &Agent,
    prefix: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<(String, FileMetadata)>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "list_dpub_files_guarded")
        .with_arg(Encode!(&prefix, &viewer_space_did, &treaty_token).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<Vec<(String, FileMetadata)>, String>).map_err(|e| e.to_string())?
}

pub async fn read_vfs_guarded(
    agent: &Agent,
    path: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<u8>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "read_vfs_guarded")
        .with_arg(Encode!(&path, &viewer_space_did, &treaty_token).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<Vec<u8>, String>).map_err(|e| e.to_string())?
}

pub async fn list_vfs_guarded(
    agent: &Agent,
    prefix: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<(String, FileMetadata)>, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "list_vfs_guarded")
        .with_arg(Encode!(&prefix, &viewer_space_did, &treaty_token).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<Vec<(String, FileMetadata)>, String>).map_err(|e| e.to_string())?
}

// -- Flow Graph API (097) --

pub async fn get_flow_graph(
    agent: &Agent,
    workflow_id: String,
    version: Option<String>,
) -> Result<FlowGraph, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "get_flow_graph")
        .with_arg(Encode!(&workflow_id, &version).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<FlowGraph, String>).map_err(|e| e.to_string())?
}

pub async fn get_flow_layout(
    agent: &Agent,
    workflow_id: String,
    graph_version: Option<String>,
) -> Result<FlowLayout, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "get_flow_layout")
        .with_arg(Encode!(&workflow_id, &graph_version).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<FlowLayout, String>).map_err(|e| e.to_string())?
}

pub async fn set_flow_layout(
    agent: &Agent,
    input: FlowLayoutInput,
) -> Result<FlowLayout, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .update(&canister_id, "set_flow_layout")
        .with_arg(Encode!(&input).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<FlowLayout, String>).map_err(|e| e.to_string())?
}

// -- Flow Layout (Nostra Backend) --

pub async fn get_flow_layout_backend(
    agent: &Agent,
    workflow_id: String,
    graph_version: Option<String>,
) -> Result<Option<FlowLayout>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getFlowLayout")
        .with_arg(Encode!(&workflow_id, &graph_version).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Option<FlowLayout>).map_err(|e| e.to_string())
}

pub async fn get_flow_layout_history_backend(
    agent: &Agent,
    workflow_id: String,
    graph_version: Option<String>,
    limit: Option<u64>,
) -> Result<Vec<FlowLayout>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getFlowLayoutHistory")
        .with_arg(
            Encode!(
                &workflow_id,
                &graph_version,
                &limit.map(|v| candid::Nat::from(v))
            )
            .unwrap(),
        )
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<FlowLayout>).map_err(|e| e.to_string())
}

pub async fn set_flow_layout_backend(
    agent: &Agent,
    input: FlowLayoutInput,
) -> Result<FlowLayout, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "setFlowLayout")
        .with_arg(Encode!(&input).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, FlowLayout).map_err(|e| e.to_string())
}

// -- dPub V1 API (080) --

pub async fn publish_dpub_edition(
    agent: &Agent,
    dpub_path: String,
    edition_version: String,
    edition_name: Option<String>,
    override_token: Option<String>,
) -> Result<DpubEditionManifest, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .update(&canister_id, "publish_dpub_edition")
        .with_arg(Encode!(
            &dpub_path,
            &edition_version,
            &edition_name,
            &override_token
        ).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<DpubEditionManifest, String>).map_err(|e| e.to_string())?
}

pub async fn get_dpub_feed(
    agent: &Agent,
    dpub_dir: String,
    limit: u32,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<String, String> {
    let canister_id = workflow_engine_canister_principal()?;
    let response = agent
        .query(&canister_id, "get_dpub_feed")
        .with_arg(Encode!(&dpub_dir, &limit, &viewer_space_did, &treaty_token).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())?
}

// -- Institution API (094) --

pub async fn create_institution(
    agent: &Agent,
    req: CreateInstitutionRequest,
) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "createInstitution")
        .with_arg(Encode!(&req).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    // Returns Result<Text, Text> which maps to Result<String, String>
    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())?
}

pub async fn get_institutions_by_space(
    agent: &Agent,
    space_id: String,
) -> Result<Vec<Institution>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getInstitutionsBySpace")
        .with_arg(Encode!(&space_id).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Vec<Institution>).map_err(|e| e.to_string())
}

pub async fn get_institution(agent: &Agent, id: String) -> Result<Option<Institution>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .query(&canister_id, "getInstitution")
        .with_arg(Encode!(&id).unwrap())
        .call()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Option<Institution>).map_err(|e| e.to_string())
}

pub async fn update_institution(
    agent: &Agent,
    req: UpdateInstitutionRequest,
) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "updateInstitution")
        .with_arg(Encode!(&req).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())?
}

pub async fn fork_institution(
    agent: &Agent,
    req: ForkInstitutionRequest,
) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "forkInstitution")
        .with_arg(Encode!(&req).unwrap())
        .call_and_wait()
        .await
        .map_err(|e| e.to_string())?;

    Decode!(&response, Result<String, String>).map_err(|e| e.to_string())?
}

pub async fn get_institution_lineage(
    _agent: &Agent,
    _id: String,
    _max_depth: u32,
) -> Result<Vec<serde_json::Value>, String> {
    // Using generic Value for LineageNode as we haven't defined it in types yet
    // Note: We might need to define LineageNode in types.rs if we want strict typing here.
    // Motoko: type LineageNode = { institution : Institution; depth : Nat };
    // For now, let's keep it simple or define a struct inline if needed, or just skip if not used in MVP UI.
    // Let's defer lineage UI to later or add the type to types.rs if strictly needed.
    // Given the plan didn't explicitly ask for lineage view in MVP, I'll stick to core CRUD.
    Err("Not implemented in frontend yet".to_string())
}
