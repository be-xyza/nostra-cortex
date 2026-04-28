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
const GATEWAY_URL_ENV: &str = "NOSTRA_GATEWAY_URL";
const CORTEX_GATEWAY_URL_ENV: &str = "CORTEX_GATEWAY_URL";
const OBSERVATION_DIR_ENV: &str = "NOSTRA_WORKER_OBSERVATION_DIR";
const VPS_STATE_ROOT_ENV: &str = "NOSTRA_VPS_STATE_ROOT";
const DEFAULT_GATEWAY_BASE_URL: &str = "http://127.0.0.1:3000";
const OBSERVE_ONCE_PACKET_ID: &str = "initiative-132-runtime-expansion-observe-once-v1";
const DEFAULT_AGENT_ID: &str = "agent:eudaemon-alpha-01";

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
    let safe_timestamp = observed_at.replace([':', '.'], "-");
    dir.join(format!("eudaemon-alpha-observe-once-{safe_timestamp}.json"))
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

    let whoami_url = format!("{gateway_base}/api/system/whoami");
    let whoami_result = reqwest::Client::new()
        .get(&whoami_url)
        .header("x-cortex-agent-id", &agent_id)
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
        authz_dev_mode,
        allow_unverified_role_header,
        agent_identity_enforcement,
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
}
