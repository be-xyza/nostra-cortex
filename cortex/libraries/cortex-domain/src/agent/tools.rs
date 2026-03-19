use async_trait::async_trait;
use serde_json::Value;

/// An individual tool that an agent can invoke.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    /// A required JSON Schema for the tool parameters.
    fn input_schema(&self) -> Value;

    /// Execute the tool asynchronously
    async fn execute(&self, params: Value) -> Result<Value, String>;
}

/// A dynamic Toolset generator patterned off Model Context Protocol (MCP).
/// Instead of hard-coded tools, Toolsets dynamically generate tools based on current context.
#[async_trait]
pub trait Toolset: Send + Sync {
    /// Retrieve tools relevant to current execution context.
    /// This allows dynamic enums (e.g. valid agent routing names at runtime).
    async fn get_tools(&self, space_id: &str) -> Result<Vec<Box<dyn Tool>>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock_test_tool"
        }
        fn description(&self) -> &str {
            "A mock tool for tests"
        }
        fn input_schema(&self) -> Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "test_param": { "type": "string" }
                }
            })
        }
        async fn execute(&self, _params: Value) -> Result<Value, String> {
            Ok(serde_json::json!({ "status": "success" }))
        }
    }

    struct MockMcpToolset;

    #[async_trait]
    impl Toolset for MockMcpToolset {
        async fn get_tools(&self, _space_id: &str) -> Result<Vec<Box<dyn Tool>>, String> {
            Ok(vec![Box::new(MockTool)])
        }
    }

    #[tokio::test]
    async fn mcp_toolset_dynamically_generates_schemas() {
        let toolset = MockMcpToolset;
        let tools = toolset.get_tools("space_test").await.unwrap();

        assert_eq!(tools.len(), 1);
        let first_tool = &tools[0];

        assert_eq!(first_tool.name(), "mock_test_tool");
        let schema = first_tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("test_param").is_some());
    }
}
