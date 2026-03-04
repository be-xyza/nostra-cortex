pub mod commons;
pub mod engine;
pub mod integrity_events;
pub mod invariant;
pub mod micro_syntax;
pub mod predicate;
pub mod rule;

pub use commons::{
    CommonsEnforcementMode, CommonsEnforcementOutcome, CommonsRuleset, evaluate_commons_ruleset,
    evaluate_commons_ruleset_with_suggested_enrichments,
};
pub use engine::{IntegrityViolation, evaluate_all, evaluate_rule};
pub use invariant::{
    GovernanceProfile, InvariantEnginePort, InvariantPolicy, InvariantViolation as PolicyViolation,
    RepoProjection, SystemIntegrityQuality,
};
pub use micro_syntax::{
    MicroSyntaxExtractor, MicroSyntaxMatch, SuggestedEnrichment, SuggestedEnrichmentKind,
    extract_micro_syntax_matches, extract_suggested_enrichments,
};
pub use predicate::{Constraint, Direction, EdgeSelector, IntegrityPredicate, NodeSelector};
pub use rule::{IntegrityRule, IntegrityScope, Severity};
