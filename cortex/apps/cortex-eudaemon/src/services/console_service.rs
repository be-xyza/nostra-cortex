use chrono::Local;
use std::sync::OnceLock;
use tokio::sync::broadcast;

#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleContent {
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleMessage {
    pub sender: String,
    pub content: ConsoleContent,
    pub timestamp: String,
}

// Global Console stream.
static CONSOLE_CHANNEL: OnceLock<broadcast::Sender<ConsoleMessage>> = OnceLock::new();

pub fn get_console_tx() -> broadcast::Sender<ConsoleMessage> {
    CONSOLE_CHANNEL
        .get_or_init(|| {
            let (tx, _) = broadcast::channel(200);
            tx
        })
        .clone()
}

pub fn send_text(sender: &str, text: impl Into<String>) {
    let msg = ConsoleMessage {
        sender: sender.to_string(),
        content: ConsoleContent::Text(text.into()),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
    };
    let _ = get_console_tx().send(msg);
}
