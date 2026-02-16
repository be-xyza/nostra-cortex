use crate::services::acp_adapter::{EnvVariable, ValidatedTerminalCreate};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::process::Stdio;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex as TokioMutex, broadcast};

pub struct TerminalService {}

static OUTPUT_CHANNEL: OnceLock<broadcast::Sender<String>> = OnceLock::new();
static INPUT_CHANNEL: OnceLock<broadcast::Sender<String>> = OnceLock::new();
static ACP_TERMINALS: OnceLock<Arc<Mutex<HashMap<String, Arc<AcpTerminalState>>>>> =
    OnceLock::new();

struct AcpTerminalState {
    child: TokioMutex<Child>,
    output: TokioMutex<VecDeque<u8>>,
    output_limit: usize,
    max_wait_ms: u64,
    stop_reason: TokioMutex<Option<String>>,
    dropped_bytes: AtomicUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalCreateResponse {
    pub terminal_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub output_byte_limit: usize,
    pub max_wait_ms: u64,
    pub max_runtime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalOutputRequest {
    pub terminal_id: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalOutputResponse {
    pub terminal_id: String,
    pub output: String,
    pub output_byte_limit: usize,
    pub truncated: bool,
    pub dropped_bytes: usize,
    pub completed: bool,
    pub exit_code: Option<i32>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalWaitRequest {
    pub terminal_id: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalWaitResponse {
    pub terminal_id: String,
    pub exited: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub effective_timeout_ms: u64,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalKillResponse {
    pub terminal_id: String,
    pub killed: bool,
    pub exit_code: Option<i32>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcpTerminalReleaseResponse {
    pub terminal_id: String,
    pub released: bool,
}

pub fn get_terminal_output_tx() -> broadcast::Sender<String> {
    OUTPUT_CHANNEL
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(1024);
            tx
        })
        .clone()
}

pub fn get_terminal_input_tx() -> broadcast::Sender<String> {
    INPUT_CHANNEL
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(1024);
            tx
        })
        .clone()
}

impl TerminalService {
    pub fn spawn_shell() {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open pty");

        let cmd = CommandBuilder::new_default_prog();
        let _child = pair
            .slave
            .spawn_command(cmd)
            .expect("failed to spawn shell");

        let mut reader = pair
            .master
            .try_clone_reader()
            .expect("failed to clone reader");
        let writer = Arc::new(Mutex::new(
            pair.master.take_writer().expect("failed to take writer"),
        ));

        tokio::spawn(async move {
            let tx = get_terminal_output_tx();
            let mut buf = [0u8; 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = tx.send(msg);
                    }
                    Ok(_) => break,
                    Err(_) => break,
                }
            }
        });

        let mut input_rx = get_terminal_input_tx().subscribe();
        let pty_writer = Arc::clone(&writer);
        tokio::spawn(async move {
            while let Ok(msg) = input_rx.recv().await {
                let mut guard = pty_writer.lock().unwrap();
                let _ = guard.write_all(msg.as_bytes());
                let _ = guard.flush();
            }
        });
    }

    pub async fn acp_terminal_create(
        validated: ValidatedTerminalCreate,
    ) -> Result<AcpTerminalCreateResponse, String> {
        let mut cmd = Command::new(&validated.command);
        cmd.args(&validated.args)
            .current_dir(&validated.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for EnvVariable { name, value } in &validated.env {
            cmd.env(name, value);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn terminal command: {}", e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let terminal_id = format!("term_{}", uuid::Uuid::new_v4().simple());
        let state = Arc::new(AcpTerminalState {
            child: TokioMutex::new(child),
            output: TokioMutex::new(VecDeque::new()),
            output_limit: validated.output_byte_limit,
            max_wait_ms: validated.max_wait_ms,
            stop_reason: TokioMutex::new(None),
            dropped_bytes: AtomicUsize::new(0),
        });

        if let Some(stream) = stdout {
            tokio::spawn(pipe_output_stream(stream, state.clone()));
        }
        if let Some(stream) = stderr {
            tokio::spawn(pipe_output_stream(stream, state.clone()));
        }

        if validated.max_runtime_ms > 0 {
            tokio::spawn(enforce_runtime_limit(
                state.clone(),
                validated.max_runtime_ms,
            ));
        }

        terminal_registry()
            .lock()
            .unwrap()
            .insert(terminal_id.clone(), state);

        Ok(AcpTerminalCreateResponse {
            terminal_id,
            command: validated.command,
            args: validated.args,
            cwd: validated.cwd.display().to_string(),
            output_byte_limit: validated.output_byte_limit,
            max_wait_ms: validated.max_wait_ms,
            max_runtime_ms: validated.max_runtime_ms,
        })
    }

    pub async fn acp_terminal_output(
        req: AcpTerminalOutputRequest,
    ) -> Result<AcpTerminalOutputResponse, String> {
        let state = get_terminal_state(&req.terminal_id)?;
        let read_limit = req
            .limit
            .unwrap_or(state.output_limit)
            .max(1)
            .min(state.output_limit);

        let output_bytes = {
            let output = state.output.lock().await;
            output.iter().copied().collect::<Vec<u8>>()
        };

        let slice = if output_bytes.len() > read_limit {
            &output_bytes[output_bytes.len() - read_limit..]
        } else {
            &output_bytes[..]
        };

        let (completed, exit_code) = {
            let mut child = state.child.lock().await;
            match child.try_wait() {
                Ok(Some(status)) => (true, status.code()),
                Ok(None) => (false, None),
                Err(_) => (false, None),
            }
        };

        let dropped = state.dropped_bytes.load(Ordering::Relaxed);
        let stop_reason = state.stop_reason.lock().await.clone();

        Ok(AcpTerminalOutputResponse {
            terminal_id: req.terminal_id,
            output: String::from_utf8_lossy(slice).to_string(),
            output_byte_limit: state.output_limit,
            truncated: dropped > 0 || output_bytes.len() > read_limit,
            dropped_bytes: dropped,
            completed,
            exit_code,
            stop_reason,
        })
    }

    pub async fn acp_terminal_wait_for_exit(
        req: AcpTerminalWaitRequest,
    ) -> Result<AcpTerminalWaitResponse, String> {
        let state = get_terminal_state(&req.terminal_id)?;
        let requested_timeout = req.timeout_ms.unwrap_or(state.max_wait_ms);
        let effective_timeout_ms = requested_timeout.clamp(1, state.max_wait_ms.max(1));
        let timeout = Duration::from_millis(effective_timeout_ms);

        let mut child = state.child.lock().await;
        match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => {
                if state.stop_reason.lock().await.is_none() {
                    *state.stop_reason.lock().await = Some("process_exited".to_string());
                }
                Ok(AcpTerminalWaitResponse {
                    terminal_id: req.terminal_id,
                    exited: true,
                    exit_code: status.code(),
                    timed_out: false,
                    effective_timeout_ms,
                    stop_reason: state.stop_reason.lock().await.clone(),
                })
            }
            Ok(Err(e)) => Err(format!("wait_for_exit failed: {}", e)),
            Err(_) => {
                *state.stop_reason.lock().await = Some("wait_timeout".to_string());
                Ok(AcpTerminalWaitResponse {
                    terminal_id: req.terminal_id,
                    exited: false,
                    exit_code: None,
                    timed_out: true,
                    effective_timeout_ms,
                    stop_reason: state.stop_reason.lock().await.clone(),
                })
            }
        }
    }

    pub async fn acp_terminal_kill(terminal_id: String) -> Result<AcpTerminalKillResponse, String> {
        let state = get_terminal_state(&terminal_id)?;
        let mut child = state.child.lock().await;

        child
            .kill()
            .await
            .map_err(|e| format!("failed to kill terminal: {}", e))?;

        let status = child.wait().await.ok().and_then(|s| s.code());
        if state.stop_reason.lock().await.is_none() {
            *state.stop_reason.lock().await = Some("killed_by_client".to_string());
        }

        let stop_reason = state.stop_reason.lock().await.clone();
        Ok(AcpTerminalKillResponse {
            terminal_id,
            killed: true,
            exit_code: status,
            stop_reason,
        })
    }

    pub fn acp_terminal_release(terminal_id: String) -> Result<AcpTerminalReleaseResponse, String> {
        let removed = terminal_registry()
            .lock()
            .unwrap()
            .remove(&terminal_id)
            .is_some();
        if !removed {
            return Err(format!("unknown terminalId: {}", terminal_id));
        }

        Ok(AcpTerminalReleaseResponse {
            terminal_id,
            released: true,
        })
    }
}

fn terminal_registry() -> Arc<Mutex<HashMap<String, Arc<AcpTerminalState>>>> {
    ACP_TERMINALS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

fn get_terminal_state(terminal_id: &str) -> Result<Arc<AcpTerminalState>, String> {
    terminal_registry()
        .lock()
        .unwrap()
        .get(terminal_id)
        .cloned()
        .ok_or_else(|| format!("unknown terminalId: {}", terminal_id))
}

async fn pipe_output_stream<R>(mut stream: R, state: Arc<AcpTerminalState>)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => append_output(&state, &buf[..n]).await,
            Err(_) => break,
        }
    }
}

async fn append_output(state: &AcpTerminalState, bytes: &[u8]) {
    let mut output = state.output.lock().await;
    output.extend(bytes.iter().copied());

    if output.len() > state.output_limit {
        let overflow = output.len() - state.output_limit;
        for _ in 0..overflow {
            let _ = output.pop_front();
        }
        state.dropped_bytes.fetch_add(overflow, Ordering::Relaxed);
    }
}

async fn enforce_runtime_limit(state: Arc<AcpTerminalState>, max_runtime_ms: u64) {
    tokio::time::sleep(Duration::from_millis(max_runtime_ms)).await;

    let mut child = state.child.lock().await;
    match child.try_wait() {
        Ok(Some(_)) => {}
        Ok(None) => {
            if child.kill().await.is_ok() {
                let _ = child.wait().await;
                *state.stop_reason.lock().await = Some("runtime_limit_exceeded".to_string());
            }
        }
        Err(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::acp_adapter::ValidatedTerminalCreate;

    fn validated_create(
        command: &str,
        args: Vec<&str>,
        output_limit: usize,
    ) -> ValidatedTerminalCreate {
        ValidatedTerminalCreate {
            command: command.to_string(),
            args: args.into_iter().map(ToString::to_string).collect(),
            env: vec![],
            cwd: std::env::current_dir().unwrap(),
            output_byte_limit: output_limit,
            max_wait_ms: 5_000,
            max_runtime_ms: 5_000,
        }
    }

    #[tokio::test]
    async fn terminal_lifecycle_create_wait_output_release() {
        let created = TerminalService::acp_terminal_create(validated_create(
            "echo",
            vec!["hello-acp"],
            4_096,
        ))
        .await
        .unwrap();

        let waited = TerminalService::acp_terminal_wait_for_exit(AcpTerminalWaitRequest {
            terminal_id: created.terminal_id.clone(),
            timeout_ms: Some(5_000),
        })
        .await
        .unwrap();
        assert!(waited.exited);

        tokio::time::sleep(Duration::from_millis(30)).await;
        let output = TerminalService::acp_terminal_output(AcpTerminalOutputRequest {
            terminal_id: created.terminal_id.clone(),
            limit: None,
        })
        .await
        .unwrap();

        assert!(output.output.contains("hello-acp"));
        let released = TerminalService::acp_terminal_release(created.terminal_id).unwrap();
        assert!(released.released);
    }

    #[tokio::test]
    async fn terminal_output_truncation_is_deterministic() {
        let created = TerminalService::acp_terminal_create(validated_create(
            "sh",
            vec!["-c", "printf 1234567890"],
            4,
        ))
        .await
        .unwrap();

        let _ = TerminalService::acp_terminal_wait_for_exit(AcpTerminalWaitRequest {
            terminal_id: created.terminal_id.clone(),
            timeout_ms: Some(5_000),
        })
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(30)).await;
        let output = TerminalService::acp_terminal_output(AcpTerminalOutputRequest {
            terminal_id: created.terminal_id.clone(),
            limit: None,
        })
        .await
        .unwrap();

        assert_eq!(output.output, "7890");
        assert!(output.truncated);
        assert!(output.dropped_bytes >= 6);

        let _ = TerminalService::acp_terminal_release(created.terminal_id);
    }

    #[tokio::test]
    async fn terminal_runtime_limit_auto_kills_process() {
        let mut validated = validated_create("sh", vec!["-c", "sleep 2"], 1_024);
        validated.max_runtime_ms = 100;
        validated.max_wait_ms = 2_000;

        let created = TerminalService::acp_terminal_create(validated)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(250)).await;
        let waited = TerminalService::acp_terminal_wait_for_exit(AcpTerminalWaitRequest {
            terminal_id: created.terminal_id.clone(),
            timeout_ms: Some(2_000),
        })
        .await
        .unwrap();

        assert!(waited.exited);
        assert_eq!(
            waited.stop_reason.as_deref(),
            Some("runtime_limit_exceeded")
        );
        let _ = TerminalService::acp_terminal_release(created.terminal_id);
    }

    #[tokio::test]
    async fn terminal_kill_sets_deterministic_stop_reason() {
        let mut validated = validated_create("sh", vec!["-c", "sleep 5"], 1_024);
        validated.max_runtime_ms = 10_000;

        let created = TerminalService::acp_terminal_create(validated)
            .await
            .unwrap();
        let killed = TerminalService::acp_terminal_kill(created.terminal_id.clone())
            .await
            .unwrap();

        assert!(killed.killed);
        assert_eq!(killed.stop_reason.as_deref(), Some("killed_by_client"));
        let _ = TerminalService::acp_terminal_release(created.terminal_id);
    }
}
