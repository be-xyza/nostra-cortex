use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    session_policy: BTreeMap<String, BTreeMap<String, DecisionKind>>,
}

#[derive(Debug, Clone, Default)]
pub struct AcpPermissionLedger {
    records: Vec<PermissionDecisionRecord>,
    session_policy: BTreeMap<String, BTreeMap<String, DecisionKind>>,
}

impl AcpPermissionLedger {
    pub fn from_json(content: Option<&str>) -> Result<Self, String> {
        let Some(content) = content else {
            return Ok(Self::default());
        };
        if content.trim().is_empty() {
            return Ok(Self::default());
        }

        let snapshot: PermissionSnapshot =
            serde_json::from_str(content).map_err(|e| e.to_string())?;
        Ok(Self {
            records: snapshot.records,
            session_policy: snapshot.session_policy,
        })
    }

    pub fn to_json(&self) -> Result<String, String> {
        let snapshot = PermissionSnapshot {
            version: LEDGER_VERSION,
            records: self.records.clone(),
            session_policy: self.session_policy.clone(),
        };
        serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())
    }

    pub fn record(
        &mut self,
        session_id: String,
        tool_call_id: String,
        option_id: String,
        kind: DecisionKind,
        source: String,
        now: u64,
    ) {
        self.records.push(PermissionDecisionRecord {
            session_id,
            tool_call_id,
            option_id,
            kind,
            source,
            timestamp: now,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_roundtrip() {
        let mut ledger = AcpPermissionLedger::default();
        ledger.set_session_policy("sess_1", "execute", DecisionKind::AllowAlways);
        ledger.record(
            "sess_1".to_string(),
            "call_1".to_string(),
            "allow_once".to_string(),
            DecisionKind::AllowOnce,
            "test".to_string(),
            10,
        );
        let json = ledger.to_json().unwrap();
        let restored = AcpPermissionLedger::from_json(Some(&json)).unwrap();
        assert_eq!(
            restored.get_session_policy("sess_1", "execute"),
            Some(DecisionKind::AllowAlways)
        );
    }
}
