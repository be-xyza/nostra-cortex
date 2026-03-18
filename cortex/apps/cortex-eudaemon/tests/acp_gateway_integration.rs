use cortex_desktop::gateway::server::GatewayService;
use cortex_desktop::services::file_system_service::FileSystemService;
use serde_json::{Value, json};
use std::net::TcpListener;
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
    for _ in 0..20 {
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

#[tokio::test]
#[ignore = "requires loopback socket permissions in test environment"]
async fn acp_gateway_pilot_gate_and_lifecycle() {
    std::env::remove_var("CORTEX_ACP_PILOT");

    let (port, handle) = start_gateway().await;
    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    // Gate off: JSON-RPC should return ACP_PILOT_DISABLED.
    let rpc_response: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rpc_response["error"]["code"], -32030);
    assert_eq!(
        rpc_response["error"]["data"]["errorCode"],
        "ACP_PILOT_DISABLED"
    );

    let terminal_off_status = client
        .post(format!("{}/api/acp/terminal/create", base))
        .json(&json!({
            "sessionId": "sess_1",
            "command": "echo",
            "args": ["hi"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        terminal_off_status.status(),
        reqwest::StatusCode::SERVICE_UNAVAILABLE
    );

    // Gate on: full lifecycle checks.
    std::env::set_var("CORTEX_ACP_PILOT", "1");

    let init_on: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "initialize",
            "params": {}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(init_on["result"]["protocolVersion"], "0.1-pilot");

    let cwd = FileSystemService::get_root_path().display().to_string();
    let new_session: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session/new",
            "params": { "cwd": cwd }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let session_id = new_session["result"]["sessionId"]
        .as_str()
        .unwrap()
        .to_string();

    let prompt: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "session/prompt",
            "params": { "sessionId": session_id, "prompt": "hello" }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(prompt["result"]["stopReason"], "completed");

    let load: Value = client
        .post(format!("{}/api/acp/rpc", base))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "session/load",
            "params": { "sessionId": new_session["result"]["sessionId"] }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(load["result"]["updates"].as_array().unwrap().len() >= 2);

    let policy_deny = client
        .post(format!("{}/api/acp/terminal/create", base))
        .json(&json!({
            "sessionId": "sess_x",
            "command": "rm",
            "args": ["-rf", "/"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(policy_deny.status(), reqwest::StatusCode::BAD_REQUEST);

    let fs_deny = client
        .post(format!("{}/api/acp/fs/read_text_file", base))
        .json(&json!({
            "sessionId": "sess_x",
            "path": "relative/path.txt"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(fs_deny.status(), reqwest::StatusCode::BAD_REQUEST);

    let acp_metrics: Value = client
        .get(format!("{}/api/metrics/acp", base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(acp_metrics.get("emit_attempts_total").is_some());
    assert!(acp_metrics.get("fallback_queue_total").is_some());

    handle.abort();
    std::env::remove_var("CORTEX_ACP_PILOT");
}
