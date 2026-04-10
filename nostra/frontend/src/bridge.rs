#![allow(dead_code)]

use dioxus::document::eval;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum GraphDelta {
    AddNode {
        id: String,
        label: String,
        x: f64,
        y: f64,
        size: f64,
        color: String,
        #[serde(default)]
        timestamp: u64, // Nanoseconds since epoch for timeline filtering
    },
    AddLink {
        id: String,
        source: String,
        target: String,
        size: f64,
        color: String,
        label: String,
    },
    Clear,
}

pub struct GraphBridge;

impl GraphBridge {
    pub fn new() -> Self {
        Self
    }

    pub fn init(&self, container_id: &str) {
        let _ = eval(&format!("window.initGraph('{}');", container_id));
    }

    pub fn apply_deltas(&self, deltas: Vec<GraphDelta>) {
        if let Ok(json) = serde_json::to_string(&deltas) {
            let _ = eval(&format!("window.updateGraph({});", json));
        } else {
            web_sys::console::error_1(&"Failed to serialize graph deltas".into());
        }
    }

    pub fn set_layout(&self, layout: &str) {
        let _ = eval(&format!("window.setGraphLayout('{}');", layout));
    }

    pub fn set_graph_mode(&self, mode: &str) {
        let _ = eval(&format!("window.setGraphMode('{}');", mode));
    }

    pub fn set_label_mode(&self, mode: &str) {
        let _ = eval(&format!("window.setLabelMode('{}');", mode));
    }

    /// Set timeline filter by percentage (0-100)
    pub fn set_timeline_percent(&self, percent: u64) {
        let _ = eval(&format!("window.setTimelinePercent({});", percent));
    }

    /// Toggle play/pause for timeline animation
    pub fn toggle_timeline_play(&self) {
        let _ = eval("window.toggleTimelinePlay();");
    }

    /// Search/filter nodes in the graph
    pub fn search_graph(&self, query: &str) {
        let escaped = query.replace("'", "\\'");
        let _ = eval(&format!("window.searchGraph('{}');", escaped));
    }
}
