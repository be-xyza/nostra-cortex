use cortex_ic_adapter::ic_cli::{DfxBackend, IcCliBackend, IcCliKind, IcpBackend};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next >= '@' && next <= '~' {
                        break;
                    }
                }
            } else if chars.peek() == Some(&'(') || chars.peek() == Some(&')') {
                chars.next();
                chars.next();
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanisterStatus {
    Running,
    Stopped,
    Stopping,
    Unknown,
}
impl std::fmt::Display for CanisterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanisterStatus::Running => write!(f, "Running"),
            CanisterStatus::Stopped => write!(f, "Stopped"),
            CanisterStatus::Stopping => write!(f, "Stopping"),
            CanisterStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanisterInfo {
    pub name: String,
    pub id: String,
    pub status: CanisterStatus,
    pub cycles: u64,
    pub memory_size: u64,
    pub module_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IcCliError {
    ParseError(String),
    CommandFailed(String),
    NoProjectFound,
}
pub type DfxError = IcCliError;

impl std::fmt::Display for IcCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IcCliError::ParseError(message) => write!(f, "ParseError: {message}"),
            IcCliError::CommandFailed(message) => write!(f, "CommandFailed: {message}"),
            IcCliError::NoProjectFound => write!(f, "NoProjectFound"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct DfxConfig {
    canisters: Option<HashMap<String, Value>>,
}

pub struct LocalIcClient {
    pub backend: Arc<dyn IcCliBackend + Send + Sync>,
    project_root: Option<PathBuf>,
}
pub type DfxClient = LocalIcClient;

impl LocalIcClient {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let kind = match std::env::var("CORTEX_IC_CLI").as_deref() {
            Ok("icp") => IcCliKind::Icp,
            _ => IcCliKind::Dfx,
        };

        let backend: Arc<dyn IcCliBackend + Send + Sync> = match kind {
            IcCliKind::Icp => Arc::new(IcpBackend {
                project_root: project_root.clone(),
            }),
            IcCliKind::Dfx => Arc::new(DfxBackend {
                project_root: project_root.clone(),
            }),
        };

        Self {
            backend,
            project_root,
        }
    }

    pub fn unwrap_candid_string(out: &str) -> Option<String> {
        cortex_ic_adapter::dfx::unwrap_candid_string(out)
    }

    pub fn is_installed() -> bool {
        // Check current backend
        let kind = match std::env::var("CORTEX_IC_CLI").as_deref() {
            Ok("icp") => IcCliKind::Icp,
            _ => IcCliKind::Dfx,
        };
        match kind {
            IcCliKind::Icp => std::process::Command::new("icp")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false),
            IcCliKind::Dfx => std::process::Command::new("dfx")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false),
        }
    }

    pub async fn is_replica_running(&self) -> bool {
        self.backend.local_status().await
    }

    pub async fn start_replica(&self) -> Result<String, IcCliError> {
        tracing::info!("Starting local IC host...");
        self.backend
            .start_local()
            .await
            .map(|_| "Local IC host started".to_string())
            .map_err(IcCliError::CommandFailed)
    }

    pub async fn stop_replica(&self) -> Result<(), IcCliError> {
        self.backend
            .stop_local()
            .await
            .map_err(IcCliError::CommandFailed)
    }

    pub fn discover_canisters(&self) -> Result<Vec<String>, IcCliError> {
        // Still fallback to dfx.json searching for now as icp.yaml support is pending
        let dfx_json = self.find_dfx_json()?;
        let content =
            std::fs::read_to_string(dfx_json).map_err(|e| IcCliError::ParseError(e.to_string()))?;
        let config: DfxConfig =
            serde_json::from_str(&content).map_err(|e| IcCliError::ParseError(e.to_string()))?;
        Ok(config
            .canisters
            .map(|c| c.keys().cloned().collect())
            .unwrap_or_default())
    }

    pub async fn get_canister_status(&self, name: &str) -> Result<CanisterInfo, IcCliError> {
        let id = self
            .backend
            .canister_id(name)
            .await
            .map_err(IcCliError::CommandFailed)?;
        let status_text = self
            .backend
            .canister_status(name)
            .await
            .map_err(IcCliError::CommandFailed)?;
        let clean_text = strip_ansi_codes(&status_text);
        self.parse_status_output(name, &id, &clean_text)
    }

    pub async fn start_canister(&self, name: &str) -> Result<(), IcCliError> {
        self.backend
            .canister_call(name, "start", None)
            .await
            .map(|_| ())
            .map_err(IcCliError::CommandFailed)
    }

    pub async fn stop_canister(&self, name: &str) -> Result<(), IcCliError> {
        self.backend
            .canister_call(name, "stop", None)
            .await
            .map(|_| ())
            .map_err(IcCliError::CommandFailed)
    }

    pub async fn call_canister(
        &self,
        name: &str,
        method: &str,
        argument: Option<&str>,
    ) -> Result<String, IcCliError> {
        self.backend
            .canister_call(name, method, argument)
            .await
            .map_err(IcCliError::CommandFailed)
    }

    fn root(&self) -> PathBuf {
        self.project_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
    }

    fn find_dfx_json(&self) -> Result<PathBuf, IcCliError> {
        let mut cur = self.root();
        loop {
            let p = cur.join("dfx.json");
            if p.exists() {
                return Ok(p);
            }
            if !cur.pop() {
                return Err(IcCliError::NoProjectFound);
            }
        }
    }

    fn parse_status_output(
        &self,
        name: &str,
        id: &str,
        output: &str,
    ) -> Result<CanisterInfo, IcCliError> {
        let mut status = CanisterStatus::Unknown;
        let mut cycles = 0;
        let mut memory = 0;
        let mut hash = None;
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("Status:") {
                status = match line
                    .strip_prefix("Status:")
                    .unwrap()
                    .trim()
                    .to_lowercase()
                    .as_str()
                {
                    "running" => CanisterStatus::Running,
                    "stopped" => CanisterStatus::Stopped,
                    "stopping" => CanisterStatus::Stopping,
                    _ => CanisterStatus::Unknown,
                };
            } else if line.starts_with("Balance:") || line.starts_with("Cycles:") {
                let prefix = if line.starts_with("Balance:") {
                    "Balance:"
                } else {
                    "Cycles:"
                };
                cycles = line
                    .strip_prefix(prefix)
                    .unwrap()
                    .trim()
                    .replace('_', "")
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            } else if line.starts_with("Memory Size:") {
                memory = line
                    .strip_prefix("Memory Size:")
                    .unwrap()
                    .trim()
                    .replace('_', "")
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            } else if line.starts_with("Module Hash:") {
                hash = Some(
                    line.strip_prefix("Module Hash:")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
            }
        }
        Ok(CanisterInfo {
            name: name.into(),
            id: id.into(),
            status,
            cycles,
            memory_size: memory,
            module_hash: hash,
        })
    }
}
