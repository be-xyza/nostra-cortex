use cortex_domain::workflow::{
    WorkflowDefinitionV1, WorkflowExecutionBindingV1, WorkflowSignalV1,
};
use candid::CandidType;
use ic_cdk_macros::{export_candid, init, query, update};
use nostra_workflow_engine::execution::StateMachine;
use nostra_workflow_engine::registry::WorkflowRegistry;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static REGISTRY: RefCell<WorkflowRegistry> = RefCell::new(WorkflowRegistry::new());
    static ENGINE: RefCell<StateMachine> = RefCell::new(StateMachine::new());
}

#[derive(CandidType, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
struct TextOperationResult {
    ok: Option<String>,
    err: Option<String>,
}

#[init]
fn init() {
    // Registry auto-initializes with defaults.
}

#[query]
fn list_workflows() -> Vec<String> {
    vec![
        "WORKFLOW_TEMPLATE_WIZARD".to_string(),
        "WORKFLOW_REQUEST_REVIEW".to_string(),
        "WORKFLOW_PUBLISH_EDITION".to_string(),
    ]
}

#[update]
fn trigger_workflow(workflow_id: String) -> String {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        engine.create_instance(workflow_id, "RENDER_FORM".to_string())
    })
}

#[query]
fn get_workflow_state(instance_id: String) -> String {
    ENGINE.with(|engine| {
        let engine = engine.borrow();
        if let Some(instance) = engine.get_instance(&instance_id) {
            if instance.current_state == "RENDER_FORM" {
                return r#"{ "surfaceUpdate": { "components": [] } }"#.to_string();
            }
            return serde_json::to_string(&instance).unwrap_or_default();
        }
        "{}".to_string()
    })
}

#[update]
fn submit_workflow_step(instance_id: String, data: HashMap<String, String>) -> String {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        match engine.transition(&instance_id, "SUBMIT", data) {
            Ok(new_state) => new_state,
            Err(e) => format!("Error: {}", e),
        }
    })
}

#[query]
fn compile_workflow_v1(definition_json: String, binding_json: String) -> TextOperationResult {
    let definition = match parse_json::<WorkflowDefinitionV1>(&definition_json, "definition_json") {
        Ok(value) => value,
        Err(err) => return err_result(err),
    };
    let binding = match parse_json::<WorkflowExecutionBindingV1>(&binding_json, "binding_json") {
        Ok(value) => value,
        Err(err) => return err_result(err),
    };

    ENGINE.with(|engine| match engine.borrow().compile_cortex_workflow(definition, binding) {
        Ok(plan) => ok_json(&plan),
        Err(err) => err_result(err),
    })
}

#[update]
fn start_workflow_v1(definition_json: String, binding_json: String) -> TextOperationResult {
    let definition = match parse_json::<WorkflowDefinitionV1>(&definition_json, "definition_json") {
        Ok(value) => value,
        Err(err) => return err_result(err),
    };
    let binding = match parse_json::<WorkflowExecutionBindingV1>(&binding_json, "binding_json") {
        Ok(value) => value,
        Err(err) => return err_result(err),
    };

    ENGINE.with(|engine| {
        match engine
            .borrow_mut()
            .start_cortex_workflow(definition, binding)
        {
            Ok(instance) => ok_json(&instance),
            Err(err) => err_result(err),
        }
    })
}

#[update]
fn signal_workflow_v1(instance_id: String, signal_json: String) -> TextOperationResult {
    let signal = match parse_json::<WorkflowSignalV1>(&signal_json, "signal_json") {
        Ok(value) => value,
        Err(err) => return err_result(err),
    };

    ENGINE.with(|engine| {
        match engine
            .borrow_mut()
            .signal_cortex_workflow(instance_id.as_str(), signal)
        {
            Ok(result) => ok_json(&result),
            Err(err) => err_result(err),
        }
    })
}

#[query]
fn snapshot_workflow_v1(instance_id: String) -> TextOperationResult {
    ENGINE.with(|engine| match engine.borrow().snapshot_cortex_workflow(instance_id.as_str()) {
        Ok(snapshot) => ok_json(&snapshot),
        Err(err) => err_result(err),
    })
}

#[update]
fn cancel_workflow_v1(instance_id: String, reason: String) -> TextOperationResult {
    ENGINE.with(|engine| {
        match engine
            .borrow_mut()
            .cancel_cortex_workflow(instance_id.as_str(), reason.as_str())
        {
            Ok(result) => ok_json(&result),
            Err(err) => err_result(err),
        }
    })
}

fn parse_json<T: serde::de::DeserializeOwned>(raw: &str, field: &str) -> Result<T, String> {
    serde_json::from_str(raw).map_err(|err| format!("invalid {}: {}", field, err))
}

fn ok_json<T: Serialize>(value: &T) -> TextOperationResult {
    match serde_json::to_string(value) {
        Ok(raw) => TextOperationResult {
            ok: Some(raw),
            err: None,
        },
        Err(err) => err_result(format!("serialization failed: {}", err)),
    }
}

fn err_result(message: String) -> TextOperationResult {
    TextOperationResult {
        ok: None,
        err: Some(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::workflow::{
        ContextContractV1, WorkflowCheckpointPolicyV1, WorkflowConfidence, WorkflowDraftPolicyV1,
        WorkflowExecutionAdapterKind, WorkflowExecutionProfileKind, WorkflowGraphV1,
        WorkflowMotifKind, WorkflowNodeKind, WorkflowNodeV1, WorkflowProvenance, WorkflowScope,
    };
    use serde_json::json;

    fn reset_engine() {
        ENGINE.with(|engine| {
            *engine.borrow_mut() = StateMachine::new();
        });
    }

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
                    checkpoint_policy: Some(WorkflowCheckpointPolicyV1 {
                        resume_allowed: true,
                        cancel_allowed: true,
                        pause_allowed: true,
                        timeout_seconds: Some(120),
                    }),
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
            checkpoint_policy: Some(WorkflowCheckpointPolicyV1 {
                resume_allowed: true,
                cancel_allowed: true,
                pause_allowed: true,
                timeout_seconds: Some(120),
            }),
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
    fn compile_start_snapshot_signal_and_cancel_contracts_work() {
        reset_engine();
        let definition = serde_json::to_string(&sample_definition(WorkflowNodeKind::HumanCheckpoint))
            .expect("definition");
        let binding = serde_json::to_string(&sample_binding()).expect("binding");

        let compile: TextOperationResult = compile_workflow_v1(definition.clone(), binding.clone());
        assert!(compile.err.is_none());
        assert!(compile.ok.as_ref().expect("ok").contains("workflow_engine_canister_v1"));

        let start: TextOperationResult = start_workflow_v1(definition, binding);
        assert!(start.err.is_none());
        let start_payload = start.ok.expect("start ok");
        let instance: serde_json::Value = serde_json::from_str(&start_payload).expect("instance");
        let instance_id = instance["instanceId"].as_str().expect("instance id").to_string();
        assert_eq!(instance["sourceOfTruth"], "workflow_engine_canister_v1");

        let snapshot: TextOperationResult = snapshot_workflow_v1(instance_id.clone());
        assert!(snapshot.err.is_none());
        assert!(snapshot.ok.as_ref().expect("snapshot").contains("checkpoints"));

        let signal: TextOperationResult = signal_workflow_v1(
            instance_id.clone(),
            serde_json::to_string(&WorkflowSignalV1 {
                signal_type: "approve".to_string(),
                checkpoint_id: None,
                payload: json!({ "decision": "approve" }),
            })
            .expect("signal"),
        );
        assert!(signal.err.is_none());

        let cancelled: TextOperationResult =
            cancel_workflow_v1(instance_id, "cancel anyway".to_string());
        assert!(cancelled.err.is_none());
    }

    #[test]
    fn unsupported_node_kinds_fail_deterministically() {
        reset_engine();
        let result: TextOperationResult = compile_workflow_v1(
            serde_json::to_string(&sample_definition(WorkflowNodeKind::Parallel))
                .expect("definition"),
            serde_json::to_string(&sample_binding()).expect("binding"),
        );
        assert!(result.ok.is_none());
        assert!(
            result
                .err
                .as_deref()
                .unwrap_or_default()
                .contains("WF_CANISTER_UNSUPPORTED_NODE_KIND")
        );
    }
}

export_candid!();
