use axum::{Json, Router, extract::State, routing::post};
use cortex_eudaemon::gateway::server::GatewayService;
use cortex_eudaemon::services::file_system_service::FileSystemService;
use cortex_eudaemon::services::local_gateway::get_gateway;
use serde_json::{Value, json};
use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

async fn start_gateway() -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let handle = tokio::spawn(async move {
        GatewayService::start(port).await;
    });

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}", port);
    for _ in 0..40 {
        if client
            .get(format!("{}/api/health", base))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    (port, handle)
}

#[derive(Clone)]
struct MockState {
    accepted: Arc<AtomicUsize>,
}

async fn mock_emit(State(state): State<MockState>, Json(_payload): Json<Value>) -> Json<Value> {
    state.accepted.fetch_add(1, Ordering::Relaxed);
    Json(json!({"status":"ok"}))
}

async fn start_mock_log_registry() -> (u16, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let accepted = Arc::new(AtomicUsize::new(0));
    let state = MockState {
        accepted: accepted.clone(),
    };

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let app = Router::new()
        .route("/emit", post(mock_emit))
        .with_state(state);
    let addr = format!("127.0.0.1:{}", port);
    let handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    (port, accepted, handle)
}

#[tokio::test]
#[ignore = "requires loopback socket permissions and local env control"]
async fn staging_drill_outage_then_recovery_updates_metrics_and_drains_queue() {
    let temp_home =
        std::env::temp_dir().join(format!("acp_staging_ops_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&temp_home).unwrap();

    std::env::set_var("HOME", &temp_home);
    std::env::set_var("CORTEX_ACP_PILOT", "1");
    std::env::set_var("CORTEX_ACP_LOG_REGISTRY_URL", "http://127.0.0.1:9/emit");

    let (port, gateway_handle) = start_gateway().await;
    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    let cwd = FileSystemService::get_root_path().display().to_string();
    let session_new: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"session/new",
            "params":{"cwd": cwd}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let session_id = session_new["result"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();

    let _prompt: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc":"2.0",
            "id":2,
            "method":"session/prompt",
            "params":{"sessionId": session_id, "prompt":"simulate outage"}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mut metrics_after_outage: Value = json!({});
    for _ in 0..100 {
        metrics_after_outage = client
            .get(format!("{}/api/metrics/acp", base))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let emit_failure = metrics_after_outage["emit_failure_total"]
            .as_u64()
            .unwrap_or(0);
        let fallback_queue = metrics_after_outage["fallback_queue_total"]
            .as_u64()
            .unwrap_or(0);

        if emit_failure >= 1 && fallback_queue >= 1 {
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(
        metrics_after_outage["emit_failure_total"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );
    assert!(
        metrics_after_outage["fallback_queue_total"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );

    let queued_before_flush = get_gateway().get_queue_size();
    assert!(queued_before_flush >= 1);

    let (mock_port, accepted, mock_handle) = start_mock_log_registry().await;
    let endpoint = format!("http://127.0.0.1:{}/emit", mock_port);
    let readiness_client = reqwest::Client::new();
    for _ in 0..20 {
        let ready = readiness_client
            .post(&endpoint)
            .json(&json!({"probe": true}))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        if ready {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let emitted = get_gateway()
        .flush_observability_events(&endpoint)
        .await
        .unwrap();

    assert!(emitted >= 1);
    assert!(accepted.load(Ordering::Relaxed) >= 1);
    assert_eq!(get_gateway().get_queue_size(), 0);

    let metrics_after_recovery: Value = client
        .get(format!("{}/api/metrics/acp", base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        metrics_after_recovery["fallback_flush_success_total"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );

    gateway_handle.abort();
    mock_handle.abort();
    std::env::remove_var("CORTEX_ACP_LOG_REGISTRY_URL");
    std::env::remove_var("CORTEX_ACP_PILOT");
}
