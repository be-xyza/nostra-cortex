use crate::services::agent_service::{
    AgentService, ChatContentPart, ChatContextBlock, ChatEvent, ChatSourceAnchor,
    RuntimeChatRequest,
};
use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq)]
pub struct ChatClientMessage {
    pub text: String,
    pub context_block_ids: Vec<String>,
    pub source_anchor: Option<ChatSourceAnchor>,
    pub thread_id: Option<String>,
    pub space_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChatClientContextEnvelope {
    #[serde(default)]
    block_ids: Vec<String>,
    #[serde(default)]
    source_anchor: Option<ChatSourceAnchor>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ChatClientEnvelope {
    #[serde(rename = "message")]
    Message {
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        content: Vec<Value>,
        #[serde(rename = "contextRefs", default)]
        context_refs: Vec<String>,
        #[serde(default)]
        context: Option<ChatClientContextEnvelope>,
        #[serde(rename = "threadId", default)]
        thread_id: Option<String>,
        #[serde(rename = "spaceId", default)]
        space_id: Option<String>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ChatServerEnvelope<'a> {
    #[serde(rename = "processing")]
    Processing {
        #[serde(skip_serializing_if = "Option::is_none")]
        agent: Option<&'a crate::services::agent_service::ChatAgentIdentity>,
    },
    #[serde(rename = "streaming")]
    Streaming {
        id: &'a str,
        delta: &'a str,
        timestamp: String,
        agent: &'a crate::services::agent_service::ChatAgentIdentity,
    },
    #[serde(rename = "message")]
    Message {
        id: &'a str,
        text: String,
        timestamp: String,
        content: &'a [ChatContentPart],
        agent: &'a crate::services::agent_service::ChatAgentIdentity,
    },
    #[serde(rename = "error")]
    Error {
        code: &'a str,
        message: &'a str,
    },
}

pub fn decode_chat_client_message(raw: &str) -> Result<ChatClientMessage, String> {
    let envelope = serde_json::from_str::<ChatClientEnvelope>(raw)
        .map_err(|err| format!("invalid chat envelope: {err}"))?;

    match envelope {
        ChatClientEnvelope::Message {
            text,
            content,
            context_refs,
            context,
            thread_id,
            space_id,
        } => {
            let extracted_text = extract_text_from_content(&content)
                .or(text)
                .unwrap_or_default()
                .trim()
                .to_string();
            if extracted_text.is_empty() {
                return Err("message text is required".to_string());
            }
            let mut context_block_ids = context_refs;
            if let Some(envelope) = context.as_ref() {
                if envelope.block_ids.is_empty() {
                    // Preserve legacy grounding when the envelope only carries source metadata.
                } else {
                    for block_id in &envelope.block_ids {
                        if !context_block_ids.iter().any(|existing| existing == block_id) {
                            context_block_ids.push(block_id.clone());
                        }
                    }
                }
            }
            Ok(ChatClientMessage {
                text: extracted_text,
                context_block_ids,
                source_anchor: context.and_then(|entry| entry.source_anchor),
                thread_id,
                space_id,
            })
        }
    }
}

fn extract_text_from_content(content: &[Value]) -> Option<String> {
    let text = content
        .iter()
        .filter_map(|part| {
            let obj = part.as_object()?;
            if obj.get("type").and_then(|value| value.as_str()) != Some("text") {
                return None;
            }
            obj.get("text")
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn translate_chat_events(events: &[ChatEvent]) -> Vec<Value> {
    let mut frames = vec![json!(ChatServerEnvelope::Processing { agent: None })];
    if events.is_empty() {
        frames.push(json!(ChatServerEnvelope::Error {
            code: "empty_response",
            message: "Chat service returned no content.",
        }));
        return frames;
    }

    let assistant_message_id = events.iter().find_map(|event| match event {
        ChatEvent::Completed { response, .. } => Some(response.response_id.as_str()),
        _ => None,
    });

    for event in events {
        match event {
            ChatEvent::TextDelta {
                message,
                timestamp,
                agent,
                ..
            } => frames.push(json!(ChatServerEnvelope::Streaming {
                id: assistant_message_id.unwrap_or("chat-stream"),
                delta: message.as_str(),
                timestamp: format_chat_timestamp(*timestamp),
                agent,
            })),
            ChatEvent::Completed {
                response,
                timestamp,
            } => frames.push(json!(ChatServerEnvelope::Message {
                id: response.response_id.as_str(),
                text: response.text.clone(),
                timestamp: format_chat_timestamp(*timestamp),
                content: response.content.as_slice(),
                agent: &response.agent,
            })),
        }
    }

    frames
}

pub async fn handle_chat_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if handle_chat_text_message(&mut sender, text.as_str())
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(_) => break,
        }
    }
}

async fn handle_chat_text_message(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    raw_text: &str,
) -> Result<(), ()> {
    let request = match decode_chat_client_message(raw_text) {
        Ok(request) => request,
        Err(err) => {
            send_error_envelope(sender, "invalid_request", &err).await?;
            return Ok(());
        }
    };

    sender
        .send(Message::Text(
            json!(ChatServerEnvelope::Processing { agent: None })
                .to_string()
                .into(),
        ))
        .await
        .map_err(|_| ())?;

    let thread_id = request
        .thread_id
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("thread-{}", uuid::Uuid::new_v4()));
    let context_blocks = crate::gateway::server::build_heap_context_bundle_for_block_ids(
        &request.context_block_ids,
    )
    .into_iter()
    .filter_map(|value| serde_json::from_value::<ChatContextBlock>(value).ok())
    .collect::<Vec<_>>();
    let history = crate::gateway::server::load_chat_thread_history(&thread_id);
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());
    let agent_message_id = format!("msg-{}", uuid::Uuid::new_v4());

    if let Err(err) = crate::gateway::server::persist_chat_message_parts(
        request
            .space_id
            .as_deref()
            .filter(|value| !value.trim().is_empty()),
        &thread_id,
        &message_id,
        "user",
        request.source_anchor.clone(),
        &[ChatContentPart::Text {
            text: request.text.clone(),
        }],
        None,
    ) {
        send_error_envelope(sender, "persistence_error", err.as_str()).await?;
        return Ok(());
    }

    let mut stream = match AgentService::send_chat_message(RuntimeChatRequest {
        author: "User".to_string(),
        text: request.text.clone(),
        space_id: request.space_id.clone(),
        thread_id: Some(thread_id.clone()),
        source_anchor: request.source_anchor.clone(),
        context_blocks,
        history,
        streaming: true,
    })
    .await
    {
        Ok(stream) => stream,
        Err(err) => {
            send_error_envelope(sender, "gateway_error", err.as_str()).await?;
            return Ok(());
        }
    };

    while let Some(event) = stream.next().await {
        match event {
            Ok(ChatEvent::TextDelta {
                message,
                timestamp,
                agent,
                ..
            }) => {
                sender
                    .send(Message::Text(
                        json!(ChatServerEnvelope::Streaming {
                            id: agent_message_id.as_str(),
                            delta: message.as_str(),
                            timestamp: format_chat_timestamp(timestamp),
                            agent: &agent,
                        })
                        .to_string()
                        .into(),
                    ))
                    .await
                    .map_err(|_| ())?;
            }
            Ok(ChatEvent::Completed {
                response,
                timestamp,
            }) => {
                if let Err(err) = crate::gateway::server::persist_chat_message_parts(
                    request
                        .space_id
                        .as_deref()
                        .filter(|value| !value.trim().is_empty()),
                    &thread_id,
                    &agent_message_id,
                    "agent",
                    request.source_anchor.clone(),
                    response.content.as_slice(),
                    Some(response.agent.clone()),
                ) {
                    send_error_envelope(sender, "persistence_error", err.as_str()).await?;
                    return Ok(());
                }
                sender
                    .send(Message::Text(
                        json!(ChatServerEnvelope::Message {
                            id: agent_message_id.as_str(),
                            text: response.text.clone(),
                            timestamp: format_chat_timestamp(timestamp),
                            content: response.content.as_slice(),
                            agent: &response.agent,
                        })
                        .to_string()
                        .into(),
                    ))
                    .await
                    .map_err(|_| ())?;
            }
            Err(err) => {
                send_error_envelope(sender, "gateway_error", err.as_str()).await?;
                return Ok(());
            }
        }
    }

    Ok(())
}

async fn send_error_envelope(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    code: &str,
    message: &str,
) -> Result<(), ()> {
    sender
        .send(Message::Text(
            json!(ChatServerEnvelope::Error { code, message })
                .to_string()
                .into(),
        ))
        .await
        .map_err(|_| ())
}

fn format_chat_timestamp(timestamp: i64) -> String {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(now_iso)
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_socket_request_parser_accepts_message_envelope() {
        let parsed = decode_chat_client_message(
            r#"{"type":"message","content":[{"type":"text","text":"Summarize this"}],"context":{"blockIds":["artifact-1"]},"threadId":"thread-1","spaceId":"nostra-space"}"#,
        )
        .expect("message envelope");

        assert_eq!(parsed.text, "Summarize this");
        assert_eq!(parsed.context_block_ids, vec!["artifact-1".to_string()]);
        assert_eq!(parsed.thread_id.as_deref(), Some("thread-1"));
        assert_eq!(parsed.space_id.as_deref(), Some("nostra-space"));
    }

    #[test]
    fn chat_socket_request_parser_accepts_legacy_text_and_context_refs() {
        let parsed = decode_chat_client_message(
            r#"{"type":"message","text":"Legacy message","contextRefs":["artifact-2"],"threadId":"thread-2","context":{"sourceAnchor":{"kind":"view","label":"Explore","href":"/explore","routeId":"/explore"}}}"#,
        )
        .expect("legacy envelope");

        assert_eq!(parsed.text, "Legacy message");
        assert_eq!(parsed.context_block_ids, vec!["artifact-2".to_string()]);
        assert_eq!(parsed.thread_id.as_deref(), Some("thread-2"));
        assert_eq!(
            parsed.source_anchor,
            Some(ChatSourceAnchor {
                kind: "view".to_string(),
                label: "Explore".to_string(),
                href: "/explore".to_string(),
                route_id: Some("/explore".to_string()),
                artifact_id: None,
                view_id: None,
                block_id: None,
                component_id: None,
            })
        );
    }

    #[test]
    fn chat_socket_request_parser_rejects_blank_message_text() {
        let error = decode_chat_client_message(
            r#"{"type":"message","content":[{"type":"text","text":"   "}],"threadId":"thread-blank"}"#,
        )
        .expect_err("blank text should fail");

        assert_eq!(error, "message text is required");
    }

    #[test]
    fn chat_socket_translator_emits_streaming_and_terminal_message() {
        let agent = crate::services::agent_service::ChatAgentIdentity {
            id: "provider".to_string(),
            label: "Cortex Runtime".to_string(),
            route: "provider-runtime.responses".to_string(),
            mode: "runtime".to_string(),
        };
        let events = vec![
            ChatEvent::TextDelta {
                author: "Cortex Runtime".to_string(),
                message: "Hello ".to_string(),
                timestamp: 1,
                agent: agent.clone(),
            },
            ChatEvent::Completed {
                response: crate::services::agent_service::RuntimeChatResponse {
                    response_id: "chat-1".to_string(),
                    text: "Hello world".to_string(),
                    content: vec![ChatContentPart::Text {
                        text: "Hello world".to_string(),
                    }],
                    agent,
                },
                timestamp: 2,
            },
        ];

        let frames = translate_chat_events(&events);

        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0]["type"], "processing");
        assert_eq!(frames[1]["type"], "streaming");
        assert_eq!(frames[1]["id"], "chat-1");
        assert_eq!(frames[1]["delta"], "Hello ");
        assert_eq!(frames[2]["type"], "message");
        assert_eq!(frames[2]["id"], "chat-1");
        assert_eq!(frames[2]["text"], "Hello world");
    }

    #[test]
    fn chat_socket_translator_returns_processing_plus_empty_response_error_for_no_events() {
        let frames = translate_chat_events(&[]);

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0]["type"], "processing");
        assert_eq!(frames[1]["type"], "error");
        assert_eq!(frames[1]["code"], "empty_response");
    }

    #[test]
    fn chat_socket_translator_uses_fallback_stream_id_without_completed_event() {
        let agent = crate::services::agent_service::ChatAgentIdentity {
            id: "provider".to_string(),
            label: "Cortex Runtime".to_string(),
            route: "provider-runtime.responses".to_string(),
            mode: "runtime".to_string(),
        };
        let events = vec![ChatEvent::TextDelta {
            author: "Cortex Runtime".to_string(),
            message: "Partial".to_string(),
            timestamp: 1,
            agent,
        }];

        let frames = translate_chat_events(&events);

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[1]["type"], "streaming");
        assert_eq!(frames[1]["id"], "chat-stream");
        assert_eq!(frames[1]["delta"], "Partial");
    }
}
