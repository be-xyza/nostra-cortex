use crate::DomainError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionUpdateKind {
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
pub enum ProjectionKind {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceContext {
    pub traceparent: Option<String>,
    pub tracestate: Option<String>,
    pub baggage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectedEvent {
    pub id: String,
    pub session_id: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub session_update_kind: SessionUpdateKind,
    pub projection_kind: ProjectionKind,
    pub timestamp: u64,
    #[serde(default)]
    pub trace: TraceContext,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct ProjectSessionUpdateInput {
    pub session_id: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub kind: SessionUpdateKind,
    pub payload: Value,
    pub trace: TraceContext,
    pub timestamp: u64,
}

pub fn deterministic_event_id(
    session_id: &str,
    turn_seq: u64,
    update_seq: u64,
    kind: SessionUpdateKind,
) -> String {
    let raw = format!("{}:{}:{}:{:?}", session_id, turn_seq, update_seq, kind);
    let digest = Sha256::digest(raw.as_bytes());
    format!("acp_evt_{}", hex::encode(digest))
}

pub fn projection_for_update(kind: SessionUpdateKind) -> ProjectionKind {
    match kind {
        SessionUpdateKind::UserMessageChunk => ProjectionKind::UserInputChunk,
        SessionUpdateKind::AgentMessageChunk => ProjectionKind::AgentOutputChunk,
        SessionUpdateKind::AgentThoughtChunk => ProjectionKind::PrivateThoughtChunk,
        SessionUpdateKind::AgentBranching => ProjectionKind::AgentBranched,
        SessionUpdateKind::ToolCall => ProjectionKind::ToolCallStarted,
        SessionUpdateKind::ToolCallUpdate => ProjectionKind::ToolCallProgress,
        SessionUpdateKind::ToolCallResult => ProjectionKind::ToolCallCompleted,
        SessionUpdateKind::Plan => ProjectionKind::PlanSnapshot,
        SessionUpdateKind::AvailableCommandsUpdate => ProjectionKind::CommandSetChanged,
        SessionUpdateKind::CurrentModeUpdate => ProjectionKind::ModeChanged,
        SessionUpdateKind::ConfigOptionUpdate => ProjectionKind::SessionConfigChanged,
    }
}

pub fn project_session_update(
    input: ProjectSessionUpdateInput,
) -> Result<ProjectedEvent, DomainError> {
    if input.session_id.is_empty() {
        return Err(DomainError::InvalidInput("session_id must not be empty"));
    }

    Ok(ProjectedEvent {
        id: deterministic_event_id(
            &input.session_id,
            input.turn_seq,
            input.update_seq,
            input.kind,
        ),
        session_id: input.session_id,
        turn_seq: input.turn_seq,
        update_seq: input.update_seq,
        projection_kind: projection_for_update(input.kind),
        session_update_kind: input.kind,
        timestamp: input.timestamp,
        trace: input.trace,
        payload: input.payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_ids_are_stable() {
        let a = deterministic_event_id("sess_1", 1, 1, SessionUpdateKind::Plan);
        let b = deterministic_event_id("sess_1", 1, 1, SessionUpdateKind::Plan);
        assert_eq!(a, b);
    }

    #[test]
    fn project_requires_non_empty_session_id() {
        let err = project_session_update(ProjectSessionUpdateInput {
            session_id: String::new(),
            turn_seq: 1,
            update_seq: 1,
            kind: SessionUpdateKind::Plan,
            payload: Value::Null,
            trace: TraceContext::default(),
            timestamp: 10,
        })
        .unwrap_err();

        assert!(matches!(err, DomainError::InvalidInput(_)));
    }
}
