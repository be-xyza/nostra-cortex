//! Manual benchmark runner for testing
//! Usage: cargo run --bin run_benchmark -- <case_id> [--mock]

use anyhow::Result;
use cortex_worker::agents::{AgentRunner, MockAgentRunner, OllamaAgentRunner};
use cortex_worker::workflows::benchmark::execute_benchmark;

use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let case_id = args.get(1).map(String::as_str).unwrap_or("gaia_val_001");
    let use_mock = args.iter().any(|s| s == "--mock");
    let model = args
        .get(2)
        .map(String::as_str)
        .filter(|s| !s.starts_with("--"))
        .unwrap_or("gemma3:4b");

    println!("=== Nostra Benchmark Runner ===");
    println!("Case ID: {}", case_id);

    // Create agent (mock for testing, Ollama for real runs)
    let agent: Arc<dyn AgentRunner> = if use_mock {
        println!("Using MockAgentRunner");
        Arc::new(MockAgentRunner::with_text_response("42"))
    } else {
        println!("Using OllamaAgentRunner ({})", model);
        Arc::new(OllamaAgentRunner::from_config(model))
    };

    println!("Agent: {} ({})", agent.name(), agent.model());
    println!("---");

    let result = execute_benchmark(case_id, agent).await?;

    println!();
    println!("=== Benchmark Result ===");
    println!("Case ID: {}", result.case_id);
    println!("Passed: {}", result.passed);
    println!("Composite Score: {:.2}", result.score);
    println!();

    // Display multi-dimensional scoring
    println!("--- Scoring Dimensions ---");
    println!(
        "  Success Rate:     {:.2}%",
        result.scoring.success_rate * 100.0
    );
    println!(
        "  Policy Compliance: {:.2}%",
        result.scoring.policy_compliance * 100.0
    );
    println!(
        "  Efficiency:        {:.2}%",
        result.scoring.efficiency * 100.0
    );
    println!("  Latency:          {}ms", result.scoring.latency_ms);
    println!("  Turns Used:       {}", result.scoring.turns_used);
    println!("  Tool Calls:       {}", result.scoring.tool_calls);
    println!("  Violations:       {}", result.scoring.policy_violations);

    // Display defect profile if present
    if let Some(ref defect) = result.defect {
        println!();
        println!("--- Defect Profile ---");
        println!("  Stage:       {:?}", defect.stage);
        println!("  Turn:        {}", defect.turn);
        println!("  Description: {}", defect.description);
        if let Some(ref ctx) = defect.context {
            println!("  Context:     {}", ctx);
        }
    }

    println!();
    println!("--- Logs ---");
    for log in &result.logs {
        for line in log.lines() {
            println!("  {}", line);
        }
    }

    Ok(())
}
