use chrono::{DateTime, Utc};
use cortex_domain::agent::contracts::{AGENT_EXECUTION_EVENT_TYPE, AgentExecutionRecord};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const EVENT_SCHEMA_VERSION: &str = "1.0.0";

pub fn normalize_event_type(value: &str) -> String {
    value.trim().to_string()
}

pub fn is_supported_event_type(value: &str) -> bool {
    normalize_event_type(value) == AGENT_EXECUTION_EVENT_TYPE
}

pub fn lifecycle_events_path(base_log_dir: &Path) -> PathBuf {
    base_log_dir
        .join("events")
        .join("agent_execution_lifecycle.jsonl")
}

fn fallback_events_path(base_log_dir: &Path) -> PathBuf {
    base_log_dir
        .join("events")
        .join("agent_execution_lifecycle_fallback.jsonl")
}

fn replay_sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn deterministic_event_id(record: &AgentExecutionRecord) -> String {
    replay_sha256(
        format!(
            "{}:{}:{:?}:{}:{}",
            record.execution_id, record.attempt_id, record.phase, record.status, record.timestamp
        )
        .as_str(),
    )
}

fn build_resource(record: &AgentExecutionRecord) -> String {
    format!(
        "nostra://workflow/{}/execution/{}",
        record.workflow_id, record.execution_id
    )
}

fn bool_env(name: &str) -> bool {
    std::env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn append_json_line<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let line = serde_json::to_string(value).map_err(|err| err.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| err.to_string())?;
    writeln!(file, "{line}").map_err(|err| err.to_string())
}

fn ensure_required_record_fields(record: &AgentExecutionRecord) -> Result<(), String> {
    let required = [
        ("schema_version", record.schema_version.as_str()),
        ("execution_id", record.execution_id.as_str()),
        ("attempt_id", record.attempt_id.as_str()),
        ("agent_id", record.agent_id.as_str()),
        ("workflow_id", record.workflow_id.as_str()),
        ("status", record.status.as_str()),
        ("input_snapshot_hash", record.input_snapshot_hash.as_str()),
        ("output_snapshot_hash", record.output_snapshot_hash.as_str()),
        ("timestamp", record.timestamp.as_str()),
    ];

    for (key, value) in required {
        if value.trim().is_empty() {
            return Err(format!("missing required execution record field '{key}'"));
        }
    }

    if !is_supported_event_type(AGENT_EXECUTION_EVENT_TYPE) {
        return Err("unsupported execution event type".to_string());
    }
    Ok(())
}

pub async fn emit_agent_execution_record(
    base_log_dir: &Path,
    record: &AgentExecutionRecord,
) -> Result<(), String> {
    ensure_required_record_fields(record)?;

    let event_id = deterministic_event_id(record);
    let resource = build_resource(record);
    let mut cloud_event = nostra_cloudevents::Event::new(
        "nostra://cortex-desktop/agent-harness",
        AGENT_EXECUTION_EVENT_TYPE.to_string(),
    )
    .with_id(event_id.clone())
    .with_subject(resource.clone())
    .with_data(record)
    .map_err(|err| err.to_string())?;
    let parsed_time = DateTime::parse_from_rfc3339(record.timestamp.as_str())
        .map_err(|err| format!("invalid execution timestamp '{}': {err}", record.timestamp))?
        .with_timezone(&Utc);
    cloud_event.time = Some(parsed_time);

    let cloud_event_json = serde_json::to_value(&cloud_event).map_err(|err| err.to_string())?;
    let line = json!({
        "schemaVersion": EVENT_SCHEMA_VERSION,
        "eventType": AGENT_EXECUTION_EVENT_TYPE,
        "emittedAt": Utc::now().to_rfc3339(),
        "idempotencyKey": event_id,
        "resource": resource,
        "cloudEvent": cloud_event_json,
        "record": record,
    });

    append_json_line(&lifecycle_events_path(base_log_dir), &line)?;

    let sink_url = optional_env("NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL");
    let sink_fail_closed = bool_env("NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED");
    if sink_fail_closed && sink_url.is_none() {
        let err = "agent execution sink fail-closed mode enabled but sink URL is not configured";
        append_json_line(
            &fallback_events_path(base_log_dir),
            &json!({
                "schemaVersion": EVENT_SCHEMA_VERSION,
                "eventType": AGENT_EXECUTION_EVENT_TYPE,
                "error": err,
                "record": record,
            }),
        )?;
        return Err(err.to_string());
    }
    if let Some(sink_url) = sink_url {
        let client = reqwest::Client::new();
        let send_result = client
            .post(sink_url)
            .header(
                "X-Idempotency-Key",
                line["idempotencyKey"].as_str().unwrap_or_default(),
            )
            .json(&cloud_event_json)
            .send()
            .await;

        match send_result {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => {
                let err = format!(
                    "agent execution sink returned non-success status {}",
                    response.status()
                );
                append_json_line(
                    &fallback_events_path(base_log_dir),
                    &json!({
                        "schemaVersion": EVENT_SCHEMA_VERSION,
                        "eventType": AGENT_EXECUTION_EVENT_TYPE,
                        "error": err,
                        "record": record,
                    }),
                )?;
                if sink_fail_closed {
                    return Err(err);
                }
            }
            Err(err) => {
                append_json_line(
                    &fallback_events_path(base_log_dir),
                    &json!({
                        "schemaVersion": EVENT_SCHEMA_VERSION,
                        "eventType": AGENT_EXECUTION_EVENT_TYPE,
                        "error": err.to_string(),
                        "record": record,
                    }),
                )?;
                if sink_fail_closed {
                    return Err(err.to_string());
                }
            }
        }
    }

    Ok(())
}

pub fn hash_json_value(value: &Option<Value>) -> String {
    let payload = value.as_ref().cloned().unwrap_or(Value::Null).to_string();
    replay_sha256(payload.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::agent::contracts::{AgentExecutionPhase, AuthorityLevel};
    use std::sync::{LazyLock, Mutex};

    fn testing_env_lock() -> &'static Mutex<()> {
        static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
        &LOCK
    }

    fn acquire_testing_env_lock() -> std::sync::MutexGuard<'static, ()> {
        testing_env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn record() -> AgentExecutionRecord {
        AgentExecutionRecord {
            schema_version: "1.0.0".to_string(),
            execution_id: "exec-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            agent_id: "agent-default".to_string(),
            workflow_id: "wf-1".to_string(),
            phase: AgentExecutionPhase::Queued,
            status: "queued".to_string(),
            authority_scope: AuthorityLevel::L1,
            input_snapshot_hash: "in".to_string(),
            output_snapshot_hash: "out".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
            space_id: Some("space-1".to_string()),
            model_fingerprint: None,
            tool_state_hash: None,
            confidence: None,
            promotion_level: None,
            started_at: None,
            ended_at: None,
            replay_contract_ref: None,
            lineage_id: None,
            evidence_refs: Vec::new(),
            benchmark: None,
        }
    }

    #[test]
    fn supports_execution_event_type() {
        assert!(is_supported_event_type("AgentExecutionLifecycle"));
    }

    #[tokio::test]
    async fn emits_lifecycle_jsonl() {
        let _lock = acquire_testing_env_lock();
        let _sink = EnvVarGuard::unset("NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL");
        let _fail_closed = EnvVarGuard::unset("NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED");
        let temp = std::env::temp_dir().join(format!(
            "agent-execution-events-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        emit_agent_execution_record(temp.as_path(), &record())
            .await
            .expect("emit");
        assert!(lifecycle_events_path(temp.as_path()).exists());
        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn sink_failure_is_best_effort_by_default() {
        let _lock = acquire_testing_env_lock();
        let _sink = EnvVarGuard::set(
            "NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL",
            "http://127.0.0.1:9/events",
        );
        let _fail_closed = EnvVarGuard::unset("NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED");
        let temp = std::env::temp_dir().join(format!(
            "agent-execution-events-best-effort-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        emit_agent_execution_record(temp.as_path(), &record())
            .await
            .expect("best-effort sink failure should not block emission");
        assert!(lifecycle_events_path(temp.as_path()).exists());
        assert!(fallback_events_path(temp.as_path()).exists());
        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn sink_failure_blocks_when_fail_closed_enabled() {
        let _lock = acquire_testing_env_lock();
        let _sink = EnvVarGuard::set(
            "NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL",
            "http://127.0.0.1:9/events",
        );
        let _fail_closed =
            EnvVarGuard::set("NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED", "true");
        let temp = std::env::temp_dir().join(format!(
            "agent-execution-events-fail-closed-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let result = emit_agent_execution_record(temp.as_path(), &record()).await;
        assert!(result.is_err());
        assert!(lifecycle_events_path(temp.as_path()).exists());
        assert!(fallback_events_path(temp.as_path()).exists());
        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn empty_sink_url_is_treated_as_disabled() {
        let _lock = acquire_testing_env_lock();
        let _sink = EnvVarGuard::set("NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL", "   ");
        let _fail_closed = EnvVarGuard::unset("NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED");
        let temp = std::env::temp_dir().join(format!(
            "agent-execution-events-empty-sink-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        emit_agent_execution_record(temp.as_path(), &record())
            .await
            .expect("blank sink url should be treated as disabled");
        assert!(lifecycle_events_path(temp.as_path()).exists());
        assert!(!fallback_events_path(temp.as_path()).exists());
        let _ = std::fs::remove_dir_all(temp);
    }

    #[tokio::test]
    async fn fail_closed_requires_configured_sink_url() {
        let _lock = acquire_testing_env_lock();
        let _sink = EnvVarGuard::unset("NOSTRA_AGENT_EXECUTION_EVENT_SINK_URL");
        let _fail_closed =
            EnvVarGuard::set("NOSTRA_AGENT_EXECUTION_EVENT_SINK_FAIL_CLOSED", "true");
        let temp = std::env::temp_dir().join(format!(
            "agent-execution-events-fail-closed-missing-url-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let result = emit_agent_execution_record(temp.as_path(), &record()).await;
        assert!(result.is_err());
        assert!(lifecycle_events_path(temp.as_path()).exists());
        assert!(fallback_events_path(temp.as_path()).exists());
        let _ = std::fs::remove_dir_all(temp);
    }
}
