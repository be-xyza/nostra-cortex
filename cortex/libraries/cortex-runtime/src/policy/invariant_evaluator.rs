use async_trait::async_trait;
use cortex_domain::error::DomainError;
use cortex_domain::graph::EdgeKind;
use cortex_domain::integrity::engine::IntegrityViolation as EngineViolation;
use cortex_domain::integrity::invariant::{
    GovernanceProfile, InvariantEnginePort, InvariantPolicy, InvariantViolation, RepoProjection,
    SystemIntegrityQuality,
};
use cortex_domain::integrity::predicate::{
    Constraint, Direction, EdgeSelector, IntegrityPredicate, NodeSelector,
};
use cortex_domain::integrity::rule::{IntegrityScope, Severity};
use cortex_domain::integrity::{IntegrityRule, evaluate_all};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementMode {
    Shadow,    // Log only, always passing
    Advisory,  // Warn, but don't block
    Enforcing, // Block on Critical/Violation
}

impl Default for EnforcementMode {
    fn default() -> Self {
        Self::Enforcing
    }
}

/// A bootstrap evaluator that runs invariant checks natively in Rust
/// using the existing `cortex-domain` integrity engine. This serves as
/// the reference implementation until a full WASM policy runtime is
/// integrated (Phase 2b).
pub struct NativeInvariantEvaluator {
    /// Static rules provided via programmatic construction
    pub rules: Vec<IntegrityRule>,
    /// How the rules should be enforced and scored
    pub mode: EnforcementMode,
}

impl NativeInvariantEvaluator {
    pub fn new(rules: Vec<IntegrityRule>) -> Self {
        Self {
            rules,
            mode: EnforcementMode::Enforcing,
        }
    }

    pub fn with_mode(mut self, mode: EnforcementMode) -> Self {
        self.mode = mode;
        self
    }
}

#[async_trait]
impl InvariantEnginePort for NativeInvariantEvaluator {
    async fn evaluate_projection(
        &self,
        projection: &RepoProjection,
        profile: &GovernanceProfile,
    ) -> Result<SystemIntegrityQuality, DomainError> {
        let mut all_engine_violations: Vec<(String, EngineViolation)> = Vec::new();

        // Combine static rules with rules derived from the profile's evaluator_refs
        let mut active_rules = self.rules.clone();
        active_rules.extend(translate_profile_to_rules(profile));

        // Evaluate rules against each graph layer in the projection
        let graphs = [
            ("file", &projection.file_graph),
            ("symbol", &projection.symbol_graph),
            ("dependency", &projection.dependency_graph),
            ("test", &projection.test_graph),
            ("workflow", &projection.workflow_graph),
            ("block", &projection.block_graph),
        ];

        for (layer_name, graph) in &graphs {
            let layer_violations = evaluate_all(&active_rules, graph);
            for v in layer_violations {
                all_engine_violations.push((layer_name.to_string(), v));
            }
        }

        let mut output_violations = Vec::new();

        // Convert EngineViolations to InvariantViolations for the output payload
        for (layer, v) in &all_engine_violations {
            output_violations.push(InvariantViolation {
                policy_id: format!("{}:{}", layer, v.rule_id),
                message: v.explanation.clone(),
                severity: format!("{:?}", v.severity),
                affected_nodes: v.affected_nodes.clone(),
            });
        }

        // Check that the profile's declared invariants are at least accounted for
        for policy in &profile.invariants {
            let matching_rule = active_rules.iter().any(|r| r.id == policy.evaluator_ref);
            if !matching_rule {
                let msg = format!(
                    "Governance profile requires evaluator '{}' but no matching rule is registered",
                    policy.evaluator_ref
                );
                output_violations.push(InvariantViolation {
                    policy_id: policy.id.clone(),
                    message: msg.clone(),
                    severity: "Warning".to_string(),
                    affected_nodes: vec![],
                });
                // Also add an artificial engine violation so penalty scoring captures it
                all_engine_violations.push((
                    "schema".to_string(),
                    EngineViolation {
                        rule_id: policy.id.clone(),
                        severity: Severity::Warning,
                        explanation: msg,
                        affected_nodes: vec![],
                    },
                ));
            }
        }

        // Compute SIQ score using severity penalty
        let engine_only: Vec<EngineViolation> =
            all_engine_violations.into_iter().map(|(_, v)| v).collect();
        let (score, mut passing) = compute_penalty_score(&engine_only);

        if self.mode == EnforcementMode::Shadow || self.mode == EnforcementMode::Advisory {
            passing = true; // Never fail in non-enforcing modes
        }

        Ok(SystemIntegrityQuality {
            score,
            passing,
            violations: output_violations,
        })
    }
}

/// Compute a 0-100 SIQ score using severity-weighted penalties.
fn compute_penalty_score(violations: &[EngineViolation]) -> (u8, bool) {
    if violations.is_empty() {
        return (100, true);
    }

    let mut penalty: u32 = 0;
    for v in violations {
        penalty += match v.severity {
            Severity::Critical => 25,
            Severity::Violation => 10,
            Severity::Warning => 3,
            Severity::Info => 1,
        };
    }

    let score = 100u32.saturating_sub(penalty).min(100) as u8;
    let passing = !violations
        .iter()
        .any(|v| matches!(v.severity, Severity::Critical | Severity::Violation));

    (score, passing)
}

/// Translate a `GovernanceProfile` into `IntegrityRule`s based on `evaluator_ref`.
///
/// In v1, this maps natively implemented rules. In v2 (WASM),
/// `evaluator_ref` would point to a WASM module hash.
fn translate_profile_to_rules(profile: &GovernanceProfile) -> Vec<IntegrityRule> {
    profile
        .invariants
        .iter()
        .filter_map(|policy| match policy.evaluator_ref.as_str() {
            "initiative.must_have_plan" => Some(rule_must_have_plan(policy)),
            "initiative.must_have_research" => Some(rule_must_have_research(policy)),
            "initiative.no_orphan_active" => Some(rule_no_orphan_active(policy)),
            "initiative.no_dependency_cycles" => Some(rule_no_dependency_cycles(policy)),
            "initiative.min_dependency" => Some(rule_min_dependency(policy)),
            "initiative.requires_constitutional_basis" => {
                Some(rule_requires_constitutional_basis(policy))
            }
            "file.no_empty_directories" => Some(rule_no_empty_directories(policy)),
            _ => None, // Unknown evaluator_ref — warning emitted dynamically during eval
        })
        .collect()
}

// --- Built-in Rule Definitions ---

fn rule_must_have_plan(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(), // Use evaluator_ref as the ID internally so matching works
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::EntityType("directory".to_string()),
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: Some("directory".to_string()),
                tags: Some(vec!["initiative".to_string()]),
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::Custom("contains".to_string()),
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::MinCount(1),
        },
        severity: Severity::Warning,
        remediation_hint: Some("Add a PLAN.md file to this initiative directory.".to_string()),
    }
}

fn rule_must_have_research(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(),
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::EntityType("directory".to_string()),
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: Some("directory".to_string()),
                tags: Some(vec!["initiative".to_string()]),
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::Custom("contains".to_string()),
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::MinCount(1),
        },
        severity: Severity::Warning,
        remediation_hint: Some("Add a RESEARCH.md file to this initiative directory.".to_string()),
    }
}

fn rule_no_orphan_active(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(),
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::EntityType("initiative".to_string()),
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: Some("initiative".to_string()),
                tags: None,
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::DependsOn,
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::MinCount(1),
        },
        severity: Severity::Violation,
        remediation_hint: Some(
            "Active initiatives must declare at least one dependency.".to_string(),
        ),
    }
}

fn rule_no_dependency_cycles(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(),
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::Global,
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: None,
                tags: None,
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::DependsOn,
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::NoCycles,
        },
        severity: Severity::Critical,
        remediation_hint: Some("Dependency cycles prevent clean build ordering.".to_string()),
    }
}

fn rule_min_dependency(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(),
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::EntityType("initiative".to_string()),
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: Some("initiative".to_string()),
                tags: None,
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::DependsOn,
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::MinCount(1),
        },
        severity: Severity::Warning,
        remediation_hint: Some("Initiatives should declare at least one dependency.".to_string()),
    }
}

fn rule_requires_constitutional_basis(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(),
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::EntityType("initiative".to_string()),
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: Some("initiative".to_string()),
                tags: None,
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::ConstitutionalBasis,
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::RequiresConstitutionalReference,
        },
        severity: Severity::Warning,
        remediation_hint: Some(
            "Initiatives should reference a constitutional basis document.".to_string(),
        ),
    }
}

fn rule_no_empty_directories(policy: &InvariantPolicy) -> IntegrityRule {
    IntegrityRule {
        id: policy.evaluator_ref.clone(),
        name: policy.name.clone(),
        description: policy.description.clone(),
        scope: IntegrityScope::EntityType("directory".to_string()),
        predicate: IntegrityPredicate {
            target: NodeSelector {
                entity_type: Some("directory".to_string()),
                tags: None,
            },
            relation: Some(EdgeSelector {
                edge_kind: EdgeKind::Custom("contains".to_string()),
                direction: Direction::Outgoing,
            }),
            constraint: Constraint::MinCount(1),
        },
        severity: Severity::Info,
        remediation_hint: Some("Empty directories should be removed or populated.".to_string()),
    }
}

/// Returns the canonical Nostra/Cortex governance profile for research repositories.
/// Loaded from `assets/default_governance_profile.json`.
pub fn default_governance_profile() -> GovernanceProfile {
    serde_json::from_str(include_str!("assets/default_governance_profile.json"))
        .expect("Failed to parse default governance profile JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::graph::{Edge, Graph, Node};
    use std::collections::BTreeMap;

    fn sample_profile() -> GovernanceProfile {
        GovernanceProfile {
            id: "test-profile".to_string(),
            name: "Test Governance".to_string(),
            invariants: vec![InvariantPolicy {
                id: "p1".to_string(),
                name: "No orphans".to_string(),
                description: "All initiatives must have dependencies".to_string(),
                evaluator_ref: "no-orphans".to_string(),
            }],
        }
    }

    fn sample_rule() -> IntegrityRule {
        IntegrityRule {
            id: "no-orphans".to_string(),
            name: "No orphans".to_string(),
            description: "Every initiative must depend on at least one node".to_string(),
            scope: IntegrityScope::EntityType("initiative".to_string()),
            predicate: IntegrityPredicate {
                target: NodeSelector {
                    entity_type: Some("initiative".to_string()),
                    tags: None,
                },
                relation: Some(EdgeSelector {
                    edge_kind: EdgeKind::DependsOn,
                    direction: Direction::Outgoing,
                }),
                constraint: Constraint::MinCount(1),
            },
            severity: Severity::Violation, // Changed from Warning to test Enforcing
            remediation_hint: None,
        }
    }

    #[tokio::test]
    async fn evaluator_passes_when_all_rules_met() {
        let mut graph = Graph::default();
        graph.add_node(Node {
            id: "i1".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_node(Node {
            id: "i2".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_edge(Edge {
            from: "i1".to_string(),
            to: "i2".to_string(),
            kind: EdgeKind::DependsOn,
        });
        graph.add_edge(Edge {
            from: "i2".to_string(),
            to: "i1".to_string(),
            kind: EdgeKind::DependsOn,
        });

        let projection = RepoProjection {
            file_graph: graph,
            ..Default::default()
        };

        let evaluator = NativeInvariantEvaluator::new(vec![sample_rule()]);
        let result = evaluator
            .evaluate_projection(&projection, &sample_profile())
            .await
            .expect("evaluation should succeed");

        assert!(result.passing);
        assert_eq!(result.score, 100);
    }

    #[tokio::test]
    async fn evaluator_fails_when_orphan_detected() {
        let mut graph = Graph::default();
        graph.add_node(Node {
            id: "i1".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_node(Node {
            id: "orphan".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_edge(Edge {
            from: "i1".to_string(),
            to: "orphan".to_string(),
            kind: EdgeKind::DependsOn,
        });

        let projection = RepoProjection {
            file_graph: graph,
            ..Default::default()
        };

        let evaluator = NativeInvariantEvaluator::new(vec![sample_rule()]);
        let result = evaluator
            .evaluate_projection(&projection, &sample_profile())
            .await
            .expect("evaluation should succeed");

        assert!(!result.passing); // Enforcing mode + Violation severity = fails
        assert!(result.score < 100);
        assert!(!result.violations.is_empty());
    }

    #[tokio::test]
    async fn evaluator_shadow_mode_passes() {
        let mut graph = Graph::default();
        // Orphan node
        graph.add_node(Node {
            id: "orphan".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });

        let projection = RepoProjection {
            file_graph: graph,
            ..Default::default()
        };

        let evaluator =
            NativeInvariantEvaluator::new(vec![sample_rule()]).with_mode(EnforcementMode::Shadow);
        let result = evaluator
            .evaluate_projection(&projection, &sample_profile())
            .await
            .expect("evaluation should succeed");

        // Shadow mode guarantees passing, but still scores the penalty
        assert!(result.passing);
        assert!(result.score < 100);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn penalty_score_computation() {
        let violations = vec![
            EngineViolation {
                rule_id: "r1".to_string(),
                affected_nodes: vec![],
                severity: Severity::Warning,
                explanation: "".to_string(),
            },
            EngineViolation {
                rule_id: "r2".to_string(),
                affected_nodes: vec![],
                severity: Severity::Violation,
                explanation: "".to_string(),
            },
        ];

        let (score, passing) = compute_penalty_score(&violations);
        // Warning = 3, Violation = 10, total penalty = 13
        assert_eq!(score, 87);
        assert!(!passing); // Violation present
    }
}
