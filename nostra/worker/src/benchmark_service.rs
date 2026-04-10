use crate::agents::{AgentRunner, OllamaAgentRunner};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BenchmarkService {
    // In-memory state for now
    _active_runs: Arc<Mutex<Vec<String>>>,
}

impl Default for BenchmarkService {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkService {
    pub fn new() -> Self {
        Self {
            _active_runs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Run a benchmark with the default Ollama agent (llama3.1:8b)
    pub async fn run_benchmark(&self, case_id: &str) -> Result<String> {
        self.run_benchmark_with_model(case_id, "llama3.1:8b").await
    }

    /// Run a benchmark with a specific model
    pub async fn run_benchmark_with_model(&self, case_id: &str, model: &str) -> Result<String> {
        println!("Starting Benchmark Run: {} with model {}", case_id, model);

        // Create agent runner from config
        let agent: Arc<dyn AgentRunner> = Arc::new(OllamaAgentRunner::from_config(model));

        // Call the workflow logic
        let result = crate::workflows::benchmark::execute_benchmark(case_id, agent).await?;

        println!(
            "Benchmark Completed. Passed: {}, Score: {:.2}",
            result.passed, result.score
        );

        Ok(format!(
            "run_{}_completed_passed={}_score={:.2}",
            case_id, result.passed, result.score
        ))
    }
}
