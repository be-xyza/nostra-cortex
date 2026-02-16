use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const LEDGER_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    AllowOnce,
    AllowAlways,
    RejectOnce,
    RejectAlways,
}

impl DecisionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionKind::AllowOnce => "allow_once",
            DecisionKind::AllowAlways => "allow_always",
            DecisionKind::RejectOnce => "reject_once",
            DecisionKind::RejectAlways => "reject_always",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecisionRecord {
    pub session_id: String,
    pub tool_call_id: String,
    pub option_id: String,
    pub kind: DecisionKind,
    pub source: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PermissionSnapshot {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    records: Vec<PermissionDecisionRecord>,
    #[serde(default)]
    session_policy: HashMap<String, HashMap<String, DecisionKind>>,
}

#[derive(Debug, Clone)]
pub struct AcpPermissionLedger {
    path: PathBuf,
    records: Vec<PermissionDecisionRecord>,
    session_policy: HashMap<String, HashMap<String, DecisionKind>>,
}

impl AcpPermissionLedger {
    pub fn load_default() -> Result<Self, String> {
        Self::load(Self::default_path())
    }

    pub fn load(path: PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self {
                path,
                records: Vec::new(),
                session_policy: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let snapshot: PermissionSnapshot =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        Ok(Self {
            path,
            records: snapshot.records,
            session_policy: snapshot.session_policy,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let snapshot = PermissionSnapshot {
            version: LEDGER_VERSION,
            records: self.records.clone(),
            session_policy: self.session_policy.clone(),
        };

        let json = serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())?;
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
        self.records.push(PermissionDecisionRecord {
            session_id,
            tool_call_id,
            option_id,
            kind,
            source,
            timestamp: now_secs(),
        });
    }

    pub fn set_session_policy(&mut self, session_id: &str, policy_key: &str, kind: DecisionKind) {
        self.session_policy
            .entry(session_id.to_string())
            .or_default()
            .insert(policy_key.to_string(), kind);
    }

    pub fn get_session_policy(&self, session_id: &str, policy_key: &str) -> Option<DecisionKind> {
        self.session_policy
            .get(session_id)
            .and_then(|m| m.get(policy_key))
            .cloned()
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
