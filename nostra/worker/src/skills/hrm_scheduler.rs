use super::Skill;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command; // Async Command
use tokio::time::{Duration, sleep};

pub struct HrmScheduler;

impl Default for HrmScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl HrmScheduler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Skill for HrmScheduler {
    fn name(&self) -> &str {
        "HrmScheduler"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec!["schedule_optimization", "job_rotation", " HRM_solver"]
    }

    async fn execute(&self, payload: String) -> Result<String> {
        println!("   [HrmScheduler] Receiving task: {}", payload);

        // Parse payload (validation step)
        let _json: Value = serde_json::from_str(&payload)?;

        // Supervisor Pattern: Retry Loop
        let max_retries = 3;
        let mut last_error = String::from("Unknown error");

        for attempt in 1..=max_retries {
            if attempt > 1 {
                println!(
                    "   [HrmScheduler] Retry attempt {}/{}...",
                    attempt, max_retries
                );
                sleep(Duration::from_millis(500 * attempt as u64)).await; // Backoff
            }

            // Execute the adapter in `run_demo` mode for Prototype Phase
            // Note: Updated to use tokio::process::Command for non-blocking execution
            let child = Command::new("python3")
                .env("DISABLE_COMPILE", "1")
                .arg("labs/hrm_scheduler/adapter.py")
                .arg("--checkpoint")
                .arg("labs/hrm_scheduler/checkpoint_sudoku")
                .current_dir("../../")
                .output()
                .await; // Async wait

            match child {
                Ok(out) => {
                    if out.status.success() {
                        let result = String::from_utf8_lossy(&out.stdout);
                        return Ok(format!(
                            "HRM Scheduled Result (Attempt {}):\n{}",
                            attempt, result
                        ));
                    } else {
                        let err = String::from_utf8_lossy(&out.stderr);
                        println!("   [HrmScheduler] Subprocess failed: {}", err);
                        last_error = err.to_string();
                    }
                }
                Err(e) => {
                    println!("   [HrmScheduler] Failed to spawn: {}", e);
                    last_error = e.to_string();
                }
            }
        }

        Err(anyhow!(
            "HRM Execution Failed after {} retries. Last error: {}",
            max_retries,
            last_error
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hrm_demo_execution() {
        let skill = HrmScheduler::new();
        // Payload doesn't matter for demo mode yet, but must be valid JSON
        let result = skill.execute("{}".to_string()).await;

        match result {
            Ok(output) => {
                println!("{}", output);
                assert!(output.contains("HRM Scheduled Result"));
                // The grid output format might vary slightly in spacing/newlines
                assert!(output.contains("1  2  6"));
            }
            Err(e) => panic!("HRM execution failed: {}", e),
        }
    }
}
