use async_trait::async_trait;
use cortex_domain::agent::tools::{Tool, Toolset};
use serde_json::Value;
use std::sync::Arc;

use crate::services::mcp::client::McpClient;
use crate::services::mcp::policy::McpPolicyGate;

pub struct McpToolsetAdapter {
    client: Arc<McpClient>,
    policy: Arc<McpPolicyGate>,
}

impl McpToolsetAdapter {
    pub fn new(client: Arc<McpClient>, policy: Arc<McpPolicyGate>) -> Self {
        Self { client, policy }
    }
}

pub struct DynamicMcpTool {
    name: String,
    description: String,
    input_schema: Value,
    client: Arc<McpClient>,
    policy: Arc<McpPolicyGate>,
}

#[async_trait]
impl Tool for DynamicMcpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn execute(&self, params: Value) -> Result<Value, String> {
        // Enforce Phase 5 Governance Security
        self.policy.validate_tool_call(&self.name, &params)?;

        let result = self
            .client
            .call_tool(&self.name, Some(params))
            .await
            .map_err(|e| e.to_string())?;

        if result.is_error {
            return Err(format!("Tool {} returned an error", self.name));
        }

        let mut out = String::new();
        for content in result.content {
            match content {
                cortex_domain::agent::mcp::protocol::ToolContent::Text { text } => {
                    out.push_str(&text);
                    out.push('\n');
                }
                _ => {
                    out.push_str("[Non-text content omitted]\n");
                }
            }
        }

        Ok(Value::String(out))
    }
}

#[async_trait]
impl Toolset for McpToolsetAdapter {
    async fn get_tools(&self, _space_id: &str) -> Result<Vec<Box<dyn Tool>>, String> {
        let mut tools: Vec<Box<dyn Tool>> = Vec::new();
        match self.client.list_tools().await {
            Ok(list) => {
                for mcp_tool in list.tools {
                    tools.push(Box::new(DynamicMcpTool {
                        name: mcp_tool.name,
                        description: mcp_tool.description.unwrap_or_default(),
                        input_schema: mcp_tool.input_schema,
                        client: self.client.clone(),
                        policy: self.policy.clone(),
                    }));
                }
            }
            Err(e) => {
                tracing::error!("Failed to list MCP tools: {:?}", e);
                return Err(e.to_string());
            }
        }
        Ok(tools)
    }
}
