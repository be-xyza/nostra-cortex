use cortex_runtime::policy::sessions as runtime_sessions;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub use runtime_sessions::{SessionRecord, StoredSessionUpdate};

#[derive(Debug, Clone)]
pub struct AcpSessionStore {
    path: PathBuf,
    inner: runtime_sessions::AcpSessionStore,
}

impl AcpSessionStore {
    pub fn load_default() -> Result<Self, String> {
        Self::load(Self::default_path())
    }

    pub fn load(path: PathBuf) -> Result<Self, String> {
        let content = if path.exists() {
            Some(fs::read_to_string(&path).map_err(|e| e.to_string())?)
        } else {
            None
        };
        let inner = runtime_sessions::AcpSessionStore::from_json(content.as_deref())?;
        Ok(Self { path, inner })
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = self.inner.to_json()?;
        fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    pub fn create_session(&mut self, session_id: String, cwd: String) -> SessionRecord {
        self.inner.create_session(session_id, cwd, now_secs())
    }

    pub fn get_session(&self, session_id: &str) -> Option<&SessionRecord> {
        self.inner.get_session(session_id)
    }

    pub fn start_turn(&mut self, session_id: &str) -> Result<u64, String> {
        self.inner.start_turn(session_id, now_secs())
    }

    pub fn next_update_seq(&mut self, session_id: &str) -> Result<u64, String> {
        self.inner.next_update_seq(session_id, now_secs())
    }

    pub fn append_update(
        &mut self,
        session_id: &str,
        update: StoredSessionUpdate,
    ) -> Result<(), String> {
        self.inner.append_update(session_id, update, now_secs())
    }

    pub fn set_cancelled(&mut self, session_id: &str, cancelled: bool) -> Result<(), String> {
        self.inner.set_cancelled(session_id, cancelled, now_secs())
    }

    pub fn set_config_option(
        &mut self,
        session_id: &str,
        config_id: &str,
        value: &str,
    ) -> Result<(), String> {
        self.inner
            .set_config_option(session_id, config_id, value, now_secs())
    }

    pub fn config_options(&self, session_id: &str) -> Result<Vec<Value>, String> {
        self.inner.config_options(session_id)
    }

    pub fn replay_updates(&self, session_id: &str) -> Result<Vec<StoredSessionUpdate>, String> {
        self.inner.replay_updates(session_id)
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
