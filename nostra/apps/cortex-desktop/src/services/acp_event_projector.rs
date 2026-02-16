use crate::services::acp_adapter::{
    AcpSessionUpdateKind, NostraProjectionKind, map_session_update_to_projection,
};
use crate::services::acp_meta_policy::TraceContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[cfg(feature = "cortex_runtime_v0")]
use cortex_domain::events::{
    ProjectedEvent as DomainProjectedEvent, ProjectionKind as DomainProjectionKind,
    SessionUpdateKind as DomainSessionUpdateKind, TraceContext as DomainTraceContext,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpProjectedEvent {
    pub id: String,
    pub session_id: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub session_update_kind: AcpSessionUpdateKind,
    pub projection_kind: NostraProjectionKind,
    pub timestamp: u64,
    #[serde(default)]
    pub trace: TraceContext,
    #[serde(default)]
    pub payload: Value,
}

pub fn deterministic_event_id(
    session_id: &str,
    turn_seq: u64,
    update_seq: u64,
    kind: AcpSessionUpdateKind,
) -> String {
    let raw = format!("{}:{}:{}:{:?}", session_id, turn_seq, update_seq, kind);
    let digest = Sha256::digest(raw.as_bytes());
    format!("acp_evt_{}", hex::encode(digest))
}

pub fn project_session_update(
    session_id: &str,
    turn_seq: u64,
    update_seq: u64,
    kind: AcpSessionUpdateKind,
    payload: Value,
    trace: TraceContext,
) -> AcpProjectedEvent {
    project_session_update_with_timestamp(
        session_id,
        turn_seq,
        update_seq,
        kind,
        payload,
        trace,
        now_secs(),
    )
}

pub fn project_session_update_with_timestamp(
    session_id: &str,
    turn_seq: u64,
    update_seq: u64,
    kind: AcpSessionUpdateKind,
    payload: Value,
    trace: TraceContext,
    timestamp: u64,
) -> AcpProjectedEvent {
    AcpProjectedEvent {
        id: deterministic_event_id(session_id, turn_seq, update_seq, kind),
        session_id: session_id.to_string(),
        turn_seq,
        update_seq,
        projection_kind: map_session_update_to_projection(kind),
        session_update_kind: kind,
        timestamp,
        trace,
        payload,
    }
}

pub fn to_cloud_event(projected: &AcpProjectedEvent) -> Result<nostra_cloudevents::Event, String> {
    let mut event = nostra_cloudevents::Event::new(
        "nostra://cortex-desktop/acp",
        format!(
            "nostra.acp.{}",
            format_projection(&projected.projection_kind)
        ),
    )
    .with_id(projected.id.clone())
    .with_subject(projected.session_id.clone())
    .with_data(serde_json::to_value(projected).map_err(|e| e.to_string())?)
    .map_err(|e| e.to_string())?;

    let event_time = DateTime::<Utc>::from_timestamp(projected.timestamp as i64, 0)
        .or_else(|| DateTime::<Utc>::from_timestamp(0, 0))
        .ok_or_else(|| "invalid timestamp".to_string())?;
    event.time = Some(event_time);
    Ok(event)
}

fn format_projection(kind: &NostraProjectionKind) -> &'static str {
    match kind {
        NostraProjectionKind::UserInputChunk => "user_input_chunk",
        NostraProjectionKind::AgentOutputChunk => "agent_output_chunk",
        NostraProjectionKind::PrivateThoughtChunk => "private_thought_chunk",
        NostraProjectionKind::ToolCallStarted => "tool_call_started",
        NostraProjectionKind::ToolCallProgress => "tool_call_progress",
        NostraProjectionKind::PlanSnapshot => "plan_snapshot",
        NostraProjectionKind::CommandSetChanged => "command_set_changed",
        NostraProjectionKind::ModeChanged => "mode_changed",
        NostraProjectionKind::SessionConfigChanged => "session_config_changed",
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(feature = "cortex_runtime_v0")]
pub fn to_domain_update_kind(kind: AcpSessionUpdateKind) -> DomainSessionUpdateKind {
    match kind {
        AcpSessionUpdateKind::UserMessageChunk => DomainSessionUpdateKind::UserMessageChunk,
        AcpSessionUpdateKind::AgentMessageChunk => DomainSessionUpdateKind::AgentMessageChunk,
        AcpSessionUpdateKind::AgentThoughtChunk => DomainSessionUpdateKind::AgentThoughtChunk,
        AcpSessionUpdateKind::ToolCall => DomainSessionUpdateKind::ToolCall,
        AcpSessionUpdateKind::ToolCallUpdate => DomainSessionUpdateKind::ToolCallUpdate,
        AcpSessionUpdateKind::Plan => DomainSessionUpdateKind::Plan,
        AcpSessionUpdateKind::AvailableCommandsUpdate => {
            DomainSessionUpdateKind::AvailableCommandsUpdate
        }
        AcpSessionUpdateKind::CurrentModeUpdate => DomainSessionUpdateKind::CurrentModeUpdate,
        AcpSessionUpdateKind::ConfigOptionUpdate => DomainSessionUpdateKind::ConfigOptionUpdate,
    }
}

#[cfg(feature = "cortex_runtime_v0")]
fn from_domain_projection_kind(kind: DomainProjectionKind) -> NostraProjectionKind {
    match kind {
        DomainProjectionKind::UserInputChunk => NostraProjectionKind::UserInputChunk,
        DomainProjectionKind::AgentOutputChunk => NostraProjectionKind::AgentOutputChunk,
        DomainProjectionKind::PrivateThoughtChunk => NostraProjectionKind::PrivateThoughtChunk,
        DomainProjectionKind::ToolCallStarted => NostraProjectionKind::ToolCallStarted,
        DomainProjectionKind::ToolCallProgress => NostraProjectionKind::ToolCallProgress,
        DomainProjectionKind::PlanSnapshot => NostraProjectionKind::PlanSnapshot,
        DomainProjectionKind::CommandSetChanged => NostraProjectionKind::CommandSetChanged,
        DomainProjectionKind::ModeChanged => NostraProjectionKind::ModeChanged,
        DomainProjectionKind::SessionConfigChanged => NostraProjectionKind::SessionConfigChanged,
    }
}

#[cfg(feature = "cortex_runtime_v0")]
pub fn to_domain_trace_context(trace: &TraceContext) -> DomainTraceContext {
    DomainTraceContext {
        traceparent: trace.traceparent.clone(),
        tracestate: trace.tracestate.clone(),
        baggage: trace.baggage.clone(),
    }
}

#[cfg(feature = "cortex_runtime_v0")]
pub fn from_domain_projected_event(event: DomainProjectedEvent) -> AcpProjectedEvent {
    AcpProjectedEvent {
        id: event.id,
        session_id: event.session_id,
        turn_seq: event.turn_seq,
        update_seq: event.update_seq,
        session_update_kind: from_domain_update_kind(event.session_update_kind),
        projection_kind: from_domain_projection_kind(event.projection_kind),
        timestamp: event.timestamp,
        trace: TraceContext {
            traceparent: event.trace.traceparent,
            tracestate: event.trace.tracestate,
            baggage: event.trace.baggage,
        },
        payload: event.payload,
    }
}

#[cfg(feature = "cortex_runtime_v0")]
fn from_domain_update_kind(kind: DomainSessionUpdateKind) -> AcpSessionUpdateKind {
    match kind {
        DomainSessionUpdateKind::UserMessageChunk => AcpSessionUpdateKind::UserMessageChunk,
        DomainSessionUpdateKind::AgentMessageChunk => AcpSessionUpdateKind::AgentMessageChunk,
        DomainSessionUpdateKind::AgentThoughtChunk => AcpSessionUpdateKind::AgentThoughtChunk,
        DomainSessionUpdateKind::ToolCall => AcpSessionUpdateKind::ToolCall,
        DomainSessionUpdateKind::ToolCallUpdate => AcpSessionUpdateKind::ToolCallUpdate,
        DomainSessionUpdateKind::Plan => AcpSessionUpdateKind::Plan,
        DomainSessionUpdateKind::AvailableCommandsUpdate => {
            AcpSessionUpdateKind::AvailableCommandsUpdate
        }
        DomainSessionUpdateKind::CurrentModeUpdate => AcpSessionUpdateKind::CurrentModeUpdate,
        DomainSessionUpdateKind::ConfigOptionUpdate => AcpSessionUpdateKind::ConfigOptionUpdate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_ids_are_stable() {
        let a = deterministic_event_id("sess_1", 1, 1, AcpSessionUpdateKind::Plan);
        let b = deterministic_event_id("sess_1", 1, 1, AcpSessionUpdateKind::Plan);
        assert_eq!(a, b);
    }

    #[test]
    fn cloud_event_conversion_works() {
        let projected = project_session_update_with_timestamp(
            "sess_1",
            1,
            1,
            AcpSessionUpdateKind::AgentMessageChunk,
            serde_json::json!({"text": "ok"}),
            TraceContext::default(),
            1,
        );
        let event = to_cloud_event(&projected).unwrap();
        assert_eq!(event.id, projected.id);
    }
}
