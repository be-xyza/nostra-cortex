//! Nostra Streaming Canister
//!
//! Real-time WebSocket streaming for AI chat responses.
//! Based on validated Echo Streaming Prototype (038).

use candid::CandidType;
use ic_cdk::{init, post_upgrade, query, update};
use ic_websocket_cdk::{
    CanisterWsCloseArguments, CanisterWsCloseResult, CanisterWsGetMessagesArguments,
    CanisterWsGetMessagesResult, CanisterWsMessageArguments, CanisterWsMessageResult,
    CanisterWsOpenArguments, CanisterWsOpenResult, ClientPrincipal, OnCloseCallbackArgs,
    OnMessageCallbackArgs, OnOpenCallbackArgs, WsHandlers, WsInitParams,
};
use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

/// Message format for client <-> canister communication
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ChatMessage {
    /// Message type: "user_message" | "ai_token" | "ai_complete" | "error"
    pub msg_type: String,
    /// Message content (text for user, token for AI streaming)
    pub content: String,
    /// Optional conversation ID for multi-thread support
    pub conversation_id: Option<String>,
}

/// A2UI streaming event (for Phase 2: A2UI integration)
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct A2UIEvent {
    /// Event type: "surfaceUpdate" | "dataModelUpdate" | "beginRendering"
    pub event_type: String,
    /// JSON payload matching A2UI protocol
    pub payload: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ChatTask {
    pub input: String,
    pub client_principal: ClientPrincipal,
    pub conversation_id: Option<String>,
}

use std::cell::RefCell;

thread_local! {
    static PENDING_TASKS: RefCell<Vec<ChatTask>> = RefCell::new(Vec::new());
}

// ============================================================================
// Initialization
// ============================================================================

#[init]
fn init() {
    let handlers = WsHandlers {
        on_open: Some(on_open),
        on_message: Some(on_message),
        on_close: Some(on_close),
    };

    let params = WsInitParams::new(handlers)
        .with_max_number_of_returned_messages(50)
        .with_send_ack_interval_ms(300_000);

    ic_websocket_cdk::init(params);
}

#[post_upgrade]
fn post_upgrade() {
    init();
}

// ============================================================================
// WebSocket Service Interface (required by ic-websocket-cdk)
// ============================================================================

#[update]
fn ws_open(args: CanisterWsOpenArguments) -> CanisterWsOpenResult {
    ic_websocket_cdk::ws_open(args)
}

#[update]
fn ws_close(args: CanisterWsCloseArguments) -> CanisterWsCloseResult {
    ic_websocket_cdk::ws_close(args)
}

#[update]
fn ws_message(
    args: CanisterWsMessageArguments,
    msg_type: Option<ChatMessage>,
) -> CanisterWsMessageResult {
    ic_websocket_cdk::ws_message(args, msg_type)
}

#[query]
fn ws_get_messages(args: CanisterWsGetMessagesArguments) -> CanisterWsGetMessagesResult {
    ic_websocket_cdk::ws_get_messages(args)
}

/// Allow external services (backend/worker) to push messages to clients
#[update]
fn send_message_to_client(client: ClientPrincipal, msg: ChatMessage) {
    send_chat_message(client, &msg.msg_type, &msg.content, msg.conversation_id);
}

#[update]
fn pop_chat_task() -> Option<ChatTask> {
    PENDING_TASKS.with(|tasks| tasks.borrow_mut().pop())
}

// ============================================================================
// Callbacks
// ============================================================================

fn on_open(args: OnOpenCallbackArgs) {
    ic_cdk::println!("Client connected: {:?}", args.client_principal);
}

fn on_message(args: OnMessageCallbackArgs) {
    let msg: ChatMessage = match candid::decode_one(&args.message) {
        Ok(m) => m,
        Err(e) => {
            ic_cdk::println!("Failed to decode message: {:?}", e);
            send_error(args.client_principal, "Invalid message format");
            return;
        }
    };

    ic_cdk::println!("Received: {:?}", msg);

    match msg.msg_type.as_str() {
        "user_message" => handle_user_message(args.client_principal, msg),
        _ => {
            send_error(args.client_principal, "Unknown message type");
        }
    }
}

fn on_close(args: OnCloseCallbackArgs) {
    ic_cdk::println!("Client disconnected: {:?}", args.client_principal);
}

// ============================================================================
// Message Handlers
// ============================================================================

/// Handle incoming user message - queue for AI Worker processing
fn handle_user_message(client: ClientPrincipal, msg: ChatMessage) {
    // Acknowledge receipt
    send_chat_message(
        client.clone(),
        "ai_token",
        "[Processing...]",
        msg.conversation_id.clone(),
    );

    // Queue message for AI Worker
    let task = ChatTask {
        input: msg.content.clone(),
        client_principal: client.clone(),
        conversation_id: msg.conversation_id.clone(),
    };
    PENDING_TASKS.with(|tasks| tasks.borrow_mut().push(task));

    // NOTE: Streaming responses are handled by the AI Worker integration.
    // Prototype simulation removed to keep build warnings clean.
}

// ============================================================================
// Utility Functions
// ============================================================================

fn send_chat_message(
    client: ClientPrincipal,
    msg_type: &str,
    content: &str,
    conversation_id: Option<String>,
) {
    let msg = ChatMessage {
        msg_type: msg_type.to_string(),
        content: content.to_string(),
        conversation_id,
    };

    if let Ok(bytes) = candid::encode_one(msg) {
        if let Err(e) = ic_websocket_cdk::send(client, bytes) {
            ic_cdk::println!("Failed to send message: {:?}", e);
        }
    }
}

fn send_error(client: ClientPrincipal, error_msg: &str) {
    send_chat_message(client, "error", error_msg, None);
}

// ============================================================================
// Future: A2UI Streaming (Phase 2)
// ============================================================================

/// Send A2UI surface update event
#[allow(dead_code)]
fn send_a2ui_event(client: ClientPrincipal, event: A2UIEvent) {
    if let Ok(bytes) = candid::encode_one(event) {
        if let Err(e) = ic_websocket_cdk::send(client, bytes) {
            ic_cdk::println!("Failed to send A2UI event: {:?}", e);
        }
    }
}
