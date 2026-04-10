use super::Skill;
use crate::kip_client::KipClient;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

pub struct Gardener {
    kip: KipClient,
}

impl Gardener {
    pub fn new(kip: KipClient) -> Self {
        Self { kip }
    }

    async fn check_sleep_tasks(&self) -> Result<String> {
        // 1. Fetch all SleepTasks
        // KQL: FIND ?x WHERE { ?x {type: "SleepTask"} }
        let kql = r#"FIND ?x WHERE { ?x {type: "SleepTask"} }"#;
        let response = self.kip.query(kql).await?;

        // 2. Parse JSON response to find pending tasks
        // Expected format: {"results": [...], "count": N}
        let v: Value = serde_json::from_str(&response)?;
        let results = v["results"].as_array();

        if results.is_none() {
            return Ok("No results found".to_string());
        }

        let mut executed_count = 0;

        for entity in results.unwrap() {
            // Check attributes
            if let Some(attrs) = entity["attributes"].as_object() {
                let status = attrs
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending");

                if status == "pending" {
                    let wake_at_str = attrs.get("wake_at").and_then(|v| v.as_str());

                    if let Some(wake_at) = wake_at_str {
                        // Parse ISO string
                        // For MVP, just string compare if ISO format is strict, or use chrono
                        // Let's assume chrono is not in dependencies yet, so use simple string compare for MVP if format is YYYY-MM-DDTHH:MM:SSZ
                        let now = chrono::Utc::now().to_rfc3339();

                        if wake_at <= now.as_str() {
                            println!("   [Gardener] Waking up task: {}", entity["name"]);

                            // Execute Task Logic based on task_type
                            let task_type = attrs
                                .get("task_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");

                            match task_type {
                                "decay_confidence" => {
                                    println!("   [Gardener] Decaying confidence...");
                                    // Logic to decay confidence
                                }
                                "aggregate_views" => {
                                    println!("   [Gardener] Aggregating views...");
                                }
                                _ => println!("   [Gardener] Unknown task type: {}", task_type),
                            }

                            // Update Task Status to 'completed'
                            // UPSERT { CONCEPT ?t { {type: "SleepTask", name: "..."} SET ATTRIBUTES { status: "completed" } } }
                            let name = entity["name"].as_str().unwrap_or("");
                            let update_cmd = format!(
                                r#"UPSERT {{ CONCEPT ?t {{ {{type: "SleepTask", name: "{}"}} SET ATTRIBUTES {{ status: "completed" }} }} }}"#,
                                name
                            );

                            let _ = self.kip.mutate(&update_cmd).await;
                            executed_count += 1;
                        }
                    }
                }
            }
        }

        Ok(format!("Checked tasks. Executed: {}", executed_count))
    }
}

#[async_trait]
impl Skill for Gardener {
    fn name(&self) -> &str {
        "Gardener"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec![
            "schedule_maintenance",
            "prune_orphans",
            "aggregate_views",
            "check_sleep_tasks",
        ]
    }

    async fn execute(&self, payload: String) -> Result<String> {
        // Parse payload to see what action is requested
        // If payload is empty or "check", run check_sleep_tasks

        if payload.contains("check_sleep_tasks") {
            return self.check_sleep_tasks().await;
        }

        Ok("Gardener: Ready to serve. Use 'check_sleep_tasks' to poll.".to_string())
    }
}
