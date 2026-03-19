//! System Operation Handlers
//!
//! Implements the SystemOp primitives for the Workflow Engine:
//! - Graph.CreateNode / Graph.LinkNodes
//! - Ledger.Transfer (Mock)
//! - Notification.Send

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a system operation execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemOpResult {
    Success { output: Option<String> },
    Failure { error: String },
}

/// Handler for SystemOp primitives.
pub struct SystemOpHandler {
    /// Mock ledger balances for testing.
    ledger_balances: HashMap<String, u64>,
    /// Mock graph node registry.
    graph_nodes: Vec<String>,
    /// Transaction counter for mock IDs.
    tx_count: u64,
}

impl Default for SystemOpHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemOpHandler {
    /// Create a new system op handler.
    pub fn new() -> Self {
        Self {
            ledger_balances: HashMap::new(),
            graph_nodes: Vec::new(),
            tx_count: 0,
        }
    }

    /// Execute a system operation.
    pub fn execute(&mut self, op_type: &str, payload: &str) -> SystemOpResult {
        match op_type {
            "Graph.CreateNode" => self.create_node(payload),
            "Graph.LinkNodes" => self.link_nodes(payload),
            "Ledger.Transfer" => self.transfer(payload),
            "Notification.Send" => self.send_notification(payload),
            _ => SystemOpResult::Failure {
                error: format!("Unknown operation type: {}", op_type),
            },
        }
    }

    /// Create a node in the knowledge graph.
    fn create_node(&mut self, payload: &str) -> SystemOpResult {
        // Parse payload as JSON
        let parsed: Result<NodePayload, _> = serde_json::from_str(payload);
        match parsed {
            Ok(node) => {
                let node_id = format!("node_{}", self.graph_nodes.len() + 1);
                self.graph_nodes.push(node_id.clone());
                log::info!("Created graph node: {} (type: {})", node_id, node.node_type);
                SystemOpResult::Success {
                    output: Some(node_id),
                }
            }
            Err(e) => {
                // Fallback: treat payload as node ID directly
                let node_id = format!("node_{}", self.graph_nodes.len() + 1);
                self.graph_nodes.push(node_id.clone());
                log::debug!("Created node with raw payload (parse error: {})", e);
                SystemOpResult::Success {
                    output: Some(node_id),
                }
            }
        }
    }

    /// Link two nodes in the knowledge graph.
    fn link_nodes(&mut self, payload: &str) -> SystemOpResult {
        let parsed: Result<LinkPayload, _> = serde_json::from_str(payload);
        match parsed {
            Ok(link) => {
                log::info!(
                    "Linked nodes: {} --[{}]--> {}",
                    link.source,
                    link.relation,
                    link.target
                );
                SystemOpResult::Success {
                    output: Some(format!("link_{}_{}", link.source, link.target)),
                }
            }
            Err(e) => SystemOpResult::Failure {
                error: format!("Invalid link payload: {}", e),
            },
        }
    }

    /// Transfer tokens between accounts (mock implementation).
    fn transfer(&mut self, payload: &str) -> SystemOpResult {
        let parsed: Result<TransferPayload, _> = serde_json::from_str(payload);
        match parsed {
            Ok(transfer) => {
                // Check sender balance
                let sender_balance = self
                    .ledger_balances
                    .get(&transfer.from)
                    .copied()
                    .unwrap_or(0);

                if sender_balance < transfer.amount {
                    return SystemOpResult::Failure {
                        error: format!(
                            "Insufficient balance: {} has {} but needs {}",
                            transfer.from, sender_balance, transfer.amount
                        ),
                    };
                }

                // Execute transfer
                *self
                    .ledger_balances
                    .entry(transfer.from.clone())
                    .or_insert(0) -= transfer.amount;
                *self.ledger_balances.entry(transfer.to.clone()).or_insert(0) += transfer.amount;

                log::info!(
                    "Transferred {} from {} to {}",
                    transfer.amount,
                    transfer.from,
                    transfer.to
                );

                self.tx_count += 1;
                SystemOpResult::Success {
                    output: Some(format!("tx_{}", self.tx_count)),
                }
            }
            Err(e) => SystemOpResult::Failure {
                error: format!("Invalid transfer payload: {}", e),
            },
        }
    }

    /// Send a notification (mock implementation).
    fn send_notification(&self, payload: &str) -> SystemOpResult {
        let parsed: Result<NotificationPayload, _> = serde_json::from_str(payload);
        match parsed {
            Ok(notif) => {
                log::info!(
                    "Notification sent to {:?}: {}",
                    notif.recipients,
                    notif.message
                );
                SystemOpResult::Success { output: None }
            }
            Err(_) => {
                // Best-effort: log raw payload
                log::info!("Notification (raw): {}", payload);
                SystemOpResult::Success { output: None }
            }
        }
    }

    /// Set a mock balance for testing.
    pub fn set_balance(&mut self, account: &str, amount: u64) {
        self.ledger_balances.insert(account.to_string(), amount);
    }

    /// Get a balance for testing.
    pub fn get_balance(&self, account: &str) -> u64 {
        self.ledger_balances.get(account).copied().unwrap_or(0)
    }

    /// Get the list of created nodes.
    pub fn get_nodes(&self) -> &[String] {
        &self.graph_nodes
    }
}

// --- Payload Types ---

#[derive(Debug, Deserialize)]
struct NodePayload {
    node_type: String,
    #[serde(default)]
    #[allow(dead_code)]
    properties: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct LinkPayload {
    source: String,
    target: String,
    relation: String,
}

#[derive(Debug, Deserialize)]
struct TransferPayload {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Debug, Deserialize)]
struct NotificationPayload {
    recipients: Vec<String>,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let mut handler = SystemOpHandler::new();
        let result = handler.execute(
            "Graph.CreateNode",
            r#"{"node_type": "Contribution", "properties": {"title": "Test"}}"#,
        );
        assert!(matches!(result, SystemOpResult::Success { .. }));
        assert_eq!(handler.get_nodes().len(), 1);
    }

    #[test]
    fn test_link_nodes() {
        let mut handler = SystemOpHandler::new();
        let result = handler.execute(
            "Graph.LinkNodes",
            r#"{"source": "node_1", "target": "node_2", "relation": "depends_on"}"#,
        );
        assert!(matches!(result, SystemOpResult::Success { .. }));
    }

    #[test]
    fn test_transfer_success() {
        let mut handler = SystemOpHandler::new();
        handler.set_balance("alice", 1000);

        let result = handler.execute(
            "Ledger.Transfer",
            r#"{"from": "alice", "to": "bob", "amount": 100}"#,
        );
        assert!(matches!(result, SystemOpResult::Success { .. }));
        assert_eq!(handler.get_balance("alice"), 900);
        assert_eq!(handler.get_balance("bob"), 100);
    }

    #[test]
    fn test_transfer_insufficient_funds() {
        let mut handler = SystemOpHandler::new();
        handler.set_balance("alice", 50);

        let result = handler.execute(
            "Ledger.Transfer",
            r#"{"from": "alice", "to": "bob", "amount": 100}"#,
        );
        assert!(matches!(result, SystemOpResult::Failure { .. }));
        assert_eq!(handler.get_balance("alice"), 50); // Unchanged
    }
}
