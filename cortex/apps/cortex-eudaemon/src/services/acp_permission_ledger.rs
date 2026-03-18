use cortex_runtime::policy::permissions as runtime_permissions;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub use runtime_permissions::DecisionKind;

#[derive(Debug, Clone)]
pub struct AcpPermissionLedger {
    path: PathBuf,
    inner: runtime_permissions::AcpPermissionLedger,
}

impl AcpPermissionLedger {
    pub fn load_default() -> Result<Self, String> {
        Self::load(Self::default_path())
    }

    pub fn load(path: PathBuf) -> Result<Self, String> {
        let content = if path.exists() {
            Some(fs::read_to_string(&path).map_err(|e| e.to_string())?)
        } else {
            None
        };
        let inner = runtime_permissions::AcpPermissionLedger::from_json(content.as_deref())?;
        Ok(Self { path, inner })
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = self.inner.to_json()?;
        fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    pub fn record(
        &mut self,
        session_id: String,
        tool_call_id: String,
        option_id: String,
        kind: DecisionKind,
        source: String,
    ) {
        self.inner.record(
            session_id,
            tool_call_id,
            option_id,
            kind,
            source,
            now_secs(),
        )
    }

    pub fn set_session_policy(&mut self, session_id: &str, policy_key: &str, kind: DecisionKind) {
        self.inner.set_session_policy(session_id, policy_key, kind)
    }

    #[cfg(test)]
    pub fn get_session_policy(&self, session_id: &str, policy_key: &str) -> Option<DecisionKind> {
        self.inner.get_session_policy(session_id, policy_key)
    }

    fn default_path() -> PathBuf {
        let base = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join(".cortex").join("acp_permission_ledger.json")
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
    fn policy_roundtrip() {
        let temp = std::env::temp_dir().join("acp_permission_ledger_test.json");
        let _ = fs::remove_file(&temp);

        let mut ledger = AcpPermissionLedger::load(temp.clone()).unwrap();
        ledger.set_session_policy("sess_1", "execute", DecisionKind::AllowAlways);
        ledger.record(
            "sess_1".to_string(),
            "call_1".to_string(),
            "allow-once".to_string(),
            DecisionKind::AllowOnce,
            "test".to_string(),
        );
        ledger.save().unwrap();

        let restored = AcpPermissionLedger::load(temp).unwrap();
        assert_eq!(
            restored.get_session_policy("sess_1", "execute"),
            Some(DecisionKind::AllowAlways)
        );
    }
}
