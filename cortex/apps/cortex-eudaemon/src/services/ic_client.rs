use cortex_ic_adapter::dfx::unwrap_candid_string as unwrap_ic_candid_string;
use cortex_ic_adapter::ic_cli::{IcCliBackend, IcCliKind, IcpBackend};
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
struct ProjectConfig {
    canisters: Option<HashMap<String, Value>>,
}

pub struct LocalIcClient {
    kind: IcCliKind,
    pub backend: Arc<dyn IcCliBackend + Send + Sync>,
    project_root: Option<PathBuf>,
}
pub type IcClient = LocalIcClient;

impl LocalIcClient {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let backend: Arc<dyn IcCliBackend + Send + Sync> = Arc::new(IcpBackend {
            project_root: project_root.clone(),
        });

        Self {
            kind: IcCliKind::Icp,
            backend,
            project_root,
        }
    }

    pub fn kind(&self) -> IcCliKind {
        self.kind
    }

    pub fn unwrap_candid_string(out: &str) -> Option<String> {
        unwrap_ic_candid_string(out)
    }

    pub fn is_installed() -> bool {
        let icp_available = std::process::Command::new("icp")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        icp_available
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
        let config_path = self.find_project_config_path()?;
        let content = std::fs::read_to_string(config_path)
            .map_err(|e| IcCliError::ParseError(e.to_string()))?;
        let config: ProjectConfig =
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

    fn find_project_config_path(&self) -> Result<PathBuf, IcCliError> {
        let mut cur = self.root();
        loop {
            for candidate in ["icp.yaml", "icp.yml", "icp.json"] {
                let p = cur.join(candidate);
                if p.exists() {
                    return Ok(p);
                }
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

#[cfg(test)]
mod tests {
    // Canonical client only; no selector matrix remains.
}
