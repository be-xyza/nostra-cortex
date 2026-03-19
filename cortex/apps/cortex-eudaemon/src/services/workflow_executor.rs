use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};

use cortex_runtime::workflow::executor::{
    WorkflowStepKind, build_execution_plan, start_message, step_analysis_message,
};

pub struct WorkflowExecutor {
    // We could hold state here, but for now we'll just have static methods or simple instance methods
}

impl WorkflowExecutor {
    pub async fn run_workflow(
        content: String,
        tx: Arc<broadcast::Sender<axum::extract::ws::Message>>,
    ) {
        let plan = build_execution_plan(&content);
        let total_steps = plan.len();
        let snapshot_service = crate::services::snapshot_service::SnapshotService::new(None);

        let _ = tx.send(axum::extract::ws::Message::Text(start_message(total_steps)));

        for step in plan {
            let trimmed = step.raw.trim();
            // Simulate "thinking" or "processing"
            let _ = tx.send(axum::extract::ws::Message::Text(step_analysis_message(
                &step,
            )));
            sleep(Duration::from_millis(500)).await;

            // Simple "Parser" logic
            if step.kind == WorkflowStepKind::Task {
                let task = trimmed.trim_start_matches("- [ ]").trim();
                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                    "[Executor] executing task: {}",
                    task
                )));
                sleep(Duration::from_millis(1000)).await;
                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                    "[Executor] ✓ Completed: {}",
                    task
                )));
            } else if step.kind == WorkflowStepKind::Command {
                let cmd = trimmed.trim_start_matches(">").trim();
                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                    "[Executor] Running command: {}",
                    cmd
                )));

                // Command Dispatch
                if cmd.starts_with("snapshot create") {
                    // split: snapshot create <name> <network>
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let name = parts[2];
                        let network = parts[3];
                        match snapshot_service.create_snapshot(name, network).await {
                            Ok(info) => {
                                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                                    "[Executor] Snapshot Created: {}",
                                    info.snapshot_id
                                )));
                            }
                            Err(e) => {
                                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                                    "[Executor] Snapshot FAILED: {}",
                                    e
                                )));
                            }
                        }
                    }
                } else if cmd.starts_with("snapshot restore") {
                    // split: snapshot restore <id/last> <network>
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let id = parts[2]; // handle "last" logic later, assume ID for now
                        let network = parts[3];
                        // Mock canister name as "nostra_backend" for now or parse from somewhere else
                        match snapshot_service
                            .restore_snapshot("nostra_backend", id, network)
                            .await
                        {
                            Ok(_) => {
                                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                                    "[Executor] Restore Successful to {}",
                                    id
                                )));
                            }
                            Err(e) => {
                                let _ = tx.send(axum::extract::ws::Message::Text(format!(
                                    "[Executor] Restore FAILED: {}",
                                    e
                                )));
                            }
                        }
                    }
                } else {
                    // Simulate generic command
                    sleep(Duration::from_millis(1500)).await;
                    let _ = tx.send(axum::extract::ws::Message::Text(format!(
                        "[Executor] Command finished: exit code 0"
                    )));
                }
            } else {
                // Just comment/text
            }
        }

        let _ = tx.send(axum::extract::ws::Message::Text(
            "[Executor] Workflow Execution Finished.".to_string(),
        ));
    }
}
