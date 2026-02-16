use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_STORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredSessionUpdate {
    pub session_update: String,
    pub turn_seq: u64,
    pub update_seq: u64,
    pub timestamp: u64,
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionRecord {
    pub session_id: String,
    pub cwd: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub next_turn_seq: u64,
    pub next_update_seq: u64,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(default)]
    pub config_options: HashMap<String, String>,
    #[serde(default)]
    pub updates: Vec<StoredSessionUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionSnapshot {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    sessions: Vec<SessionRecord>,
}

#[derive(Debug, Clone)]
pub struct AcpSessionStore {
    path: PathBuf,
    sessions: HashMap<String, SessionRecord>,
}

impl AcpSessionStore {
    pub fn load_default() -> Result<Self, String> {
        Self::load(Self::default_path())
    }

    pub fn load(path: PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self {
                path,
                sessions: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let snapshot: SessionSnapshot =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let sessions = snapshot
            .sessions
            .into_iter()
            .map(|s| (s.session_id.clone(), s))
            .collect();

        Ok(Self { path, sessions })
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let snapshot = SessionSnapshot {
            version: SESSION_STORE_VERSION,
            sessions: self.sessions.values().cloned().collect(),
        };

        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
        fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    pub fn create_session(&mut self, session_id: String, cwd: String) -> SessionRecord {
        let now = now_secs();
        let record = SessionRecord {
            session_id: session_id.clone(),
            cwd,
            created_at: now,
            updated_at: now,
            next_turn_seq: 1,
            next_update_seq: 1,
            cancelled: false,
            config_options: HashMap::from([
                ("mode".to_string(), "ask".to_string()),
                ("model".to_string(), "model-1".to_string()),
            ]),
            updates: Vec::new(),
        };
        self.sessions.insert(session_id, record.clone());
        record
    }

    pub fn get_session(&self, session_id: &str) -> Option<&SessionRecord> {
        self.sessions.get(session_id)
    }

    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut SessionRecord> {
        self.sessions.get_mut(session_id)
    }

    pub fn start_turn(&mut self, session_id: &str) -> Result<u64, String> {
        let now = now_secs();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;
        let turn = session.next_turn_seq;
        session.next_turn_seq += 1;
        session.cancelled = false;
        session.updated_at = now;
        Ok(turn)
    }

    pub fn next_update_seq(&mut self, session_id: &str) -> Result<u64, String> {
        let now = now_secs();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;
        let seq = session.next_update_seq;
        session.next_update_seq += 1;
        session.updated_at = now;
        Ok(seq)
    }

    pub fn append_update(
        &mut self,
        session_id: &str,
        update: StoredSessionUpdate,
    ) -> Result<(), String> {
        let now = now_secs();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;
        session.updated_at = now;
        session.updates.push(update);
        Ok(())
    }

    pub fn set_cancelled(&mut self, session_id: &str, cancelled: bool) -> Result<(), String> {
        let now = now_secs();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;
        session.cancelled = cancelled;
        session.updated_at = now;
        Ok(())
    }

    pub fn set_config_option(
        &mut self,
        session_id: &str,
        config_id: &str,
        value: &str,
    ) -> Result<(), String> {
        let now = now_secs();
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;
        session
            .config_options
            .insert(config_id.to_string(), value.to_string());
        session.updated_at = now;
        Ok(())
    }

    pub fn config_options(&self, session_id: &str) -> Result<Vec<Value>, String> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;

        let mode_value = session
            .config_options
            .get("mode")
            .cloned()
            .unwrap_or_else(|| "ask".to_string());
        let model_value = session
            .config_options
            .get("model")
            .cloned()
            .unwrap_or_else(|| "model-1".to_string());

        Ok(vec![
            serde_json::json!({
                "id": "mode",
                "name": "Session Mode",
                "category": "mode",
                "type": "select",
                "currentValue": mode_value,
                "options": [
                    { "value": "ask", "name": "Ask", "description": "Request permission before actions" },
                    { "value": "code", "name": "Code", "description": "Run with allowed tool policy" }
                ]
            }),
            serde_json::json!({
                "id": "model",
                "name": "Model",
                "category": "model",
                "type": "select",
                "currentValue": model_value,
                "options": [
                    { "value": "model-1", "name": "Model 1", "description": "Balanced" },
                    { "value": "model-2", "name": "Model 2", "description": "Higher reasoning" }
                ]
            }),
        ])
    }

    pub fn replay_updates(&self, session_id: &str) -> Result<Vec<StoredSessionUpdate>, String> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;

        let mut updates = session.updates.clone();
        updates.sort_by_key(|u| (u.turn_seq, u.update_seq));
        Ok(updates)
    }

    fn default_path() -> PathBuf {
        let base = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join(".cortex").join("acp_sessions.json")
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_session_store() {
        let temp = std::env::temp_dir().join("acp_sessions_test.json");
        let _ = std::fs::remove_file(&temp);

        let mut store = AcpSessionStore::load(temp.clone()).unwrap();
        store.create_session("sess_1".to_string(), "/tmp".to_string());
        let turn = store.start_turn("sess_1").unwrap();
        let update_seq = store.next_update_seq("sess_1").unwrap();
        store
            .append_update(
                "sess_1",
                StoredSessionUpdate {
                    session_update: "agent_message_chunk".to_string(),
                    turn_seq: turn,
                    update_seq,
                    timestamp: 1,
                    event_id: Some("evt_1".to_string()),
                    payload: serde_json::json!({"text": "ok"}),
                },
            )
            .unwrap();
        store.save().unwrap();

        let restored = AcpSessionStore::load(temp).unwrap();
        assert!(restored.get_session("sess_1").is_some());
        assert_eq!(restored.replay_updates("sess_1").unwrap().len(), 1);
    }
}
