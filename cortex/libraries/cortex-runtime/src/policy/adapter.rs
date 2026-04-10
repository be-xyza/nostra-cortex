use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpPolicyError {
    EmptyCommand,
    PathNotAbsolute(String),
    PathOutsideAllowedRoots(String),
    InvalidLineNumber(usize),
    InvalidLimit(usize),
    CommandNotAllowed(String),
    EnvVarNotAllowed(String),
    OutputLimitExceeded { requested: usize, max: usize },
    NoAllowedRootsConfigured,
    Io(String),
}

impl Display for AcpPolicyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AcpPolicyError::EmptyCommand => write!(f, "terminal command is required"),
            AcpPolicyError::PathNotAbsolute(path) => write!(f, "path must be absolute: {}", path),
            AcpPolicyError::PathOutsideAllowedRoots(path) => {
                write!(f, "path outside allowed roots: {}", path)
            }
            AcpPolicyError::InvalidLineNumber(line) => {
                write!(f, "line number must be 1-based and > 0: {}", line)
            }
            AcpPolicyError::InvalidLimit(limit) => write!(f, "limit must be > 0: {}", limit),
            AcpPolicyError::CommandNotAllowed(cmd) => {
                write!(f, "terminal command not allowed by policy: {}", cmd)
            }
            AcpPolicyError::EnvVarNotAllowed(name) => {
                write!(f, "environment variable is not allowed: {}", name)
            }
            AcpPolicyError::OutputLimitExceeded { requested, max } => write!(
                f,
                "output byte limit {} exceeds policy max {}",
                requested, max
            ),
            AcpPolicyError::NoAllowedRootsConfigured => {
                write!(f, "at least one allowed root is required")
            }
            AcpPolicyError::Io(err) => write!(f, "io error: {}", err),
        }
    }
}

impl std::error::Error for AcpPolicyError {}

#[derive(Debug, Clone)]
pub struct AcpPolicyConfig {
    pub allowed_roots: Vec<PathBuf>,
    pub allowed_terminal_commands: HashSet<String>,
    pub allowed_env_vars: HashSet<String>,
    pub max_read_lines: usize,
    pub max_output_byte_limit: usize,
    pub max_terminal_wait_ms: u64,
    pub max_terminal_runtime_ms: u64,
}

impl AcpPolicyConfig {
    pub fn baseline(allowed_roots: Vec<PathBuf>) -> Self {
        Self {
            allowed_roots,
            allowed_terminal_commands: HashSet::from_iter([
                "cargo".to_string(),
                "git".to_string(),
                "icp".to_string(),
                "ls".to_string(),
                "cat".to_string(),
            ]),
            allowed_env_vars: HashSet::from_iter([
                "RUST_LOG".to_string(),
                "CARGO_TERM_COLOR".to_string(),
            ]),
            max_read_lines: 2_000,
            max_output_byte_limit: 1_048_576,
            max_terminal_wait_ms: 60_000,
            max_terminal_runtime_ms: 300_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FsReadTextFileRequest {
    pub session_id: String,
    pub path: String,
    pub line: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FsReadTextFileResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FsWriteTextFileRequest {
    pub session_id: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TerminalCreateRequest {
    pub session_id: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<EnvVariable>,
    pub cwd: Option<String>,
    pub output_byte_limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTerminalCreate {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<EnvVariable>,
    pub cwd: PathBuf,
    pub output_byte_limit: usize,
    pub max_wait_ms: u64,
    pub max_runtime_ms: u64,
}

pub trait OperationAdapter {
    fn validate_workspace_path(&self, path: &str) -> Result<PathBuf, AcpPolicyError>;
    fn validate_terminal_create(
        &self,
        req: TerminalCreateRequest,
    ) -> Result<ValidatedTerminalCreate, AcpPolicyError>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcpSessionUpdateKind {
    UserMessageChunk,
    AgentMessageChunk,
    AgentThoughtChunk,
    AgentBranching,
    ToolCall,
    ToolCallUpdate,
    ToolCallResult,
    Plan,
    AvailableCommandsUpdate,
    CurrentModeUpdate,
    ConfigOptionUpdate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NostraProjectionKind {
    UserInputChunk,
    AgentOutputChunk,
    PrivateThoughtChunk,
    AgentBranched,
    ToolCallStarted,
    ToolCallProgress,
    ToolCallCompleted,
    PlanSnapshot,
    CommandSetChanged,
    ModeChanged,
    SessionConfigChanged,
}

pub fn map_session_update_to_projection(kind: AcpSessionUpdateKind) -> NostraProjectionKind {
    match kind {
        AcpSessionUpdateKind::UserMessageChunk => NostraProjectionKind::UserInputChunk,
        AcpSessionUpdateKind::AgentMessageChunk => NostraProjectionKind::AgentOutputChunk,
        AcpSessionUpdateKind::AgentThoughtChunk => NostraProjectionKind::PrivateThoughtChunk,
        AcpSessionUpdateKind::AgentBranching => NostraProjectionKind::AgentBranched,
        AcpSessionUpdateKind::ToolCall => NostraProjectionKind::ToolCallStarted,
        AcpSessionUpdateKind::ToolCallUpdate => NostraProjectionKind::ToolCallProgress,
        AcpSessionUpdateKind::ToolCallResult => NostraProjectionKind::ToolCallCompleted,
        AcpSessionUpdateKind::Plan => NostraProjectionKind::PlanSnapshot,
        AcpSessionUpdateKind::AvailableCommandsUpdate => NostraProjectionKind::CommandSetChanged,
        AcpSessionUpdateKind::CurrentModeUpdate => NostraProjectionKind::ModeChanged,
        AcpSessionUpdateKind::ConfigOptionUpdate => NostraProjectionKind::SessionConfigChanged,
    }
}
