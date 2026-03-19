pub mod activities;
pub mod agent_workflows;
pub mod temporal;
pub mod temporal_native;
pub mod temporal_sdk;
pub mod translator;

use activities::reasoning::ReasoningActivity;
use activities::simulation::EvaluateSimulationPlanActivity;
use agent_workflows::ArchitectAndEvaluateWorkflow;
use temporal::{Activity, RuntimeMode, TemporalExecutionBackend, Workflow};
use temporal_native::run_temporal_bridge_worker;
use temporal_sdk::run_temporal_sdk_worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cortex Temporal Worker initializing...");
    let runtime_mode = RuntimeMode::from_env();
    let temporal_backend = TemporalExecutionBackend::from_env();
    println!("Runtime mode: {}", runtime_mode.as_str());
    println!("Temporal backend: {}", temporal_backend.as_str());

    // In a production Temporal worker, this maps to:
    // let mut worker = Worker::new("SIMULATION_TASK_QUEUE");
    // worker.register_workflow(ArchitectAndEvaluateWorkflow::NAME, ArchitectAndEvaluateWorkflow);
    // worker.register_activity(EvaluateSimulationPlanActivity::NAME, EvaluateSimulationPlanActivity);
    // worker.run().await?;

    println!(
        "Registered Workflow: {}",
        ArchitectAndEvaluateWorkflow::NAME
    );
    println!(
        "Registered Activity: {}",
        EvaluateSimulationPlanActivity::NAME
    );
    println!("Registered Activity: {}", ReasoningActivity::NAME);

    match runtime_mode {
        RuntimeMode::GatewayPrimary => {
            println!("Worker in gateway_primary mode; temporal runtime disabled.");
        }
        RuntimeMode::TemporalShadow | RuntimeMode::TemporalPrimary => {
            println!("Worker listening on queue: SIMULATION_TASK_QUEUE");
            match temporal_backend {
                TemporalExecutionBackend::Bridge => {
                    run_temporal_bridge_worker(runtime_mode)
                        .await
                        .map_err(|err| format!("Temporal bridge runtime failed: {err}"))?;
                }
                TemporalExecutionBackend::Sdk => {
                    match run_temporal_sdk_worker(runtime_mode.clone()).await {
                        Ok(()) => {}
                        Err(err) => {
                            let allow_bridge_fallback =
                                std::env::var("CORTEX_TEMPORAL_SDK_FALLBACK_BRIDGE")
                                    .ok()
                                    .map(|value| {
                                        matches!(
                                            value.trim().to_ascii_lowercase().as_str(),
                                            "1" | "true" | "yes" | "on"
                                        )
                                    })
                                    .unwrap_or(true);
                            if !allow_bridge_fallback {
                                return Err(format!("Temporal SDK runtime failed: {err}").into());
                            }
                            println!(
                                "Temporal SDK backend unavailable; falling back to bridge runtime: {}",
                                err
                            );
                            run_temporal_bridge_worker(runtime_mode).await.map_err(|bridge_err| {
                                format!("Temporal bridge runtime failed after SDK fallback: {bridge_err}")
                            })?;
                        }
                    }
                }
            }
        }
    }

    // Keeping the process alive for local testing
    // std::future::pending::<()>().await;

    Ok(())
}
