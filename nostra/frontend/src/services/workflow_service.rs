use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowInstance {
    pub id: String,
    pub status: String,
    pub current_step: Option<String>,
    pub history: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingTask {
    pub instance_id: String,
    pub step_id: String,
    pub description: String,
    pub a2ui_schema: Option<String>,
}

pub struct WorkflowService {
    base_url: String,
}

impl WorkflowService {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:3003".to_string(), // Worker API
        }
    }

    pub async fn start_workflow(&self, template: &str) -> Result<String> {
        let resp = reqwest::Client::new()
            .post(format!("{}/workflows/start/{}", self.base_url, template))
            .send()
            .await?;

        let json: serde_json::Value = resp.json().await?;
        Ok(json["workflow_id"].as_str().unwrap_or_default().to_string())
    }

    pub async fn get_pending_tasks(&self) -> Result<Vec<PendingTask>> {
        let tasks = reqwest::get(format!("{}/tasks", self.base_url))
            .await?
            .json::<Vec<PendingTask>>()
            .await?;
        Ok(tasks)
    }

    pub async fn get_workflow_details(&self, id: &str) -> Result<WorkflowInstance> {
        let instance = reqwest::get(format!("{}/workflows/{}", self.base_url, id))
            .await?
            .json::<WorkflowInstance>()
            .await?;
        Ok(instance)
    }

    pub async fn complete_task(&self, instance_id: &str, payload: String) -> Result<()> {
        let client = reqwest::Client::new();
        // Try to parse as JSON map, enable simple text otherwise
        let map: HashMap<String, String> = serde_json::from_str(&payload).unwrap_or_else(|_| {
            let mut m = HashMap::new();
            m.insert("input".to_string(), payload);
            m
        });

        let body = serde_json::json!({ "payload": map });

        client
            .post(format!("{}/tasks/{}/complete", self.base_url, instance_id))
            .json(&body)
            .send()
            .await?;

        Ok(())
    }

    pub async fn cancel_workflow(&self, instance_id: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let _res = client
            .post(format!(
                "{}/workflows/{}/cancel",
                self.base_url, instance_id
            ))
            .send()
            .await?;
        Ok(())
    }

    pub async fn retry_workflow(&self, instance_id: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let _res = client
            .post(format!("{}/workflows/{}/retry", self.base_url, instance_id))
            .send()
            .await?;
        Ok(())
    }

    pub async fn generate_workflow(&self, intention: &str) -> Result<(String, String)> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/workflows/generate", self.base_url))
            .json(&serde_json::json!({ "intention": intention }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let workflow_json = res["workflow_json"].as_str().unwrap_or("").to_string();
        let preview = res["preview"].as_str().unwrap_or("").to_string();
        Ok((workflow_json, preview))
    }
}
