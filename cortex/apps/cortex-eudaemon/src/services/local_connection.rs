use crate::services::ic_client::LocalIcClient;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkContext {
    Local,
    Disconnected,
}
impl std::fmt::Display for NetworkContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkContext::Local => write!(f, "Local IC Host"),
            NetworkContext::Disconnected => write!(f, "Disconnected"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub context: NetworkContext,
    pub message: String,
}

pub struct LocalConnection {
    client: LocalIcClient,
}
impl LocalConnection {
    pub fn new() -> Self {
        Self {
            client: LocalIcClient::new(None),
        }
    }
    pub async fn connect(&self) -> Result<ConnectionStatus, String> {
        if !LocalIcClient::is_installed() {
            return Ok(ConnectionStatus {
                context: NetworkContext::Disconnected,
                message: "icp-cli not installed".to_string(),
            });
        }

        let version = std::process::Command::new("icp")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        if self.client.is_replica_running().await {
            let message = version
                .as_deref()
                .map(|v| format!("Connected to local IC host ({v})"))
                .unwrap_or_else(|| "Connected to local IC host".to_string());
            return Ok(ConnectionStatus {
                context: NetworkContext::Local,
                message,
            });
        }
        let message = version
            .as_deref()
            .map(|v| format!("No local IC host running. ({v})"))
            .unwrap_or_else(|| "No local IC host running.".to_string());
        Ok(ConnectionStatus {
            context: NetworkContext::Disconnected,
            message,
        })
    }
}
