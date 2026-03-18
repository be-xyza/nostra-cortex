use crate::ports::TimeProvider;
use crate::workflow::adapter::WorkflowExecutionAdapter;
use crate::workflow::digest::workflow_digest_hex;
use crate::RuntimeError;
use async_trait::async_trait;
use cortex_domain::agent::contracts::{
    TemporalBridgeRunSnapshot, TemporalBridgeSignalCommand, TemporalBridgeStartCommand,
};
use cortex_domain::workflow::{
    WorkflowCheckpointPolicyV1, WorkflowCheckpointResultV1, WorkflowCheckpointStatus,
    WorkflowCheckpointV1, WorkflowDefinitionV1, WorkflowExecutionBindingV1,
    WorkflowExecutionPlanV1, WorkflowInstanceStatus, WorkflowInstanceV1, WorkflowNodeKind,
    WorkflowOutcomeStatus, WorkflowOutcomeV1, WorkflowScope, WorkflowSignalV1,
    WorkflowSnapshotV1, WorkflowTraceEventV1,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LocalDurableWorkerAdapter<T: TimeProvider> {
    runtime_root: PathBuf,
    time: Arc<T>,
    default_task_queue: String,
    default_namespace: String,
    default_approval_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct LocalWorkflowInstanceRecord {
    instance: WorkflowInstanceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    checkpoint_policy: Option<WorkflowCheckpointPolicyV1>,
}

impl<T: TimeProvider> LocalDurableWorkerAdapter<T> {
    pub fn new(runtime_root: PathBuf, time: Arc<T>) -> Self {
        Self {
            runtime_root,
            time,
            default_task_queue: "SIMULATION_TASK_QUEUE".to_string(),
            default_namespace: "default".to_string(),
            default_approval_timeout_seconds: 3600,
        }
    }

    pub fn with_defaults(
        mut self,
        task_queue: impl Into<String>,
        namespace: impl Into<String>,
        approval_timeout_seconds: u64,
    ) -> Self {
        self.default_task_queue = task_queue.into();
        self.default_namespace = namespace.into();
        self.default_approval_timeout_seconds = approval_timeout_seconds.max(1);
        self
    }

    fn now_iso(&self) -> Result<String, RuntimeError> {
        let now = self.time.now_unix_secs();
        self.time.to_rfc3339(now)
    }

    fn command_nonce(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    }

    fn commands_dir(&self, kind: &str) -> PathBuf {
        self.runtime_root.join("commands").join(kind)
    }

    fn snapshots_dir(&self) -> PathBuf {
        self.runtime_root.join("snapshots")
    }

    fn instances_dir(&self) -> PathBuf {
        self.runtime_root.join("instances")
    }

    fn start_command_path(&self, instance_id: &str) -> PathBuf {
        self.commands_dir("start").join(format!(
            "{}_{}.json",
            sanitize_fs_component(instance_id),
            self.command_nonce()
        ))
    }

    fn signal_command_path(&self, instance_id: &str) -> PathBuf {
        self.commands_dir("signal").join(format!(
            "{}_{}.json",
            sanitize_fs_component(instance_id),
            self.command_nonce()
        ))
    }

    fn snapshot_path(&self, instance_id: &str) -> PathBuf {
        self.snapshots_dir()
            .join(format!("{}.json", sanitize_fs_component(instance_id)))
    }

    fn instance_record_path(&self, instance_id: &str) -> PathBuf {
        self.instances_dir()
            .join(format!("{}.json", sanitize_fs_component(instance_id)))
    }

    fn ensure_runtime_dirs(&self) -> Result<(), RuntimeError> {
        fs::create_dir_all(self.commands_dir("start"))
            .map_err(|err| RuntimeError::Storage(err.to_string()))?;
        fs::create_dir_all(self.commands_dir("signal"))
            .map_err(|err| RuntimeError::Storage(err.to_string()))?;
        fs::create_dir_all(self.snapshots_dir())
            .map_err(|err| RuntimeError::Storage(err.to_string()))?;
        fs::create_dir_all(self.instances_dir())
            .map_err(|err| RuntimeError::Storage(err.to_string()))?;
        Ok(())
    }

    fn write_json<TValue: Serialize>(&self, path: &Path, value: &TValue) -> Result<(), RuntimeError> {
        let raw = serde_json::to_vec_pretty(value)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        fs::write(path, raw).map_err(|err| RuntimeError::Storage(err.to_string()))
    }

    fn read_json<TValue: for<'de> Deserialize<'de>>(
        &self,
        path: &Path,
    ) -> Result<TValue, RuntimeError> {
        let raw = fs::read_to_string(path).map_err(|err| RuntimeError::Storage(err.to_string()))?;
        serde_json::from_str(&raw).map_err(|err| RuntimeError::Serialization(err.to_string()))
    }

    fn read_json_optional<TValue: for<'de> Deserialize<'de>>(
        &self,
        path: &Path,
    ) -> Result<Option<TValue>, RuntimeError> {
        match fs::read_to_string(path) {
            Ok(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|err| RuntimeError::Serialization(err.to_string())),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(RuntimeError::Storage(err.to_string())),
        }
    }

    fn binding_string_limit(binding: &WorkflowExecutionBindingV1, key: &str) -> Option<String> {
        binding
            .runtime_limits
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn binding_u64_limit(binding: &WorkflowExecutionBindingV1, key: &str) -> Option<u64> {
        binding.runtime_limits.get(key).and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_i64().and_then(|entry| u64::try_from(entry).ok()))
                .or_else(|| value.as_str().and_then(|entry| entry.trim().parse::<u64>().ok()))
        })
    }

    fn workflow_scope(
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> WorkflowScope {
        let mut scope = definition.scope.clone();
        if scope.space_id.is_none() {
            scope.space_id = Self::binding_string_limit(binding, "spaceId");
        }
        scope
    }

    fn workflow_id(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> String {
        Self::binding_string_limit(binding, "workflowId").unwrap_or_else(|| definition.definition_id.clone())
    }

    fn contribution_id(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> String {
        Self::binding_string_limit(binding, "contributionId")
            .unwrap_or_else(|| format!("workflow-definition:{}", definition.definition_id))
    }

    fn approval_timeout_seconds(&self, binding: &WorkflowExecutionBindingV1) -> u64 {
        Self::binding_u64_limit(binding, "approvalTimeoutSeconds")
            .unwrap_or(self.default_approval_timeout_seconds)
            .max(1)
    }

    fn definition_digest(&self, definition: &WorkflowDefinitionV1) -> Result<String, RuntimeError> {
        match definition.digest.clone() {
            Some(digest) if !digest.trim().is_empty() => Ok(digest),
            _ => {
                let value = serde_json::to_value(definition)
                    .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
                Ok(workflow_digest_hex(&value))
            }
        }
    }

    fn binding_digest(&self, binding: &WorkflowExecutionBindingV1) -> Result<String, RuntimeError> {
        let value = serde_json::to_value(binding)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        Ok(workflow_digest_hex(&value))
    }

    fn snapshot_to_status(status: &str) -> WorkflowInstanceStatus {
        match status.trim().to_ascii_lowercase().as_str() {
            "queued" => WorkflowInstanceStatus::Queued,
            "waiting_approval" => WorkflowInstanceStatus::WaitingCheckpoint,
            "completed" => WorkflowInstanceStatus::Completed,
            "rejected" => WorkflowInstanceStatus::Cancelled,
            "failed" => WorkflowInstanceStatus::Failed,
            "paused" => WorkflowInstanceStatus::Paused,
            _ => WorkflowInstanceStatus::Running,
        }
    }

    fn outcome_from_snapshot(snapshot: &TemporalBridgeRunSnapshot) -> Option<WorkflowOutcomeV1> {
        let status = match snapshot.status.trim().to_ascii_lowercase().as_str() {
            "completed" => WorkflowOutcomeStatus::Completed,
            "failed" => WorkflowOutcomeStatus::Failed,
            "rejected" => WorkflowOutcomeStatus::Cancelled,
            "paused" => WorkflowOutcomeStatus::Paused,
            _ => return None,
        };
        Some(WorkflowOutcomeV1 {
            outcome_id: format!("workflow_outcome_{}", sanitize_fs_component(&snapshot.run_id)),
            instance_id: snapshot.run_id.clone(),
            status,
            completed_at: snapshot.updated_at.clone(),
            summary: snapshot
                .error
                .clone()
                .unwrap_or_else(|| format!("workflow {}", snapshot.status)),
            contribution_refs: vec![snapshot.contribution_id.clone()],
            global_event_refs: Vec::new(),
            evidence_refs: Vec::new(),
        })
    }
}

#[async_trait]
impl<T> WorkflowExecutionAdapter for LocalDurableWorkerAdapter<T>
where
    T: TimeProvider + 'static,
{
    fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, RuntimeError> {
        let projection = json!({
            "runtime": "local_durable_worker_v1",
            "runtimeRoot": self.runtime_root.to_string_lossy(),
            "workflowId": self.workflow_id(definition, binding),
            "contributionId": self.contribution_id(definition, binding),
            "taskQueue": Self::binding_string_limit(binding, "taskQueue")
                .unwrap_or_else(|| self.default_task_queue.clone()),
            "namespace": Self::binding_string_limit(binding, "namespace")
                .unwrap_or_else(|| self.default_namespace.clone()),
            "approvalTimeoutSeconds": self.approval_timeout_seconds(binding),
        });
        Ok(WorkflowExecutionPlanV1 {
            plan_id: format!(
                "workflow_plan_{}_{}",
                sanitize_fs_component(&binding.binding_id),
                self.time.now_unix_secs()
            ),
            definition_id: definition.definition_id.clone(),
            binding_id: binding.binding_id.clone(),
            adapter: binding.adapter.clone(),
            projection,
        })
    }

    async fn start(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, RuntimeError> {
        self.ensure_runtime_dirs()?;

        let scope = Self::workflow_scope(definition, binding);
        let instance_id = Self::binding_string_limit(binding, "instanceId")
            .unwrap_or_else(|| format!("workflow_instance_{}", self.command_nonce()));
        let created_at = self.now_iso()?;
        let task_queue = Self::binding_string_limit(binding, "taskQueue")
            .unwrap_or_else(|| self.default_task_queue.clone());
        let namespace = Self::binding_string_limit(binding, "namespace")
            .unwrap_or_else(|| self.default_namespace.clone());
        let approval_timeout_seconds = self.approval_timeout_seconds(binding);
        let command = TemporalBridgeStartCommand {
            run_id: instance_id.clone(),
            workflow_id: self.workflow_id(definition, binding),
            space_id: scope
                .space_id
                .clone()
                .unwrap_or_else(|| "workflow-space".to_string()),
            contribution_id: self.contribution_id(definition, binding),
            approval_timeout_seconds,
            task_queue,
            namespace,
        };
        self.write_json(&self.start_command_path(&instance_id), &command)?;

        let instance = WorkflowInstanceV1 {
            schema_version: "1.0.0".to_string(),
            instance_id: instance_id.clone(),
            definition_id: definition.definition_id.clone(),
            binding_id: binding.binding_id.clone(),
            status: WorkflowInstanceStatus::Queued,
            scope,
            created_at: created_at.clone(),
            updated_at: created_at,
            definition_digest: self.definition_digest(definition)?,
            binding_digest: self.binding_digest(binding)?,
            source_of_truth: "local_durable_worker_v1".to_string(),
            replay_contract_ref: binding
                .governance_ref
                .as_ref()
                .map(|entry| entry.replay_contract_ref.clone()),
            lineage_id: binding
                .governance_ref
                .as_ref()
                .map(|entry| entry.lineage_id.clone()),
            degraded_reason: binding
                .governance_ref
                .as_ref()
                .and_then(|entry| entry.degraded_reason.clone()),
        };
        let record = LocalWorkflowInstanceRecord {
            instance: instance.clone(),
            checkpoint_policy: binding.checkpoint_policy.clone(),
        };
        self.write_json(&self.instance_record_path(&instance_id), &record)?;
        Ok(instance)
    }

    async fn signal(
        &self,
        instance_id: &str,
        signal: WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        self.ensure_runtime_dirs()?;
        let decision = signal
            .payload
            .get("decision")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| signal.signal_type.clone());
        let command = TemporalBridgeSignalCommand {
            run_id: instance_id.to_string(),
            decision: decision.clone(),
            rationale: signal
                .payload
                .get("rationale")
                .and_then(Value::as_str)
                .map(str::to_string),
            actor: signal
                .payload
                .get("actor")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| "workflow-operator".to_string()),
            decision_ref: signal
                .payload
                .get("decisionRef")
                .and_then(Value::as_str)
                .map(str::to_string),
        };
        self.write_json(&self.signal_command_path(instance_id), &command)?;
        Ok(WorkflowCheckpointResultV1 {
            instance_id: instance_id.to_string(),
            checkpoint_id: signal.checkpoint_id,
            status: if decision.eq_ignore_ascii_case("cancel")
                || decision.eq_ignore_ascii_case("cancelled")
                || decision.eq_ignore_ascii_case("rejected")
            {
                WorkflowCheckpointStatus::Cancelled
            } else {
                WorkflowCheckpointStatus::Resolved
            },
            updated_at: self.now_iso()?,
        })
    }

    async fn snapshot(&self, instance_id: &str) -> Result<WorkflowSnapshotV1, RuntimeError> {
        let record: LocalWorkflowInstanceRecord = self.read_json(&self.instance_record_path(instance_id))?;
        let snapshot = self.read_json_optional::<TemporalBridgeRunSnapshot>(&self.snapshot_path(instance_id))?;
        let Some(snapshot) = snapshot else {
            return Ok(WorkflowSnapshotV1 {
                instance: record.instance,
                trace: Vec::new(),
                checkpoints: Vec::new(),
                outcome: None,
            });
        };

        let mut instance = record.instance;
        instance.status = Self::snapshot_to_status(&snapshot.status);
        instance.updated_at = snapshot.updated_at.clone();

        let trace = snapshot
            .events
            .iter()
            .map(|event| WorkflowTraceEventV1 {
                event_id: format!(
                    "workflow_trace_{}_{}",
                    sanitize_fs_component(&snapshot.run_id),
                    event.sequence
                ),
                instance_id: snapshot.run_id.clone(),
                event_type: event.event_type.clone(),
                sequence: event.sequence,
                timestamp: event.timestamp.clone(),
                payload: event.payload.clone(),
            })
            .collect::<Vec<_>>();

        let checkpoints = if snapshot.status == "waiting_approval" {
            vec![WorkflowCheckpointV1 {
                checkpoint_id: format!("workflow_checkpoint_{}", sanitize_fs_component(&snapshot.run_id)),
                instance_id: snapshot.run_id.clone(),
                node_id: "human_approval".to_string(),
                kind: WorkflowNodeKind::HumanCheckpoint,
                status: WorkflowCheckpointStatus::Pending,
                created_at: snapshot.updated_at.clone(),
                resolved_at: None,
                surface_ref: Some(format!("a2ui://workflow-checkpoint/{}", snapshot.run_id)),
                policy: record.checkpoint_policy.unwrap_or(WorkflowCheckpointPolicyV1 {
                    resume_allowed: true,
                    cancel_allowed: true,
                    pause_allowed: true,
                    timeout_seconds: Some(snapshot.approval_timeout_seconds),
                }),
            }]
        } else {
            Vec::new()
        };

        Ok(WorkflowSnapshotV1 {
            instance,
            trace,
            checkpoints,
            outcome: Self::outcome_from_snapshot(&snapshot),
        })
    }

    async fn cancel(
        &self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        self.signal(
            instance_id,
            WorkflowSignalV1 {
                signal_type: "cancel".to_string(),
                checkpoint_id: None,
                payload: json!({
                    "decision": "cancelled",
                    "rationale": reason,
                    "actor": "workflow-system"
                }),
            },
        )
        .await
    }
}

fn sanitize_fs_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::agent::contracts::AgentRunEvent;
    use cortex_domain::workflow::{
        WorkflowCheckpointPolicyV1, WorkflowExecutionAdapterKind, WorkflowExecutionProfileKind,
        WorkflowGenerationMode, WorkflowGovernanceRef, WorkflowMotifKind, generate_candidate_set,
    };

    struct FixedTime;

    impl TimeProvider for FixedTime {
        fn now_unix_secs(&self) -> u64 {
            1_741_651_200
        }

        fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
            Ok(format!("{unix_secs}Z"))
        }
    }

    fn temp_runtime_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "cortex-workflow-runtime-{}-{}",
            label,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir_all(&root).expect("runtime root");
        root
    }

    fn sample_definition_and_binding(
        runtime_root: &Path,
    ) -> (LocalDurableWorkerAdapter<FixedTime>, WorkflowDefinitionV1, WorkflowExecutionBindingV1) {
        let candidate_set = generate_candidate_set(
            WorkflowScope {
                space_id: Some("space-a".to_string()),
                route_id: Some("/workflows".to_string()),
                role: Some("operator".to_string()),
            },
            "Compare and approve",
            WorkflowMotifKind::ParallelCompare,
            &[],
            1,
            "tester",
            "human",
            WorkflowGenerationMode::DeterministicScaffold,
            "candidate_set_1",
            "2026-03-11T00:00:00Z",
            "seed_1",
        );
        let draft = candidate_set.candidates[0].workflow_draft.clone();
        let definition = WorkflowDefinitionV1 {
            schema_version: draft.schema_version.clone(),
            definition_id: "workflow_def_test".to_string(),
            scope: draft.scope.clone(),
            intent_ref: draft.intent_ref.clone(),
            intent: draft.intent.clone(),
            motif_kind: draft.motif_kind.clone(),
            constraints: draft.constraints.clone(),
            graph: draft.graph.clone(),
            context_contract: draft.context_contract.clone(),
            confidence: draft.confidence.clone(),
            lineage: draft.lineage.clone(),
            policy: draft.policy.clone(),
            provenance: draft.provenance.clone(),
            governance_ref: Some(WorkflowGovernanceRef {
                gate_level: "release_blocker".to_string(),
                gate_status: "allow".to_string(),
                decision_gate_id: "gate-1".to_string(),
                replay_contract_ref: "replay-1".to_string(),
                source_of_truth: "fallback".to_string(),
                lineage_id: "lineage-1".to_string(),
                degraded_reason: Some("test".to_string()),
                definition_digest: "definition-digest".to_string(),
                binding_digest: "binding-digest".to_string(),
            }),
            digest: Some("definition-digest".to_string()),
        };
        let binding = WorkflowExecutionBindingV1 {
            schema_version: "1.0.0".to_string(),
            binding_id: "workflow_binding_test".to_string(),
            definition_id: definition.definition_id.clone(),
            adapter: WorkflowExecutionAdapterKind::LocalDurableWorkerV1,
            execution_profile: WorkflowExecutionProfileKind::Async,
            checkpoint_policy: Some(WorkflowCheckpointPolicyV1 {
                resume_allowed: true,
                cancel_allowed: true,
                pause_allowed: true,
                timeout_seconds: Some(120),
            }),
            runtime_limits: [
                ("contributionId".to_string(), json!("contrib-1")),
                ("taskQueue".to_string(), json!("queue-a")),
                ("namespace".to_string(), json!("default")),
                ("approvalTimeoutSeconds".to_string(), json!(120)),
            ]
            .into_iter()
            .collect(),
            governance_ref: definition.governance_ref.clone(),
            provenance: definition.provenance.clone(),
        };
        let adapter =
            LocalDurableWorkerAdapter::new(runtime_root.to_path_buf(), Arc::new(FixedTime));
        (adapter, definition, binding)
    }

    #[tokio::test]
    async fn start_writes_bridge_command_and_instance_record() {
        let runtime_root = temp_runtime_root("start");
        let (adapter, definition, binding) = sample_definition_and_binding(&runtime_root);

        let instance = adapter.start(&definition, &binding).await.expect("start");
        assert_eq!(instance.status, WorkflowInstanceStatus::Queued);

        let start_dir = runtime_root.join("commands").join("start");
        let command_path = fs::read_dir(&start_dir)
            .expect("start dir")
            .next()
            .expect("command")
            .expect("entry")
            .path();
        let command: TemporalBridgeStartCommand = serde_json::from_str(
            &fs::read_to_string(command_path).expect("read command"),
        )
        .expect("parse command");
        assert_eq!(command.run_id, instance.instance_id);
        assert_eq!(command.contribution_id, "contrib-1");

        let record_path = runtime_root
            .join("instances")
            .join(format!("{}.json", sanitize_fs_component(&instance.instance_id)));
        assert!(record_path.exists(), "instance record should exist");
    }

    #[tokio::test]
    async fn signal_and_cancel_write_bridge_signal_commands() {
        let runtime_root = temp_runtime_root("signal");
        let (adapter, definition, binding) = sample_definition_and_binding(&runtime_root);
        let instance = adapter.start(&definition, &binding).await.expect("start");

        let result = adapter
            .signal(
                &instance.instance_id,
                WorkflowSignalV1 {
                    signal_type: "resume".to_string(),
                    checkpoint_id: Some("checkpoint-1".to_string()),
                    payload: json!({
                        "decision": "approved",
                        "actor": "reviewer",
                        "rationale": "looks good"
                    }),
                },
            )
            .await
            .expect("signal");
        assert_eq!(result.status, WorkflowCheckpointStatus::Resolved);

        let cancel_result = adapter
            .cancel(&instance.instance_id, "operator cancelled")
            .await
            .expect("cancel");
        assert_eq!(cancel_result.status, WorkflowCheckpointStatus::Cancelled);

        let signal_dir = runtime_root.join("commands").join("signal");
        let command_count = fs::read_dir(signal_dir).expect("signal dir").count();
        assert_eq!(command_count, 2);
    }

    #[tokio::test]
    async fn snapshot_reads_bridge_snapshot_and_projects_checkpoint() {
        let runtime_root = temp_runtime_root("snapshot");
        let (adapter, definition, binding) = sample_definition_and_binding(&runtime_root);
        let instance = adapter.start(&definition, &binding).await.expect("start");

        let snapshot = TemporalBridgeRunSnapshot {
            schema_version: "1.0.0".to_string(),
            run_id: instance.instance_id.clone(),
            workflow_id: "workflow_def_test".to_string(),
            space_id: "space-a".to_string(),
            contribution_id: "contrib-1".to_string(),
            status: "waiting_approval".to_string(),
            started_at: "2026-03-11T00:00:00Z".to_string(),
            updated_at: "2026-03-11T00:05:00Z".to_string(),
            sequence: 2,
            events: vec![AgentRunEvent {
                event_type: "approval_required".to_string(),
                run_id: instance.instance_id.clone(),
                space_id: "space-a".to_string(),
                timestamp: "2026-03-11T00:05:00Z".to_string(),
                sequence: 2,
                payload: json!({ "status": "waiting_approval" }),
            }],
            simulation: None,
            surface_update: Some(json!({ "surface": "checkpoint" })),
            authority_outcome: None,
            provider_trace: None,
            approval_timeout_seconds: 120,
            terminal: false,
            error: None,
        };
        fs::write(
            runtime_root.join("snapshots").join(format!(
                "{}.json",
                sanitize_fs_component(&instance.instance_id)
            )),
            serde_json::to_vec_pretty(&snapshot).expect("encode snapshot"),
        )
        .expect("write snapshot");

        let projected = adapter.snapshot(&instance.instance_id).await.expect("snapshot");
        assert_eq!(projected.instance.status, WorkflowInstanceStatus::WaitingCheckpoint);
        assert_eq!(projected.trace.len(), 1);
        assert_eq!(projected.checkpoints.len(), 1);
        assert_eq!(projected.checkpoints[0].status, WorkflowCheckpointStatus::Pending);
        assert_eq!(
            projected.checkpoints[0].policy.timeout_seconds,
            Some(120)
        );
        assert!(projected.outcome.is_none());
    }
}
