use anyhow::Result;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

use cortex_worker::config_service::ConfigService;

const WORKER_KEYS_PATH_ENV: &str = "NOSTRA_WORKER_KEYS_PATH";
const DEFAULT_WORKER_KEYS_PATH: &str = "worker_keys.json";
const RUN_ONCE_ENV: &str = "NOSTRA_WORKER_RUN_ONCE";

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
    std::env::var(RUN_ONCE_ENV)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
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
