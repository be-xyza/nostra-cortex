use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<&str>;
    async fn execute(&self, payload: String) -> Result<String>;
}

pub mod analyst;
pub mod architect;
pub mod dev;
pub mod extraction;
pub mod gardener;
pub mod hrm_scheduler;
pub mod librarian;
pub mod pm;
pub mod qa;
