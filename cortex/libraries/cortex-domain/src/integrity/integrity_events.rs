use super::invariant::{InvariantViolation, SystemIntegrityQuality};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// An event representing the result of an Invariant Engine evaluation.
/// Emitted after a `GovernanceProfile` is evaluated against a `RepoProjection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvariantEvaluationEvent {
    pub event_id: String,
    pub sandbox_id: String,
    pub profile_id: String,
    pub siq: SystemIntegrityQuality,
    pub timestamp: u64,
    pub projection_hash: String,
}

impl InvariantEvaluationEvent {
    /// Create a new evaluation event with a deterministic event ID.
    pub fn new(
        sandbox_id: String,
        profile_id: String,
        siq: SystemIntegrityQuality,
        timestamp: u64,
        projection_hash: String,
    ) -> Self {
        let event_id = deterministic_evaluation_id(&sandbox_id, &profile_id, timestamp);
        Self {
            event_id,
            sandbox_id,
            profile_id,
            siq,
            timestamp,
            projection_hash,
        }
    }

    /// Serialize to a CloudEvent-compatible JSON value.
    pub fn to_cloud_event(&self) -> serde_json::Value {
        serde_json::json!({
            "specversion": "1.0",
            "type": "nostra.cortex.invariant.evaluation",
            "source": format!("nostra://cortex/sandbox/{}", self.sandbox_id),
            "id": self.event_id,
            "time": self.timestamp,
            "datacontenttype": "application/json",
            "data": {
                "sandbox_id": self.sandbox_id,
                "profile_id": self.profile_id,
                "score": self.siq.score,
                "passing": self.siq.passing,
                "violation_count": self.siq.violations.len(),
                "projection_hash": self.projection_hash,
                "violations": self.siq.violations,
            }
        })
    }
}

/// Generate a deterministic event ID from sandbox, profile, and timestamp.
fn deterministic_evaluation_id(sandbox_id: &str, profile_id: &str, timestamp: u64) -> String {
    let raw = format!("{}:{}:{}", sandbox_id, profile_id, timestamp);
    let digest = Sha256::digest(raw.as_bytes());
    format!("siq_eval_{}", hex::encode(&digest[..16]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_id_is_deterministic() {
        let a = deterministic_evaluation_id("sb1", "prof1", 1000);
        let b = deterministic_evaluation_id("sb1", "prof1", 1000);
        assert_eq!(a, b);
        assert!(a.starts_with("siq_eval_"));
    }

    #[test]
    fn cloud_event_has_required_fields() {
        let siq = SystemIntegrityQuality {
            score: 85,
            passing: false,
            violations: vec![InvariantViolation {
                policy_id: "inv-001".to_string(),
                message: "Orphan detected".to_string(),
                severity: "violation".to_string(),
                affected_nodes: vec!["init-127".to_string()],
            }],
        };

        let event = InvariantEvaluationEvent::new(
            "test-sb".to_string(),
            "nostra-research-v1".to_string(),
            siq,
            1000,
            "abc123".to_string(),
        );

        let ce = event.to_cloud_event();
        assert_eq!(ce["specversion"], "1.0");
        assert_eq!(ce["type"], "nostra.cortex.invariant.evaluation");
        assert_eq!(ce["data"]["score"], 85);
        assert_eq!(ce["data"]["violation_count"], 1);
    }
}
