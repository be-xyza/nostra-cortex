use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    #[serde(flatten, default)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseOutputMessage {
    #[serde(rename = "type")]
    pub item_type: ResponseOutputItemType,
    #[serde(flatten, default)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResponseOutputItem {
    FunctionCall(ResponseOutputFunctionCall),
    Message(ResponseOutputMessage),
    Other(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseModel {
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub output: Vec<ResponseOutputItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponsesEventEnvelope {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub response: Option<ResponseModel>,
    #[serde(default)]
    pub delta: Option<String>,
    #[serde(flatten, default)]
    pub extra: Map<String, Value>,
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
