use async_trait::async_trait;
use cortex_domain::agent::mcp::protocol::{
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpError,
};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, message: JsonRpcMessage) -> Result<(), McpError>;
    async fn receive(&self) -> Result<JsonRpcMessage, McpError>;
}

pub struct StdioTransport {
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    // Hold onto the child process so it doesn't drop
    _child: Arc<Mutex<Child>>,
}

impl StdioTransport {
    /// Phase 1: Local development escape-hatch execution
    pub async fn start_local(command: &str, args: &[String], cwd: &str) -> Result<Self, McpError> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut child = cmd
            .spawn()
            .map_err(|e| McpError::Transport(e.to_string()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Transport("Failed to capture stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Transport("Failed to capture stdout".into()))?;

        Ok(Self {
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            _child: Arc::new(Mutex::new(child)),
        })
    }

    /// Phase 2: IronClaw Docker Sandbox execution for genuine isolation
    pub async fn start_isolated(
        image: &str,
        command: &str,
        args: &[String],
        cwd: &str,
    ) -> Result<Self, McpError> {
        let mut docker_args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "--read-only".to_string(),
            "--user".to_string(),
            "1000:1000".to_string(),
            "-i".to_string(),
            "-v".to_string(),
            format!("{}:/workspace:rw", cwd),
            "-w".to_string(),
            "/workspace".to_string(),
            "--cap-drop=ALL".to_string(),
            "--security-opt=no-new-privileges:true".to_string(),
            image.to_string(),
            command.to_string(),
        ];
        docker_args.extend_from_slice(args);

        Self::start_local("docker", &docker_args, cwd).await
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&self, message: JsonRpcMessage) -> Result<(), McpError> {
        let mut payload =
            serde_json::to_string(&message).map_err(|e| McpError::Transport(e.to_string()))?;
        payload.push('\n');

        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;
        stdin
            .flush()
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;
        Ok(())
    }

    async fn receive(&self) -> Result<JsonRpcMessage, McpError> {
        let mut stdout = self.stdout.lock().await;
        let mut line = String::new();
        let bytes_read = stdout
            .read_line(&mut line)
            .await
            .map_err(|e| McpError::Transport(e.to_string()))?;

        if bytes_read == 0 {
            return Err(McpError::Transport("EOF reached".into()));
        }

        let msg: JsonRpcMessage =
            serde_json::from_str(&line).map_err(|e| McpError::Parse(e.to_string()))?;
        Ok(msg)
    }
}
