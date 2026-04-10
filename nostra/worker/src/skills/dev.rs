use super::Skill;
use anyhow::Result;
use async_trait::async_trait;

pub struct Dev;

impl Default for Dev {
    fn default() -> Self {
        Self::new()
    }
}

impl Dev {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Skill for Dev {
    fn name(&self) -> &str {
        "Developer"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec!["coding", "implementation", "debugging"]
    }

    async fn execute(&self, payload: String) -> Result<String> {
        println!("   [Dev] Implementing code changes...");
        Ok(format!("Commit: Implemented '{}'.", payload))
    }
}
