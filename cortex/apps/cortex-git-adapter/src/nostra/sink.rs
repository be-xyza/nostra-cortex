use crate::config::AppConfig;
use async_trait::async_trait;
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;
use ic_agent::identity::AnonymousIdentity;
use serde_json::Value;
use std::path::PathBuf;
use tokio::process::Command;

#[async_trait]
pub trait NostraSink: Send + Sync {
    async fn execute_kip(&self, command: &str) -> Result<Value, String>;
}

#[derive(Clone)]
pub struct NostraSinkHandle(std::sync::Arc<dyn NostraSink>);

impl NostraSinkHandle {
    pub fn new(inner: std::sync::Arc<dyn NostraSink>) -> Self {
        Self(inner)
    }

    pub async fn execute_kip(&self, command: &str) -> Result<Value, String> {
        self.0.execute_kip(command).await
    }
}

pub async fn build_sink(config: &AppConfig) -> anyhow::Result<NostraSinkHandle> {
    let method = config.kip_method.trim().to_string();
    if config.use_dfx {
        let cwd = config
            .dfx_project_root
            .clone()
            .ok_or_else(|| anyhow::anyhow!("CORTEX_GIT_ADAPTER_USE_DFX is enabled but CORTEX_GIT_ADAPTER_DFX_PROJECT_ROOT is not set"))?;
        return Ok(NostraSinkHandle::new(std::sync::Arc::new(DfxSink {
            cwd,
            canister: config.dfx_canister_name.clone(),
            method,
        })));
    }

    let canister_id_text = config
        .nostra_kip_canister_id
        .as_deref()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing KIP canister id; set CANISTER_ID_NOSTRA_KIP (or enable dfx mode)"
            )
        })?;
    let canister_id = Principal::from_text(canister_id_text)?;
    let agent = build_agent(&config.nostra_ic_host).await?;
    Ok(NostraSinkHandle::new(std::sync::Arc::new(IcAgentSink {
        agent: std::sync::Arc::new(agent),
        canister_id,
        method,
    })))
}

async fn build_agent(url: &str) -> anyhow::Result<Agent> {
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(AnonymousIdentity)
        .build()?;
    if url.contains("127.0.0.1") || url.contains("localhost") {
        agent.fetch_root_key().await?;
    }
    Ok(agent)
}

struct DfxSink {
    cwd: PathBuf,
    canister: String,
    method: String,
}

#[async_trait]
impl NostraSink for DfxSink {
    async fn execute_kip(&self, command: &str) -> Result<Value, String> {
        // dfx arg is a candid tuple: ("<cmd>")
        let arg = format!("(\"{}\")", escape_candid_string(command));
        let out = Command::new("dfx")
            .args([
                "canister",
                "call",
                &self.canister,
                self.method.as_str(),
                &arg,
            ])
            .current_dir(&self.cwd)
            .output()
            .await
            .map_err(|e| format!("failed to run dfx: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "dfx canister call failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        parse_execute_kip_candid_output(stdout.as_str())
    }
}

struct IcAgentSink {
    agent: std::sync::Arc<Agent>,
    canister_id: Principal,
    method: String,
}

#[allow(non_camel_case_types)]
#[derive(candid::CandidType, candid::Deserialize, Debug)]
enum KipResult {
    ok((String, Option<KipEntity>)),
    err(String),
}

#[allow(non_camel_case_types)]
#[derive(candid::CandidType, candid::Deserialize, Debug)]
enum LegacyResult {
    ok(String),
    err(String),
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
struct KipEntity {
    id: String,
}

#[async_trait]
impl NostraSink for IcAgentSink {
    async fn execute_kip(&self, command: &str) -> Result<Value, String> {
        let arg = Encode!(&command.to_string()).map_err(|e| e.to_string())?;
        let bytes = self
            .agent
            .update(&self.canister_id, self.method.as_str())
            .with_arg(arg)
            .call_and_wait()
            .await
            .map_err(|e| format!("ic-agent update failed: {e}"))?;

        if self.method == "executeKip" {
            let decoded =
                Decode!(&bytes, LegacyResult).map_err(|e| format!("decode failed: {e}"))?;
            match decoded {
                LegacyResult::ok(json) => {
                    serde_json::from_str(&json).map_err(|e| e.to_string())
                }
                LegacyResult::err(message) => Err(message),
            }
        } else {
            let decoded =
                Decode!(&bytes, KipResult).map_err(|e| format!("decode failed: {e}"))?;
            match decoded {
                KipResult::ok((json, _entity)) => {
                    serde_json::from_str(&json).map_err(|e| e.to_string())
                }
                KipResult::err(message) => Err(message),
            }
        }
    }
}

fn escape_candid_string(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

fn parse_execute_kip_candid_output(raw: &str) -> Result<Value, String> {
    let trimmed = raw.trim();
    let is_ok = trimmed.contains("variant { ok =");
    let is_err = trimmed.contains("variant { err =");
    if !is_ok && !is_err {
        return Ok(serde_json::json!({ "raw": trimmed }));
    }

    let extracted = unwrap_candid_string(trimmed).unwrap_or_else(|| "".to_string());
    if is_err {
        return Err(if extracted.is_empty() {
            "executeKip returned err (unparseable)".to_string()
        } else {
            extracted
        });
    }

    if extracted.is_empty() {
        return Ok(serde_json::json!({ "raw": trimmed }));
    }

    serde_json::from_str(&extracted).or_else(|_| Ok(serde_json::json!({ "raw": extracted })))
}

fn unwrap_candid_string(out: &str) -> Option<String> {
    let s = out.trim();
    if !s.starts_with('(') {
        return None;
    }
    let start_quote = s.find('"')?;
    let mut escaped = false;
    let mut end_quote: Option<usize> = None;
    for (i, ch) in s[start_quote + 1..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            end_quote = Some(start_quote + 1 + i);
            break;
        }
    }
    let end_quote = end_quote?;
    let literal = &s[start_quote + 1..end_quote];
    let json = format!("\"{}\"", literal);
    serde_json::from_str::<String>(&json).ok()
}
