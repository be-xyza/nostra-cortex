use anyhow::Result;
use chrono::Utc;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

use cortex_worker::config_service::ConfigService;

const WORKER_KEYS_PATH_ENV: &str = "NOSTRA_WORKER_KEYS_PATH";
const DEFAULT_WORKER_KEYS_PATH: &str = "worker_keys.json";
const RUN_ONCE_ENV: &str = "NOSTRA_WORKER_RUN_ONCE";
const OBSERVE_ONCE_ENV: &str = "NOSTRA_WORKER_OBSERVE_ONCE";
const READONLY_HEAP_DELTA_ENV: &str = "NOSTRA_WORKER_READONLY_HEAP_DELTA";
const CONTEXT_BUNDLE_PREP_ENV: &str = "NOSTRA_WORKER_CONTEXT_BUNDLE_PREP";
const STEWARD_REVIEWED_HEAP_EMIT_ENV: &str = "NOSTRA_WORKER_STEWARD_REVIEWED_HEAP_EMIT";
const GATEWAY_URL_ENV: &str = "NOSTRA_GATEWAY_URL";
const CORTEX_GATEWAY_URL_ENV: &str = "CORTEX_GATEWAY_URL";
const OBSERVATION_DIR_ENV: &str = "NOSTRA_WORKER_OBSERVATION_DIR";
const HEAP_CHANGED_SINCE_ENV: &str = "NOSTRA_WORKER_HEAP_CHANGED_SINCE";
const HEAP_SPACE_ID_ENV: &str = "NOSTRA_WORKER_HEAP_SPACE_ID";
const HEAP_LIMIT_ENV: &str = "NOSTRA_WORKER_HEAP_LIMIT";
const CONTEXT_BLOCK_IDS_ENV: &str = "NOSTRA_WORKER_CONTEXT_BLOCK_IDS";
const CONTEXT_BLOCK_LIMIT_ENV: &str = "NOSTRA_WORKER_CONTEXT_BLOCK_LIMIT";
const HEAP_EMIT_SPACE_ID_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_SPACE_ID";
const HEAP_EMIT_TITLE_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_TITLE";
const HEAP_EMIT_BODY_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_BODY";
const HEAP_EMIT_APPROVAL_REF_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_APPROVAL_REF";
const HEAP_EMIT_SOURCE_ARTIFACT_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_SOURCE_ARTIFACT";
const HEAP_EMIT_BODY_LIMIT_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_BODY_LIMIT";
const HEAP_EMIT_AUTH_MODE_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_AUTH_MODE";
const HEAP_EMIT_PRINCIPAL_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_PRINCIPAL";
const HEAP_EMIT_ROLE_ENV: &str = "NOSTRA_WORKER_HEAP_EMIT_ROLE";
const VPS_STATE_ROOT_ENV: &str = "NOSTRA_VPS_STATE_ROOT";
const DEFAULT_GATEWAY_BASE_URL: &str = "http://127.0.0.1:3000";
const OBSERVE_ONCE_PACKET_ID: &str = "initiative-132-runtime-expansion-observe-once-v1";
const READONLY_HEAP_DELTA_PACKET_ID: &str =
    "initiative-132-runtime-expansion-readonly-heap-delta-v1";
const CONTEXT_BUNDLE_PREP_PACKET_ID: &str =
    "initiative-132-runtime-expansion-context-bundle-prep-v1";
const STEWARD_REVIEWED_HEAP_EMIT_PACKET_ID: &str =
    "initiative-132-runtime-expansion-steward-reviewed-heap-emission-v1";
const DEFAULT_AGENT_ID: &str = "agent:eudaemon-alpha-01";
const DEFAULT_HEAP_LIMIT: usize = 25;
const MAX_HEAP_LIMIT: usize = 25;
const DEFAULT_CONTEXT_BLOCK_LIMIT: usize = 5;
const MAX_CONTEXT_BLOCK_LIMIT: usize = 5;
const DEFAULT_HEAP_EMIT_BODY_LIMIT: usize = 4000;
const MAX_HEAP_EMIT_BODY_LIMIT: usize = 4000;

#[derive(Serialize, Deserialize)]
struct WorkerKeyStoreV1 {
    key_id: String,
    rsa_private_key_der: Vec<u8>,
    rsa_public_key_der: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WorkerKeyStoreV2 {
    rsa_key_id: String,
    rsa_private_key_der: Vec<u8>,
    rsa_public_key_der: Vec<u8>,
    hpke_key_id: String,
    hpke_private_key: Vec<u8>,
    hpke_public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WorkerKeyStoreV3 {
    hpke_key_id: String,
    hpke_private_key: Vec<u8>,
    hpke_public_key: Vec<u8>,
}

struct WorkerKeys {
    hpke_public_key: Vec<u8>,
    hpke_key_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObserveOnceArtifact {
    schema_version: String,
    packet_id: String,
    observed_at: String,
    agent_id: String,
    gateway_base_url: String,
    authz_dev_mode: Option<bool>,
    allow_unverified_role_header: Option<bool>,
    agent_identity_enforcement: Option<bool>,
    worker_mode: String,
    checks: Vec<String>,
    errors: Vec<String>,
    exit_status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadOnlyHeapDeltaArtifact {
    schema_version: String,
    packet_id: String,
    observed_at: String,
    agent_id: String,
    gateway_base_url: String,
    space_id: Option<String>,
    changed_since: String,
    limit: usize,
    authz_dev_mode: Option<bool>,
    allow_unverified_role_header: Option<bool>,
    agent_identity_enforcement: Option<bool>,
    worker_mode: String,
    heap_read: HeapReadSummary,
    checks: Vec<String>,
    errors: Vec<String>,
    exit_status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextBundlePrepArtifact {
    schema_version: String,
    packet_id: String,
    observed_at: String,
    agent_id: String,
    gateway_base_url: String,
    requested_block_ids: Vec<String>,
    block_limit: usize,
    authz_dev_mode: Option<bool>,
    allow_unverified_role_header: Option<bool>,
    agent_identity_enforcement: Option<bool>,
    worker_mode: String,
    context_bundle: ContextBundleSummary,
    checks: Vec<String>,
    errors: Vec<String>,
    exit_status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StewardReviewedHeapEmitArtifact {
    schema_version: String,
    packet_id: String,
    observed_at: String,
    agent_id: String,
    gateway_base_url: String,
    space_id: Option<String>,
    approval_ref: Option<String>,
    source_observation_artifact: Option<String>,
    body_length: usize,
    body_limit: usize,
    authz_dev_mode: Option<bool>,
    allow_unverified_role_header: Option<bool>,
    agent_identity_enforcement: Option<bool>,
    worker_mode: String,
    authz: HeapEmitAuthzSummary,
    heap_emit: HeapEmitSummary,
    checks: Vec<String>,
    errors: Vec<String>,
    exit_status: String,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct HeapReadSummary {
    endpoint: String,
    count: Option<usize>,
    changed_count: Option<usize>,
    deleted_count: Option<usize>,
    has_more: Option<bool>,
    next_cursor_present: bool,
    items: Vec<HeapItemSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeapItemSummary {
    artifact_id: Option<String>,
    block_id: Option<String>,
    title: Option<String>,
    space_id: Option<String>,
    block_type: Option<String>,
    emitted_at: Option<String>,
    updated_at: Option<String>,
    deleted_at: Option<String>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextBundleSummary {
    endpoint: String,
    block_count: Option<usize>,
    prepared_at: Option<String>,
    requested_count: usize,
    returned_count: usize,
    total_surface_json_bytes: usize,
    blocks: Vec<ContextBlockSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextBlockSummary {
    artifact_id: Option<String>,
    title: Option<String>,
    block_type: Option<String>,
    tag_count: usize,
    mention_count: usize,
    updated_at: Option<String>,
    surface_json_bytes: usize,
}

#[derive(Default)]
struct GatewayAuthPosture {
    authz_dev_mode: Option<bool>,
    allow_unverified_role_header: Option<bool>,
    agent_identity_enforcement: Option<bool>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct HeapEmitAuthzSummary {
    auth_mode: Option<String>,
    effective_role: Option<String>,
    identity_verified: Option<bool>,
    identity_source: Option<String>,
    principal_present: bool,
    verified_operator_or_higher: bool,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct HeapEmitSummary {
    endpoint: String,
    attempted: bool,
    status: Option<u16>,
    accepted: Option<bool>,
    artifact_id: Option<String>,
    block_id: Option<String>,
    op_id: Option<String>,
    idempotent: Option<bool>,
    source_of_truth: Option<String>,
    fallback_active: Option<bool>,
}

#[derive(Clone)]
struct HeapEmitAuthHeaders {
    auth_mode: Option<String>,
    principal: Option<String>,
    role: String,
}

fn worker_keys_path() -> String {
    std::env::var(WORKER_KEYS_PATH_ENV).unwrap_or_else(|_| DEFAULT_WORKER_KEYS_PATH.to_string())
}

fn load_or_generate_keys() -> Result<WorkerKeys> {
    let path = worker_keys_path();
    if Path::new(&path).exists() {
        let data = fs::read_to_string(&path)?;
        if let Ok(stored) = serde_json::from_str::<WorkerKeyStoreV3>(&data) {
            return Ok(WorkerKeys {
                hpke_public_key: stored.hpke_public_key,
                hpke_key_id: stored.hpke_key_id,
            });
        }

        if let Ok(stored) = serde_json::from_str::<WorkerKeyStoreV2>(&data) {
            let upgraded = WorkerKeyStoreV3 {
                hpke_key_id: stored.hpke_key_id.clone(),
                hpke_private_key: stored.hpke_private_key,
                hpke_public_key: stored.hpke_public_key.clone(),
            };
            fs::write(&path, serde_json::to_string_pretty(&upgraded)?)?;
            return Ok(WorkerKeys {
                hpke_public_key: stored.hpke_public_key,
                hpke_key_id: stored.hpke_key_id,
            });
        }

        if serde_json::from_str::<WorkerKeyStoreV1>(&data).is_ok() {
            println!("   ! Legacy RSA-only worker key store detected; generating HPKE keys.");
            return generate_and_store_hpke_keys(&path);
        }
    }

    println!("   > Generating HPKE (X25519) keypair...");
    generate_and_store_hpke_keys(&path)
}

fn generate_and_store_hpke_keys(path: &str) -> Result<WorkerKeys> {
    let secret = X25519StaticSecret::random_from_rng(OsRng);
    let public = X25519PublicKey::from(&secret);
    let key_id = uuid::Uuid::new_v4().to_string();
    let stored = WorkerKeyStoreV3 {
        hpke_key_id: key_id.clone(),
        hpke_private_key: secret.to_bytes().to_vec(),
        hpke_public_key: public.as_bytes().to_vec(),
    };
    fs::write(path, serde_json::to_string_pretty(&stored)?)?;

    Ok(WorkerKeys {
        hpke_public_key: stored.hpke_public_key,
        hpke_key_id: key_id,
    })
}

fn run_once_enabled() -> bool {
    env_flag_enabled(RUN_ONCE_ENV)
}

fn observe_once_enabled() -> bool {
    env_flag_enabled(OBSERVE_ONCE_ENV)
}

fn readonly_heap_delta_enabled() -> bool {
    env_flag_enabled(READONLY_HEAP_DELTA_ENV)
}

fn context_bundle_prep_enabled() -> bool {
    env_flag_enabled(CONTEXT_BUNDLE_PREP_ENV)
}

fn steward_reviewed_heap_emit_enabled() -> bool {
    env_flag_enabled(STEWARD_REVIEWED_HEAP_EMIT_ENV)
}

fn env_flag_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn configured_agent_id() -> String {
    std::env::var("NOSTRA_AGENT_ID").unwrap_or_else(|_| DEFAULT_AGENT_ID.to_string())
}

fn gateway_base_url() -> String {
    let raw = std::env::var(CORTEX_GATEWAY_URL_ENV)
        .or_else(|_| std::env::var(GATEWAY_URL_ENV))
        .unwrap_or_else(|_| DEFAULT_GATEWAY_BASE_URL.to_string());
    normalize_gateway_base_url(&raw)
}

fn normalize_gateway_base_url(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    let without_studio = trimmed
        .strip_suffix("/api/cortex/studio")
        .or_else(|| trimmed.strip_suffix("/api/cortex"))
        .or_else(|| trimmed.strip_suffix("/api"))
        .unwrap_or(trimmed);
    without_studio.to_string()
}

fn observation_dir() -> PathBuf {
    if let Ok(path) = std::env::var(OBSERVATION_DIR_ENV) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if let Ok(path) = std::env::var(VPS_STATE_ROOT_ENV) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed).join("observations");
        }
    }

    PathBuf::from("logs/eudaemon-alpha/observations")
}

fn observation_path(dir: &Path, observed_at: &str) -> PathBuf {
    mode_observation_path(dir, "observe-once", observed_at)
}

fn readonly_heap_delta_observation_path(dir: &Path, observed_at: &str) -> PathBuf {
    mode_observation_path(dir, "readonly-heap-delta", observed_at)
}

fn context_bundle_prep_observation_path(dir: &Path, observed_at: &str) -> PathBuf {
    mode_observation_path(dir, "context-bundle-prep", observed_at)
}

fn steward_reviewed_heap_emit_observation_path(dir: &Path, observed_at: &str) -> PathBuf {
    mode_observation_path(dir, "steward-reviewed-heap-emission", observed_at)
}

fn mode_observation_path(dir: &Path, mode: &str, observed_at: &str) -> PathBuf {
    let safe_timestamp = observed_at.replace([':', '.'], "-");
    dir.join(format!("eudaemon-alpha-{mode}-{safe_timestamp}.json"))
}

fn env_text(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn heap_emit_body_limit() -> usize {
    std::env::var(HEAP_EMIT_BODY_LIMIT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_HEAP_EMIT_BODY_LIMIT)
        .clamp(1, MAX_HEAP_EMIT_BODY_LIMIT)
}

fn heap_emit_auth_headers() -> HeapEmitAuthHeaders {
    HeapEmitAuthHeaders {
        auth_mode: env_text(HEAP_EMIT_AUTH_MODE_ENV),
        principal: env_text(HEAP_EMIT_PRINCIPAL_ENV),
        role: env_text(HEAP_EMIT_ROLE_ENV).unwrap_or_else(|| "operator".to_string()),
    }
}

fn apply_heap_emit_auth_headers(
    builder: reqwest::RequestBuilder,
    agent_id: &str,
    auth: &HeapEmitAuthHeaders,
) -> reqwest::RequestBuilder {
    let mut builder = builder.header("x-cortex-agent-id", agent_id);
    if let Some(principal) = auth.principal.as_deref() {
        builder = builder.header("x-ic-principal", principal);
        builder = builder.header("x-cortex-role", auth.role.as_str());
    }
    builder
}

fn role_rank(role: &str) -> u8 {
    match role.trim().to_ascii_lowercase().as_str() {
        "viewer" => 1,
        "editor" => 2,
        "operator" => 3,
        "steward" => 4,
        "admin" => 5,
        _ => 0,
    }
}

fn is_heap_workspace_ulid(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() == 26
        && trimmed
            .chars()
            .all(|ch| matches!(ch, '0'..='9' | 'A'..='H' | 'J'..='K' | 'M'..='N' | 'P'..='T' | 'V'..='Z'))
}

fn heap_emit_url(gateway_base: &str) -> Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "{gateway_base}/api/cortex/studio/heap/emit"
    ))?)
}

async fn fetch_gateway_auth_posture(
    gateway_base: &str,
    agent_id: &str,
    checks: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> GatewayAuthPosture {
    let whoami_url = format!("{gateway_base}/api/system/whoami");
    let whoami_result = reqwest::Client::new()
        .get(&whoami_url)
        .header("x-cortex-agent-id", agent_id)
        .send()
        .await;

    let mut authz_dev_mode = None;
    let mut allow_unverified_role_header = None;
    let mut agent_identity_enforcement = std::env::var("NOSTRA_AGENT_IDENTITY_ENFORCEMENT")
        .ok()
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"));

    match whoami_result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                checks.push("gateway_whoami:ok".to_string());
                match response.json::<Value>().await {
                    Ok(payload) => {
                        authz_dev_mode = payload.get("authzDevMode").and_then(Value::as_bool);
                        allow_unverified_role_header = payload
                            .get("allowUnverifiedRoleHeader")
                            .and_then(Value::as_bool);
                        if agent_identity_enforcement.is_none() {
                            agent_identity_enforcement = payload
                                .get("agentIdentityEnforcement")
                                .and_then(Value::as_bool);
                        }
                    }
                    Err(error) => errors.push(format!("gateway_whoami_json:{error}")),
                }
            } else {
                errors.push(format!("gateway_whoami_status:{status}"));
            }
        }
        Err(error) => errors.push(format!("gateway_whoami_request:{error}")),
    }

    if authz_dev_mode == Some(false) {
        checks.push("authz_dev_mode:false".to_string());
    }
    if allow_unverified_role_header == Some(false) {
        checks.push("allow_unverified_role_header:false".to_string());
    }
    if agent_identity_enforcement == Some(true) {
        checks.push("agent_identity_enforcement:true".to_string());
    }

    GatewayAuthPosture {
        authz_dev_mode,
        allow_unverified_role_header,
        agent_identity_enforcement,
    }
}

async fn fetch_heap_emit_authz(
    gateway_base: &str,
    agent_id: &str,
    auth_headers: &HeapEmitAuthHeaders,
    checks: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> (GatewayAuthPosture, HeapEmitAuthzSummary) {
    let whoami_url = format!("{gateway_base}/api/system/whoami");
    let whoami_result = apply_heap_emit_auth_headers(
        reqwest::Client::new().get(&whoami_url),
        agent_id,
        auth_headers,
    )
    .send()
    .await;

    let mut posture = GatewayAuthPosture {
        agent_identity_enforcement: std::env::var("NOSTRA_AGENT_IDENTITY_ENFORCEMENT")
            .ok()
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES")),
        ..GatewayAuthPosture::default()
    };
    let mut authz = HeapEmitAuthzSummary {
        auth_mode: auth_headers.auth_mode.clone(),
        principal_present: auth_headers.principal.is_some(),
        ..HeapEmitAuthzSummary::default()
    };

    match whoami_result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                checks.push("heap_emit_whoami:ok".to_string());
                match response.json::<Value>().await {
                    Ok(payload) => {
                        posture.authz_dev_mode =
                            payload.get("authzDevMode").and_then(Value::as_bool);
                        posture.allow_unverified_role_header = payload
                            .get("allowUnverifiedRoleHeader")
                            .and_then(Value::as_bool);
                        if posture.agent_identity_enforcement.is_none() {
                            posture.agent_identity_enforcement = payload
                                .get("agentIdentityEnforcement")
                                .and_then(Value::as_bool);
                        }
                        authz.effective_role = string_field(&payload, "effectiveRole");
                        authz.identity_verified =
                            payload.get("identityVerified").and_then(Value::as_bool);
                        authz.identity_source = string_field(&payload, "identitySource");
                        authz.verified_operator_or_higher = authz.identity_verified == Some(true)
                            && authz
                                .effective_role
                                .as_deref()
                                .map(|role| role_rank(role) >= role_rank("operator"))
                                .unwrap_or(false);
                    }
                    Err(error) => errors.push(format!("heap_emit_whoami_json:{error}")),
                }
            } else {
                errors.push(format!("heap_emit_whoami_status:{status}"));
            }
        }
        Err(error) => errors.push(format!("heap_emit_whoami_request:{error}")),
    }

    if posture.authz_dev_mode == Some(false) {
        checks.push("authz_dev_mode:false".to_string());
    }
    if posture.allow_unverified_role_header == Some(false) {
        checks.push("allow_unverified_role_header:false".to_string());
    }
    if posture.agent_identity_enforcement == Some(true) {
        checks.push("agent_identity_enforcement:true".to_string());
    }
    if authz.verified_operator_or_higher {
        checks.push("heap_emit_authz:verified_operator_or_higher".to_string());
    } else {
        errors.push("heap_emit_authz:verified_operator_or_higher_required".to_string());
    }

    (posture, authz)
}

async fn run_observe_once() -> Result<PathBuf> {
    let agent_id = configured_agent_id();
    let gateway_base = gateway_base_url();
    let observed_at = Utc::now().to_rfc3339();
    let mut checks = vec![
        format!("packet:{OBSERVE_ONCE_PACKET_ID}"),
        format!("agent_id:{agent_id}"),
        "mode:observe_once".to_string(),
    ];
    let mut errors = Vec::new();
    let posture =
        fetch_gateway_auth_posture(&gateway_base, &agent_id, &mut checks, &mut errors).await;

    let exit_status = if errors.is_empty() {
        "pass"
    } else {
        "needs_review"
    };
    let artifact = ObserveOnceArtifact {
        schema_version: "1.0.0".to_string(),
        packet_id: OBSERVE_ONCE_PACKET_ID.to_string(),
        observed_at: observed_at.clone(),
        agent_id,
        gateway_base_url: gateway_base,
        authz_dev_mode: posture.authz_dev_mode,
        allow_unverified_role_header: posture.allow_unverified_role_header,
        agent_identity_enforcement: posture.agent_identity_enforcement,
        worker_mode: "observe_once".to_string(),
        checks,
        errors,
        exit_status: exit_status.to_string(),
    };

    let dir = observation_dir();
    fs::create_dir_all(&dir)?;
    let path = observation_path(&dir, &observed_at);
    fs::write(&path, serde_json::to_string_pretty(&artifact)?)?;
    Ok(path)
}

fn heap_limit() -> usize {
    std::env::var(HEAP_LIMIT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_HEAP_LIMIT)
        .clamp(1, MAX_HEAP_LIMIT)
}

fn heap_changed_since() -> Option<String> {
    std::env::var(HEAP_CHANGED_SINCE_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn heap_space_id() -> Option<String> {
    std::env::var(HEAP_SPACE_ID_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn context_block_limit() -> usize {
    std::env::var(CONTEXT_BLOCK_LIMIT_ENV)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_CONTEXT_BLOCK_LIMIT)
        .clamp(1, MAX_CONTEXT_BLOCK_LIMIT)
}

fn context_block_ids(limit: usize) -> Vec<String> {
    let raw = std::env::var(CONTEXT_BLOCK_IDS_ENV).unwrap_or_default();
    let mut seen = std::collections::HashSet::<String>::new();
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| {
            let id = value.to_string();
            if seen.insert(id.clone()) {
                Some(id)
            } else {
                None
            }
        })
        .take(limit)
        .collect()
}

fn heap_delta_url(
    gateway_base: &str,
    changed_since: &str,
    space_id: Option<&str>,
    limit: usize,
) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(&format!(
        "{gateway_base}/api/cortex/studio/heap/changed_blocks"
    ))?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("changedSince", changed_since);
        query.append_pair("limit", &limit.to_string());
        query.append_pair("includeDeleted", "false");
        if let Some(space_id) = space_id {
            query.append_pair("spaceId", space_id);
        }
    }
    Ok(url)
}

fn summarize_heap_item(value: &Value) -> HeapItemSummary {
    let projection = value.get("projection").unwrap_or(value);
    HeapItemSummary {
        artifact_id: string_field(projection, "artifactId")
            .or_else(|| string_field(value, "artifactId")),
        block_id: string_field(projection, "blockId").or_else(|| string_field(value, "blockId")),
        title: string_field(projection, "title").or_else(|| string_field(value, "title")),
        space_id: string_field(projection, "workspaceId")
            .or_else(|| string_field(projection, "spaceId"))
            .or_else(|| string_field(value, "spaceId")),
        block_type: string_field(projection, "blockType").or_else(|| string_field(value, "type")),
        emitted_at: string_field(projection, "emittedAt")
            .or_else(|| string_field(value, "emittedAt")),
        updated_at: string_field(projection, "updatedAt")
            .or_else(|| string_field(value, "updatedAt")),
        deleted_at: string_field(value, "deletedAt")
            .or_else(|| string_field(projection, "deletedAt")),
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.is_empty())
}

fn heap_read_summary(endpoint: &str, payload: &Value) -> HeapReadSummary {
    let changed_items = payload
        .get("changed")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let deleted_items = payload
        .get("deleted")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let list_items = payload
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let source_items = if !changed_items.is_empty() {
        changed_items
    } else {
        list_items
    };

    HeapReadSummary {
        endpoint: endpoint.to_string(),
        count: payload
            .get("count")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        changed_count: Some(source_items.len()),
        deleted_count: Some(deleted_items.len()),
        has_more: payload.get("hasMore").and_then(Value::as_bool),
        next_cursor_present: payload.get("nextCursor").and_then(Value::as_str).is_some(),
        items: source_items
            .iter()
            .take(MAX_HEAP_LIMIT)
            .map(summarize_heap_item)
            .collect(),
    }
}

fn context_bundle_url(gateway_base: &str) -> Result<reqwest::Url> {
    Ok(reqwest::Url::parse(&format!(
        "{gateway_base}/api/cortex/studio/heap/blocks/context"
    ))?)
}

fn json_approx_bytes(value: &Value) -> usize {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or_default()
}

fn summarize_context_block(value: &Value) -> ContextBlockSummary {
    let tags = value
        .get("tags")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let mentions = value
        .get("mentions")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let surface_json_bytes = value
        .get("surface_json")
        .map(json_approx_bytes)
        .unwrap_or_default();

    ContextBlockSummary {
        artifact_id: string_field(value, "artifact_id")
            .or_else(|| string_field(value, "artifactId")),
        title: string_field(value, "title"),
        block_type: string_field(value, "block_type").or_else(|| string_field(value, "blockType")),
        tag_count: tags,
        mention_count: mentions,
        updated_at: string_field(value, "updated_at").or_else(|| string_field(value, "updatedAt")),
        surface_json_bytes,
    }
}

fn context_bundle_summary(
    endpoint: &str,
    requested_count: usize,
    payload: &Value,
) -> ContextBundleSummary {
    let bundle = payload.get("context_bundle").unwrap_or(payload);
    let blocks = bundle
        .get("blocks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let block_summaries = blocks
        .iter()
        .take(MAX_CONTEXT_BLOCK_LIMIT)
        .map(summarize_context_block)
        .collect::<Vec<_>>();
    let total_surface_json_bytes = block_summaries
        .iter()
        .map(|block| block.surface_json_bytes)
        .sum();

    ContextBundleSummary {
        endpoint: endpoint.to_string(),
        block_count: bundle
            .get("block_count")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        prepared_at: string_field(bundle, "prepared_at")
            .or_else(|| string_field(bundle, "preparedAt")),
        requested_count,
        returned_count: block_summaries.len(),
        total_surface_json_bytes,
        blocks: block_summaries,
    }
}

async fn run_readonly_heap_delta() -> Result<PathBuf> {
    let agent_id = configured_agent_id();
    let gateway_base = gateway_base_url();
    let observed_at = Utc::now().to_rfc3339();
    let limit = heap_limit();
    let changed_since = heap_changed_since().unwrap_or_else(|| observed_at.clone());
    let space_id = heap_space_id();
    let mut checks = vec![
        format!("packet:{READONLY_HEAP_DELTA_PACKET_ID}"),
        format!("agent_id:{agent_id}"),
        "mode:readonly_heap_delta".to_string(),
        format!("heap_limit:{limit}"),
    ];
    let mut errors = Vec::new();
    let posture =
        fetch_gateway_auth_posture(&gateway_base, &agent_id, &mut checks, &mut errors).await;

    let endpoint = "/api/cortex/studio/heap/changed_blocks";
    let mut heap_read = HeapReadSummary {
        endpoint: endpoint.to_string(),
        ..HeapReadSummary::default()
    };

    match heap_delta_url(&gateway_base, &changed_since, space_id.as_deref(), limit) {
        Ok(url) => match reqwest::Client::new()
            .get(url)
            .header("x-cortex-agent-id", &agent_id)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    checks.push("heap_changed_blocks:ok".to_string());
                    match response.json::<Value>().await {
                        Ok(payload) => {
                            heap_read = heap_read_summary(endpoint, &payload);
                        }
                        Err(error) => errors.push(format!("heap_changed_blocks_json:{error}")),
                    }
                } else {
                    errors.push(format!("heap_changed_blocks_status:{status}"));
                }
            }
            Err(error) => errors.push(format!("heap_changed_blocks_request:{error}")),
        },
        Err(error) => errors.push(format!("heap_changed_blocks_url:{error}")),
    }

    let exit_status = if errors.is_empty() {
        "pass"
    } else {
        "needs_review"
    };
    let artifact = ReadOnlyHeapDeltaArtifact {
        schema_version: "1.0.0".to_string(),
        packet_id: READONLY_HEAP_DELTA_PACKET_ID.to_string(),
        observed_at: observed_at.clone(),
        agent_id,
        gateway_base_url: gateway_base,
        space_id,
        changed_since,
        limit,
        authz_dev_mode: posture.authz_dev_mode,
        allow_unverified_role_header: posture.allow_unverified_role_header,
        agent_identity_enforcement: posture.agent_identity_enforcement,
        worker_mode: "readonly_heap_delta".to_string(),
        heap_read,
        checks,
        errors,
        exit_status: exit_status.to_string(),
    };

    let dir = observation_dir();
    fs::create_dir_all(&dir)?;
    let path = readonly_heap_delta_observation_path(&dir, &observed_at);
    fs::write(&path, serde_json::to_string_pretty(&artifact)?)?;
    Ok(path)
}

async fn run_context_bundle_prep() -> Result<PathBuf> {
    let agent_id = configured_agent_id();
    let gateway_base = gateway_base_url();
    let observed_at = Utc::now().to_rfc3339();
    let block_limit = context_block_limit();
    let requested_block_ids = context_block_ids(block_limit);
    let mut checks = vec![
        format!("packet:{CONTEXT_BUNDLE_PREP_PACKET_ID}"),
        format!("agent_id:{agent_id}"),
        "mode:context_bundle_prep".to_string(),
        format!("context_block_limit:{block_limit}"),
        format!("context_requested_count:{}", requested_block_ids.len()),
    ];
    let mut errors = Vec::new();
    let posture =
        fetch_gateway_auth_posture(&gateway_base, &agent_id, &mut checks, &mut errors).await;

    let endpoint = "/api/cortex/studio/heap/blocks/context";
    let mut context_bundle = ContextBundleSummary {
        endpoint: endpoint.to_string(),
        requested_count: requested_block_ids.len(),
        ..ContextBundleSummary::default()
    };

    if requested_block_ids.is_empty() {
        errors.push(format!("{CONTEXT_BLOCK_IDS_ENV}:empty"));
    } else {
        match context_bundle_url(&gateway_base) {
            Ok(url) => match reqwest::Client::new()
                .post(url)
                .header("x-cortex-agent-id", &agent_id)
                .json(&json!({ "block_ids": &requested_block_ids }))
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        checks.push("heap_blocks_context:ok".to_string());
                        match response.json::<Value>().await {
                            Ok(payload) => {
                                context_bundle = context_bundle_summary(
                                    endpoint,
                                    requested_block_ids.len(),
                                    &payload,
                                );
                            }
                            Err(error) => errors.push(format!("heap_blocks_context_json:{error}")),
                        }
                    } else {
                        errors.push(format!("heap_blocks_context_status:{status}"));
                    }
                }
                Err(error) => errors.push(format!("heap_blocks_context_request:{error}")),
            },
            Err(error) => errors.push(format!("heap_blocks_context_url:{error}")),
        }
    }

    let exit_status = if errors.is_empty() {
        "pass"
    } else {
        "needs_review"
    };
    let artifact = ContextBundlePrepArtifact {
        schema_version: "1.0.0".to_string(),
        packet_id: CONTEXT_BUNDLE_PREP_PACKET_ID.to_string(),
        observed_at: observed_at.clone(),
        agent_id,
        gateway_base_url: gateway_base,
        requested_block_ids,
        block_limit,
        authz_dev_mode: posture.authz_dev_mode,
        allow_unverified_role_header: posture.allow_unverified_role_header,
        agent_identity_enforcement: posture.agent_identity_enforcement,
        worker_mode: "context_bundle_prep".to_string(),
        context_bundle,
        checks,
        errors,
        exit_status: exit_status.to_string(),
    };

    let dir = observation_dir();
    fs::create_dir_all(&dir)?;
    let path = context_bundle_prep_observation_path(&dir, &observed_at);
    fs::write(&path, serde_json::to_string_pretty(&artifact)?)?;
    Ok(path)
}

async fn run_steward_reviewed_heap_emit() -> Result<PathBuf> {
    let agent_id = configured_agent_id();
    let gateway_base = gateway_base_url();
    let observed_at = Utc::now().to_rfc3339();
    let body_limit = heap_emit_body_limit();
    let space_id = env_text(HEAP_EMIT_SPACE_ID_ENV);
    let title = env_text(HEAP_EMIT_TITLE_ENV);
    let body = env_text(HEAP_EMIT_BODY_ENV);
    let approval_ref = env_text(HEAP_EMIT_APPROVAL_REF_ENV);
    let source_observation_artifact = env_text(HEAP_EMIT_SOURCE_ARTIFACT_ENV);
    let auth_headers = heap_emit_auth_headers();
    let mut checks = vec![
        format!("packet:{STEWARD_REVIEWED_HEAP_EMIT_PACKET_ID}"),
        format!("agent_id:{agent_id}"),
        "mode:steward_reviewed_heap_emit".to_string(),
        format!("body_limit:{body_limit}"),
    ];
    let mut errors = Vec::new();

    if space_id.is_none() {
        errors.push(format!("{HEAP_EMIT_SPACE_ID_ENV}:missing"));
    } else if !space_id
        .as_deref()
        .map(is_heap_workspace_ulid)
        .unwrap_or(false)
    {
        errors.push(format!(
            "{HEAP_EMIT_SPACE_ID_ENV}:must_be_heap_workspace_ulid"
        ));
    }
    if title.is_none() {
        errors.push(format!("{HEAP_EMIT_TITLE_ENV}:missing"));
    }
    if body.is_none() {
        errors.push(format!("{HEAP_EMIT_BODY_ENV}:missing"));
    }
    if approval_ref.is_none() {
        errors.push(format!("{HEAP_EMIT_APPROVAL_REF_ENV}:missing"));
    }

    let body_length = body.as_deref().map(str::len).unwrap_or_default();
    if body_length > body_limit {
        errors.push(format!(
            "{HEAP_EMIT_BODY_ENV}:exceeds_limit:{body_length}>{body_limit}"
        ));
    }

    let (posture, authz) = fetch_heap_emit_authz(
        &gateway_base,
        &agent_id,
        &auth_headers,
        &mut checks,
        &mut errors,
    )
    .await;

    let endpoint = "/api/cortex/studio/heap/emit";
    let mut heap_emit = HeapEmitSummary {
        endpoint: endpoint.to_string(),
        ..HeapEmitSummary::default()
    };

    if errors.is_empty() && authz.verified_operator_or_higher {
        let payload = json!({
            "schema_version": "1.0.0",
            "mode": "heap",
            "space_id": space_id.as_deref().unwrap_or_default(),
            "source": {
                "agent_id": agent_id,
                "session_id": "initiative-132-eudaemon-alpha",
                "request_id": format!(
                    "initiative-132:{}:{}",
                    STEWARD_REVIEWED_HEAP_EMIT_PACKET_ID,
                    approval_ref.as_deref().unwrap_or_default()
                ),
                "emitted_at": observed_at
            },
            "block": {
                "type": "eudaemon_evidence_note",
                "title": title.as_deref().unwrap_or_default(),
                "attributes": {
                    "initiative_id": "132",
                    "packet_id": STEWARD_REVIEWED_HEAP_EMIT_PACKET_ID,
                    "approval_ref": approval_ref.as_deref().unwrap_or_default(),
                    "source_observation_artifact": source_observation_artifact
                }
            },
            "content": {
                "payload_type": "rich_text",
                "rich_text": {
                    "plain_text": body.as_deref().unwrap_or_default()
                }
            },
            "relations": {},
            "files": [],
            "apps": [],
            "meta": {
                "schema_version": "1.0.0",
                "request_path": "initiative-132/steward-reviewed-heap-emission-v1"
            },
            "projection_hints": {},
            "crdt_projection": {}
        });

        match heap_emit_url(&gateway_base) {
            Ok(url) => {
                heap_emit.attempted = true;
                match apply_heap_emit_auth_headers(
                    reqwest::Client::new().post(url),
                    &agent_id,
                    &auth_headers,
                )
                .json(&payload)
                .send()
                .await
                {
                    Ok(response) => {
                        let status = response.status();
                        heap_emit.status = Some(status.as_u16());
                        if status.is_success() {
                            checks.push("heap_emit:ok".to_string());
                            match response.json::<Value>().await {
                                Ok(payload) => {
                                    heap_emit.accepted =
                                        payload.get("accepted").and_then(Value::as_bool);
                                    heap_emit.artifact_id = string_field(&payload, "artifactId");
                                    heap_emit.block_id = string_field(&payload, "blockId");
                                    heap_emit.op_id = string_field(&payload, "opId");
                                    heap_emit.idempotent =
                                        payload.get("idempotent").and_then(Value::as_bool);
                                    heap_emit.source_of_truth =
                                        string_field(&payload, "sourceOfTruth");
                                    heap_emit.fallback_active =
                                        payload.get("fallbackActive").and_then(Value::as_bool);
                                }
                                Err(error) => errors.push(format!("heap_emit_json:{error}")),
                            }
                        } else {
                            errors.push(format!("heap_emit_status:{status}"));
                        }
                    }
                    Err(error) => errors.push(format!("heap_emit_request:{error}")),
                }
            }
            Err(error) => errors.push(format!("heap_emit_url:{error}")),
        }
    } else {
        checks.push("heap_emit:skipped".to_string());
    }

    let exit_status = if errors.is_empty() {
        "pass"
    } else {
        "needs_review"
    };
    let artifact = StewardReviewedHeapEmitArtifact {
        schema_version: "1.0.0".to_string(),
        packet_id: STEWARD_REVIEWED_HEAP_EMIT_PACKET_ID.to_string(),
        observed_at: observed_at.clone(),
        agent_id,
        gateway_base_url: gateway_base,
        space_id,
        approval_ref,
        source_observation_artifact,
        body_length,
        body_limit,
        authz_dev_mode: posture.authz_dev_mode,
        allow_unverified_role_header: posture.allow_unverified_role_header,
        agent_identity_enforcement: posture.agent_identity_enforcement,
        worker_mode: "steward_reviewed_heap_emit".to_string(),
        authz,
        heap_emit,
        checks,
        errors,
        exit_status: exit_status.to_string(),
    };

    let dir = observation_dir();
    fs::create_dir_all(&dir)?;
    let path = steward_reviewed_heap_emit_observation_path(&dir, &observed_at);
    fs::write(&path, serde_json::to_string_pretty(&artifact)?)?;
    Ok(path)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config = ConfigService::get();
    let keys = load_or_generate_keys()?;

    println!("Nostra Cortex worker starting in passive preflight mode");
    println!("   > Environment: {:?}", config.get_env());
    println!("   > HPKE key id: {}", keys.hpke_key_id);
    println!("   > HPKE public key bytes: {}", keys.hpke_public_key.len());
    println!(
        "   > Primary canister configured: {}",
        config.get_canister_id("primary").is_some()
    );
    println!(
        "   > Streaming canister configured: {}",
        config.get_canister_id("streaming").is_some()
    );
    println!(
        "   > Backend canister configured: {}",
        config.get_canister_id("backend").is_some()
    );

    if run_once_enabled() {
        println!("   > Passive preflight complete; exiting because {RUN_ONCE_ENV}=true.");
        return Ok(());
    }

    if observe_once_enabled() {
        let path = run_observe_once().await?;
        println!("   > Observe-once artifact written to {}", path.display());
        return Ok(());
    }

    if readonly_heap_delta_enabled() {
        let path = run_readonly_heap_delta().await?;
        println!(
            "   > Read-only heap delta artifact written to {}",
            path.display()
        );
        return Ok(());
    }

    if context_bundle_prep_enabled() {
        let path = run_context_bundle_prep().await?;
        println!(
            "   > Context bundle prep artifact written to {}",
            path.display()
        );
        return Ok(());
    }

    if steward_reviewed_heap_emit_enabled() {
        let path = run_steward_reviewed_heap_emit().await?;
        println!(
            "   > Steward-reviewed heap emission artifact written to {}",
            path.display()
        );
        return Ok(());
    }

    println!("   > Runtime polling remains disabled pending Initiative 132 authority proof.");
    println!("   > Waiting for shutdown signal.");

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("   > Shutdown signal received.");
                return Ok(());
            }
            _ = sleep(Duration::from_secs(60)) => {
                println!("   > Passive worker heartbeat.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_gateway_base_url_from_studio_url() {
        assert_eq!(
            normalize_gateway_base_url("http://127.0.0.1:3000/api/cortex/studio"),
            "http://127.0.0.1:3000"
        );
    }

    #[test]
    fn normalizes_gateway_base_url_from_api_url() {
        assert_eq!(
            normalize_gateway_base_url("http://127.0.0.1:3000/api/"),
            "http://127.0.0.1:3000"
        );
    }

    #[test]
    fn observation_path_is_filesystem_safe() {
        let path = observation_path(
            Path::new("/tmp/eudaemon-observations"),
            "2026-04-28T12:34:56.789Z",
        );
        assert_eq!(
            path,
            PathBuf::from(
                "/tmp/eudaemon-observations/eudaemon-alpha-observe-once-2026-04-28T12-34-56-789Z.json"
            )
        );
    }

    #[test]
    fn readonly_heap_delta_observation_path_is_filesystem_safe() {
        let path = readonly_heap_delta_observation_path(
            Path::new("/tmp/eudaemon-observations"),
            "2026-04-28T12:34:56.789Z",
        );
        assert_eq!(
            path,
            PathBuf::from(
                "/tmp/eudaemon-observations/eudaemon-alpha-readonly-heap-delta-2026-04-28T12-34-56-789Z.json"
            )
        );
    }

    #[test]
    fn context_bundle_prep_observation_path_is_filesystem_safe() {
        let path = context_bundle_prep_observation_path(
            Path::new("/tmp/eudaemon-observations"),
            "2026-04-28T12:34:56.789Z",
        );
        assert_eq!(
            path,
            PathBuf::from(
                "/tmp/eudaemon-observations/eudaemon-alpha-context-bundle-prep-2026-04-28T12-34-56-789Z.json"
            )
        );
    }

    #[test]
    fn steward_reviewed_heap_emit_observation_path_is_filesystem_safe() {
        let path = steward_reviewed_heap_emit_observation_path(
            Path::new("/tmp/eudaemon-observations"),
            "2026-04-29T12:34:56.789Z",
        );
        assert_eq!(
            path,
            PathBuf::from(
                "/tmp/eudaemon-observations/eudaemon-alpha-steward-reviewed-heap-emission-2026-04-29T12-34-56-789Z.json"
            )
        );
    }

    #[test]
    fn role_rank_requires_operator_or_higher_for_heap_emit() {
        assert!(role_rank("operator") >= role_rank("operator"));
        assert!(role_rank("steward") >= role_rank("operator"));
        assert!(role_rank("admin") >= role_rank("operator"));
        assert!(role_rank("viewer") < role_rank("operator"));
        assert!(role_rank("editor") < role_rank("operator"));
    }

    #[test]
    fn heap_emit_body_limit_caps_operator_input() {
        std::env::set_var(HEAP_EMIT_BODY_LIMIT_ENV, "9999");
        assert_eq!(heap_emit_body_limit(), MAX_HEAP_EMIT_BODY_LIMIT);
        std::env::set_var(HEAP_EMIT_BODY_LIMIT_ENV, "0");
        assert_eq!(heap_emit_body_limit(), 1);
        std::env::remove_var(HEAP_EMIT_BODY_LIMIT_ENV);
    }

    #[test]
    fn heap_emit_workspace_id_requires_ulid() {
        assert!(is_heap_workspace_ulid("01KM4C04QY37V9RV9H2HH9J1NM"));
        assert!(!is_heap_workspace_ulid("initiative-132"));
        assert!(!is_heap_workspace_ulid("01KM4C04QY37V9RV9H2HH9J1NI"));
        assert!(!is_heap_workspace_ulid("01KM4C04QY37V9RV9H2HH9J1N"));
    }

    #[test]
    fn heap_read_summary_redacts_to_stable_fields() {
        let payload = serde_json::json!({
            "count": 1,
            "hasMore": false,
            "nextCursor": null,
            "changed": [{
                "projection": {
                    "artifactId": "artifact-1",
                    "blockId": "block-1",
                    "title": "Read me",
                    "workspaceId": "space-1",
                    "blockType": "note",
                    "emittedAt": "2026-04-28T00:00:00Z",
                    "updatedAt": "2026-04-28T00:01:00Z"
                },
                "surfaceJson": {"large": "payload"}
            }],
            "deleted": []
        });

        let summary = heap_read_summary("/api/cortex/studio/heap/changed_blocks", &payload);
        assert_eq!(summary.count, Some(1));
        assert_eq!(summary.changed_count, Some(1));
        assert_eq!(summary.deleted_count, Some(0));
        assert_eq!(summary.items[0].artifact_id.as_deref(), Some("artifact-1"));
        assert_eq!(summary.items[0].space_id.as_deref(), Some("space-1"));
    }

    #[test]
    fn context_block_ids_dedupes_and_caps_operator_input() {
        std::env::set_var(
            CONTEXT_BLOCK_IDS_ENV,
            " artifact-1,artifact-2,artifact-1,,artifact-3 ",
        );
        let ids = context_block_ids(2);
        std::env::remove_var(CONTEXT_BLOCK_IDS_ENV);

        assert_eq!(
            ids,
            vec!["artifact-1".to_string(), "artifact-2".to_string()]
        );
    }

    #[test]
    fn context_bundle_summary_redacts_surface_json_to_size() {
        let payload = serde_json::json!({
            "context_bundle": {
                "block_count": 1,
                "prepared_at": "2026-04-28T00:00:00Z",
                "blocks": [{
                    "artifact_id": "artifact-1",
                    "title": "Context me",
                    "block_type": "note",
                    "tags": ["alpha", "beta"],
                    "mentions": ["agent:eudaemon-alpha-01"],
                    "surface_json": {"secret": "do-not-copy", "text": "large payload"},
                    "updated_at": "2026-04-28T00:01:00Z"
                }]
            }
        });

        let summary = context_bundle_summary("/api/cortex/studio/heap/blocks/context", 1, &payload);

        assert_eq!(summary.block_count, Some(1));
        assert_eq!(summary.prepared_at.as_deref(), Some("2026-04-28T00:00:00Z"));
        assert_eq!(summary.requested_count, 1);
        assert_eq!(summary.returned_count, 1);
        assert_eq!(summary.blocks[0].artifact_id.as_deref(), Some("artifact-1"));
        assert_eq!(summary.blocks[0].tag_count, 2);
        assert_eq!(summary.blocks[0].mention_count, 1);
        assert!(summary.blocks[0].surface_json_bytes > 0);
        assert_eq!(
            serde_json::to_value(&summary)
                .unwrap()
                .to_string()
                .contains("do-not-copy"),
            false
        );
    }
}
