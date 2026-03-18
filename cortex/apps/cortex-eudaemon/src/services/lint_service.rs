use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LintReport {
    pub timestamp: String,
    pub status: String, // "PASSED" | "FAILED"
    pub summary: Summary,
    #[serde(default)]
    pub violations: Vec<Violation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Summary {
    pub errors: usize,
    pub warnings: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Violation {
    pub file: String,
    pub message: String,
    pub rule: String,
    pub severity: String,
}

pub struct LintService;

impl LintService {
    pub async fn get_report() -> LintReport {
        match Self::read_report() {
            Ok(report) => report,
            Err(e) => {
                println!("Error reading lint report: {}", e);
                LintReport {
                    status: "UNKNOWN".to_string(),
                    ..Default::default()
                }
            }
        }
    }

    fn read_report() -> anyhow::Result<LintReport> {
        // Try to find the report in the workspace root
        // We assume we are running from a target directory or the project root
        let path = Self::find_report_file()?;
        let content = fs::read_to_string(path)?;
        let report: LintReport = serde_json::from_str(&content)?;
        Ok(report)
    }

    fn find_report_file() -> anyhow::Result<PathBuf> {
        // Simple heuristic: look in current dir and up to 3 parents
        let filename = "nostra-lint-report.json";
        let mut current_dir = std::env::current_dir()?;

        for _ in 0..4 {
            let target = current_dir.join(filename);
            if target.exists() {
                return Ok(target);
            }
            if !current_dir.pop() {
                break;
            }
        }

        anyhow::bail!("Report file not found")
    }
}
