use crate::services::acp_metrics::{record_fallback_flush, record_fallback_queued};
use crate::services::ic_client::IcClient;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Preconditions {
    #[serde(default)]
    pub base_version: Option<String>,
    #[serde(default)]
    pub target_exists: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mutation {
    pub id: String,
    #[serde(default)]
    pub idempotency_key: String,
    #[serde(default)]
    pub space_id: Option<String>,
    pub kip_command: String,
    pub timestamp: u64,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_attempt_at: Option<u64>,
    #[serde(default)]
    pub preconditions: Option<Preconditions>,
}

impl Mutation {
    pub fn new(kip_command: String) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = now_secs();
        Self {
            id: id.clone(),
            idempotency_key: id,
            space_id: None,
            kip_command,
            timestamp: now,
            attempts: 0,
            last_error: None,
            last_attempt_at: None,
            preconditions: None,
        }
    }

    fn normalize(&mut self) {
        if self.id.is_empty() {
            self.id = Uuid::new_v4().to_string();
        }
        if self.idempotency_key.is_empty() {
            self.idempotency_key = self.id.clone();
        }
        if self.timestamp == 0 {
            self.timestamp = now_secs();
        }
    }
}

#[derive(Clone)]
pub struct LocalGateway {
    queue: Arc<Mutex<Vec<Mutation>>>,
    is_online: Arc<Mutex<bool>>,
    storage_path: PathBuf,
}

// Global Singleton
static GATEWAY: LazyLock<LocalGateway> = LazyLock::new(LocalGateway::new);

pub fn get_gateway() -> &'static LocalGateway {
    &GATEWAY
}

enum ReplayResult {
    Success,
    Transient(String),
    Conflict(String),
    Rejected(String),
}

impl LocalGateway {
    pub fn new() -> Self {
        Self::with_storage_path(Self::default_storage_path())
    }

    pub fn with_storage_path(storage_path: PathBuf) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            is_online: Arc::new(Mutex::new(true)), // Assume online by default
            storage_path,
        }
    }

    pub fn init(&self) {
        if let Err(e) = self.load_queue() {
            tracing::warn!("LocalGateway load_queue failed: {}", e);
        }
        if self.is_network_online() {
            self.reconcile_async();
        }
    }

    pub fn set_online(&self, status: bool) {
        let mut online = self.is_online.lock().unwrap();

        // Only reconcile if transitioning from Offline -> Online
        if status && !*online {
            *online = true;
            drop(online);
            self.reconcile_async();
        } else {
            *online = status;
        }
    }

    pub fn is_network_online(&self) -> bool {
        *self.is_online.lock().unwrap()
    }

    // Returns: Ok(ProcessedResult) or Ok("Queued")
    pub fn submit_mutation(&self, mut mutation: Mutation) -> Result<String, String> {
        let online = *self.is_online.lock().unwrap();
        mutation.normalize();

        if online {
            Ok("Synced".to_string())
        } else {
            self.enqueue_mutation(mutation)
        }
    }

    pub fn queue_observability_payload(&self, payload: &serde_json::Value) -> Result<String, String> {
        let command = format!(
            "observability.emit {}",
            serde_json::to_string(payload).map_err(|e| e.to_string())?
        );
        self.enqueue_mutation(Mutation::new(command))
    }

    pub async fn flush_observability_events(&self, endpoint: &str) -> Result<usize, String> {
        let started = std::time::Instant::now();
        let mut pending = {
            let q = self.queue.lock().unwrap();
            q.clone()
        };

        if pending.is_empty() {
            return Ok(0);
        }

        let client = reqwest::Client::new();
        let mut emitted = 0usize;
        let mut transient_failure = false;
        let mut rebuilt_queue = Vec::new();
        let mut iter = pending.drain(..);

        while let Some(mut mutation) = iter.next() {
            mutation.normalize();

            let Some(payload) = parse_observability_payload(&mutation.kip_command) else {
                rebuilt_queue.push(mutation);
                continue;
            };

            match emit_observability_payload(&client, endpoint, &mutation.idempotency_key, &payload)
                .await
            {
                EmitResult::Success => {
                    emitted += 1;
                }
                EmitResult::Transient(err) => {
                    self.mark_failure(&mut mutation, now_secs(), &err);
                    rebuilt_queue.push(mutation);
                    rebuilt_queue.extend(iter);
                    transient_failure = true;
                    break;
                }
                EmitResult::Rejected(err) => {
                    tracing::warn!(
                        "dropping rejected observability event {}: {}",
                        mutation.id,
                        err
                    );
                }
            }
        }

        {
            let mut q = self.queue.lock().unwrap();
            *q = rebuilt_queue;
        }
        if let Err(err) = self.save_queue() {
            record_fallback_flush(false, elapsed_ms(started));
            return Err(err);
        }
        record_fallback_flush(!transient_failure, elapsed_ms(started));
        Ok(emitted)
    }

    pub fn retry_mutation(&self, id: &str) -> Result<(), String> {
        let target_id = id.trim();
        if target_id.is_empty() {
            return Err("mutation_id is required".to_string());
        }
        let mut updated = false;
        {
            let mut q = self.queue.lock().unwrap();
            if let Some(m) = q.iter_mut().find(|m| m.id == target_id) {
                m.last_error = None;
                m.last_attempt_at = None;
                updated = true;
            }
        }
        if !updated {
            return Err(format!("mutation {target_id} not found"));
        }
        let _ = self.save_queue();
        self.send_conflict_decision(target_id, "retry");
        if self.is_network_online() {
            self.reconcile_async();
        }
        Ok(())
    }

    pub fn discard_mutation(&self, id: &str) -> Result<(), String> {
        let target_id = id.trim();
        if target_id.is_empty() {
            return Err("mutation_id is required".to_string());
        }
        let removed = {
            let mut q = self.queue.lock().unwrap();
            let before = q.len();
            q.retain(|m| m.id != target_id);
            q.len() != before
        };
        if !removed {
            return Err(format!("mutation {target_id} not found"));
        }
        self.save_queue()?;
        self.send_conflict_decision(target_id, "discard");
        Ok(())
    }

    pub fn mark_fork_needed(&self, id: &str) -> Result<(), String> {
        let target_id = id.trim();
        if target_id.is_empty() {
            return Err("mutation_id is required".to_string());
        }
        let mut updated = false;
        {
            let mut q = self.queue.lock().unwrap();
            if let Some(m) = q.iter_mut().find(|m| m.id == target_id) {
                m.last_error = Some("Marked for fork".to_string());
                updated = true;
            }
        }
        if !updated {
            return Err(format!("mutation {target_id} not found"));
        }
        self.save_queue()?;
        self.send_conflict_decision(target_id, "fork");
        Ok(())
    }

    pub fn get_queue_size(&self) -> usize {
        let q = self.queue.lock().unwrap();
        q.len()
    }

    pub fn queue_snapshot(&self) -> Vec<Mutation> {
        let q = self.queue.lock().unwrap();
        q.clone()
    }

    pub fn export_queue_json(&self) -> Result<String, String> {
        if !self.storage_path.exists() {
            return Ok("[]".to_string());
        }
        fs::read_to_string(&self.storage_path).map_err(|e| e.to_string())
    }

    pub fn load_queue(&self) -> Result<(), String> {
        if !self.storage_path.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(&self.storage_path).map_err(|e| e.to_string())?;
        let mut items: Vec<Mutation> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        for item in items.iter_mut() {
            item.normalize();
        }
        let mut q = self.queue.lock().unwrap();
        *q = items;
        Ok(())
    }

    pub fn save_queue(&self) -> Result<(), String> {
        self.ensure_storage_dir()?;
        let q = self.queue.lock().unwrap();
        let data = serde_json::to_string_pretty(&*q).map_err(|e| e.to_string())?;
        fs::write(&self.storage_path, data).map_err(|e| e.to_string())
    }

    pub fn reconcile_async(&self) {
        let gateway = self.clone();
        tokio::spawn(async move {
            gateway.reconcile().await;
        });
    }

    async fn reconcile(&self) {
        let mut pending = {
            let q = self.queue.lock().unwrap();
            q.clone()
        };
        if pending.is_empty() {
            return;
        }

        println!("Reconciling {} mutations...", pending.len());

        let mut new_queue = Vec::new();
        let mut iter = pending.drain(..);
        while let Some(mut mutation) = iter.next() {
            mutation.normalize();
            let now = now_secs();

            if !self.can_retry(&mutation, now) {
                new_queue.push(mutation);
                new_queue.extend(iter);
                break;
            }

            match self.replay_mutation(&mutation).await {
                ReplayResult::Success => {}
                ReplayResult::Transient(err) => {
                    self.mark_failure(&mut mutation, now, &err);
                    new_queue.push(mutation);
                    new_queue.extend(iter);
                    break;
                }
                ReplayResult::Conflict(err) => {
                    self.mark_failure(&mut mutation, now, &err);
                    self.notify_conflict(&mutation, "Conflict", &err);
                    new_queue.push(mutation);
                    new_queue.extend(iter);
                    break;
                }
                ReplayResult::Rejected(err) => {
                    self.mark_failure(&mut mutation, now, &err);
                    self.notify_conflict(&mutation, "Rejected", &err);
                    new_queue.push(mutation);
                    new_queue.extend(iter);
                    break;
                }
            }
        }

        {
            let mut q = self.queue.lock().unwrap();
            *q = new_queue;
        }
        let _ = self.save_queue();
    }

    async fn replay_mutation(&self, mutation: &Mutation) -> ReplayResult {
        let client = IcClient::new(None);
        let arg = format!("(\"{}\")", mutation.kip_command.replace("\"", "\\\""));
        match client
            .call_canister("workflow-engine", "execute_script", Some(&arg))
            .await
        {
            Ok(_) => ReplayResult::Success,
            Err(e) => self.classify_error(&e.to_string()),
        }
    }

    fn classify_error(&self, err: &str) -> ReplayResult {
        let msg = err.to_lowercase();
        if msg.contains("timeout")
            || msg.contains("timed out")
            || msg.contains("not running")
            || msg.contains("offline")
            || msg.contains("connection")
        {
            ReplayResult::Transient(err.to_string())
        } else if msg.contains("permission")
            || msg.contains("unauthorized")
            || msg.contains("rejected")
            || msg.contains("invalid")
        {
            ReplayResult::Rejected(err.to_string())
        } else {
            ReplayResult::Conflict(err.to_string())
        }
    }

    fn mark_failure(&self, mutation: &mut Mutation, now: u64, err: &str) {
        mutation.attempts = mutation.attempts.saturating_add(1);
        mutation.last_attempt_at = Some(now);
        mutation.last_error = Some(err.to_string());
    }

    fn notify_conflict(&self, mutation: &Mutation, kind: &str, err: &str) {
        self.send_conflict_task(mutation, kind, err);
        tracing::warn!("Offline Conflict Detected: [{}] {}", kind, err);
    }

    fn can_retry(&self, mutation: &Mutation, now: u64) -> bool {
        if mutation.attempts == 0 {
            return true;
        }
        let delay = backoff_delay_secs(mutation.attempts);
        match mutation.last_attempt_at {
            None => true,
            Some(ts) => now.saturating_sub(ts) >= delay,
        }
    }

    fn ensure_storage_dir(&self) -> Result<(), String> {
        if let Some(dir) = self.storage_path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn enqueue_mutation(&self, mutation: Mutation) -> Result<String, String> {
        let mut q = self.queue.lock().unwrap();
        q.push(mutation);
        let size = q.len();
        drop(q);
        record_fallback_queued();
        self.save_queue()?;
        println!("Queued mutation locally. Queue size: {:?}", size);
        Ok("Queued".to_string())
    }

    fn default_storage_path() -> PathBuf {
        let base = home::home_dir().unwrap_or_else(std::env::temp_dir);
        base.join(".nostra")
            .join("cortex")
            .join("local_gateway_queue.json")
    }

    fn send_conflict_task(&self, mutation: &Mutation, kind: &str, err: &str) {
        let payload = self.conflict_payload(mutation, kind, err, None);
        tokio::spawn(async move {
            let client = IcClient::new(None);
            let template_arg = format!("(\"{}\")", "offline_conflict");
            let workflow_id = client
                .call_canister("workflow-engine", "start_workflow", Some(&template_arg))
                .await
                .ok();

            let payload_with_id = if let Some(id) = workflow_id {
                serde_json::json!({
                    "type": "offline_conflict",
                    "workflow_id": id,
                    "payload": payload
                })
                .to_string()
            } else {
                payload
            };

            let arg = format!("(\"{}\")", payload_with_id.replace("\"", "\\\""));
            let _ = client
                .call_canister("workflow-engine", "process_message", Some(&arg))
                .await;
        });
    }

    fn send_conflict_decision(&self, mutation_id: &str, decision: &str) {
        let payload = serde_json::json!({
            "type": "offline_conflict_decision",
            "mutation_id": mutation_id,
            "decision": decision,
            "timestamp": now_secs(),
            "source": "cortex-desktop"
        })
        .to_string();

        tokio::spawn(async move {
            let client = IcClient::new(None);
            let arg = format!("(\"{}\")", payload.replace("\"", "\\\""));
            let _ = client
                .call_canister("workflow-engine", "process_message", Some(&arg))
                .await;
        });
    }

    fn conflict_payload(
        &self,
        mutation: &Mutation,
        kind: &str,
        err: &str,
        workflow_id: Option<String>,
    ) -> String {
        serde_json::json!({
            "type": "offline_conflict",
            "kind": kind,
            "workflow_id": workflow_id,
            "mutation": {
                "id": mutation.id,
                "idempotency_key": mutation.idempotency_key,
                "space_id": mutation.space_id,
                "kip_command": mutation.kip_command,
                "timestamp": mutation.timestamp,
                "attempts": mutation.attempts,
                "last_error": mutation.last_error,
                "last_attempt_at": mutation.last_attempt_at,
            },
            "error": err,
            "source": "cortex-desktop"
        })
        .to_string()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn backoff_delay_secs(attempts: u32) -> u64 {
    let exp = attempts.min(6); // cap exponential growth
    5u64.saturating_mul(2u64.saturating_pow(exp))
}

fn elapsed_ms(started: std::time::Instant) -> u64 {
    started.elapsed().as_millis().min(u64::MAX as u128) as u64
}

enum EmitResult {
    Success,
    Transient(String),
    Rejected(String),
}

fn parse_observability_payload(kip_command: &str) -> Option<serde_json::Value> {
    let prefix = "observability.emit ";
    if !kip_command.starts_with(prefix) {
        return None;
    }
    serde_json::from_str::<serde_json::Value>(&kip_command[prefix.len()..]).ok()
}

async fn emit_observability_payload(
    client: &reqwest::Client,
    endpoint: &str,
    idempotency_key: &str,
    payload: &serde_json::Value,
) -> EmitResult {
    match client
        .post(endpoint)
        .header("X-Idempotency-Key", idempotency_key)
        .json(payload)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => EmitResult::Success,
        Ok(resp) => classify_emit_status(resp.status()),
        Err(err) => EmitResult::Transient(err.to_string()),
    }
}

fn classify_emit_status(status: StatusCode) -> EmitResult {
    if status.as_u16() == 429 || status.is_server_error() {
        EmitResult::Transient(format!("emit returned {}", status))
    } else {
        EmitResult::Rejected(format!("emit returned {}", status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_path(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join("cortex_desktop_tests");
        let _ = fs::create_dir_all(&base);
        base.join(format!("{}_{}.json", name, Uuid::new_v4()))
    }

    #[test]
    fn persistence_roundtrip() {
        let path = temp_path("roundtrip");
        let gateway = LocalGateway::with_storage_path(path.clone());
        gateway.set_online(false);
        let mut mutation = Mutation::new("test:command".to_string());
        mutation.idempotency_key.clear();
        let _ = gateway.submit_mutation(mutation);

        let gateway2 = LocalGateway::with_storage_path(path);
        gateway2.load_queue().unwrap();
        assert_eq!(gateway2.get_queue_size(), 1);

        let q = gateway2.queue.lock().unwrap();
        assert_eq!(q[0].idempotency_key, q[0].id);
    }

    #[test]
    fn load_defaults_for_old_schema() {
        let path = temp_path("defaults");
        let raw = r#"[{"id":"1","kip_command":"x","timestamp":1}]"#;
        fs::write(&path, raw).unwrap();

        let gateway = LocalGateway::with_storage_path(path);
        gateway.load_queue().unwrap();

        let q = gateway.queue.lock().unwrap();
        assert_eq!(q[0].idempotency_key, "1");
        assert_eq!(q[0].attempts, 0);
        assert!(q[0].last_error.is_none());
    }

    #[test]
    fn fifo_order_preserved() {
        let path = temp_path("fifo");
        let gateway = LocalGateway::with_storage_path(path.clone());
        gateway.set_online(false);

        let mut a1 = Mutation::new("a1".to_string());
        a1.space_id = Some("A".to_string());
        let mut b1 = Mutation::new("b1".to_string());
        b1.space_id = Some("B".to_string());
        let mut a2 = Mutation::new("a2".to_string());
        a2.space_id = Some("A".to_string());

        let _ = gateway.submit_mutation(a1);
        let _ = gateway.submit_mutation(b1);
        let _ = gateway.submit_mutation(a2);

        let gateway2 = LocalGateway::with_storage_path(path);
        gateway2.load_queue().unwrap();
        let q = gateway2.queue.lock().unwrap();
        assert_eq!(q[0].kip_command, "a1");
        assert_eq!(q[1].kip_command, "b1");
        assert_eq!(q[2].kip_command, "a2");
    }

    #[test]
    fn observability_payload_parser_accepts_prefixed_json() {
        let payload = serde_json::json!({"kind":"acp_observability_event","ok":true});
        let cmd = format!("observability.emit {}", payload);
        let parsed = parse_observability_payload(&cmd).unwrap();
        assert_eq!(parsed["ok"], true);
    }

    #[test]
    fn emit_status_classification_is_stable() {
        assert!(matches!(
            classify_emit_status(StatusCode::INTERNAL_SERVER_ERROR),
            EmitResult::Transient(_)
        ));
        assert!(matches!(
            classify_emit_status(StatusCode::BAD_REQUEST),
            EmitResult::Rejected(_)
        ));
    }

    #[test]
    fn queue_actions_reject_unknown_mutation_ids() {
        let path = temp_path("queue_actions");
        let gateway = LocalGateway::with_storage_path(path);
        assert!(gateway.retry_mutation("missing").is_err());
        assert!(gateway.discard_mutation("missing").is_err());
        assert!(gateway.mark_fork_needed("missing").is_err());
    }
}
