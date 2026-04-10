use crate::a2ui::{surface_from_v1, A2UIMeta, Component, ComponentProperties, Surface};
use futures::future::{select, Either};
use futures::pin_mut;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex, OnceLock};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Event, IdbDatabase, IdbFactory, IdbOpenDbRequest, IdbRequest, IdbTransactionMode};

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
        let id = uuid::Uuid::new_v4().to_string();
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
            self.id = uuid::Uuid::new_v4().to_string();
        }
        if self.idempotency_key.is_empty() {
            self.idempotency_key = self.id.clone();
        }
        if self.timestamp == 0 {
            self.timestamp = now_secs();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictEvent {
    pub id: String,
    pub kind: String,
    pub error: String,
    pub command: String,
    pub attempts: u32,
    pub timestamp: u64,
    pub workflow_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InboxItemRecord {
    #[serde(default)]
    dedup_key: String,
    surface: Surface,
    #[serde(default)]
    created_at: u64,
    #[serde(default)]
    updated_at: u64,
    #[serde(default)]
    expires_at: Option<u64>,
    #[serde(default)]
    snoozed_until: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InboxSnapshot {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    items: Vec<InboxItemRecord>,
}

#[derive(Clone)]
pub struct LocalGateway {
    queue: Arc<Mutex<Vec<Mutation>>>,
    is_online: Arc<Mutex<bool>>,
    conflicts: Arc<Mutex<Vec<ConflictEvent>>>,
    a2ui_inbox: Arc<Mutex<Vec<InboxItemRecord>>>,
}

// Global Singleton
static GATEWAY: OnceLock<LocalGateway> = OnceLock::new();

pub fn get_gateway() -> &'static LocalGateway {
    GATEWAY.get_or_init(LocalGateway::new)
}

enum ReplayResult {
    Success,
    Transient(String),
    Conflict(String),
    Rejected(String),
}

const INBOX_SCHEMA_VERSION: u32 = 2;
const MAX_INBOX_ITEMS: usize = 50;
const DECISION_ACK_TTL_SECS: u64 = 20 * 60;
const MIN_OVERRIDE_JUSTIFICATION_LEN: usize = 32;
const MIN_OVERRIDE_JUSTIFICATION_WORDS: usize = 6;

impl LocalGateway {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            is_online: Arc::new(Mutex::new(true)), // Assume online by default
            conflicts: Arc::new(Mutex::new(Vec::new())),
            a2ui_inbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn init(&self) {
        let gateway = self.clone();
        spawn_local(async move {
            if let Ok(items) = load_queue_from_idb().await {
                if !items.is_empty() {
                    let mut q = gateway.queue.lock().unwrap();
                    *q = items;
                }
            }
            if let Ok(items) = load_inbox_from_idb().await {
                if !items.is_empty() {
                    let mut inbox = gateway.a2ui_inbox.lock().unwrap();
                    *inbox = items;
                    let changed = Self::prune_expired_inbox_locked(&mut inbox, now_secs());
                    if changed {
                        let snapshot = inbox.clone();
                        spawn_local(async move {
                            let _ = save_inbox_to_idb(&snapshot).await;
                        });
                    }
                }
            }
            if gateway.is_network_online() {
                gateway.reconcile_async();
            }
        });
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
            // Online: Return "Synced" to indicate immediate success (mock)
            Ok("Synced".to_string())
        } else {
            // Offline: Queue locally
            let mut q = self.queue.lock().unwrap();
            q.push(mutation);
            let to_save = q.clone();
            spawn_local(async move {
                let _ = save_queue_to_idb(&to_save).await;
            });
            web_sys::console::log_1(
                &format!("Queued mutation locally. Queue size: {:?}", q.len()).into(),
            );
            Ok("Queued".to_string())
        }
    }

    pub fn retry_mutation(&self, id: &str) -> Result<(), String> {
        let mut q = self.queue.lock().unwrap();
        if let Some(m) = q.iter_mut().find(|m| m.id == id) {
            m.last_error = None;
            m.last_attempt_at = None;
            let to_save = q.clone();
            spawn_local(async move {
                let _ = save_queue_to_idb(&to_save).await;
            });
        }
        drop(q);
        self.remove_conflict(id);
        self.send_decision("retry", id);
        if self.is_network_online() {
            self.reconcile_async();
        }
        Ok(())
    }

    pub fn discard_mutation(&self, id: &str) -> Result<(), String> {
        let mut q = self.queue.lock().unwrap();
        q.retain(|m| m.id != id);
        let to_save = q.clone();
        spawn_local(async move {
            let _ = save_queue_to_idb(&to_save).await;
        });
        drop(q);
        self.remove_conflict(id);
        self.send_decision("discard", id);
        Ok(())
    }

    pub fn mark_fork_needed(&self, id: &str) -> Result<(), String> {
        let mut q = self.queue.lock().unwrap();
        if let Some(m) = q.iter_mut().find(|m| m.id == id) {
            m.last_error = Some("Marked for fork".to_string());
        }
        let to_save = q.clone();
        spawn_local(async move {
            let _ = save_queue_to_idb(&to_save).await;
        });
        drop(q);
        self.remove_conflict(id);
        self.send_decision("fork", id);
        Ok(())
    }

    pub fn request_epistemic_gate(
        &self,
        workflow_id: &str,
        mutation_id: &str,
        decision_class: Option<&str>,
    ) -> Result<(), String> {
        let workflow_id = workflow_id.trim();
        let mutation_id = mutation_id.trim();
        if workflow_id.is_empty() || mutation_id.is_empty() {
            return Err("workflow_id and mutation_id are required".to_string());
        }

        let decision_class_value = decision_class
            .map(|value| value.to_string())
            .unwrap_or_else(|| "governance".to_string());
        let mut pending_surface = build_blackwell_gate_fallback_surface(
            workflow_id,
            mutation_id,
            &decision_class_value,
            Some("Awaiting workflow-engine assessment."),
        );
        pending_surface.id = Some(format!("blackwell_pending:{}", mutation_id));
        self.push_a2ui_inbox(pending_surface);

        let payload = serde_json::json!({
            "type": "epistemic_gate_request",
            "workflow_id": workflow_id,
            "mutation_id": mutation_id,
            "decision_class": decision_class_value,
            "source": "nostra-web"
        })
        .to_string();
        let workflow_id = workflow_id.to_string();
        let mutation_id = mutation_id.to_string();
        let decision_class = decision_class_value.to_string();
        let gateway = self.clone();
        spawn_local(async move {
            match send_workflow_message_with_timeout(&payload, 1_200).await {
                Ok(response) => {
                    let captured = gateway.capture_a2ui_response(&response);
                    if !captured {
                        let surface = build_blackwell_gate_fallback_surface(
                            &workflow_id,
                            &mutation_id,
                            &decision_class,
                            Some("Workflow engine returned non-surface payload."),
                        );
                        gateway.push_a2ui_inbox(surface);
                    }
                }
                Err(err) => {
                    let surface = build_blackwell_gate_fallback_surface(
                        &workflow_id,
                        &mutation_id,
                        &decision_class,
                        Some(&format!("Workflow engine unavailable: {}", err)),
                    );
                    gateway.push_a2ui_inbox(surface);
                }
            }
        });
        Ok(())
    }

    pub fn ack_epistemic_override(
        &self,
        workflow_id: &str,
        mutation_id: &str,
        justification: &str,
    ) -> Result<(), String> {
        let workflow_id = workflow_id.trim();
        let mutation_id = mutation_id.trim();
        let justification = justification.trim();
        if workflow_id.is_empty() || mutation_id.is_empty() {
            return Err("workflow_id and mutation_id are required".to_string());
        }
        validate_override_justification(justification)?;

        let assessment_id = blackwell_assessment_id(workflow_id, mutation_id);
        let payload = serde_json::json!({
            "type": "blackwell_override_ack",
            "assessment_id": assessment_id,
            "workflow_id": workflow_id,
            "mutation_id": mutation_id,
            "justification": justification,
            "source": "nostra-web"
        })
        .to_string();
        let workflow_id = workflow_id.to_string();
        let mutation_id = mutation_id.to_string();
        let justification = justification.to_string();
        let gateway = self.clone();
        spawn_local(async move {
            match send_workflow_message_with_timeout(&payload, 1_200).await {
                Ok(response) => {
                    let captured = gateway.capture_a2ui_response(&response);
                    if !captured {
                        let surface = build_blackwell_ack_fallback_surface(
                            &workflow_id,
                            &mutation_id,
                            &justification,
                        );
                        gateway.push_a2ui_inbox(surface);
                    }
                }
                Err(_err) => {
                    let surface = build_blackwell_ack_fallback_surface(
                        &workflow_id,
                        &mutation_id,
                        &justification,
                    );
                    gateway.push_a2ui_inbox(surface);
                }
            }
        });
        Ok(())
    }

    pub fn get_queue_size(&self) -> usize {
        let q = self.queue.lock().unwrap();
        q.len()
    }

    pub fn get_conflicts(&self) -> Vec<ConflictEvent> {
        let c = self.conflicts.lock().unwrap();
        c.clone()
    }

    pub fn get_a2ui_inbox(&self) -> Vec<Surface> {
        let mut inbox = self.a2ui_inbox.lock().unwrap();
        let changed = Self::prune_expired_inbox_locked(&mut inbox, now_secs());
        let now = now_secs();
        let surfaces = inbox
            .iter()
            .filter(|item| !Self::is_snoozed(item, now))
            .map(|item| item.surface.clone())
            .collect();
        if changed {
            let snapshot = inbox.clone();
            spawn_local(async move {
                let _ = save_inbox_to_idb(&snapshot).await;
            });
        }
        surfaces
    }

    pub fn upsert_surface(&self, surface: Surface) {
        self.push_a2ui_inbox(surface);
    }

    pub fn snooze_inbox_item(&self, surface_id: &str, seconds: u64) -> Result<(), String> {
        let wake_at = now_secs().saturating_add(seconds);
        let mut inbox = self.a2ui_inbox.lock().unwrap();
        let mut updated = false;
        for item in inbox.iter_mut() {
            if item.surface.id.as_deref() == Some(surface_id) {
                item.snoozed_until = Some(wake_at);
                item.updated_at = now_secs();
                updated = true;
            }
        }
        if updated {
            let snapshot = inbox.clone();
            drop(inbox);
            spawn_local(async move {
                let _ = save_inbox_to_idb(&snapshot).await;
            });
            Ok(())
        } else {
            Err("surface not found".to_string())
        }
    }

    fn remove_conflict(&self, id: &str) {
        let mut c = self.conflicts.lock().unwrap();
        c.retain(|e| e.id != id);
    }

    fn reconcile_async(&self) {
        let gateway = self.clone();
        spawn_local(async move {
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

        web_sys::console::log_1(&format!("Reconciling {} mutations...", pending.len()).into());

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
                    self.record_conflict(&mutation, "Conflict", &err, None);
                    new_queue.push(mutation);
                    new_queue.extend(iter);
                    break;
                }
                ReplayResult::Rejected(err) => {
                    self.mark_failure(&mut mutation, now, &err);
                    self.record_conflict(&mutation, "Rejected", &err, None);
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
        let to_save = {
            let q = self.queue.lock().unwrap();
            q.clone()
        };
        spawn_local(async move {
            let _ = save_queue_to_idb(&to_save).await;
        });
    }

    async fn replay_mutation(&self, mutation: &Mutation) -> ReplayResult {
        let payload = serde_json::json!({
            "type": "offline_replay",
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
            "source": "nostra-web"
        })
        .to_string();

        match send_workflow_message(&payload).await {
            Ok(_) => ReplayResult::Success,
            Err(e) => self.classify_error(&e),
        }
    }

    fn classify_error(&self, err: &str) -> ReplayResult {
        let msg = err.to_lowercase();
        if msg.contains("timeout")
            || msg.contains("timed out")
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

    fn record_conflict(&self, mutation: &Mutation, kind: &str, err: &str, workflow_id: Option<String>) {
        let mut c = self.conflicts.lock().unwrap();
        c.push(ConflictEvent {
            id: mutation.id.clone(),
            kind: kind.to_string(),
            error: err.to_string(),
            command: mutation.kip_command.clone(),
            attempts: mutation.attempts,
            timestamp: now_secs(),
            workflow_id,
        });
        self.send_conflict_task(mutation, kind, err);
    }

    fn mark_failure(&self, mutation: &mut Mutation, now: u64, err: &str) {
        mutation.attempts = mutation.attempts.saturating_add(1);
        mutation.last_attempt_at = Some(now);
        mutation.last_error = Some(err.to_string());
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

    fn send_conflict_task(&self, mutation: &Mutation, kind: &str, err: &str) {
        let payload = serde_json::json!({
            "type": "offline_conflict",
            "kind": kind,
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
            "source": "nostra-web"
        })
        .to_string();
        let gateway = self.clone();
        spawn_local(async move {
            if let Ok(response) = send_conflict_task_to_engine(&payload).await {
                gateway.capture_a2ui_response(&response);
            }
        });
    }

    fn send_decision(&self, decision: &str, mutation_id: &str) {
        let payload = serde_json::json!({
            "type": "offline_conflict_decision",
            "decision": decision,
            "mutation_id": mutation_id,
            "timestamp": now_secs(),
            "source": "nostra-web"
        })
        .to_string();
        let gateway = self.clone();
        spawn_local(async move {
            if let Ok(response) = send_workflow_message(&payload).await {
                gateway.capture_a2ui_response(&response);
            }
        });
    }

    fn capture_a2ui_response(&self, response: &str) -> bool {
        if let Some(surface) = surface_from_v1(response) {
            self.push_a2ui_inbox(surface);
            return true;
        }
        false
    }

    fn push_a2ui_inbox(&self, surface: Surface) {
        let now = now_secs();
        let dedup_key = Self::dedup_key_for_surface(&surface, now);
        let is_decision_ack = surface
            .id
            .as_deref()
            .map(|id| {
                id.starts_with("offline_conflict_decision_")
                    || id.starts_with("blackwell_override_ack:")
            })
            .unwrap_or(false);
        let workflow_id = Self::workflow_id_for_surface(&surface);
        let mutation_id = Self::mutation_id_for_surface(&surface);
        let expires_at = if is_decision_ack {
            Some(now.saturating_add(DECISION_ACK_TTL_SECS))
        } else {
            None
        };

        let mut inbox = self.a2ui_inbox.lock().unwrap();
        Self::prune_expired_inbox_locked(&mut inbox, now);

        if is_decision_ack {
            inbox.retain(|item| {
                !Self::is_pending_resolution_match(item, workflow_id.as_deref(), mutation_id.as_deref())
            });
        }

        if let Some(existing) = inbox.iter_mut().find(|item| item.dedup_key == dedup_key) {
            existing.surface = surface;
            existing.updated_at = now;
            existing.expires_at = expires_at;
        } else {
            inbox.push(InboxItemRecord {
                dedup_key,
                surface,
                created_at: now,
                updated_at: now,
                expires_at,
                snoozed_until: None,
            });
        }

        if inbox.len() > MAX_INBOX_ITEMS {
            let overflow = inbox.len() - MAX_INBOX_ITEMS;
            inbox.drain(0..overflow);
        }

        let snapshot = inbox.clone();
        drop(inbox);
        spawn_local(async move {
            let _ = save_inbox_to_idb(&snapshot).await;
        });
    }

    fn prune_expired_inbox_locked(inbox: &mut Vec<InboxItemRecord>, now: u64) -> bool {
        let before = inbox.len();
        inbox.retain(|item| item.expires_at.map(|exp| exp > now).unwrap_or(true));
        inbox.len() != before
    }

    fn is_snoozed(item: &InboxItemRecord, now: u64) -> bool {
        item.snoozed_until
            .map(|wake_at| wake_at > now)
            .unwrap_or(false)
    }

    fn workflow_id_for_surface(surface: &Surface) -> Option<String> {
        if let Some(workflow_id) = surface
            .meta
            .as_ref()
            .or(surface.root.meta.as_ref())
            .and_then(|meta| meta.workflow_id.clone())
        {
            return Some(workflow_id);
        }
        if let Some(id) = surface.id.as_deref() {
            if let Some(rest) = id.strip_prefix("offline_conflict_decision_") {
                return Some(rest.to_string());
            }
            if let Some(rest) = id.strip_prefix("offline_conflict_") {
                return Some(rest.to_string());
            }
        }
        None
    }

    fn mutation_id_for_surface(surface: &Surface) -> Option<String> {
        if let Some(mutation_id) = surface
            .meta
            .as_ref()
            .or(surface.root.meta.as_ref())
            .and_then(|meta| meta.mutation_id.clone())
        {
            return Some(mutation_id);
        }
        Self::first_action_with_prefix(surface, &["conflict_", "blackwell_"])
            .and_then(|action| action.split_once(':').map(|(_, id)| id.to_string()))
    }

    fn first_action_with_prefix(surface: &Surface, prefixes: &[&str]) -> Option<String> {
        Self::find_first_action(&surface.root)
            .filter(|action| prefixes.iter().any(|prefix| action.starts_with(prefix)))
    }

    fn find_first_action(component: &Component) -> Option<String> {
        match &component.properties {
            ComponentProperties::Button { action, .. } => Some(action.clone()),
            ComponentProperties::Container { children }
            | ComponentProperties::Column { children }
            | ComponentProperties::Row { children } => {
                children.iter().find_map(Self::find_first_action)
            }
            ComponentProperties::Card { child } => Self::find_first_action(child),
            ComponentProperties::Tabs { tab_items } => tab_items
                .iter()
                .find_map(|tab| Self::find_first_action(&tab.child)),
            ComponentProperties::Modal {
                entry_point_child,
                content_child,
            } => Self::find_first_action(entry_point_child)
                .or_else(|| Self::find_first_action(content_child)),
            _ => None,
        }
    }

    fn is_pending_resolution_match(
        item: &InboxItemRecord,
        workflow_id: Option<&str>,
        mutation_id: Option<&str>,
    ) -> bool {
        if item
            .surface
            .id
            .as_deref()
            .map(|id| {
                id.starts_with("offline_conflict_decision_")
                    || id.starts_with("blackwell_override_ack:")
            })
            .unwrap_or(false)
        {
            return false;
        }
        if let Some(workflow_id) = workflow_id {
            if item
                .surface
                .id
                .as_deref()
                .map(|id| {
                    id == format!("offline_conflict_{}", workflow_id)
                        || id.starts_with("blackwell_")
                })
                .unwrap_or(false)
            {
                return true;
            }
            if Self::workflow_id_for_surface(&item.surface).as_deref() == Some(workflow_id) {
                return true;
            }
        }
        if let Some(mutation_id) = mutation_id {
            return Self::surface_contains_mutation_action(&item.surface, mutation_id);
        }
        false
    }

    fn surface_contains_mutation_action(surface: &Surface, mutation_id: &str) -> bool {
        Self::collect_actions(&surface.root)
            .iter()
            .any(|action| {
                (action.starts_with("conflict_") || action.starts_with("blackwell_"))
                    && action.ends_with(mutation_id)
            })
    }

    fn collect_actions(component: &Component) -> Vec<String> {
        match &component.properties {
            ComponentProperties::Button { action, .. } => vec![action.clone()],
            ComponentProperties::Container { children }
            | ComponentProperties::Column { children }
            | ComponentProperties::Row { children } => children
                .iter()
                .flat_map(Self::collect_actions)
                .collect(),
            ComponentProperties::Card { child } => Self::collect_actions(child),
            ComponentProperties::Tabs { tab_items } => tab_items
                .iter()
                .flat_map(|tab| Self::collect_actions(&tab.child))
                .collect(),
            ComponentProperties::Modal {
                entry_point_child,
                content_child,
            } => {
                let mut actions = Self::collect_actions(entry_point_child);
                actions.extend(Self::collect_actions(content_child));
                actions
            }
            _ => Vec::new(),
        }
    }

    fn dedup_key_for_surface(surface: &Surface, now: u64) -> String {
        if let Some(surface_id) = surface.id.as_deref() {
            return format!("surface:{}", surface_id);
        }
        if let Some(action) = Self::find_first_action(&surface.root) {
            let normalized = action.trim().to_ascii_lowercase();
            let digest = Sha256::digest(normalized.as_bytes());
            return format!("action:{}", hex::encode(digest));
        }
        format!("anon:{}:{}", now, uuid::Uuid::new_v4())
    }
}

fn blackwell_assessment_id(workflow_id: &str, mutation_id: &str) -> String {
    let material = format!("{}::{}", workflow_id, mutation_id);
    let digest = Sha256::digest(material.as_bytes());
    format!("epi_{}", hex::encode(digest))
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn validate_override_justification(justification: &str) -> Result<(), String> {
    if justification.is_empty() {
        return Err("override justification is required".to_string());
    }
    if justification.len() < MIN_OVERRIDE_JUSTIFICATION_LEN {
        return Err(format!(
            "override justification must be at least {} characters",
            MIN_OVERRIDE_JUSTIFICATION_LEN
        ));
    }
    let word_count = justification.split_whitespace().count();
    if word_count < MIN_OVERRIDE_JUSTIFICATION_WORDS {
        return Err(format!(
            "override justification must include at least {} words",
            MIN_OVERRIDE_JUSTIFICATION_WORDS
        ));
    }

    let normalized = justification.to_ascii_lowercase();
    let has_risk = contains_any(
        &normalized,
        &["risk", "impact", "blast radius", "safety", "uncertain"],
    );
    let has_rollback = contains_any(
        &normalized,
        &["rollback", "revert", "fallback", "undo", "backout"],
    );
    let has_evidence = contains_any(
        &normalized,
        &["evidence", "metric", "log", "test", "runbook", "ticket", "incident"],
    );

    if !has_risk {
        return Err(
            "override justification must explain risk/impact (include terms like risk, impact, or blast radius)"
                .to_string(),
        );
    }
    if !has_rollback {
        return Err(
            "override justification must include rollback/backout plan (include rollback, revert, or fallback)"
                .to_string(),
        );
    }
    if !has_evidence {
        return Err(
            "override justification must reference supporting evidence (include evidence, test, log, metric, or runbook)"
                .to_string(),
        );
    }
    Ok(())
}

fn build_blackwell_meta(workflow_id: &str, mutation_id: &str, source: &str) -> A2UIMeta {
    A2UIMeta {
        tone: Some("warn".to_string()),
        context: Some("ops".to_string()),
        priority: Some("p1".to_string()),
        intent: Some("warning".to_string()),
        gate_level: Some("release_blocker".to_string()),
        gate_status: Some("pending".to_string()),
        decision_gate_id: Some(format!("blackwell_gate:{}", mutation_id)),
        replay_contract_ref: Some(format!("system_replay_contract:{}", mutation_id)),
        workflow_id: Some(workflow_id.to_string()),
        mutation_id: Some(mutation_id.to_string()),
        source: Some(source.to_string()),
        timestamp: Some(now_secs()),
        ..A2UIMeta::default()
    }
}

fn text_component(id: &str, text: String, meta: Option<A2UIMeta>) -> Component {
    Component {
        id: id.to_string(),
        properties: ComponentProperties::Text { text },
        a11y: None,
        meta,
    }
}

fn heading_component(id: &str, text: String, meta: Option<A2UIMeta>) -> Component {
    Component {
        id: id.to_string(),
        properties: ComponentProperties::Heading { text },
        a11y: None,
        meta,
    }
}

fn button_component(id: &str, label: &str, action: String, meta: Option<A2UIMeta>) -> Component {
    Component {
        id: id.to_string(),
        properties: ComponentProperties::Button {
            label: label.to_string(),
            action,
        },
        a11y: None,
        meta,
    }
}

fn build_blackwell_gate_fallback_surface(
    workflow_id: &str,
    mutation_id: &str,
    decision_class: &str,
    note: Option<&str>,
) -> Surface {
    let assessment_id = blackwell_assessment_id(workflow_id, mutation_id);
    let meta = build_blackwell_meta(workflow_id, mutation_id, "local_gateway_fallback");
    let mut children = vec![
        heading_component(
            "blackwell_heading",
            "Blackwell Guardrail Review".to_string(),
            Some(meta.clone()),
        ),
        text_component(
            "blackwell_summary",
            format!("Decision Class: {}", decision_class),
            Some(meta.clone()),
        ),
        text_component(
            "blackwell_workflow",
            format!("Workflow: {}", workflow_id),
            Some(meta.clone()),
        ),
        text_component(
            "blackwell_mutation",
            format!("Mutation: {}", mutation_id),
            Some(meta.clone()),
        ),
        text_component(
            "blackwell_assessment",
            format!("Assessment: {}", assessment_id),
            Some(meta.clone()),
        ),
    ];
    if let Some(note) = note {
        children.push(text_component(
            "blackwell_note",
            note.to_string(),
            Some(meta.clone()),
        ));
    }
    children.push(Component {
        id: "blackwell_actions".to_string(),
        properties: ComponentProperties::Row {
            children: vec![
                button_component(
                    "blackwell_btn_evidence",
                    "Add Evidence",
                    format!("blackwell_add_evidence:{}", mutation_id),
                    Some(meta.clone()),
                ),
                button_component(
                    "blackwell_btn_simulate",
                    "Run Simulation",
                    format!("blackwell_run_simulation:{}", mutation_id),
                    Some(meta.clone()),
                ),
                button_component(
                    "blackwell_btn_review",
                    "Request Review",
                    format!("blackwell_request_review:{}", mutation_id),
                    Some(meta.clone()),
                ),
                button_component(
                    "blackwell_btn_override",
                    "Override",
                    format!("blackwell_override:{}", mutation_id),
                    Some(meta.clone()),
                ),
            ],
        },
        a11y: None,
        meta: Some(meta.clone()),
    });

    Surface {
        id: Some(format!("blackwell_warning:{}", mutation_id)),
        root: Component {
            id: "blackwell_gate_surface".to_string(),
            properties: ComponentProperties::Column { children },
            a11y: None,
            meta: Some(meta.clone()),
        },
        meta: Some(meta),
    }
}

fn build_blackwell_ack_fallback_surface(
    workflow_id: &str,
    mutation_id: &str,
    justification: &str,
) -> Surface {
    let assessment_id = blackwell_assessment_id(workflow_id, mutation_id);
    let mut meta = build_blackwell_meta(workflow_id, mutation_id, "local_gateway_fallback");
    meta.tone = Some("info".to_string());
    meta.intent = Some("success".to_string());

    Surface {
        id: Some(format!("blackwell_override_ack:{}", mutation_id)),
        root: Component {
            id: "blackwell_ack_surface".to_string(),
            properties: ComponentProperties::Column {
                children: vec![
                    heading_component(
                        "blackwell_ack_heading",
                        "Override Recorded".to_string(),
                        Some(meta.clone()),
                    ),
                    text_component(
                        "blackwell_ack_assessment",
                        format!("Override for assessment {}", assessment_id),
                        Some(meta.clone()),
                    ),
                    text_component(
                        "blackwell_ack_justification",
                        format!("Justification: {}", justification),
                        Some(meta.clone()),
                    ),
                ],
            },
            a11y: None,
            meta: Some(meta.clone()),
        },
        meta: Some(meta),
    }
}

fn now_secs() -> u64 {
    js_sys::Date::now() as u64 / 1000
}

fn backoff_delay_secs(attempts: u32) -> u64 {
    let exp = attempts.min(6);
    5u64.saturating_mul(2u64.saturating_pow(exp))
}

fn idb_request_promise(req: IdbRequest) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, reject| {
        let success_req = req.clone();
        let success_cb = Closure::once(move |_event: Event| {
            let result = success_req.result().unwrap_or(JsValue::UNDEFINED);
            let _ = resolve.call1(&JsValue::NULL, &result);
        });
        let error_cb = Closure::once(move |_event: Event| {
            let err = JsValue::from_str("indexeddb request failed");
            let _ = reject.call1(&JsValue::NULL, &err);
        });
        req.set_onsuccess(Some(success_cb.as_ref().unchecked_ref()));
        req.set_onerror(Some(error_cb.as_ref().unchecked_ref()));
        success_cb.forget();
        error_cb.forget();
    })
}

async fn idb_request_await(req: IdbRequest) -> Result<JsValue, JsValue> {
    let promise = idb_request_promise(req);
    JsFuture::from(promise).await
}

async fn load_queue_from_idb() -> Result<Vec<Mutation>, String> {
    let db = open_db().await?;
    let tx = db
        .transaction_with_str_and_mode("local_gateway", IdbTransactionMode::Readonly)
        .map_err(|e| format!("{:?}", e))?;
    let store = tx
        .object_store("local_gateway")
        .map_err(|e| format!("{:?}", e))?;
    let req = store.get(&JsValue::from_str("queue")).map_err(|e| format!("{:?}", e))?;
    let value = idb_request_await(req).await.map_err(|e| format!("{:?}", e))?;
    if value.is_undefined() || value.is_null() {
        return Ok(vec![]);
    }
    let raw = value.as_string().ok_or("queue value not string")?;
    let mut items: Vec<Mutation> = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    for item in items.iter_mut() {
        item.normalize();
    }
    Ok(items)
}

async fn save_queue_to_idb(queue: &[Mutation]) -> Result<(), String> {
    let db = open_db().await?;
    let tx = db
        .transaction_with_str_and_mode("local_gateway", IdbTransactionMode::Readwrite)
        .map_err(|e| format!("{:?}", e))?;
    let store = tx
        .object_store("local_gateway")
        .map_err(|e| format!("{:?}", e))?;
    let data = serde_json::to_string(queue).map_err(|e| e.to_string())?;
    store
        .put_with_key(&JsValue::from_str(&data), &JsValue::from_str("queue"))
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

async fn load_inbox_from_idb() -> Result<Vec<InboxItemRecord>, String> {
    let db = open_db().await?;
    let tx = db
        .transaction_with_str_and_mode("local_gateway", IdbTransactionMode::Readonly)
        .map_err(|e| format!("{:?}", e))?;
    let store = tx
        .object_store("local_gateway")
        .map_err(|e| format!("{:?}", e))?;
    let req = store
        .get(&JsValue::from_str("inbox_v1"))
        .map_err(|e| format!("{:?}", e))?;
    let value = idb_request_await(req).await.map_err(|e| format!("{:?}", e))?;
    if value.is_undefined() || value.is_null() {
        return Ok(vec![]);
    }
    let raw = value.as_string().ok_or("inbox value not string")?;
    let snapshot: InboxSnapshot = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if snapshot.version > INBOX_SCHEMA_VERSION {
        return Ok(vec![]);
    }
    Ok(snapshot.items)
}

async fn save_inbox_to_idb(items: &[InboxItemRecord]) -> Result<(), String> {
    let db = open_db().await?;
    let tx = db
        .transaction_with_str_and_mode("local_gateway", IdbTransactionMode::Readwrite)
        .map_err(|e| format!("{:?}", e))?;
    let store = tx
        .object_store("local_gateway")
        .map_err(|e| format!("{:?}", e))?;
    let payload = InboxSnapshot {
        version: INBOX_SCHEMA_VERSION,
        items: items.to_vec(),
    };
    let data = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
    store
        .put_with_key(&JsValue::from_str(&data), &JsValue::from_str("inbox_v1"))
        .map_err(|e| format!("{:?}", e))?;
    Ok(())
}

async fn open_db() -> Result<IdbDatabase, String> {
    let window = web_sys::window().ok_or("no window")?;
    let idb: IdbFactory = window
        .indexed_db()
        .map_err(|e| format!("{:?}", e))?
        .ok_or("indexeddb not available")?;
    let request: IdbOpenDbRequest = idb
        .open_with_u32("nostra_local_gateway", 1)
        .map_err(|e| format!("{:?}", e))?;

    let on_upgrade = Closure::wrap(Box::new(move |event: web_sys::IdbVersionChangeEvent| {
        let target = event
            .target()
            .and_then(|t| t.dyn_into::<IdbOpenDbRequest>().ok());
        if let Some(req) = target {
            if let Ok(db_value) = req.result() {
                if let Ok(db) = db_value.dyn_into::<IdbDatabase>() {
                    if !db.object_store_names().contains("local_gateway") {
                        let _ = db.create_object_store("local_gateway");
                    }
                } else {
                    web_sys::console::warn_1(
                        &"[LocalGateway] onupgradeneeded result was not an IdbDatabase".into(),
                    );
                }
            }
        }
    }) as Box<dyn FnMut(web_sys::IdbVersionChangeEvent)>);
    request.set_onupgradeneeded(Some(on_upgrade.as_ref().unchecked_ref()));
    on_upgrade.forget();

    let req: IdbRequest = request
        .dyn_into()
        .map_err(|_| "failed to cast idb request")?;
    let db_value = idb_request_await(req).await.map_err(|e| format!("{:?}", e))?;
    let db: IdbDatabase = db_value.dyn_into().map_err(|_| "failed to open db")?;
    Ok(db)
}

async fn send_workflow_message(payload: &str) -> Result<String, String> {
    let agent = crate::api::create_agent().await;
    crate::api::workflow_process_message(&agent, payload.to_string()).await
}

async fn send_workflow_message_with_timeout(payload: &str, timeout_ms: u32) -> Result<String, String> {
    let send_fut = send_workflow_message(payload);
    let timeout_fut = gloo_timers::future::TimeoutFuture::new(timeout_ms);
    pin_mut!(send_fut);
    pin_mut!(timeout_fut);
    match select(send_fut, timeout_fut).await {
        Either::Left((result, _)) => result,
        Either::Right((_elapsed, _pending)) => {
            Err(format!("workflow message timed out after {}ms", timeout_ms))
        }
    }
}

async fn send_conflict_task_to_engine(payload: &str) -> Result<String, String> {
    let agent = crate::api::create_agent().await;
    let workflow_id = crate::api::workflow_start_workflow(&agent, "offline_conflict".to_string())
        .await
        .ok();
    let with_id = if let Some(id) = workflow_id {
        serde_json::json!({
            "type": "offline_conflict",
            "workflow_id": id,
            "payload": payload
        })
        .to_string()
    } else {
        payload.to_string()
    };
    crate::api::workflow_process_message(&agent, with_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button_surface(id: Option<&str>, action: &str) -> Surface {
        Surface {
            id: id.map(|v| v.to_string()),
            root: Component {
                id: "action".to_string(),
                properties: ComponentProperties::Button {
                    label: "Action".to_string(),
                    action: action.to_string(),
                },
                a11y: None,
                meta: None,
            },
            meta: None,
        }
    }

    #[test]
    fn dedup_key_prefers_surface_id() {
        let surface = button_surface(Some("offline_conflict_abc"), "conflict_retry:m1");
        let dedup = LocalGateway::dedup_key_for_surface(&surface, 123);
        assert_eq!(dedup, "surface:offline_conflict_abc");
    }

    #[test]
    fn dedup_key_hashes_first_action_when_surface_missing() {
        let surface = button_surface(None, "conflict_retry:MyMutation");
        let dedup = LocalGateway::dedup_key_for_surface(&surface, 123);
        let digest = Sha256::digest("conflict_retry:mymutation".as_bytes());
        let expected = format!("action:{}", hex::encode(digest));
        assert_eq!(dedup, expected);
    }

    #[test]
    fn pending_conflict_match_detects_workflow_and_mutation() {
        let pending = InboxItemRecord {
            dedup_key: "surface:offline_conflict_wf1".to_string(),
            surface: button_surface(Some("offline_conflict_wf1"), "conflict_retry:m1"),
            created_at: 1,
            updated_at: 1,
            expires_at: None,
            snoozed_until: None,
        };
        assert!(LocalGateway::is_pending_resolution_match(
            &pending,
            Some("wf1"),
            Some("m1")
        ));

        let ack = InboxItemRecord {
            dedup_key: "surface:offline_conflict_decision_wf1".to_string(),
            surface: button_surface(Some("offline_conflict_decision_wf1"), "conflict_retry:m1"),
            created_at: 1,
            updated_at: 1,
            expires_at: Some(2),
            snoozed_until: None,
        };
        assert!(!LocalGateway::is_pending_resolution_match(
            &ack,
            Some("wf1"),
            Some("m1")
        ));
    }

    #[test]
    fn blackwell_ack_is_not_pending_item() {
        let ack = InboxItemRecord {
            dedup_key: "surface:blackwell_override_ack:m1".to_string(),
            surface: button_surface(Some("blackwell_override_ack:m1"), "blackwell_override:m1"),
            created_at: 1,
            updated_at: 1,
            expires_at: Some(10),
            snoozed_until: None,
        };
        assert!(!LocalGateway::is_pending_resolution_match(
            &ack,
            Some("wf1"),
            Some("m1")
        ));
    }

    #[test]
    fn build_blackwell_assessment_id_matches_engine_pattern() {
        let id = blackwell_assessment_id("wf-123", "mut-123");
        assert!(id.starts_with("epi_"));
        assert_eq!(id.len(), 68);
    }

    #[test]
    fn prune_expired_inbox_entries() {
        let mut inbox = vec![
            InboxItemRecord {
                dedup_key: "a".to_string(),
                surface: button_surface(Some("a"), "conflict_retry:m1"),
                created_at: 1,
                updated_at: 1,
                expires_at: Some(10),
                snoozed_until: None,
            },
            InboxItemRecord {
                dedup_key: "b".to_string(),
                surface: button_surface(Some("b"), "conflict_retry:m2"),
                created_at: 1,
                updated_at: 1,
                expires_at: Some(30),
                snoozed_until: None,
            },
            InboxItemRecord {
                dedup_key: "c".to_string(),
                surface: button_surface(Some("c"), "conflict_retry:m3"),
                created_at: 1,
                updated_at: 1,
                expires_at: None,
                snoozed_until: None,
            },
        ];
        let changed = LocalGateway::prune_expired_inbox_locked(&mut inbox, 20);
        assert!(changed);
        assert_eq!(inbox.len(), 2);
        assert!(inbox.iter().all(|item| item.dedup_key != "a"));
    }

    #[test]
    fn snoozed_records_are_hidden_until_wake_time() {
        let record = InboxItemRecord {
            dedup_key: "s".to_string(),
            surface: button_surface(Some("surface_s"), "conflict_retry:m1"),
            created_at: 1,
            updated_at: 1,
            expires_at: None,
            snoozed_until: Some(200),
        };
        assert!(LocalGateway::is_snoozed(&record, 100));
        assert!(!LocalGateway::is_snoozed(&record, 250));
    }

    #[test]
    fn override_justification_validation_enforces_quality_dimensions() {
        let empty = validate_override_justification("");
        assert!(empty.is_err());

        let missing_evidence = validate_override_justification(
            "Operator approved: bounded blast radius and rollback path exists.",
        );
        assert!(missing_evidence.is_err());

        let valid = validate_override_justification(
            "Operator approved with bounded blast radius risk, rollback path tested, and evidence from logs and test run local_ide_phase_next.",
        );
        assert!(valid.is_ok());
    }
}
