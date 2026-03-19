use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub simulate: bool,
    pub space_id: String,
    pub user_did: String,
}

impl WorkflowContext {
    pub fn new(space_id: &str, user_did: &str, simulate: bool) -> Self {
        Self {
            simulate,
            space_id: space_id.to_string(),
            user_did: user_did.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStepKind {
    Task,
    Command,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    pub index: usize,
    pub raw: String,
    pub kind: WorkflowStepKind,
}

pub fn build_execution_plan(content: &str) -> Vec<WorkflowStep> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(idx, line)| WorkflowStep {
            index: idx + 1,
            raw: line.to_string(),
            kind: classify_step(line),
        })
        .collect()
}

pub fn classify_step(line: &str) -> WorkflowStepKind {
    if line.starts_with("- [ ]") {
        WorkflowStepKind::Task
    } else if line.starts_with('>') {
        WorkflowStepKind::Command
    } else {
        WorkflowStepKind::Note
    }
}

pub fn start_message(total_steps: usize) -> String {
    format!("[Executor] Starting Workflow ({} steps)...", total_steps)
}

pub fn step_analysis_message(step: &WorkflowStep) -> String {
    format!("[Executor] Step {}: Analyzing '{}'", step.index, step.raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_classifies_expected_step_kinds() {
        let plan = build_execution_plan(
            r#"
- [ ] write tests
> snapshot create pre local
plain note
"#,
        );
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].kind, WorkflowStepKind::Task);
        assert_eq!(plan[1].kind, WorkflowStepKind::Command);
        assert_eq!(plan[2].kind, WorkflowStepKind::Note);
    }
}
