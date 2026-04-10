#![allow(dead_code)]

//! Streaming Client (JS Bridge)
//! Wraps the window.NostraStreaming JS adapter.

use dioxus::document::eval;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub msg_type: String,
    pub content: String,
    pub conversation_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StreamEvent {
    #[serde(rename = "message")]
    Message(ChatMessage),
    #[serde(rename = "state")]
    State(String),
}

/// Initialize the streaming listener and return a channel for events
pub fn use_streaming_listener() -> Coroutine<StreamEvent> {
    use_coroutine(|_rx: UnboundedReceiver<StreamEvent>| async move {
        // This is a placeholder.
        // Actual implementation requires mapping the eval recv loop to a Dioxus coroutine or signal.
        // But use_coroutine is for SENDING into it.
        // We want to RECEIVE from JS.
    })
}

/// Connect to the streaming gateway
pub fn connect(gateway_url: &str, canister_id: &str, ic_url: &str) {
    let script = format!(
        "window.NostraStreaming.connect('{}', '{}', '{}');",
        gateway_url, canister_id, ic_url
    );
    let _ = eval(&script);
}

/// Send a message
pub fn send_message(content: &str) {
    // Escape quotes in content
    let safe_content = content.replace("'", "\\'"); // Basic escaping
    let script = format!("window.NostraStreaming.send('{}');", safe_content);
    let _ = eval(&script);
}
