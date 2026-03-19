use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Host-agnostic actor identifier. Replaces `candid::Principal` to keep
/// this crate free of ICP-specific dependencies (118 Purity Contract).
pub type ActorId = String;

/// Represents the type of alignment signal derived from agent behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    /// Frequency of human edits to agent output.
    CorrectionDensity,
    /// Ratio of Forks vs Merges.
    ForkPressure,
    /// Discrepancy between confidence and acceptance.
    ConfidenceDrift,
    /// Frequency of manual overrides.
    HumanInterventionRate,
    /// Heuristic detection of norm violations.
    NormViolation,
}

/// A specific instance of a measured signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentSignal {
    pub agent_id: ActorId,
    pub signal_type: SignalType,
    /// Normalized value 0.0 to 1.0
    pub value: f64,
    /// The window of time this signal derived from
    pub time_window_seconds: u64,
    pub context_space_id: String,
    pub context_workflow_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    GreaterThan,
    LessThan,
    EqualTo,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTrigger {
    pub signal: SignalType,
    pub operator: Operator,
    pub threshold: f64,
    /// How long this condition must persist (seconds)
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrchestrationAction {
    LogWarning,
    RequireReview,
    DowngradeAutonomy,
    ThrottleExecution,
    SwitchModel,
    PauseExecution,
    RevokeAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPolicy {
    pub id: String,
    pub target_role: String,
    pub triggers: Vec<PolicyTrigger>,
    pub action: OrchestrationAction,
    pub recovery_condition: Option<PolicyTrigger>,
}

impl OrchestrationPolicy {
    pub fn evaluate(&self, signal: &AlignmentSignal) -> Option<OrchestrationAction> {
        for trigger in &self.triggers {
            if trigger.signal == signal.signal_type {
                let triggered = match trigger.operator {
                    Operator::GreaterThan => signal.value > trigger.threshold,
                    Operator::LessThan => signal.value < trigger.threshold,
                    Operator::EqualTo => (signal.value - trigger.threshold).abs() < f64::EPSILON,
                    Operator::GreaterThanOrEqual => signal.value >= trigger.threshold,
                    Operator::LessThanOrEqual => signal.value <= trigger.threshold,
                };

                // NOTE: In a real system, we would also check duration here using a history provider.
                // For this primitive implementation, we assume instantaneous evaluation if the signal is present.
                if triggered {
                    return Some(self.action.clone());
                }
            }
        }
        None
    }
}

/// Trait for components that can extract signals from history.
pub trait SignalExtractor {
    fn extract(&self, agent_id: ActorId, window: Duration) -> Vec<AlignmentSignal>;
}
