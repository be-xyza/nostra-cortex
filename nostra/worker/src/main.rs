use anyhow::Result;
use chrono::Utc;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
const GATEWAY_URL_ENV: &str = "NOSTRA_GATEWAY_URL";
const CORTEX_GATEWAY_URL_ENV: &str = "CORTEX_GATEWAY_URL";
const OBSERVATION_DIR_ENV: &str = "NOSTRA_WORKER_OBSERVATION_DIR";
const HEAP_CHANGED_SINCE_ENV: &str = "NOSTRA_WORKER_HEAP_CHANGED_SINCE";
const HEAP_SPACE_ID_ENV: &str = "NOSTRA_WORKER_HEAP_SPACE_ID";
const HEAP_LIMIT_ENV: &str = "NOSTRA_WORKER_HEAP_LIMIT";
const VPS_STATE_ROOT_ENV: &str = "NOSTRA_VPS_STATE_ROOT";
const DEFAULT_GATEWAY_BASE_URL: &str = "http://127.0.0.1:3000";
const OBSERVE_ONCE_PACKET_ID: &str = "initiative-132-runtime-expansion-observe-once-v1";
const READONLY_HEAP_DELTA_PACKET_ID: &str =
    "initiative-132-runtime-expansion-readonly-heap-delta-v1";
const DEFAULT_AGENT_ID: &str = "agent:eudaemon-alpha-01";
const DEFAULT_HEAP_LIMIT: usize = 25;
const MAX_HEAP_LIMIT: usize = 25;

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

#[derive(Default)]
struct GatewayAuthPosture {
    authz_dev_mode: Option<bool>,
    allow_unverified_role_header: Option<bool>,
    agent_identity_enforcement: Option<bool>,
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

fn mode_observation_path(dir: &Path, mode: &str, observed_at: &str) -> PathBuf {
    let safe_timestamp = observed_at.replace([':', '.'], "-");
    dir.join(format!("eudaemon-alpha-{mode}-{safe_timestamp}.json"))
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
}
