use crate::services::agent_service::{AgentService, ChatEvent};
use axum::extract::ws::{Message, WebSocket};
use chrono::{DateTime, Utc};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatClientAttachmentDescriptor {
    pub name: String,
    pub r#type: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatClientMessage {
    pub text: String,
    pub context_refs: Vec<String>,
    pub attachments: Vec<ChatClientAttachmentDescriptor>,
    pub thread_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ChatClientEnvelope {
    #[serde(rename = "message")]
    Message {
        text: String,
        #[serde(rename = "contextRefs", default)]
        context_refs: Vec<String>,
        #[serde(default)]
        attachments: Vec<ChatClientAttachmentDescriptor>,
        #[serde(rename = "threadId", default)]
        thread_id: Option<String>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ChatServerEnvelope<'a> {
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "streaming")]
    Streaming {
        id: &'a str,
        delta: &'a str,
        timestamp: String,
    },
    #[serde(rename = "message")]
    Message {
        id: &'a str,
        text: String,
        timestamp: String,
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
            context_refs,
            attachments,
            thread_id,
        } => {
            if text.trim().is_empty() && attachments.is_empty() {
                return Err("message text or attachments are required".to_string());
            }
            Ok(ChatClientMessage {
                text,
                context_refs,
                attachments,
                thread_id,
            })
        }
    }
}

pub fn translate_chat_events(message_id: &str, events: &[ChatEvent]) -> Vec<Value> {
    let mut frames = vec![json!(ChatServerEnvelope::Processing)];
    if events.is_empty() {
        frames.push(json!(ChatServerEnvelope::Error {
            code: "empty_response",
            message: "Chat service returned no content.",
        }));
        return frames;
    }

    if events.len() == 1 {
        let event = &events[0];
        frames.push(json!(ChatServerEnvelope::Message {
            id: message_id,
            text: event.message.clone(),
            timestamp: format_chat_timestamp(event.timestamp),
        }));
        return frames;
    }

    let mut accumulated = String::new();
    for event in &events[..events.len() - 1] {
        accumulated.push_str(&event.message);
        frames.push(json!(ChatServerEnvelope::Streaming {
            id: message_id,
            delta: event.message.as_str(),
            timestamp: format_chat_timestamp(event.timestamp),
        }));
    }

    if let Some(last) = events.last() {
        accumulated.push_str(&last.message);
        frames.push(json!(ChatServerEnvelope::Message {
            id: message_id,
            text: accumulated,
            timestamp: format_chat_timestamp(last.timestamp),
        }));
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
            json!(ChatServerEnvelope::Processing).to_string().into(),
        ))
        .await
        .map_err(|_| ())?;

    let mut stream = match AgentService::send_chat_message(
        "User".to_string(),
        build_agent_prompt(&request),
        None,
        true,
    )
    .await
    {
        Ok(stream) => stream,
        Err(err) => {
            send_error_envelope(sender, "gateway_error", err.as_str()).await?;
            return Ok(());
        }
    };

    let message_id = uuid::Uuid::new_v4().to_string();
    let mut accumulated = String::new();
    let mut pending: Option<ChatEvent> = None;

    while let Some(event) = stream.next().await {
        match event {
            Ok(next_event) => {
                if let Some(previous) = pending.replace(next_event) {
                    accumulated.push_str(&previous.message);
                    sender
                        .send(Message::Text(
                            json!(ChatServerEnvelope::Streaming {
                                id: message_id.as_str(),
                                delta: previous.message.as_str(),
                                timestamp: format_chat_timestamp(previous.timestamp),
                            })
                            .to_string()
                            .into(),
                        ))
                        .await
                        .map_err(|_| ())?;
                }
            }
            Err(err) => {
                send_error_envelope(sender, "gateway_error", err.as_str()).await?;
                return Ok(());
            }
        }
    }

    if let Some(last) = pending {
        accumulated.push_str(&last.message);
        sender
            .send(Message::Text(
                json!(ChatServerEnvelope::Message {
                    id: message_id.as_str(),
                    text: accumulated,
                    timestamp: format_chat_timestamp(last.timestamp),
                })
                .to_string()
                .into(),
            ))
            .await
            .map_err(|_| ())?;
    } else {
        send_error_envelope(sender, "empty_response", "Chat service returned no content.").await?;
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

fn build_agent_prompt(request: &ChatClientMessage) -> String {
    let mut sections = Vec::new();

    if !request.context_refs.is_empty() {
        sections.push(format!(
            "Context references: {}",
            request.context_refs.join(", ")
        ));
    }

    if !request.attachments.is_empty() {
        let attachment_summary = request
            .attachments
            .iter()
            .map(|attachment| format!("{} ({}, {} bytes)", attachment.name, attachment.r#type, attachment.size))
            .collect::<Vec<_>>()
            .join("; ");
        sections.push(format!("Attachments: {attachment_summary}"));
    }

    sections.push(format!("User request: {}", request.text.trim()));

    sections.join("\n")
}

fn format_chat_timestamp(timestamp: i64) -> String {
    DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(Utc::now)
        .to_rfc3339()
}
