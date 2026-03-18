use std::path::{Path, PathBuf};
use tokio::process::Command;

pub fn unwrap_candid_string(out: &str) -> Option<String> {
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

pub async fn resolve_canister_id(env_key: &str, canister_name: &str) -> Result<String, String> {
    if let Ok(id) = std::env::var(env_key) {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    resolve_canister_id_from_cli(canister_name, None).await
}

pub async fn resolve_canister_id_any(
    env_keys: &[&str],
    canister_name: &str,
) -> Result<String, String> {
    for key in env_keys {
        if let Ok(id) = std::env::var(key) {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    let project_root = std::env::var("CORTEX_IC_PROJECT_ROOT")
        .map(PathBuf::from)
        .ok();

    resolve_canister_id_from_cli(canister_name, project_root.as_deref()).await
}

pub async fn resolve_canister_id_from_cli(
    canister_name: &str,
    project_root: Option<&Path>,
) -> Result<String, String> {
    let kind = match std::env::var("CORTEX_IC_CLI").as_deref() {
        Ok("icp") => "icp",
        _ => "dfx",
    };

    let mut cmd = Command::new(kind);
    cmd.args(["canister", "id", canister_name]);

    if let Some(root) = project_root {
        cmd.current_dir(root);
    }

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[derive(Clone, Debug, Default)]
pub struct DfxCli {
    project_root: Option<PathBuf>,
}

impl DfxCli {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        Self { project_root }
    }

    pub async fn is_installed() -> bool {
        Command::new("dfx")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn ping(&self) -> bool {
        self.output(&["ping"])
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub async fn canister_id(&self, name: &str) -> Result<String, String> {
        let output = self.output(&["canister", "id", name]).await?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub async fn canister_status_raw(&self, name: &str) -> Result<String, String> {
        let output = self.output(&["canister", "status", name]).await?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn run_canister_command(&self, cmd: &str, name: &str) -> Result<(), String> {
        let output = self.output(&["canister", cmd, name]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub async fn call_canister(
        &self,
        name: &str,
        method: &str,
        argument: Option<&str>,
    ) -> Result<String, String> {
        let mut command = Command::new("dfx");
        command.args(["canister", "call", name, method]);
        if let Some(arg) = argument {
            command.arg(arg);
        }
        if let Some(root) = self.project_root.as_deref() {
            command.current_dir(root);
        }
        let output = command.output().await.map_err(|err| err.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn output(&self, args: &[&str]) -> Result<std::process::Output, String> {
        let mut command = Command::new("dfx");
        command.args(args);
        if let Some(root) = self.project_root.as_deref() {
            command.current_dir(root);
        }
        command.output().await.map_err(|err| err.to_string())
    }
}
