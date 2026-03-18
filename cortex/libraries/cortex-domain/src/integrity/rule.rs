use super::predicate::IntegrityPredicate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scope: IntegrityScope,
    pub predicate: IntegrityPredicate,
    pub severity: Severity,
    #[serde(default)]
    pub remediation_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityScope {
    Global,
    EntityType(String),
    Space(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Violation,
    Critical,
}
