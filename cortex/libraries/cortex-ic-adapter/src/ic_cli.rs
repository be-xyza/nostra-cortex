use async_trait::async_trait;
use ic_agent::identity::AnonymousIdentity;
use candid::Principal;
use ic_agent::Agent;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IcCliKind {
    Dfx,
    Icp,
}

#[derive(Clone, Debug)]
pub struct IcCliConfig {
    pub kind: IcCliKind,
    pub project_root: Option<PathBuf>,
    pub network: String,
    pub local_host: Option<String>,
    pub output_env_file: Option<PathBuf>,
}

#[async_trait]
pub trait IcCliBackend: Send + Sync {
    fn kind(&self) -> IcCliKind;
    async fn version(&self) -> Result<String, String>;
    async fn is_installed(&self) -> bool;
    async fn local_status(&self) -> bool;
    async fn start_local(&self) -> Result<(), String>;
    async fn stop_local(&self) -> Result<(), String>;
    async fn build_check(&self, project_root: &Path) -> Result<(), String>;
    async fn deploy(
        &self,
        project_root: &Path,
        network: &str,
        mode: Option<&str>,
    ) -> Result<(), String>;
    async fn canister_id(&self, name: &str) -> Result<String, String>;
    async fn canister_status(&self, name: &str) -> Result<String, String>;
    async fn canister_call(
        &self,
        name: &str,
        method: &str,
        argument: Option<&str>,
    ) -> Result<String, String>;
    async fn snapshot_create(&self, name: &str, network: &str) -> Result<String, String>;
    async fn snapshot_load(
        &self,
        snapshot_id: &str,
        name: &str,
        network: &str,
    ) -> Result<(), String>;
}

pub struct IcpBackend {
    pub project_root: Option<PathBuf>,
}

impl IcpBackend {
    fn run_cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("icp");
        if let Some(ref root) = self.project_root {
            cmd.current_dir(root);
        }
        cmd.args(args);
        cmd
    }

    async fn exec(&self, args: &[&str]) -> Result<std::process::Output, String> {
        self.run_cmd(args)
            .output()
            .await
            .map_err(|e| format!("failed to execute icp {}: {}", args.join(" "), e))
    }
}

#[async_trait]
impl IcCliBackend for IcpBackend {
    fn kind(&self) -> IcCliKind {
        IcCliKind::Icp
    }

    async fn version(&self) -> Result<String, String> {
        let output = self.exec(&["--version"]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn is_installed(&self) -> bool {
        Command::new("icp")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn local_status(&self) -> bool {
        self.exec(&["network", "status"])
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn start_local(&self) -> Result<(), String> {
        let output = self.exec(&["network", "start", "-d"]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn stop_local(&self) -> Result<(), String> {
        let output = self.exec(&["network", "stop"]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn build_check(&self, _project_root: &Path) -> Result<(), String> {
        let output = self.exec(&["build"]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn deploy(
        &self,
        _project_root: &Path,
        network: &str,
        _mode: Option<&str>,
    ) -> Result<(), String> {
        let output = self.exec(&["deploy", "-e", network]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn canister_id(&self, name: &str) -> Result<String, String> {
        let output = self.exec(&["canister", "id", name]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn canister_status(&self, name: &str) -> Result<String, String> {
        let output = self.exec(&["canister", "status", name]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn canister_call(
        &self,
        name: &str,
        method: &str,
        argument: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["canister", "call", name, method];
        if let Some(arg) = argument {
            args.push(arg);
        }
        let output = self.exec(&args).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn snapshot_create(&self, name: &str, _network: &str) -> Result<String, String> {
        let output = self.exec(&["canister", "snapshot", "create", name]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn snapshot_load(
        &self,
        snapshot_id: &str,
        name: &str,
        _network: &str,
    ) -> Result<(), String> {
        let output = self
            .exec(&["canister", "snapshot", "load", name, snapshot_id])
            .await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

pub struct DfxBackend {
    pub project_root: Option<PathBuf>,
}

impl DfxBackend {
    fn run_cmd(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("dfx");
        if let Some(ref root) = self.project_root {
            cmd.current_dir(root);
        }
        cmd.args(args);
        cmd
    }

    async fn exec(&self, args: &[&str]) -> Result<std::process::Output, String> {
        self.run_cmd(args)
            .output()
            .await
            .map_err(|e| format!("failed to execute dfx {}: {}", args.join(" "), e))
    }
}

#[async_trait]
impl IcCliBackend for DfxBackend {
    fn kind(&self) -> IcCliKind {
        IcCliKind::Dfx
    }

    async fn version(&self) -> Result<String, String> {
        let output = self.exec(&["--version"]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn is_installed(&self) -> bool {
        Command::new("dfx")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn local_status(&self) -> bool {
        self.exec(&["ping"])
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn start_local(&self) -> Result<(), String> {
        let output = self.exec(&["start", "--background"]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn stop_local(&self) -> Result<(), String> {
        let output = self.exec(&["stop"]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn build_check(&self, _project_root: &Path) -> Result<(), String> {
        let output = self.exec(&["build", "--check"]).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn deploy(
        &self,
        _project_root: &Path,
        network: &str,
        mode: Option<&str>,
    ) -> Result<(), String> {
        let mut args = vec!["deploy", "--network", network];
        if let Some(m) = mode {
            args.push("--mode");
            args.push(m);
        }
        let output = self.exec(&args).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn canister_id(&self, name: &str) -> Result<String, String> {
        let output = self.exec(&["canister", "id", name]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn canister_status(&self, name: &str) -> Result<String, String> {
        let output = self.exec(&["canister", "status", name]).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn canister_call(
        &self,
        name: &str,
        method: &str,
        argument: Option<&str>,
    ) -> Result<String, String> {
        let mut args = vec!["canister", "call", name, method];
        if let Some(arg) = argument {
            args.push(arg);
        }
        let output = self.exec(&args).await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn snapshot_create(&self, name: &str, network: &str) -> Result<String, String> {
        let output = self
            .exec(&["canister", "--network", network, "snapshot", "create", name])
            .await?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn snapshot_load(
        &self,
        snapshot_id: &str,
        name: &str,
        network: &str,
    ) -> Result<(), String> {
        let output = self
            .exec(&[
                "canister",
                "--network",
                network,
                "snapshot",
                "load",
                name,
                snapshot_id,
            ])
            .await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

pub struct DirectAgentBackend {
    pub cli_backend: Box<dyn IcCliBackend>,
    pub agent: Agent,
}

impl DirectAgentBackend {
    pub async fn new(project_root: Option<PathBuf>, host: &str) -> Result<Self, String> {
        let cli_backend: Box<dyn IcCliBackend> = Box::new(IcpBackend { project_root });

        let agent: Agent = Agent::builder()
            .with_url(host)
            .with_identity(AnonymousIdentity)
            .build()
            .map_err(|e| format!("failed to build agent: {}", e))?;

        if host.contains("127.0.0.1") || host.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|e| format!("failed to fetch root key: {}", e))?;
        }

        Ok(Self { cli_backend, agent })
    }
}

#[async_trait]
impl IcCliBackend for DirectAgentBackend {
    fn kind(&self) -> IcCliKind {
        self.cli_backend.kind()
    }

    async fn version(&self) -> Result<String, String> {
        self.cli_backend.version().await
    }

    async fn is_installed(&self) -> bool {
        self.cli_backend.is_installed().await
    }

    async fn local_status(&self) -> bool {
        match self.agent.status().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn start_local(&self) -> Result<(), String> {
        self.cli_backend.start_local().await
    }

    async fn stop_local(&self) -> Result<(), String> {
        self.cli_backend.stop_local().await
    }

    async fn build_check(&self, project_root: &Path) -> Result<(), String> {
        self.cli_backend.build_check(project_root).await
    }

    async fn deploy(
        &self,
        project_root: &Path,
        network: &str,
        mode: Option<&str>,
    ) -> Result<(), String> {
        self.cli_backend.deploy(project_root, network, mode).await
    }

    async fn canister_id(&self, name: &str) -> Result<String, String> {
        self.cli_backend.canister_id(name).await
    }

    async fn canister_status(&self, name: &str) -> Result<String, String> {
        let canister_id = self.canister_id(name).await?;
        let _principal = Principal::from_text(&canister_id).map_err(|err| format!("Invalid principal: {err}"))?;

        // ic-agent's canister_status requires the agent to be an controller or use a specific management canister call
        // For local development, we might just fallback to CLI if we are not the controller.
        // But let's try querying management canister.
        self.cli_backend.canister_status(name).await
    }

    async fn canister_call(
        &self,
        name: &str,
        method: &str,
        argument: Option<&str>,
    ) -> Result<String, String> {
        // Here we could use ic-agent for actual calls.
        // For now, let's keep it as a placeholder to show we can intercept.
        // Real implementation would need Candid serialization/deserialization.
        self.cli_backend.canister_call(name, method, argument).await
    }

    async fn snapshot_create(&self, name: &str, network: &str) -> Result<String, String> {
        self.cli_backend.snapshot_create(name, network).await
    }

    async fn snapshot_load(
        &self,
        snapshot_id: &str,
        name: &str,
        network: &str,
    ) -> Result<(), String> {
        self.cli_backend
            .snapshot_load(snapshot_id, name, network)
            .await
    }
}
