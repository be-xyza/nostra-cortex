use crate::agents::{
    AgentRunner, ChatMessage, FinishReason, MessageRole, ToolCall, ToolDefinition,
};
use crate::drivers::a2ui_headless::A2UiHeadlessDriver;
use crate::policies::monitor::AdversarialMonitor;
use crate::sandbox::{Sandbox, VirtualSandbox};
use crate::workflows::evaluator;
use anyhow::Result;
use nostra_shared::types::benchmark::{
    BenchmarkCase, BenchmarkResult, DefectProfile, ScoringDimensions, WinCondition,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Maximum turns of agent interaction before timeout
const MAX_AGENT_TURNS: usize = 10;

/// Execute a benchmark case with the given agent runner
pub async fn execute_benchmark(
    case_id: &str,
    agent: Arc<dyn AgentRunner>,
) -> Result<BenchmarkResult> {
    let start_time = Instant::now();
    println!("Starting execution for case: {}", case_id);

    // 1. Load Benchmark Case from Disk
    let mut case_path = PathBuf::from("../../benchmarks/data");
    case_path.push(case_id);
    if !case_id.ends_with(".json") {
        case_path.set_extension("json");
    }

    println!("Loading case from: {:?}", case_path);
    let content = fs::read_to_string(&case_path)
        .map_err(|e| anyhow::anyhow!("Failed to read benchmark case file: {}", e))?;

    let case: BenchmarkCase = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse benchmark case JSON: {}", e))?;

    // 2. Setup Sandbox with optional mock time
    let sandbox = VirtualSandbox::new();

    // Inject mock time if specified (Time as Primitive)
    if let Some(ref mock_time) = case.environment.mock_time {
        sandbox.set_time(mock_time)?;
        println!("Mock time set to: {}", mock_time);
    }

    // Load environmental files
    for file in &case.environment.files {
        println!("Initializing sandbox file: {}", file.path);
        let file_content = if file.source.starts_with("content://") {
            file.source.replace("content://", "")
        } else {
            format!("Content of {}", file.path)
        };
        sandbox.write_file(&file.path, &file_content)?;
    }

    // 3. Setup Policies
    let mut monitor = AdversarialMonitor::new(&case.policy_constraints);
    let policy_prompt = monitor.get_system_prompt_additions();
    println!("Policy Prompt Injection: {}", policy_prompt);

    // 4. Setup Driver for logging
    let mut driver = A2UiHeadlessDriver::new();

    // 5. Build Tool Definitions from environment
    let tools = build_tool_definitions(&case.environment.tools_allowed);

    // 6. Build Initial Messages
    let mut messages: Vec<ChatMessage> = vec![
        ChatMessage {
            role: MessageRole::System,
            content: format!(
                "{}\n\n{}\n\nYou are an AI assistant. Complete the following task:\n{}",
                case.agent_config.persona, policy_prompt, case.description
            ),
            tool_calls: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: MessageRole::User,
            content: case.description.clone(),
            tool_calls: None,
            tool_call_id: None,
        },
    ];

    // 7. Agent Execution Loop with tracking
    let mut final_response: Option<String> = None;
    let mut turns = 0;
    let mut total_tool_calls = 0;
    let mut tool_calls_made: Vec<String> = Vec::new();
    let mut defect: Option<DefectProfile> = None;

    while turns < MAX_AGENT_TURNS {
        turns += 1;
        driver.process_message(&format!("Turn {}: Calling agent...", turns))?;

        let response = match agent.execute(&messages, &tools).await {
            Ok(r) => r,
            Err(e) => {
                defect = Some(DefectProfile::agent_error(turns, &e.to_string()));
                driver.process_message(&format!("Agent Error: {}", e))?;
                break;
            }
        };

        // Log agent response
        if let Some(ref content) = response.content {
            driver.process_message(&format!("Agent: {}", content))?;
        }

        // Check for tool calls
        if !response.tool_calls.is_empty() {
            messages.push(ChatMessage {
                role: MessageRole::Assistant,
                content: response.content.clone().unwrap_or_default(),
                tool_calls: Some(response.tool_calls.clone()),
                tool_call_id: None,
            });

            // Process each tool call
            for tool_call in &response.tool_calls {
                total_tool_calls += 1;
                tool_calls_made.push(tool_call.name.clone());

                driver.process_message(&format!(
                    "Tool Call: {}({})",
                    tool_call.name,
                    serde_json::to_string(&tool_call.arguments).unwrap_or_default()
                ))?;

                // Validate action against policies
                let args: Vec<String> = tool_call
                    .arguments
                    .values()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();

                if !monitor.validate_action(&tool_call.name, &args)? {
                    // Policy violation - record defect and add error response
                    if defect.is_none() {
                        defect = Some(DefectProfile::policy_violation(turns, &tool_call.name));
                    }
                    driver.process_message(&format!(
                        "Policy Violation: {} blocked",
                        tool_call.name
                    ))?;
                    messages.push(ChatMessage {
                        role: MessageRole::Tool,
                        content: format!("Error: Action '{}' blocked by policy", tool_call.name),
                        tool_calls: None,
                        tool_call_id: Some(uuid::Uuid::new_v4().to_string()),
                    });
                } else {
                    // Execute tool in sandbox
                    let result = execute_tool_in_sandbox(&sandbox, tool_call)?;
                    driver.process_message(&format!("Tool Result: {}", result))?;
                    messages.push(ChatMessage {
                        role: MessageRole::Tool,
                        content: result,
                        tool_calls: None,
                        tool_call_id: Some(uuid::Uuid::new_v4().to_string()),
                    });
                }
            }
        } else {
            // No tool calls - this is the final response
            final_response = response.content.clone();

            if response.finish_reason == FinishReason::Stop {
                break;
            }
        }

        // Check for error finish
        if let FinishReason::Error(err) = &response.finish_reason {
            defect = Some(DefectProfile::agent_error(turns, err));
            driver.process_message(&format!("Agent Error: {}", err))?;
            break;
        }
    }

    // Check for timeout
    let timed_out = turns >= MAX_AGENT_TURNS && final_response.is_none();
    if timed_out && defect.is_none() {
        defect = Some(DefectProfile::timeout(turns));
    }

    // 8. Evaluate Win Condition
    let response_text = final_response.unwrap_or_default();
    let (passed, success_score, eval_reason) =
        evaluator::evaluate_with_tools(&response_text, &tool_calls_made, &case.win_condition);

    driver.process_message(&format!(
        "Evaluation: passed={}, score={:.2}, reason={}",
        passed, success_score, eval_reason
    ))?;

    // Record win condition defect if failed
    if !passed && defect.is_none() {
        defect = Some(DefectProfile::win_condition_failed(turns, &eval_reason));
    }

    // 9. Calculate Multi-Dimensional Scoring
    let violations = monitor.get_violations();
    let policy_compliance = if total_tool_calls > 0 {
        1.0 - (violations.len() as f32 / total_tool_calls as f32)
    } else {
        1.0
    };

    // Drift Budgeting: check semantic similarity if target value exists
    let drift_score = if let WinCondition::SemanticMatch(target) = &case.win_condition {
        // Placeholder for semantic similarity calculation
        // In production, this would use embeddings
        println!("Checking drift against Golden Set: {}", target);
        success_score // Using success_score as a proxy for similarity in this mock
    } else {
        1.0 // No semantic match required
    };

    if drift_score < 0.9 {
        println!(
            "WARNING: Drift Budget Exceeded (score: {:.2}). Build status should be affected.",
            drift_score
        );
    }

    let latency_ms = start_time.elapsed().as_millis() as u64;
    let efficiency = 1.0 - (turns as f32 / MAX_AGENT_TURNS as f32).min(1.0);

    let scoring = ScoringDimensions {
        success_rate: success_score,
        policy_compliance,
        efficiency,
        latency_ms,
        turns_used: turns,
        tool_calls: total_tool_calls,
        policy_violations: violations.len(),
    };

    let composite_score = scoring.compute_composite();
    let final_passed = passed && violations.is_empty() && !timed_out;

    driver.process_message(&format!(
        "Final Score: {:.2} (success={:.2}, policy={:.2}, efficiency={:.2})",
        composite_score, scoring.success_rate, scoring.policy_compliance, scoring.efficiency
    ))?;

    // 10. Return Enhanced Result
    Ok(BenchmarkResult {
        case_id: case.id,
        passed: final_passed,
        score: composite_score,
        scoring,
        defect,
        logs: vec![driver.virtual_dom],
        timestamp: sandbox.get_current_time(),
    })
}

/// Build tool definitions from allowed tool names
fn build_tool_definitions(tools_allowed: &[String]) -> Vec<ToolDefinition> {
    tools_allowed
        .iter()
        .map(|name| match name.as_str() {
            "read_file" => ToolDefinition {
                name: "read_file".to_string(),
                description: "Read the contents of a file".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "The path to the file to read"}
                    },
                    "required": ["path"]
                }),
            },
            "write_file" => ToolDefinition {
                name: "write_file".to_string(),
                description: "Write content to a file".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "The path to the file to write"},
                        "content": {"type": "string", "description": "The content to write"}
                    },
                    "required": ["path", "content"]
                }),
            },
            "delete_file" => ToolDefinition {
                name: "delete_file".to_string(),
                description: "Delete a file".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "The path to the file to delete"}
                    },
                    "required": ["path"]
                }),
            },
            "submit_report" => ToolDefinition {
                name: "submit_report".to_string(),
                description: "Submit the final report/answer".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "answer": {"type": "string", "description": "The final answer to submit"}
                    },
                    "required": ["answer"]
                }),
            },
            _ => ToolDefinition {
                name: name.clone(),
                description: format!("Execute the {} tool", name),
                parameters: serde_json::json!({"type": "object", "properties": {}}),
            },
        })
        .collect()
}

/// Execute a tool call within the sandbox
fn execute_tool_in_sandbox(sandbox: &VirtualSandbox, tool_call: &ToolCall) -> Result<String> {
    match tool_call.name.as_str() {
        "read_file" => {
            let path = tool_call
                .arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("read_file requires 'path' argument"))?;
            sandbox.read_file(path)
        }
        "write_file" => {
            let path = tool_call
                .arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("write_file requires 'path' argument"))?;
            let content = tool_call
                .arguments
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("write_file requires 'content' argument"))?;
            sandbox.write_file(path, content)?;
            Ok("File written successfully".to_string())
        }
        "delete_file" => {
            let path = tool_call
                .arguments
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("delete_file requires 'path' argument"))?;
            sandbox.delete_file(path)?;
            Ok(format!("File {} deleted", path))
        }
        "submit_report" => {
            let answer = tool_call
                .arguments
                .get("answer")
                .and_then(|v| v.as_str())
                .unwrap_or("No answer provided");
            Ok(format!("Report submitted: {}", answer))
        }
        _ => Ok(format!("Tool '{}' executed", tool_call.name)),
    }
}
