use crate::activities::reasoning::{
    ProviderExecutionTrace, ReasoningActivity, ReasoningInput, ReasoningOutput,
};
use crate::activities::simulation::{
    EvaluatePlanInput, EvaluatePlanOutput, EvaluateSimulationPlanActivity,
};
use crate::temporal::{Activity, RuntimeMode};
use cortex_domain::agent::contracts::{
    ActionTarget, AgentIntent, AgentRunEvent, AuthorityExecutionOutcome, TemporalBridgeRunSnapshot,
    TemporalBridgeSignalCommand, TemporalBridgeStartCommand,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct TemporalNativeApprovalSignal {
    decision: String,
    rationale: Option<String>,
    actor: String,
    decision_ref: Option<String>,
}

#[derive(Default)]
struct TemporalNativeState {
    run_signals: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<TemporalNativeApprovalSignal>>>>,
    run_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

pub async fn run_temporal_bridge_worker(mode: RuntimeMode) -> Result<(), String> {
    if mode == RuntimeMode::GatewayPrimary {
        return Ok(());
    }

    ensure_temporal_runtime_dirs()?;
    let state = TemporalNativeState::default();
    tracing::info!(
        "Temporal-native worker loop started (mode={})",
        mode.as_str()
    );

    loop {
        process_start_commands(&state).await?;
        process_signal_commands(&state)?;
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub async fn run_temporal_native_worker(mode: RuntimeMode) -> Result<(), String> {
    run_temporal_bridge_worker(mode).await
}

fn ensure_temporal_runtime_dirs() -> Result<(), String> {
    fs::create_dir_all(temporal_start_commands_dir()).map_err(|err| err.to_string())?;
    fs::create_dir_all(temporal_signal_commands_dir()).map_err(|err| err.to_string())?;
    fs::create_dir_all(temporal_snapshots_dir()).map_err(|err| err.to_string())
}

async fn process_start_commands(state: &TemporalNativeState) -> Result<(), String> {
    for entry in read_sorted_json_files(temporal_start_commands_dir())? {
        let raw = fs::read_to_string(&entry).map_err(|err| err.to_string())?;
        let command = match serde_json::from_str::<TemporalBridgeStartCommand>(&raw) {
            Ok(command) => command,
            Err(err) => {
                tracing::warn!(
                    "Ignoring invalid temporal start command {}: {}",
                    entry.display(),
                    err
                );
                let _ = fs::remove_file(&entry);
                continue;
            }
        };
        let _ = fs::remove_file(&entry);

        let already_running = state
            .run_tasks
            .lock()
            .ok()
            .map(|tasks| tasks.contains_key(&command.run_id))
            .unwrap_or(false);
        if already_running {
            continue;
        }

        let (signal_tx, signal_rx) = mpsc::unbounded_channel::<TemporalNativeApprovalSignal>();
        if let Ok(mut signals) = state.run_signals.lock() {
            signals.insert(command.run_id.clone(), signal_tx);
        }

        let state_signals = state.run_signals.clone();
        let run_id = command.run_id.clone();
        let run_id_for_task = run_id.clone();
        let task = tokio::spawn(async move {
            run_temporal_workflow(command, signal_rx).await;
            if let Ok(mut signals) = state_signals.lock() {
                signals.remove(&run_id_for_task);
            }
        });
        if let Ok(mut tasks) = state.run_tasks.lock() {
            tasks.insert(run_id, task);
        }
    }
    Ok(())
}

fn process_signal_commands(state: &TemporalNativeState) -> Result<(), String> {
    for entry in read_sorted_json_files(temporal_signal_commands_dir())? {
        let raw = fs::read_to_string(&entry).map_err(|err| err.to_string())?;
        let command = match serde_json::from_str::<TemporalBridgeSignalCommand>(&raw) {
            Ok(command) => command,
            Err(err) => {
                tracing::warn!(
                    "Ignoring invalid temporal signal command {}: {}",
                    entry.display(),
                    err
                );
                let _ = fs::remove_file(&entry);
                continue;
            }
        };
        let _ = fs::remove_file(&entry);
        let sender = state
            .run_signals
            .lock()
            .ok()
            .and_then(|signals| signals.get(&command.run_id).cloned());
        if let Some(sender) = sender {
            let _ = sender.send(TemporalNativeApprovalSignal {
                decision: command.decision,
                rationale: command.rationale,
                actor: command.actor,
                decision_ref: command.decision_ref,
            });
        } else {
            tracing::warn!(
                "Temporal-native signal received for unknown run {}",
                command.run_id
            );
        }
    }
    Ok(())
}

async fn run_temporal_workflow(
    command: TemporalBridgeStartCommand,
    mut signal_rx: mpsc::UnboundedReceiver<TemporalNativeApprovalSignal>,
) {
    let mut snapshot = TemporalBridgeRunSnapshot {
        schema_version: "1.0.0".to_string(),
        run_id: command.run_id.clone(),
        workflow_id: command.workflow_id.clone(),
        space_id: command.space_id.clone(),
        contribution_id: command.contribution_id.clone(),
        status: "queued".to_string(),
        started_at: now_iso(),
        updated_at: now_iso(),
        sequence: 0,
        events: Vec::new(),
        simulation: None,
        surface_update: None,
        authority_outcome: None,
        provider_trace: None,
        approval_timeout_seconds: command.approval_timeout_seconds,
        terminal: false,
        error: None,
    };
    append_event(
        &mut snapshot,
        "run_started",
        json!({ "status": "queued", "contributionId": command.contribution_id }),
    );
    let _ = persist_snapshot(&snapshot);

    snapshot.status = "simulating".to_string();
    let contribution_id = snapshot.contribution_id.clone();
    append_event(
        &mut snapshot,
        "run_started",
        json!({ "status": "simulating", "contributionId": contribution_id }),
    );
    let _ = persist_snapshot(&snapshot);

    let reasoning_result = ReasoningActivity
        .execute(ReasoningInput {
            contribution_id: snapshot.contribution_id.clone(),
            space_id: snapshot.space_id.clone(),
        })
        .await;
    let reasoning_result = match reasoning_result {
        Ok(output) => output,
        Err(err) => {
            snapshot.status = "failed".to_string();
            snapshot.terminal = true;
            snapshot.error = Some(err.clone());
            append_event(&mut snapshot, "run_failed", json!({ "error": err }));
            let _ = persist_snapshot(&snapshot);
            return;
        }
    };
    snapshot.provider_trace = Some(provider_trace_json(&reasoning_result.provider_trace));

    let simulation_result = EvaluateSimulationPlanActivity
        .execute(EvaluatePlanInput {
            scenario_id: format!("sim-{}", snapshot.contribution_id),
            action_targets_json: reasoning_result.action_targets_json.clone(),
        })
        .await;
    let simulation_result = match simulation_result {
        Ok(output) => output,
        Err(err) => {
            snapshot.status = "failed".to_string();
            snapshot.terminal = true;
            snapshot.error = Some(err.clone());
            append_event(&mut snapshot, "run_failed", json!({ "error": err }));
            let _ = persist_snapshot(&snapshot);
            return;
        }
    };

    let simulation_value = simulation_json(&simulation_result);
    snapshot.simulation = Some(simulation_value.clone());
    snapshot.surface_update = Some(build_surface_update(
        &snapshot.space_id,
        &snapshot.run_id,
        &simulation_result,
    ));
    snapshot.status = "waiting_approval".to_string();
    append_event(&mut snapshot, "simulation_ready", simulation_value);
    let surface_update = snapshot.surface_update.clone();
    append_event(
        &mut snapshot,
        "surface_update",
        json!({ "surfaceUpdate": surface_update }),
    );
    append_event(
        &mut snapshot,
        "approval_required",
        json!({ "status": "waiting_approval" }),
    );
    let _ = persist_snapshot(&snapshot);

    let timeout = Duration::from_secs(snapshot.approval_timeout_seconds.max(1));
    let deadline = Instant::now() + timeout;
    let signal = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break None;
        }
        match tokio::time::timeout(remaining, signal_rx.recv()).await {
            Ok(Some(signal)) => break Some(signal),
            Ok(None) => break None,
            Err(_) => break None,
        }
    };

    let Some(signal) = signal else {
        snapshot.status = "failed".to_string();
        snapshot.terminal = true;
        snapshot.error = Some("approval_timeout".to_string());
        append_event(
            &mut snapshot,
            "run_failed",
            json!({ "error": "approval_timeout" }),
        );
        let _ = persist_snapshot(&snapshot);
        return;
    };

    if signal.decision.eq_ignore_ascii_case("approved") {
        snapshot.status = "applying".to_string();
        append_event(
            &mut snapshot,
            "run_started",
            json!({ "status": "applying" }),
        );
        let _ = persist_snapshot(&snapshot);

        let action_target = action_target_from_reasoning(&reasoning_result);
        let outcome = AuthorityExecutionOutcome {
            accepted: true,
            action_target,
            applied_at: now_iso(),
            host_receipt: Some(format!("temporal-native:{}", snapshot.run_id)),
            error: None,
        };
        snapshot.authority_outcome = Some(outcome.clone());
        snapshot.status = "completed".to_string();
        snapshot.terminal = true;
        append_event(
            &mut snapshot,
            "run_completed",
            json!({ "status": "completed", "authorityOutcome": outcome }),
        );
        let _ = persist_snapshot(&snapshot);
    } else {
        snapshot.status = "rejected".to_string();
        snapshot.terminal = true;
        append_event(
            &mut snapshot,
            "run_completed",
            json!({
                "status": "rejected",
                "decision": signal.decision,
                "actor": signal.actor,
                "rationale": signal.rationale,
                "decisionRef": signal.decision_ref
            }),
        );
        let _ = persist_snapshot(&snapshot);
    }
}

fn simulation_json(output: &EvaluatePlanOutput) -> Value {
    json!({
        "success": output.success,
        "violationCount": output.violation_count,
        "riskScore": output.risk_score,
        "siqsScore": output.siqs_score,
        "sessionId": output.session_id,
        "structuralDiffSummary": output.structural_diff_summary
    })
}

fn action_target_from_reasoning(reasoning: &ReasoningOutput) -> ActionTarget {
    let Some(first) = reasoning.action_targets_json.first() else {
        return ActionTarget {
            protocol: "ic".to_string(),
            address: "kg-canister".to_string(),
            method: "create_context_node".to_string(),
            payload: Vec::new(),
        };
    };

    if let Ok(intent) = serde_json::from_str::<AgentIntent>(first) {
        return match intent {
            AgentIntent::CreateContextNode { node_id, content } => ActionTarget {
                protocol: "ic".to_string(),
                address: "kg-canister".to_string(),
                method: "create_context_node".to_string(),
                payload: serde_json::to_vec(&json!({
                    "nodeId": node_id,
                    "content": content
                }))
                .unwrap_or_default(),
            },
            AgentIntent::ProposeSchemaMutation { schema_json } => ActionTarget {
                protocol: "ic".to_string(),
                address: "kg-canister".to_string(),
                method: "propose_schema_mutation".to_string(),
                payload: serde_json::to_vec(&json!({ "schemaJson": schema_json }))
                    .unwrap_or_default(),
            },
            AgentIntent::ExecuteSimulation { scenario_id } => ActionTarget {
                protocol: "ic".to_string(),
                address: "simulation-canister".to_string(),
                method: "execute_simulation".to_string(),
                payload: serde_json::to_vec(&json!({ "scenarioId": scenario_id }))
                    .unwrap_or_default(),
            },
            AgentIntent::ApplyActionTarget { action_target } => action_target,
        };
    }

    serde_json::from_str(first).unwrap_or(ActionTarget {
        protocol: "ic".to_string(),
        address: "kg-canister".to_string(),
        method: "create_context_node".to_string(),
        payload: Vec::new(),
    })
}

fn build_surface_update(space_id: &str, run_id: &str, simulation: &EvaluatePlanOutput) -> Value {
    json!({
        "type": "Container",
        "children": {
            "explicitList": [
                {
                    "id": "sim-header",
                    "componentProperties": {
                        "Heading": {
                            "text": format!("Temporal Evaluation For Space: {}", space_id)
                        }
                    }
                },
                {
                    "id": "sim-summary",
                    "componentProperties": {
                        "Text": {
                            "text": format!(
                                "session={} violations={} risk={} SIQS={:.2}",
                                simulation.session_id,
                                simulation.violation_count,
                                simulation.risk_score,
                                simulation.siqs_score
                            )
                        }
                    }
                },
                {
                    "id": "sim-approval",
                    "componentProperties": {
                        "ApprovalControls": {
                            "spaceId": space_id,
                            "runId": run_id,
                            "scenarioId": simulation.session_id,
                            "decisionRef": format!("DEC-{}", run_id)
                        }
                    }
                }
            ]
        }
    })
}

fn append_event(snapshot: &mut TemporalBridgeRunSnapshot, event_type: &str, payload: Value) {
    let sequence = snapshot
        .events
        .last()
        .map(|event| event.sequence.saturating_add(1))
        .unwrap_or(1);
    let event = AgentRunEvent {
        event_type: event_type.to_string(),
        run_id: snapshot.run_id.clone(),
        space_id: snapshot.space_id.clone(),
        timestamp: now_iso(),
        sequence,
        payload,
    };
    snapshot.sequence = sequence;
    snapshot.updated_at = now_iso();
    snapshot.events.push(event);
}

fn persist_snapshot(snapshot: &TemporalBridgeRunSnapshot) -> Result<(), String> {
    let path = temporal_snapshot_path(&snapshot.run_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(snapshot).map_err(|err| err.to_string())?;
    fs::write(path, bytes).map_err(|err| err.to_string())
}

fn read_sorted_json_files(dir: PathBuf) -> Result<Vec<PathBuf>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = fs::read_dir(dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn decision_surface_log_dir() -> PathBuf {
    std::env::var("NOSTRA_DECISION_SURFACE_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CORTEX_IC_PROJECT_ROOT")
                .map(|root| PathBuf::from(root).join("logs/system/decision_surfaces"))
        })
        .unwrap_or_else(|_| PathBuf::from("logs/system/decision_surfaces"))
}

fn temporal_runtime_root() -> PathBuf {
    decision_surface_log_dir().join("temporal_bridge_runtime")
}

fn temporal_start_commands_dir() -> PathBuf {
    temporal_runtime_root().join("commands").join("start")
}

fn temporal_signal_commands_dir() -> PathBuf {
    temporal_runtime_root().join("commands").join("signal")
}

fn temporal_snapshots_dir() -> PathBuf {
    temporal_runtime_root().join("snapshots")
}

fn temporal_snapshot_path(run_id: &str) -> PathBuf {
    temporal_snapshots_dir().join(format!("{}.json", sanitize_fs_component(run_id)))
}

fn sanitize_fs_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

fn provider_trace_json(trace: &ProviderExecutionTrace) -> Value {
    serde_json::to_value(trace).unwrap_or_else(|_| json!(null))
}
