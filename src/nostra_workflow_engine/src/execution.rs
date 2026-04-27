use candid::{CandidType, Deserialize};
use cortex_domain::workflow::{
    WorkflowCheckpointPolicyV1, WorkflowCheckpointResultV1, WorkflowCheckpointStatus,
    WorkflowCheckpointV1, WorkflowDefinitionV1, WorkflowExecutionBindingV1,
    WorkflowExecutionPlanV1, WorkflowInstanceStatus, WorkflowInstanceV1, WorkflowNodeKind,
    WorkflowOutcomeStatus, WorkflowOutcomeV1, WorkflowScope, WorkflowSignalV1, WorkflowSnapshotV1,
    WorkflowTraceEventV1,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a running legacy workflow instance
#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub current_state: String,
    pub data: HashMap<String, String>,
    pub history: Vec<StateTransition>,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub timestamp: u64,
    pub trigger: String,
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum ActionType {
    RenderForm {
        fields: Vec<FormField>,
    },
    TemplateHydrate {
        template_id: String,
        data: HashMap<String, String>,
    },
    EditorOpen {
        path: String,
    },
    TaskCreate {
        title: String,
        assignee: String,
        description: String,
    },
    NotificationSend {
        recipients: Vec<String>,
        message: String,
    },
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CortexWorkflowRecord {
    pub definition: WorkflowDefinitionV1,
    pub binding: WorkflowExecutionBindingV1,
    pub snapshot: WorkflowSnapshotV1,
}

pub struct StateMachine {
    instances: HashMap<String, WorkflowInstance>,
    cortex_instances: HashMap<String, CortexWorkflowRecord>,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            cortex_instances: HashMap::new(),
        }
    }

    pub fn create_instance(&mut self, workflow_id: String, initial_state: String) -> String {
        let id = format!("wf_{}", self.instances.len() + 1);
        let instance = WorkflowInstance {
            id: id.clone(),
            workflow_id,
            current_state: initial_state,
            data: HashMap::new(),
            history: Vec::new(),
        };
        self.instances.insert(id.clone(), instance);
        id
    }

    pub fn transition(
        &mut self,
        instance_id: &str,
        trigger: &str,
        data: HashMap<String, String>,
    ) -> Result<String, String> {
        let instance = self
            .instances
            .get_mut(instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;

        instance.data.extend(data);

        let next_state = match instance.current_state.as_str() {
            "RENDER_FORM" => "GENERATE_ARTIFACT",
            "GENERATE_ARTIFACT" => "OPEN_EDITOR",
            "OPEN_EDITOR" => "COMPLETE",
            _ => "COMPLETE",
        };

        instance.history.push(StateTransition {
            from_state: instance.current_state.clone(),
            to_state: next_state.to_string(),
            timestamp: 0,
            trigger: trigger.to_string(),
        });

        instance.current_state = next_state.to_string();
        Ok(next_state.to_string())
    }

    pub fn get_instance(&self, instance_id: &str) -> Option<&WorkflowInstance> {
        self.instances.get(instance_id)
    }

    pub fn compile_cortex_workflow(
        &self,
        definition: WorkflowDefinitionV1,
        binding: WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, String> {
        validate_supported_definition(&definition)?;
        let plan_digest = digest_json(&json!({
            "definitionId": definition.definition_id,
            "bindingId": binding.binding_id,
            "adapter": binding.adapter,
        }))?;
        Ok(WorkflowExecutionPlanV1 {
            plan_id: format!("workflow_plan_{}", plan_digest),
            definition_id: definition.definition_id.clone(),
            binding_id: binding.binding_id.clone(),
            adapter: binding.adapter,
            projection: json!({
                "runtime": "workflow_engine_canister_v1",
                "supportedNodeKinds": [
                    "capability_call",
                    "human_checkpoint",
                    "loop",
                    "terminal"
                ],
                "executionProfile": binding.execution_profile,
                "scope": definition.scope,
            }),
        })
    }

    pub fn start_cortex_workflow(
        &mut self,
        definition: WorkflowDefinitionV1,
        binding: WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, String> {
        validate_supported_definition(&definition)?;

        let scope = workflow_scope(&definition, &binding);
        let now = now_token();
        let instance_id = binding
            .runtime_limits
            .get("instanceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("workflow_instance_{}", self.cortex_instances.len() + 1));

        let mut instance = WorkflowInstanceV1 {
            schema_version: "1.0.0".to_string(),
            instance_id: instance_id.clone(),
            definition_id: definition.definition_id.clone(),
            binding_id: binding.binding_id.clone(),
            status: WorkflowInstanceStatus::Running,
            scope,
            created_at: now.clone(),
            updated_at: now.clone(),
            definition_digest: digest_json(&definition)?,
            binding_digest: digest_json(&binding)?,
            source_of_truth: "workflow_engine_canister_v1".to_string(),
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

        let mut trace = vec![WorkflowTraceEventV1 {
            event_id: format!("workflow_event_{}_1", instance_id),
            instance_id: instance_id.clone(),
            event_type: "workflow_started".to_string(),
            sequence: 1,
            timestamp: now.clone(),
            payload: json!({
                "adapter": "workflow_engine_canister_v1",
                "definitionId": definition.definition_id,
                "bindingId": binding.binding_id,
            }),
        }];

        let mut checkpoints = Vec::new();
        let mut outcome = None;
        if let Some(node) = definition
            .graph
            .nodes
            .iter()
            .find(|node| matches!(node.kind, WorkflowNodeKind::HumanCheckpoint))
        {
            let checkpoint = WorkflowCheckpointV1 {
                checkpoint_id: format!("workflow_checkpoint_{}_1", instance_id),
                instance_id: instance_id.clone(),
                node_id: node.node_id.clone(),
                kind: WorkflowNodeKind::HumanCheckpoint,
                status: WorkflowCheckpointStatus::Pending,
                created_at: now.clone(),
                resolved_at: None,
                surface_ref: None,
                policy: node
                    .checkpoint_policy
                    .clone()
                    .or_else(|| binding.checkpoint_policy.clone())
                    .unwrap_or_else(default_checkpoint_policy),
            };
            instance.status = WorkflowInstanceStatus::WaitingCheckpoint;
            trace.push(WorkflowTraceEventV1 {
                event_id: format!("workflow_event_{}_2", instance_id),
                instance_id: instance_id.clone(),
                event_type: "checkpoint_created".to_string(),
                sequence: 2,
                timestamp: now.clone(),
                payload: json!({
                    "checkpointId": checkpoint.checkpoint_id,
                    "nodeId": checkpoint.node_id,
                    "kind": "human_checkpoint",
                }),
            });
            checkpoints.push(checkpoint);
        } else {
            instance.status = WorkflowInstanceStatus::Completed;
            outcome = Some(WorkflowOutcomeV1 {
                outcome_id: format!("workflow_outcome_{}_1", instance_id),
                instance_id: instance_id.clone(),
                status: WorkflowOutcomeStatus::Completed,
                completed_at: now.clone(),
                summary: "Workflow completed without a human checkpoint.".to_string(),
                contribution_refs: Vec::new(),
                global_event_refs: Vec::new(),
                evidence_refs: Vec::new(),
            });
            trace.push(WorkflowTraceEventV1 {
                event_id: format!("workflow_event_{}_2", instance_id),
                instance_id: instance_id.clone(),
                event_type: "workflow_completed".to_string(),
                sequence: 2,
                timestamp: now.clone(),
                payload: json!({
                    "status": "completed",
                }),
            });
        }

        let snapshot = WorkflowSnapshotV1 {
            instance: instance.clone(),
            trace,
            checkpoints,
            outcome,
        };
        self.cortex_instances.insert(
            instance_id,
            CortexWorkflowRecord {
                definition,
                binding,
                snapshot,
            },
        );
        Ok(instance)
    }

    pub fn signal_cortex_workflow(
        &mut self,
        instance_id: &str,
        signal: WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, String> {
        let record = self
            .cortex_instances
            .get_mut(instance_id)
            .ok_or_else(|| "Workflow instance not found".to_string())?;
        let now = now_token();
        let decision = signal
            .payload
            .get("decision")
            .and_then(Value::as_str)
            .unwrap_or(signal.signal_type.as_str())
            .to_ascii_lowercase();

        let next_sequence = (record.snapshot.trace.len() as u64) + 1;
        record.snapshot.trace.push(WorkflowTraceEventV1 {
            event_id: format!("workflow_event_{}_{}", instance_id, next_sequence),
            instance_id: instance_id.to_string(),
            event_type: "signal_received".to_string(),
            sequence: next_sequence,
            timestamp: now.clone(),
            payload: json!({
                "signalType": signal.signal_type,
                "checkpointId": signal.checkpoint_id,
                "decision": decision,
            }),
        });

        if decision == "cancel" || decision == "cancelled" || decision == "rejected" {
            let checkpoint_id = record
                .snapshot
                .checkpoints
                .iter_mut()
                .find(|checkpoint| checkpoint.status == WorkflowCheckpointStatus::Pending)
                .map(|checkpoint| {
                    checkpoint.status = WorkflowCheckpointStatus::Cancelled;
                    checkpoint.resolved_at = Some(now.clone());
                    checkpoint.checkpoint_id.clone()
                });
            record.snapshot.instance.status = WorkflowInstanceStatus::Cancelled;
            record.snapshot.instance.updated_at = now.clone();
            record.snapshot.outcome = Some(WorkflowOutcomeV1 {
                outcome_id: format!("workflow_outcome_{}_{}", instance_id, next_sequence),
                instance_id: instance_id.to_string(),
                status: WorkflowOutcomeStatus::Cancelled,
                completed_at: now.clone(),
                summary: "Cancelled by workflow signal".to_string(),
                contribution_refs: Vec::new(),
                global_event_refs: Vec::new(),
                evidence_refs: Vec::new(),
            });
            return Ok(WorkflowCheckpointResultV1 {
                instance_id: instance_id.to_string(),
                checkpoint_id,
                status: WorkflowCheckpointStatus::Cancelled,
                updated_at: now,
            });
        }

        let pending_checkpoint = record
            .snapshot
            .checkpoints
            .iter_mut()
            .find(|checkpoint| checkpoint.status == WorkflowCheckpointStatus::Pending);
        let Some(checkpoint) = pending_checkpoint else {
            return Err("Workflow instance has no pending checkpoint".to_string());
        };

        checkpoint.status = WorkflowCheckpointStatus::Resolved;
        checkpoint.resolved_at = Some(now.clone());

        record.snapshot.instance.status = if decision == "pause" {
            WorkflowInstanceStatus::Paused
        } else {
            WorkflowInstanceStatus::Completed
        };
        record.snapshot.instance.updated_at = now.clone();
        record.snapshot.outcome = Some(WorkflowOutcomeV1 {
            outcome_id: format!("workflow_outcome_{}_{}", instance_id, next_sequence),
            instance_id: instance_id.to_string(),
            status: if decision == "pause" {
                WorkflowOutcomeStatus::Paused
            } else {
                WorkflowOutcomeStatus::Completed
            },
            completed_at: now.clone(),
            summary: if decision == "pause" {
                "Workflow paused by operator.".to_string()
            } else {
                "Workflow completed after checkpoint resolution.".to_string()
            },
            contribution_refs: Vec::new(),
            global_event_refs: Vec::new(),
            evidence_refs: Vec::new(),
        });

        Ok(WorkflowCheckpointResultV1 {
            instance_id: instance_id.to_string(),
            checkpoint_id: Some(checkpoint.checkpoint_id.clone()),
            status: WorkflowCheckpointStatus::Resolved,
            updated_at: now,
        })
    }

    pub fn snapshot_cortex_workflow(
        &self,
        instance_id: &str,
    ) -> Result<WorkflowSnapshotV1, String> {
        self.cortex_instances
            .get(instance_id)
            .map(|record| record.snapshot.clone())
            .ok_or_else(|| "Workflow instance not found".to_string())
    }

    pub fn cancel_cortex_workflow(
        &mut self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, String> {
        let record = self
            .cortex_instances
            .get_mut(instance_id)
            .ok_or_else(|| "Workflow instance not found".to_string())?;
        let now = now_token();

        let checkpoint_id = record
            .snapshot
            .checkpoints
            .iter_mut()
            .find(|checkpoint| checkpoint.status == WorkflowCheckpointStatus::Pending)
            .map(|checkpoint| {
                checkpoint.status = WorkflowCheckpointStatus::Cancelled;
                checkpoint.resolved_at = Some(now.clone());
                checkpoint.checkpoint_id.clone()
            });

        let sequence = (record.snapshot.trace.len() as u64) + 1;
        record.snapshot.trace.push(WorkflowTraceEventV1 {
            event_id: format!("workflow_event_{}_{}", instance_id, sequence),
            instance_id: instance_id.to_string(),
            event_type: "workflow_cancelled".to_string(),
            sequence,
            timestamp: now.clone(),
            payload: json!({
                "reason": reason,
            }),
        });

        record.snapshot.instance.status = WorkflowInstanceStatus::Cancelled;
        record.snapshot.instance.updated_at = now.clone();
        record.snapshot.outcome = Some(WorkflowOutcomeV1 {
            outcome_id: format!("workflow_outcome_{}_{}", instance_id, sequence),
            instance_id: instance_id.to_string(),
            status: WorkflowOutcomeStatus::Cancelled,
            completed_at: now.clone(),
            summary: reason.to_string(),
            contribution_refs: Vec::new(),
            global_event_refs: Vec::new(),
            evidence_refs: Vec::new(),
        });

        Ok(WorkflowCheckpointResultV1 {
            instance_id: instance_id.to_string(),
            checkpoint_id,
            status: WorkflowCheckpointStatus::Cancelled,
            updated_at: now,
        })
    }
}

fn workflow_scope(
    definition: &WorkflowDefinitionV1,
    binding: &WorkflowExecutionBindingV1,
) -> WorkflowScope {
    let mut scope = definition.scope.clone();
    if scope.space_id.is_none() {
        scope.space_id = binding
            .runtime_limits
            .get("spaceId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
    }
    scope
}

fn validate_supported_definition(definition: &WorkflowDefinitionV1) -> Result<(), String> {
    let unsupported = definition
        .graph
        .nodes
        .iter()
        .filter_map(|node| {
            let code = match node.kind {
                WorkflowNodeKind::EvaluationGate => Some("evaluation_gate"),
                WorkflowNodeKind::Parallel => Some("parallel"),
                WorkflowNodeKind::Switch => Some("switch"),
                WorkflowNodeKind::SubflowRef => Some("subflow_ref"),
                _ => None,
            }?;
            Some(format!(
                "WF_CANISTER_UNSUPPORTED_NODE_KIND:{}:{}",
                node.node_id, code
            ))
        })
        .collect::<Vec<_>>();

    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "workflow_engine_canister_v1 cannot compile unsupported node kinds: {}",
            unsupported.join(", ")
        ))
    }
}

fn digest_json<T: Serialize>(value: &T) -> Result<String, String> {
    let raw = serde_json::to_vec(value).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(raw);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn default_checkpoint_policy() -> WorkflowCheckpointPolicyV1 {
    WorkflowCheckpointPolicyV1 {
        resume_allowed: true,
        cancel_allowed: true,
        pause_allowed: true,
        timeout_seconds: None,
    }
}

fn now_token() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return (ic_cdk::api::time() / 1_000_000).to_string();
    }

    #[cfg(not(target_arch = "wasm32"))]
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::workflow::{
        ContextContractV1, WorkflowConfidence, WorkflowDraftPolicyV1, WorkflowExecutionAdapterKind,
        WorkflowExecutionProfileKind, WorkflowGraphV1, WorkflowMotifKind, WorkflowNodeV1,
        WorkflowProvenance,
    };

    fn sample_definition(kind: WorkflowNodeKind) -> WorkflowDefinitionV1 {
        WorkflowDefinitionV1 {
            schema_version: "1.0.0".to_string(),
            definition_id: "workflow_def_test".to_string(),
            scope: WorkflowScope {
                space_id: Some("space-alpha".to_string()),
                route_id: None,
                role: None,
            },
            intent_ref: None,
            intent: "test".to_string(),
            motif_kind: WorkflowMotifKind::Sequential,
            constraints: Vec::new(),
            graph: WorkflowGraphV1 {
                nodes: vec![WorkflowNodeV1 {
                    node_id: "node-1".to_string(),
                    label: "Node".to_string(),
                    kind,
                    reads: Vec::new(),
                    writes: Vec::new(),
                    evidence_outputs: Vec::new(),
                    authority_requirements: Vec::new(),
                    checkpoint_policy: Some(default_checkpoint_policy()),
                    loop_policy: None,
                    subflow_ref: None,
                    config: json!({}),
                }],
                edges: Vec::new(),
            },
            context_contract: ContextContractV1::default(),
            confidence: WorkflowConfidence {
                score: 0.9,
                rationale: "test".to_string(),
            },
            lineage: Default::default(),
            policy: WorkflowDraftPolicyV1 {
                recommendation_only: false,
                require_review: true,
                allow_shadow_execution: false,
            },
            provenance: WorkflowProvenance {
                created_by: "tester".to_string(),
                created_at: "0".to_string(),
                source_mode: "test".to_string(),
            },
            governance_ref: None,
            digest: None,
        }
    }

    fn sample_binding() -> WorkflowExecutionBindingV1 {
        WorkflowExecutionBindingV1 {
            schema_version: "1.0.0".to_string(),
            binding_id: "binding-test".to_string(),
            definition_id: "workflow_def_test".to_string(),
            adapter: WorkflowExecutionAdapterKind::WorkflowEngineCanisterV1,
            execution_profile: WorkflowExecutionProfileKind::Async,
            checkpoint_policy: Some(default_checkpoint_policy()),
            runtime_limits: Default::default(),
            governance_ref: None,
            provenance: WorkflowProvenance {
                created_by: "tester".to_string(),
                created_at: "0".to_string(),
                source_mode: "test".to_string(),
            },
        }
    }

    #[test]
    fn compile_rejects_unsupported_node_kinds() {
        let machine = StateMachine::new();
        let result = machine.compile_cortex_workflow(
            sample_definition(WorkflowNodeKind::Parallel),
            sample_binding(),
        );
        assert!(result.is_err());
        assert!(result
            .err()
            .expect("error")
            .contains("WF_CANISTER_UNSUPPORTED_NODE_KIND"));
    }

    #[test]
    fn start_signal_and_snapshot_round_trip() {
        let mut machine = StateMachine::new();
        let instance = machine
            .start_cortex_workflow(
                sample_definition(WorkflowNodeKind::HumanCheckpoint),
                sample_binding(),
            )
            .expect("start");
        assert_eq!(instance.status, WorkflowInstanceStatus::WaitingCheckpoint);

        let before = machine
            .snapshot_cortex_workflow(instance.instance_id.as_str())
            .expect("snapshot");
        assert_eq!(before.checkpoints.len(), 1);
        assert_eq!(
            before.checkpoints[0].status,
            WorkflowCheckpointStatus::Pending
        );

        let result = machine
            .signal_cortex_workflow(
                instance.instance_id.as_str(),
                WorkflowSignalV1 {
                    signal_type: "approve".to_string(),
                    checkpoint_id: before.checkpoints[0].checkpoint_id.clone().into(),
                    payload: json!({ "decision": "approve" }),
                },
            )
            .expect("signal");
        assert_eq!(result.status, WorkflowCheckpointStatus::Resolved);

        let after = machine
            .snapshot_cortex_workflow(instance.instance_id.as_str())
            .expect("snapshot");
        assert_eq!(after.instance.status, WorkflowInstanceStatus::Completed);
        assert_eq!(
            after.checkpoints[0].status,
            WorkflowCheckpointStatus::Resolved
        );
        assert_eq!(
            after.outcome.as_ref().map(|outcome| outcome.status.clone()),
            Some(WorkflowOutcomeStatus::Completed)
        );
    }

    #[test]
    fn cancel_marks_instance_cancelled() {
        let mut machine = StateMachine::new();
        let instance = machine
            .start_cortex_workflow(
                sample_definition(WorkflowNodeKind::HumanCheckpoint),
                sample_binding(),
            )
            .expect("start");
        let result = machine
            .cancel_cortex_workflow(instance.instance_id.as_str(), "operator requested cancel")
            .expect("cancel");
        assert_eq!(result.status, WorkflowCheckpointStatus::Cancelled);

        let snapshot = machine
            .snapshot_cortex_workflow(instance.instance_id.as_str())
            .expect("snapshot");
        assert_eq!(snapshot.instance.status, WorkflowInstanceStatus::Cancelled);
        assert_eq!(
            snapshot
                .outcome
                .as_ref()
                .map(|outcome| outcome.status.clone()),
            Some(WorkflowOutcomeStatus::Cancelled)
        );
    }
}
