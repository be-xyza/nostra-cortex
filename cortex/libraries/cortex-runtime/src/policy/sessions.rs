use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
    pub config_options: BTreeMap<String, String>,
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

#[derive(Debug, Clone, Default)]
pub struct AcpSessionStore {
    sessions: BTreeMap<String, SessionRecord>,
}

impl AcpSessionStore {
    pub fn from_json(content: Option<&str>) -> Result<Self, String> {
        let Some(content) = content else {
            return Ok(Self::default());
        };
        if content.trim().is_empty() {
            return Ok(Self::default());
        }

        let snapshot: SessionSnapshot = serde_json::from_str(content).map_err(|e| e.to_string())?;
        let sessions = snapshot
            .sessions
            .into_iter()
            .map(|s| (s.session_id.clone(), s))
            .collect();

        Ok(Self { sessions })
    }

    pub fn to_json(&self) -> Result<String, String> {
        let snapshot = SessionSnapshot {
            version: SESSION_STORE_VERSION,
            sessions: self.sessions.values().cloned().collect(),
        };
        serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())
    }

    pub fn create_session(&mut self, session_id: String, cwd: String, now: u64) -> SessionRecord {
        let record = SessionRecord {
            session_id: session_id.clone(),
            cwd,
            created_at: now,
            updated_at: now,
            next_turn_seq: 1,
            next_update_seq: 1,
            cancelled: false,
            config_options: BTreeMap::from([
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

    pub fn start_turn(&mut self, session_id: &str, now: u64) -> Result<u64, String> {
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

    pub fn next_update_seq(&mut self, session_id: &str, now: u64) -> Result<u64, String> {
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
        now: u64,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("unknown session: {}", session_id))?;
        session.updated_at = now;
        session.updates.push(update);
        Ok(())
    }

    pub fn set_cancelled(
        &mut self,
        session_id: &str,
        cancelled: bool,
        now: u64,
    ) -> Result<(), String> {
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
        now: u64,
    ) -> Result<(), String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_store_roundtrip_and_ordering() {
        let mut store = AcpSessionStore::default();
        store.create_session("sess_1".to_string(), "/tmp".to_string(), 10);
        let turn = store.start_turn("sess_1", 11).unwrap();
        let first = store.next_update_seq("sess_1", 12).unwrap();
        let second = store.next_update_seq("sess_1", 13).unwrap();

        store
            .append_update(
                "sess_1",
                StoredSessionUpdate {
                    session_update: "agent_message_chunk".to_string(),
                    turn_seq: turn,
                    update_seq: second,
                    timestamp: 2,
                    event_id: Some("evt_2".to_string()),
                    payload: serde_json::json!({"text":"b"}),
                },
                14,
            )
            .unwrap();
        store
            .append_update(
                "sess_1",
                StoredSessionUpdate {
                    session_update: "user_message_chunk".to_string(),
                    turn_seq: turn,
                    update_seq: first,
                    timestamp: 1,
                    event_id: Some("evt_1".to_string()),
                    payload: serde_json::json!({"text":"a"}),
                },
                15,
            )
            .unwrap();

        let json = store.to_json().unwrap();
        let restored = AcpSessionStore::from_json(Some(&json)).unwrap();
        let replay = restored.replay_updates("sess_1").unwrap();
        assert_eq!(replay.len(), 2);
        assert_eq!(replay[0].update_seq, 1);
        assert_eq!(replay[1].update_seq, 2);
    }
}
