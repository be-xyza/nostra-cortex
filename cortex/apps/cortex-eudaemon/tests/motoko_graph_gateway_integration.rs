use cortex_desktop::gateway::server::GatewayService;
use serde_json::{Value, json};
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;

async fn start_gateway() -> Option<(u16, tokio::task::JoinHandle<()>)> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!(
                "Skipping gateway integration test (socket bind failed): {}",
                err
            );
            return None;
        }
    };
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);

    let handle = tokio::spawn(async move {
        GatewayService::start(port).await;
    });

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}", port);
    for _ in 0..30 {
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

    Some((port, handle))
}

struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("{}-{}-{}", prefix, std::process::id(), nonce));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_snapshot_fixture(root: &Path) {
    let history = root.join("history");
    let pending = root.join("decisions").join("pending");
    let monitoring_runs = root.join("monitoring_runs");
    fs::create_dir_all(&history).expect("history dir");
    fs::create_dir_all(&pending).expect("pending dir");
    fs::create_dir_all(&monitoring_runs).expect("monitoring runs dir");

    let snapshot = json!({
      "schema_version": "1.0.0",
      "generated_at": "2026-02-08T19:00:00Z",
      "contribution_id": "078",
      "status": {
        "gate_result": "G2_DUAL_PATH_PASS",
        "posture": "watch-first",
        "authority_mode": "recommendation_only",
        "runtime_dependency_promotion": "deferred"
      },
      "workloads": [],
      "stability": [],
      "workflow_stages": [],
      "evidence": {
        "gate_file": "/tmp/m16_dual_path/gate.txt",
        "m4_metrics_file": "/tmp/m16_dual_path/m4_metrics.tsv",
        "m8_metrics_file": "/tmp/m16_dual_path/m8_metrics.tsv",
        "stability_file": "/tmp/m16_dual_path/path_stability.tsv",
        "analysis_file": "/Users/xaoj/ICP/research/reference/analysis/motoko-graph.md",
        "m8_pass_count": 2
      },
      "history_event_id": "kg_snapshot_fixture"
    });

    fs::write(
        root.join("snapshot_latest.json"),
        serde_json::to_vec_pretty(&snapshot).expect("snapshot json"),
    )
    .expect("write snapshot");

    let run = json!({
      "schema_version": "1.0.0",
      "run_id": "monitor_fixture_int_001",
      "started_at": "2026-02-08T19:10:00Z",
      "finished_at": "2026-02-08T19:10:08Z",
      "gateway_base": "http://127.0.0.1:3000",
      "overall_status": "warn",
      "required_failures": 0,
      "warnings": 1,
      "checks": [
        {"name":"Generate motoko-graph snapshot","required":true,"status":"pass","details":""},
        {"name":"Check gateway health endpoint","required":false,"status":"warn","details":"gateway offline"}
      ]
    });
    fs::write(
        monitoring_runs.join("monitor_fixture_int_001.json"),
        serde_json::to_vec_pretty(&run).expect("run json"),
    )
    .expect("write run");

    let trend = json!({
      "schema_version": "1.0.0",
      "generated_at": "2026-02-08T19:11:00Z",
      "windows": {
        "7d": {
          "total_runs": 1,
          "pass_runs": 0,
          "warn_runs": 1,
          "fail_runs": 0,
          "reliability_percent": 100.0,
          "warning_rate_percent": 100.0,
          "required_failure_rate_percent": 0.0,
          "gateway_warning_rate_percent": 100.0,
          "mean_duration_seconds": 8.0,
          "p95_duration_seconds": 8.0,
          "last_success_at": "2026-02-08T19:10:08Z"
        },
        "30d": {
          "total_runs": 1,
          "pass_runs": 0,
          "warn_runs": 1,
          "fail_runs": 0,
          "reliability_percent": 100.0,
          "warning_rate_percent": 100.0,
          "required_failure_rate_percent": 0.0,
          "gateway_warning_rate_percent": 100.0,
          "mean_duration_seconds": 8.0,
          "p95_duration_seconds": 8.0,
          "last_success_at": "2026-02-08T19:10:08Z"
        }
      },
      "latest": {
        "run_id": "monitor_fixture_int_001",
        "overall_status": "warn",
        "required_failures": 0,
        "warnings": 1,
        "duration_seconds": 8.0,
        "started_at": "2026-02-08T19:10:00Z",
        "finished_at": "2026-02-08T19:10:08Z"
      },
      "last_applied_decision_event_id": null,
      "next_action": "START_GATEWAY",
      "advisory_recommendation": "Hold Deferred"
    });
    fs::write(
        root.join("monitoring_trend_latest.json"),
        serde_json::to_vec_pretty(&trend).expect("trend json"),
    )
    .expect("write trend");
}

#[tokio::test]
#[ignore = "requires loopback socket permissions in test environment"]
async fn motoko_graph_gateway_capture_and_query_flow() {
    let temp = TempDir::new("motoko-graph-gateway");
    write_snapshot_fixture(temp.path());
    std::env::set_var(
        "NOSTRA_MOTOKO_GRAPH_LOG_DIR",
        temp.path().display().to_string(),
    );

    let Some((port, handle)) = start_gateway().await else {
        std::env::remove_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR");
        return;
    };
    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    let snapshot: Value = client
        .get(format!("{}/api/kg/motoko-graph/snapshot", base))
        .send()
        .await
        .expect("snapshot response")
        .json()
        .await
        .expect("snapshot json");
    assert_eq!(snapshot["contribution_id"], "078");

    let health: Value = client
        .get(format!("{}/api/kg/motoko-graph/health", base))
        .send()
        .await
        .expect("health response")
        .json()
        .await
        .expect("health json");
    assert_eq!(health["status"], "ok");

    let trends: Value = client
        .get(format!("{}/api/kg/motoko-graph/monitoring-trends", base))
        .send()
        .await
        .expect("trends response")
        .json()
        .await
        .expect("trends json");
    assert_eq!(trends["next_action"], "START_GATEWAY");

    let runs: Value = client
        .get(format!(
            "{}/api/kg/motoko-graph/monitoring-runs?limit=10",
            base
        ))
        .send()
        .await
        .expect("runs response")
        .json()
        .await
        .expect("runs json");
    assert_eq!(runs.as_array().map(|v| v.len()).unwrap_or(0), 1);

    for option in [
        "Hold Deferred",
        "Conditional Progression",
        "Request Additional Evidence",
    ] {
        let capture_status = client
            .post(format!("{}/api/kg/motoko-graph/decision-capture", base))
            .json(&json!({
              "schema_version": "1.0.0",
              "contribution": "078",
              "decision_date": "2026-02-08",
              "selected_option": option,
              "rationale": format!("fixture rationale: {}", option),
              "posture_before": "watch-first",
              "posture_after": "watch-first",
              "authority_mode": "recommendation_only",
              "evidence_refs": ["/tmp/m16_dual_path/gate.txt"],
              "steward": "Research Steward",
              "owner": "Nostra Architecture Team",
              "follow_up_actions": ["Continue monitoring"],
              "source": "integration-test"
            }))
            .send()
            .await
            .expect("capture response");
        assert_eq!(capture_status.status(), reqwest::StatusCode::OK);
    }

    let history: Value = client
        .get(format!("{}/api/kg/motoko-graph/decision-history", base))
        .send()
        .await
        .expect("history response")
        .json()
        .await
        .expect("history json");
    assert!(history.as_array().map(|v| v.len()).unwrap_or(0) >= 3);

    handle.abort();
    std::env::remove_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR");
}

#[tokio::test]
#[ignore = "requires loopback socket permissions in test environment"]
async fn motoko_graph_gateway_rejects_invalid_capture_payload() {
    let temp = TempDir::new("motoko-graph-gateway-invalid");
    write_snapshot_fixture(temp.path());
    std::env::set_var(
        "NOSTRA_MOTOKO_GRAPH_LOG_DIR",
        temp.path().display().to_string(),
    );

    let Some((port, handle)) = start_gateway().await else {
        std::env::remove_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR");
        return;
    };
    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{}", port);

    let body: Value = client
        .post(format!("{}/api/kg/motoko-graph/decision-capture", base))
        .json(&json!({
          "schema_version": "1.0.0",
          "contribution": "078",
          "decision_date": "2026-02-08",
          "selected_option": "Promote Now",
          "rationale": "invalid option",
          "posture_before": "watch-first",
          "posture_after": "conditional",
          "authority_mode": "recommendation_only",
          "evidence_refs": ["/tmp/m16_dual_path/gate.txt"],
          "steward": "Research Steward",
          "owner": "Nostra Architecture Team",
          "follow_up_actions": ["n/a"],
          "source": "integration-test"
        }))
        .send()
        .await
        .expect("invalid capture response")
        .json()
        .await
        .expect("invalid capture json");

    assert_eq!(body["errorCode"], "INVALID_DECISION_CAPTURE");

    handle.abort();
    std::env::remove_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR");
}
