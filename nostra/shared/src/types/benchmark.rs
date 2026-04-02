use candid::{CandidType, Deserialize};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq)]
pub enum BenchmarkType {
    #[serde(rename = "logic")]
    Logic,
    #[serde(rename = "tool_use")]
    ToolUse,
    #[serde(rename = "compliance")]
    Compliance,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq)]
pub enum PolicyConstraint {
    #[serde(rename = "gdpr_v1")]
    GdprV1,
    #[serde(rename = "hipaa_phi")]
    HipaaPhi,
    #[serde(rename = "readonly")]
    ReadOnly,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq)]
pub enum WinCondition {
    #[serde(rename = "exact_match")]
    ExactMatch(String),
    #[serde(rename = "fuzzy_match")]
    FuzzyMatch(String),
    #[serde(rename = "function_call")]
    FunctionCall(String),
    #[serde(rename = "semantic_match")]
    SemanticMatch(String), // Embedding-based similarity
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct MockFile {
    pub path: String,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct EnvironmentConfig {
    pub files: Vec<MockFile>,
    pub tools_allowed: Vec<String>,
    pub internet_access: String,
    pub mock_time: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct AgentConfig {
    pub memory_tier: String,
    pub persona: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct BenchmarkCase {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(rename = "type")]
    pub case_type: BenchmarkType,
    pub description: String,
    pub environment: EnvironmentConfig,
    pub agent_config: AgentConfig,
    pub policy_constraints: Vec<PolicyConstraint>,
    pub win_condition: WinCondition,
    pub metadata: HashMap<String, String>,
}

/// Multi-dimensional benchmark result with detailed scoring
#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct BenchmarkResult {
    pub case_id: String,
    pub passed: bool,
    /// Composite score (0.0 - 1.0)
    pub score: f32,
    /// Detailed scoring dimensions
    pub scoring: ScoringDimensions,
    /// Defect profile if the benchmark failed
    pub defect: Option<DefectProfile>,
    /// Execution logs
    pub logs: Vec<String>,
    /// Unix timestamp of execution
    pub timestamp: u64,
}

/// Multi-dimensional scoring breakdown (from 059 spec)
#[derive(Clone, Debug, Serialize, Deserialize, CandidType, Default)]
pub struct ScoringDimensions {
    /// Win condition success rate (0.0 - 1.0)
    pub success_rate: f32,
    /// Policy compliance (1.0 - violations/total_actions)
    pub policy_compliance: f32,
    /// Efficiency: inverted normalized cost (tokens, calls)
    pub efficiency: f32,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Number of agent turns used
    pub turns_used: usize,
    /// Total tool calls made
    pub tool_calls: usize,
    /// Policy violations encountered
    pub policy_violations: usize,
}

impl ScoringDimensions {
    /// Calculate composite score with weights (from 059 spec)
    pub fn compute_composite(&self) -> f32 {
        const W_SUCCESS: f32 = 0.5;
        const W_POLICY: f32 = 0.3;
        const W_EFFICIENCY: f32 = 0.2;

        (self.success_rate * W_SUCCESS)
            + (self.policy_compliance * W_POLICY)
            + (self.efficiency * W_EFFICIENCY)
    }
}

/// Defect profile for failure taxonomy (from 063 spec)
#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct DefectProfile {
    /// Stage where failure occurred
    pub stage: DefectStage,
    /// Human-readable description
    pub description: String,
    /// Agent turn number when defect was detected
    pub turn: usize,
    /// Optional additional context
    pub context: Option<String>,
}

/// Failure stages for defect classification
#[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq)]
pub enum DefectStage {
    /// Failed during context/memory retrieval
    Retrieval,
    /// Failed during reasoning/planning
    Reasoning,
    /// Failed during tool execution
    ToolExecution,
    /// Violated a policy constraint
    PolicyViolation,
    /// Failed to meet win condition
    WinCondition,
    /// Hit max turns without completion
    Timeout,
    /// Agent returned an error
    AgentError,
}

impl DefectProfile {
    pub fn policy_violation(turn: usize, action: &str) -> Self {
        Self {
            stage: DefectStage::PolicyViolation,
            description: format!("Policy violation on action: {}", action),
            turn,
            context: Some(action.to_string()),
        }
    }

    pub fn win_condition_failed(turn: usize, reason: &str) -> Self {
        Self {
            stage: DefectStage::WinCondition,
            description: format!("Win condition not met: {}", reason),
            turn,
            context: None,
        }
    }

    pub fn timeout(turn: usize) -> Self {
        Self {
            stage: DefectStage::Timeout,
            description: "Maximum turns exceeded without completion".to_string(),
            turn,
            context: None,
        }
    }

    pub fn agent_error(turn: usize, error: &str) -> Self {
        Self {
            stage: DefectStage::AgentError,
            description: format!("Agent error: {}", error),
            turn,
            context: Some(error.to_string()),
        }
    }
}
