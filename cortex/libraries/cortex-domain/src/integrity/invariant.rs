use crate::graph::Graph;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The unified materialized graph representation of a sandboxed repository.
/// This matches the Code Property Graph (CPG) abstraction pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RepoProjection {
    /// Captures the filesystem structure, directories, and markdown links
    pub file_graph: Graph,
    /// Captures AST parsed nodes, traits, structs, and function calls
    pub symbol_graph: Graph,
    /// Captures upstream packages and peer module dependencies
    pub dependency_graph: Graph,
    /// Captures test fixtures, mocks, and coverage edges
    pub test_graph: Graph,
    /// Captures Temporal signals and shell tasks
    pub workflow_graph: Graph,
    /// Captures relationships between Polymorphic Blocks (A2UI, Rich Text, Pointers, etc.)
    pub block_graph: Graph,
    /// High-level metadata (Stewardship, Status, Authors)
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl RepoProjection {
    /// Compute a deterministic hash of the entire projection for idempotent evaluation.
    /// This combines the `root_hash_hex()` of all graph layers.
    pub fn content_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(self.file_graph.root_hash_hex().as_bytes());
        hasher.update(b"|");
        hasher.update(self.symbol_graph.root_hash_hex().as_bytes());
        hasher.update(b"|");
        hasher.update(self.dependency_graph.root_hash_hex().as_bytes());
        hasher.update(b"|");
        hasher.update(self.test_graph.root_hash_hex().as_bytes());
        hasher.update(b"|");
        hasher.update(self.workflow_graph.root_hash_hex().as_bytes());
        hasher.update(b"|");
        hasher.update(self.block_graph.root_hash_hex().as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// A collection of declarative policies that a repository must abide by.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceProfile {
    pub id: String,
    pub name: String,
    pub invariants: Vec<InvariantPolicy>,
}

/// A specific governance rule executed by the WASM Policy Engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvariantPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The identifier for the WASM module or Rego policy to evaluate
    pub evaluator_ref: String,
}

/// The UI-visible scorecard resulting from a Governance Profile evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemIntegrityQuality {
    /// A 0-100 metric of structural health
    pub score: u8,
    /// Whether the sandbox passed the minimum constitutional bounds
    pub passing: bool,
    /// The specific invariant checks that failed
    #[serde(default)]
    pub violations: Vec<InvariantViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvariantViolation {
    pub policy_id: String,
    pub message: String,
    pub severity: String,
    #[serde(default)]
    pub affected_nodes: Vec<String>,
}

/// The core Substrate port that executes policies against a projection.
#[async_trait::async_trait]
pub trait InvariantEnginePort: Send + Sync {
    async fn evaluate_projection(
        &self,
        projection: &RepoProjection,
        profile: &GovernanceProfile,
    ) -> Result<SystemIntegrityQuality, crate::error::DomainError>;
}
