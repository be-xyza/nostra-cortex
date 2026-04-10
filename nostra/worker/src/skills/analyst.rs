use super::Skill;
use crate::kip_client::KipClient;
use anyhow::Result;
use async_trait::async_trait;

pub struct Analyst {
    kip: KipClient,
}

impl Analyst {
    pub fn new(kip: KipClient) -> Self {
        Self { kip }
    }
}

#[async_trait]
impl Skill for Analyst {
    fn name(&self) -> &str {
        "Analyst"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec!["research", "synthesis", "feedback_analysis", "kip_query"]
    }

    async fn execute(&self, payload: String) -> Result<String> {
        println!("   [Analyst] Synthesizing feedback from payload...");

        // demonstrate KIP query
        println!("   [Analyst] Querying Knowledge Graph to contextualize...");
        // In real scenario, we'd construct KQL based on payload
        let kql = r#"SEARCH "Nostra""#;
        match self.kip.query(kql).await {
            Ok(json) => {
                let preview: String = json.chars().take(100).collect();
                println!("   [Analyst] KIP Context: {}...", preview);
            }
            Err(e) => println!("   [Analyst] KIP Error: {}", e),
        }

        // Call OpenAI here (mocked)
        Ok(format!(
            "Report: Analysis of '{}' complete (Contextualized via KIP).",
            payload
        ))
    }
}
