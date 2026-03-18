use crate::temporal::RuntimeMode;

pub const HUMAN_APPROVAL_SIGNAL: &str = "human_approval";
pub const RUN_SNAPSHOT_QUERY: &str = "run_snapshot";

#[cfg(feature = "temporal-sdk")]
pub async fn run_temporal_sdk_worker(mode: RuntimeMode) -> Result<(), String> {
    tracing::info!(
        "Temporal SDK backend selected for mode {}; awaiting termination signal.",
        mode.as_str()
    );
    tokio::signal::ctrl_c()
        .await
        .map_err(|err| format!("temporal_sdk_ctrl_c_wait_failed: {}", err))?;
    Ok(())
}

#[cfg(not(feature = "temporal-sdk"))]
pub async fn run_temporal_sdk_worker(mode: RuntimeMode) -> Result<(), String> {
    let _ = mode;
    Err("temporal_sdk_backend_unavailable_build_with_feature_temporal-sdk".to_string())
}
