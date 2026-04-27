use super::engine::{evaluate_all, IntegrityViolation};
use super::micro_syntax::SuggestedEnrichment;
use super::rule::IntegrityRule;
use crate::graph::Graph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommonsEnforcementMode {
    Shadow,
    WarnOrBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommonsRuleset {
    pub commons_id: String,
    pub commons_version: String,
    #[serde(default)]
    pub rules: Vec<IntegrityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CommonsEnforcementOutcome {
    pub mode: CommonsEnforcementMode,
    pub should_block: bool,
    pub should_warn: bool,
    #[serde(default)]
    pub violations: Vec<IntegrityViolation>,
    #[serde(default)]
    pub suggested_enrichments: Vec<SuggestedEnrichment>,
}

pub fn evaluate_commons_ruleset(
    graph: &Graph,
    ruleset: &CommonsRuleset,
    mode: CommonsEnforcementMode,
) -> CommonsEnforcementOutcome {
    evaluate_commons_ruleset_with_suggested_enrichments(graph, ruleset, mode, Vec::new())
}

pub fn evaluate_commons_ruleset_with_suggested_enrichments(
    graph: &Graph,
    ruleset: &CommonsRuleset,
    mode: CommonsEnforcementMode,
    suggested_enrichments: Vec<SuggestedEnrichment>,
) -> CommonsEnforcementOutcome {
    let violations = evaluate_all(&ruleset.rules, graph);
    let should_block = matches!(mode, CommonsEnforcementMode::WarnOrBlock)
        && violations
            .iter()
            .any(|entry| matches!(entry.severity, super::rule::Severity::Critical));
    let should_warn = !violations.is_empty() || !suggested_enrichments.is_empty();

    CommonsEnforcementOutcome {
        mode,
        should_block,
        should_warn,
        violations,
        suggested_enrichments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, EdgeKind, Node};
    use crate::integrity::micro_syntax::{extract_suggested_enrichments, SuggestedEnrichmentKind};
    use crate::integrity::predicate::{
        Constraint, Direction, EdgeSelector, IntegrityPredicate, NodeSelector,
    };
    use crate::integrity::rule::{IntegrityRule, IntegrityScope, Severity};
    use std::collections::BTreeMap;

    #[test]
    fn critical_violation_blocks_in_warn_or_block_mode() {
        let mut graph = Graph::default();
        graph.add_node(Node {
            id: "initiative-1".to_string(),
            node_type: "initiative".to_string(),
            attributes: BTreeMap::new(),
        });

        let ruleset = CommonsRuleset {
            commons_id: "commons-1".to_string(),
            commons_version: "v1".to_string(),
            rules: vec![IntegrityRule {
                id: "rule-1".to_string(),
                name: "Must reference constitution".to_string(),
                description: "Rule".to_string(),
                scope: IntegrityScope::Global,
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
                severity: Severity::Critical,
                remediation_hint: None,
            }],
        };

        let outcome =
            evaluate_commons_ruleset(&graph, &ruleset, CommonsEnforcementMode::WarnOrBlock);
        assert!(outcome.should_block);
        assert!(outcome.should_warn);

        graph.add_node(Node {
            id: "constitution-1".to_string(),
            node_type: "constitution".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_edge(Edge {
            from: "initiative-1".to_string(),
            to: "constitution-1".to_string(),
            kind: EdgeKind::ConstitutionalBasis,
        });

        let clean = evaluate_commons_ruleset(&graph, &ruleset, CommonsEnforcementMode::WarnOrBlock);
        assert!(!clean.should_block);
    }

    #[test]
    fn evaluate_commons_ruleset_includes_suggested_enrichments() {
        let graph = Graph::default();
        let ruleset = CommonsRuleset {
            commons_id: "commons-1".to_string(),
            commons_version: "v1".to_string(),
            rules: vec![],
        };
        let enrichments = extract_suggested_enrichments("We must abide by #constitution");
        let outcome = evaluate_commons_ruleset_with_suggested_enrichments(
            &graph,
            &ruleset,
            CommonsEnforcementMode::WarnOrBlock,
            enrichments,
        );

        assert!(outcome.should_warn);
        assert_eq!(outcome.violations.len(), 0);
        assert!(outcome
            .suggested_enrichments
            .iter()
            .any(|entry| entry.kind == SuggestedEnrichmentKind::Tag
                && entry.matched_text == "#constitution"));
    }
}
