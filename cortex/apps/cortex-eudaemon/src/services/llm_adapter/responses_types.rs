use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseOutputItemType {
    Message,
    FunctionCall,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseOutputFunctionCall {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub item_type: ResponseOutputItemType,
    pub name: Option<String>,
    pub arguments: Option<String>,
    pub call_id: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseModel {
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub output: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponsesEventEnvelope {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub response: Option<ResponseModel>,
    #[serde(flatten)]
    #[serde(default)]
    pub extra: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallRequest {
    pub call_id: String,
    pub name: String,
    pub arguments_json: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedResponse {
    pub response_id: String,
    pub full_text: String,
    pub tool_calls: Vec<ToolCallRequest>,
    pub raw: Value,
}
