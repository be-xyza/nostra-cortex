use std::io;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tracing_subscriber::fmt::MakeWriter;

pub static LOG_BROADCAST: OnceLock<broadcast::Sender<String>> = OnceLock::new();

pub fn init_broadcast() -> broadcast::Sender<String> {
    let (tx, _) = broadcast::channel(1024);
    let _ = LOG_BROADCAST.set(tx.clone());
    tx
}

pub fn subscribe() -> broadcast::Receiver<String> {
    LOG_BROADCAST
        .get()
        .expect("Telemetry broadcast not initialized")
        .subscribe()
}

pub struct BroadcastWriter;

impl io::Write for BroadcastWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(sender) = LOG_BROADCAST.get() {
            if let Ok(s) = std::str::from_utf8(buf) {
                // Suppress empty sends or solely newline sends if desired
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    // Send to all connected websocket clients
                    let _ = sender.send(s.to_string());
                }
            }
        }
        // Always return success so tracing doesn't fail
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for BroadcastWriter {
    type Writer = BroadcastWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BroadcastWriter
    }
}
