use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppRoute {
    Home,
    Institutions,
    Questions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildStatus {
    pub mode: &'static str,
    pub source: &'static str,
    pub last_verified: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstitutionSummary {
    pub id: &'static str,
    pub name: &'static str,
    pub stewardship_domain: &'static str,
    pub status: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KipEntity {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub description: String,
    #[serde(default)]
    pub attributes: Vec<(String, String)>,
}

impl KipEntity {
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(attr_key, _)| attr_key == key)
            .map(|(_, value)| value.as_str())
    }

    pub fn display_title(&self) -> &str {
        self.attr("title").unwrap_or(self.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::KipEntity;

    #[test]
    fn display_title_prefers_explicit_title_attribute() {
        let entity = KipEntity {
            id: "question_1".to_string(),
            name: "Question: raw-id".to_string(),
            entity_type: "Question".to_string(),
            description: "Why now?".to_string(),
            attributes: vec![("title".to_string(), "Should we ship?".to_string())],
        };

        assert_eq!(entity.display_title(), "Should we ship?");
    }

    #[test]
    fn display_title_falls_back_to_name() {
        let entity = KipEntity {
            id: "question_2".to_string(),
            name: "Should we ship?".to_string(),
            entity_type: "Question".to_string(),
            description: "Why now?".to_string(),
            attributes: vec![],
        };

        assert_eq!(entity.display_title(), "Should we ship?");
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiqHealth {
    pub status: String,
    pub siq_log_dir: String,
    pub schema_version: String,
    pub runs_count: usize,
    pub gate_exists: bool,
    pub projection_exists: bool,
    pub coverage_fresh: bool,
    pub gate_fresh: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqGateCounts {
    pub pass: u64,
    pub fail: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqGateSummary {
    pub schema_version: String,
    pub mode: String,
    pub latest_run_id: String,
    pub overall_verdict: String,
    pub counts: SiqGateCounts,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqCoverageContribution {
    pub contribution_id: String,
    #[serde(default)]
    pub directory: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub closure_state: String,
    #[serde(default)]
    pub rules: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqCoverage {
    pub schema_version: String,
    pub generated_at: String,
    pub integrity_set: Vec<String>,
    pub contributions: Vec<SiqCoverageContribution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqDependencyClosureRow {
    pub contribution_id: String,
    #[serde(default)]
    pub required_dependencies: Vec<String>,
    #[serde(default)]
    pub satisfied_dependencies: Vec<String>,
    #[serde(default)]
    pub missing_dependencies: Vec<String>,
    #[serde(default)]
    pub closure_state: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqDependencyClosure {
    pub schema_version: String,
    pub generated_at: String,
    pub integrity_set: Vec<String>,
    pub overall_closure_state: String,
    pub rows: Vec<SiqDependencyClosureRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiqSnapshot {
    pub health: SiqHealth,
    pub gates: SiqGateSummary,
    pub coverage: SiqCoverage,
    pub dependency_closure: SiqDependencyClosure,
    pub runs: Vec<SiqRunArtifact>,
    pub latest_run: Option<SiqRunArtifact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqRuleResult {
    pub id: String,
    pub severity: String,
    pub owner: String,
    pub source_standard: String,
    pub status: String,
    #[serde(default)]
    pub failures: Vec<Value>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiqRunArtifact {
    pub schema_version: String,
    pub run_id: String,
    pub generated_at: String,
    pub mode: String,
    pub policy_path: String,
    pub policy_version: u64,
    pub overall_verdict: String,
    pub required_gates_pass: bool,
    pub counts: SiqGateCounts,
    #[serde(default)]
    pub failures: Vec<Value>,
    #[serde(default)]
    pub results: Vec<SiqRuleResult>,
    pub git_commit: String,
}
