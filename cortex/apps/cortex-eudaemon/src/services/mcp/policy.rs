use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct McpPolicyConfig {
    pub allowed_tools: HashSet<String>,
    pub space_id: String,
}

#[derive(Debug, Clone)]
pub struct McpPolicyGate {
    config: McpPolicyConfig,
}

impl McpPolicyGate {
    pub fn new(config: McpPolicyConfig) -> Self {
        Self { config }
    }

    pub fn validate_tool_call(&self, tool_name: &str, _params: &Value) -> Result<(), String> {
        // Governance Rule 1: Tool Allowlisting
        // A tool must be explicitly allowed for this SpaceId
        // Allows all if empty, or enforce strict. For MVK, if it's empty, we deny all for strict security,
        // or allow all for dev? Let's say if `allowed_tools` is empty, it's explicitly denying all.
        // Wait, for testing we might want a wildcard or just assume we populate it.
        // We will do a strict check: must contain the tool name or wildcard "*".
        if !self.config.allowed_tools.contains(tool_name)
            && !self.config.allowed_tools.contains("*")
        {
            tracing::warn!(
                "MCP Governance Blocked Tool: '{}' in space '{}'",
                tool_name,
                self.config.space_id
            );
            return Err(format!(
                "MCP Governance Violation: Tool '{}' is not allowed in space '{}'.",
                tool_name, self.config.space_id
            ));
        }

        // Future: Inspect _params for A2UI / HITL gating on sensitive operations
        Ok(())
    }
}
