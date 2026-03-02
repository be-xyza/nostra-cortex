use cortex_domain::capabilities::navigation_graph::{
    CapabilityEdge, CapabilityId, CapabilityNode, EdgeRelationship, IntentType,
    PlatformCapabilityGraph,
};
use std::path::PathBuf;
use tracing::{info, warn};

/// Service responsible for scanning the local workspace and injecting discovered
/// capabilities into the Navigation Graph. Integrates with the Invariant Engine
/// conceptually by verifying bounding contexts before node creation.
pub struct CapabilitiesScanner;

impl CapabilitiesScanner {
    pub fn scan_and_build() -> PlatformCapabilityGraph {
        let mut graph = PlatformCapabilityGraph::new();
        let workspace_root = resolve_workspace_root();

        info!("Scanning for localized capabilities to build Navigation Graph");

        if !scanner_fallback_enabled() {
            warn!(
                "CapabilitiesScanner fallback is disabled; returning baseline root-only graph. \
Enable CORTEX_CAPABILITY_GRAPH_SCANNER_ENABLED=1 for explicit fallback scanning."
            );
            let root_id = CapabilityId("cortex.workbench.root".to_string());
            graph.unverified_add_node(CapabilityNode {
                id: root_id,
                name: "Cortex Workbench".to_string(),
                description: "Capability scanner fallback root".to_string(),
                intent_type: IntentType::Monitor,
                root_path: None,
                invariant_violations: vec![],
            });
            return graph;
        }

        // 1. Root / Core Capability
        let root_id = CapabilityId("cortex.core.system".to_string());
        graph.unverified_add_node(CapabilityNode {
            id: root_id.clone(),
            name: "Cortex Native System".to_string(),
            description: "Root kernel of the Cortex Daemon ecosystem".to_string(),
            intent_type: IntentType::Monitor,
            root_path: None,
            invariant_violations: vec![],
        });

        // 2. Headless Daemon Sandbox (Demonstrating architectural layout)
        let daemon_id = CapabilityId("cortex.daemon.sandbox".to_string());
        graph.unverified_add_node(CapabilityNode {
            id: daemon_id.clone(),
            name: "Cortex Daemon (Headless)".to_string(),
            description: "MVK-aligned gateway server execution context".to_string(),
            intent_type: IntentType::Execute,
            root_path: Some(
                workspace_root
                    .join("cortex/apps/cortex-eudaemon")
                    .display()
                    .to_string(),
            ),
            invariant_violations: vec![],
        });

        graph.unverified_add_edge(CapabilityEdge {
            source: root_id.clone(),
            target: daemon_id.clone(),
            relationship: EdgeRelationship::ChildOf,
        });

        // 3. Cortex Web (Visualizer Shell)
        let web_ui_id = CapabilityId("cortex.web.shell".to_string());
        graph.unverified_add_node(CapabilityNode {
            id: web_ui_id.clone(),
            name: "Cortex Web UI".to_string(),
            description: "D3/React Flow Capability Visualizer and A2UI bridge".to_string(),
            intent_type: IntentType::Visualize,
            root_path: Some(
                workspace_root
                    .join("cortex/apps/cortex-web")
                    .display()
                    .to_string(),
            ),
            invariant_violations: vec![cortex_domain::integrity::PolicyViolation {
                policy_id: "ORG-004-SPA-COMPLEXITY".to_string(),
                severity: "Warning".to_string(),
                message: "React SPA introduces high bundle complexity compared to raw DOM"
                    .to_string(),
                affected_nodes: vec![],
            }],
        });

        graph.unverified_add_edge(CapabilityEdge {
            source: root_id.clone(),
            target: web_ui_id.clone(),
            relationship: EdgeRelationship::ChildOf,
        });

        // 4. Invariant Engine Hook (Demonstrating policy monitoring context)
        let invariant_id = CapabilityId("nostra.invariant.engine".to_string());
        graph.unverified_add_node(CapabilityNode {
            id: invariant_id.clone(),
            name: "Invariant Engine".to_string(),
            description: "Enforces MVK and Execution constraints on capabilities".to_string(),
            intent_type: IntentType::Mutate,
            root_path: Some(workspace_root.join("nostra/extraction").display().to_string()),
            invariant_violations: vec![],
        });

        graph.unverified_add_edge(CapabilityEdge {
            source: root_id.clone(),
            target: invariant_id.clone(),
            relationship: EdgeRelationship::ProvidesContextTo,
        });

        // Example: Scanning active workspaces (Simulated for initial phase)
        let workspace_id = CapabilityId("nostra.gaming.protocol".to_string());
        graph.unverified_add_node(CapabilityNode {
            id: workspace_id.clone(),
            name: "Nostra Gaming Protocol (Init 049)".to_string(),
            description: "Active research and Godot compute layer".to_string(),
            intent_type: IntentType::Execute,
            root_path: Some(
                workspace_root
                    .join("research/049-nostra-gaming-protocol")
                    .display()
                    .to_string(),
            ),
            invariant_violations: vec![],
        });

        graph.unverified_add_edge(CapabilityEdge {
            source: root_id.clone(),
            target: workspace_id.clone(),
            relationship: EdgeRelationship::ChildOf,
        });

        info!("Built Navigation Graph with {} nodes", graph.nodes.len());
        graph
    }
}

fn resolve_workspace_root() -> PathBuf {
    if let Ok(root) = std::env::var("NOSTRA_WORKSPACE_ROOT") {
        let trimmed = root.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn scanner_fallback_enabled() -> bool {
    std::env::var("CORTEX_CAPABILITY_GRAPH_SCANNER_ENABLED")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}
