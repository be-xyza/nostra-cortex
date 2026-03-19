use crate::ports::{EventBus, LogAdapter, NetworkAdapter, StorageAdapter, TimeProvider};
use crate::{CortexRuntime, RuntimeConfig, RuntimeError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cortex_domain::events::{
    ProjectSessionUpdateInput, ProjectedEvent, ProjectionKind, SessionUpdateKind, TraceContext,
    project_session_update,
};
use nostra_cloudevents::Event;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSessionUpdateRequest {
    pub session_id: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub update_kind: SessionUpdateKind,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub trace: TraceContext,
    pub timestamp_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RuntimeSessionUpdateResult {
    pub projected: ProjectedEvent,
    pub cloud_event: Event,
    pub network_error: Option<String>,
}

pub struct EventOrchestrator {
    config: RuntimeConfig,
    time_provider: Arc<dyn TimeProvider>,
    log_adapter: Arc<dyn LogAdapter>,
    event_bus: Arc<dyn EventBus>,
    #[allow(dead_code)]
    storage_adapter: Option<Arc<dyn StorageAdapter>>,
    network_adapter: Option<Arc<dyn NetworkAdapter>>,
}

impl EventOrchestrator {
    pub fn new(
        config: RuntimeConfig,
        time_provider: Arc<dyn TimeProvider>,
        log_adapter: Arc<dyn LogAdapter>,
        event_bus: Arc<dyn EventBus>,
        storage_adapter: Option<Arc<dyn StorageAdapter>>,
        network_adapter: Option<Arc<dyn NetworkAdapter>>,
    ) -> Self {
        Self {
            config,
            time_provider,
            log_adapter,
            event_bus,
            storage_adapter,
            network_adapter,
        }
    }

    pub async fn publish_session_update(
        &self,
        request: RuntimeSessionUpdateRequest,
    ) -> Result<RuntimeSessionUpdateResult, RuntimeError> {
        let timestamp = request
            .timestamp_secs
            .unwrap_or_else(|| self.time_provider.now_unix_secs());

        let projected = project_session_update(ProjectSessionUpdateInput {
            session_id: request.session_id,
            turn_seq: request.turn_seq,
            update_seq: request.update_seq,
            kind: request.update_kind,
            payload: request.payload,
            trace: request.trace,
            timestamp,
        })
        .map_err(|e| RuntimeError::Domain(e.to_string()))?;

        let cloud_event = to_cloud_event(
            &projected,
            &self.config.event_source,
            &self.config.event_type_prefix,
            self.time_provider.as_ref(),
        )?;

        self.event_bus.append_projected_event(&projected).await?;

        let mut network_error = None;
        if let (Some(endpoint), Some(network_adapter)) =
            (&self.config.remote_endpoint, &self.network_adapter)
        {
            let body = serde_json::to_value(&cloud_event)
                .map_err(|e| RuntimeError::Serialization(e.to_string()))?;
            if let Err(err) = network_adapter
                .post_json(endpoint, &projected.id, &body)
                .await
            {
                network_error = Some(err.to_string());
                if self.config.fail_on_network_error {
                    return Err(err);
                }
                self.log_adapter.warn(&format!(
                    "network emit failed for event {}: {}",
                    projected.id, err
                ));
            }
        }

        Ok(RuntimeSessionUpdateResult {
            projected,
            cloud_event,
            network_error,
        })
    }
}

#[async_trait]
impl CortexRuntime for EventOrchestrator {
    async fn publish_session_update(
        &self,
        request: RuntimeSessionUpdateRequest,
    ) -> Result<RuntimeSessionUpdateResult, RuntimeError> {
        EventOrchestrator::publish_session_update(self, request).await
    }
}

pub fn to_cloud_event(
    projected: &ProjectedEvent,
    source: &str,
    event_type_prefix: &str,
    time_provider: &dyn TimeProvider,
) -> Result<Event, RuntimeError> {
    let event_type = format!(
        "{}.{}",
        event_type_prefix,
        projection_kind_key(projected.projection_kind)
    );
    let mut cloud_event = Event::new(source, event_type)
        .with_id(projected.id.clone())
        .with_subject(projected.session_id.clone())
        .with_data(projected)
        .map_err(|e| RuntimeError::Serialization(e.to_string()))?;

    let rendered = time_provider.to_rfc3339(projected.timestamp)?;
    let timestamp = DateTime::parse_from_rfc3339(&rendered)
        .map_err(|_| RuntimeError::InvalidTimestamp(projected.timestamp))?
        .with_timezone(&Utc);
    cloud_event.time = Some(timestamp);

    Ok(cloud_event)
}

fn projection_kind_key(kind: ProjectionKind) -> &'static str {
    match kind {
        ProjectionKind::UserInputChunk => "user_input_chunk",
        ProjectionKind::AgentOutputChunk => "agent_output_chunk",
        ProjectionKind::PrivateThoughtChunk => "private_thought_chunk",
        ProjectionKind::ToolCallStarted => "tool_call_started",
        ProjectionKind::ToolCallProgress => "tool_call_progress",
        ProjectionKind::PlanSnapshot => "plan_snapshot",
        ProjectionKind::CommandSetChanged => "command_set_changed",
        ProjectionKind::ModeChanged => "mode_changed",
        ProjectionKind::SessionConfigChanged => "session_config_changed",
        ProjectionKind::AgentBranched => "agent_branched",
        ProjectionKind::ToolCallCompleted => "tool_call_completed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{EventBus, LogAdapter, TimeProvider};
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct FixedTime;

    impl TimeProvider for FixedTime {
        fn now_unix_secs(&self) -> u64 {
            10
        }

        fn to_rfc3339(&self, _unix_secs: u64) -> Result<String, RuntimeError> {
            Ok("1970-01-01T00:00:10Z".to_string())
        }
    }

    struct TestLog;

    impl LogAdapter for TestLog {
        fn info(&self, _message: &str) {}
        fn warn(&self, _message: &str) {}
        fn error(&self, _message: &str) {}
    }

    #[derive(Default)]
    struct MemoryEventBus {
        events: Mutex<Vec<ProjectedEvent>>,
    }

    #[async_trait]
    impl EventBus for MemoryEventBus {
        async fn append_projected_event(&self, event: &ProjectedEvent) -> Result<(), RuntimeError> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    #[test]
    fn deterministic_replay_is_stable() {
        futures::executor::block_on(async {
            let bus = Arc::new(MemoryEventBus::default());
            let orchestrator = EventOrchestrator::new(
                RuntimeConfig::default(),
                Arc::new(FixedTime),
                Arc::new(TestLog),
                bus,
                None,
                None,
            );

            let request = RuntimeSessionUpdateRequest {
                session_id: "sess_1".to_string(),
                turn_seq: 1,
                update_seq: 1,
                update_kind: SessionUpdateKind::Plan,
                payload: Value::Null,
                trace: TraceContext::default(),
                timestamp_secs: Some(100),
            };

            let left = orchestrator
                .publish_session_update(request.clone())
                .await
                .unwrap();
            let right = orchestrator.publish_session_update(request).await.unwrap();

            assert_eq!(left.projected.id, right.projected.id);
            assert_eq!(left.cloud_event.type_, right.cloud_event.type_);
        });
    }
}
