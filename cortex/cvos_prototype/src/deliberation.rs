//! Deliberation Components for CVOS
//!
//! ECS components for visualizing the Deliberation layer in the infinite canvas.
//! These components represent Arguments, Critiques, and Evaluations as renderable entities.
//!
//! Per Strategy 034c: Epistemic & Deliberative Governance.

use bevy_ecs::prelude::*;

/// An Argument node in the canvas.
/// Represents a structured reasoning unit from `nostra.argument`.
#[derive(Component, Debug)]
pub struct ArgumentNode {
    /// The argument's unique ID (matches graph node)
    pub id: String,
    /// The claim being made
    pub claim: String,
    /// Stance: support, oppose, alternative
    pub stance: ArgumentStance,
    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Visual state for rendering
    pub visual_state: NodeVisualState,
}

/// Argument stance variants
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentStance {
    Support,
    Oppose,
    Alternative,
}

impl From<&str> for ArgumentStance {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "oppose" => ArgumentStance::Oppose,
            "alternative" => ArgumentStance::Alternative,
            _ => ArgumentStance::Support,
        }
    }
}

/// An Evaluation node in the canvas.
/// Represents lightweight feedback from `nostra.evaluation`.
#[derive(Component, Debug)]
pub struct EvaluationNode {
    /// The evaluation's unique ID
    pub id: String,
    /// Reference to the target artifact
    pub target_id: String,
    /// Stance: supportive, skeptical, neutral
    pub stance: EvaluationStance,
    /// Brief summary
    pub summary: String,
    /// Optional score (0-10)
    pub score: Option<f32>,
}

/// Evaluation stance variants
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationStance {
    Supportive,
    Skeptical,
    Neutral,
}

impl From<&str> for EvaluationStance {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "skeptical" => EvaluationStance::Skeptical,
            "neutral" => EvaluationStance::Neutral,
            _ => EvaluationStance::Supportive,
        }
    }
}

/// A Critique node in the canvas.
/// Represents a formal, dPub-backed challenge from `nostra.critique`.
#[derive(Component, Debug)]
pub struct CritiqueNode {
    /// The critique's unique ID
    pub id: String,
    /// Reference to the target artifact
    pub target_id: String,
    /// The core thesis/challenge
    pub thesis: String,
    /// Severity: substantial or critical
    pub severity: CritiqueSeverity,
    /// Status: submitted, acknowledged, addressed, unresolved
    pub status: CritiqueStatus,
    /// Reference to the dPub document
    pub body_ref: String,
}

/// Critique severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum CritiqueSeverity {
    Substantial,
    Critical,
}

impl From<&str> for CritiqueSeverity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => CritiqueSeverity::Critical,
            _ => CritiqueSeverity::Substantial,
        }
    }
}

/// Critique status variants
#[derive(Debug, Clone, PartialEq)]
pub enum CritiqueStatus {
    Submitted,
    Acknowledged,
    Addressed,
    Unresolved,
}

impl From<&str> for CritiqueStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "acknowledged" => CritiqueStatus::Acknowledged,
            "addressed" => CritiqueStatus::Addressed,
            "unresolved" => CritiqueStatus::Unresolved,
            _ => CritiqueStatus::Submitted,
        }
    }
}

/// Visual state for canvas nodes
#[derive(Debug, Clone, Default)]
pub struct NodeVisualState {
    /// Whether the node is selected
    pub selected: bool,
    /// Whether the node is hovered
    pub hovered: bool,
    /// Animation progress (0.0 - 1.0)
    pub animation_t: f32,
}

/// Edge between deliberation nodes (argument → argument, critique → argument, etc.)
#[derive(Component, Debug)]
pub struct DeliberationEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Edge type
    pub edge_type: EdgeType,
}

/// Types of edges in the deliberation graph
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    /// Argument supports proposal/theory
    Supports,
    /// Argument opposes proposal/theory
    Opposes,
    /// Argument rebuts another argument
    Rebuts,
    /// Critique challenges an artifact
    Challenges,
    /// Evaluation assesses an artifact
    Assesses,
}

/// Marker component for nodes that are part of an active Debate workflow
#[derive(Component)]
pub struct InActiveDebate {
    /// The debate workflow instance ID
    pub debate_id: String,
    /// Current phase of the debate
    pub phase: DebatePhase,
}

/// Debate workflow phases (mirrors debates.rs)
#[derive(Debug, Clone, PartialEq)]
pub enum DebatePhase {
    Open,
    Deliberation,
    Rebuttal,
    Synthesis,
    Resolved,
}

impl From<&str> for DebatePhase {
    fn from(s: &str) -> Self {
        match s {
            "debate.open" => DebatePhase::Open,
            "debate.deliberation" => DebatePhase::Deliberation,
            "debate.rebuttal" => DebatePhase::Rebuttal,
            "debate.synthesis" => DebatePhase::Synthesis,
            "debate.resolved" => DebatePhase::Resolved,
            _ => DebatePhase::Open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_stance_from_str() {
        assert_eq!(ArgumentStance::from("support"), ArgumentStance::Support);
        assert_eq!(ArgumentStance::from("oppose"), ArgumentStance::Oppose);
        assert_eq!(
            ArgumentStance::from("alternative"),
            ArgumentStance::Alternative
        );
    }

    #[test]
    fn test_critique_severity_from_str() {
        assert_eq!(
            CritiqueSeverity::from("substantial"),
            CritiqueSeverity::Substantial
        );
        assert_eq!(
            CritiqueSeverity::from("critical"),
            CritiqueSeverity::Critical
        );
    }
}
