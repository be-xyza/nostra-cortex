use crate::services::vfs_service::{NodeType, VfsService};
use dioxus::prelude::*;
use serde::Deserialize;

#[derive(PartialEq, Clone, Props)]
pub struct TraceProps {
    pub agent_name: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct AuditTrace {
    #[serde(rename = "type")]
    trace_type: String,
    action: String,
    dpub_id: Option<String>,
    edition_id: Option<String>,
    timestamp: Option<String>,
    inputs: Option<serde_json::Value>,
    outputs: Option<serde_json::Value>,
}

#[component]
pub fn InspectorTrace(props: TraceProps) -> Element {
    let vfs = use_context::<VfsService>();
    let mut traces = use_signal(|| Vec::<AuditTrace>::new());

    use_effect(move || {
        let mut out = Vec::<AuditTrace>::new();
        for node in vfs.list_dir("/lib/audit_traces") {
            if !matches!(node.node_type, NodeType::File { .. }) {
                continue;
            }
            let path = format!("/lib/audit_traces/{}", node.name);
            if let Some(bytes) = vfs.read_file_bytes(&path) {
                if let Ok(t) = serde_json::from_slice::<AuditTrace>(&bytes) {
                    if t.trace_type.starts_with("audit_trace") {
                        out.push(t);
                    }
                }
            }
        }
        // newest-ish first (timestamp strings are either RFC3339 or ns string)
        out.reverse();
        traces.set(out);
    });

    rsx! {
        div { class: "flex flex-col gap-3 p-4 bg-gray-900 rounded-lg border border-gray-700",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    span { class: "text-blue-400 font-mono text-sm", "🤖 {props.agent_name}" }
                    span { class: "text-gray-500 text-xs", "audit_traces" }
                }
                div { class: "text-[10px] text-gray-500 uppercase tracking-wider", "{traces.read().len()} events" }
            }

            if traces.read().is_empty() {
                div { class: "text-xs text-gray-500", "No audit traces found yet. Publish an edition to generate a trace." }
            } else {
                div { class: "flex flex-col gap-2",
                    for t in traces.read().iter().take(10) {
                        div { class: "p-3 bg-black/30 rounded border border-gray-800",
                            div { class: "flex items-center justify-between text-xs text-gray-500 font-mono",
                                span { "{t.timestamp.clone().unwrap_or_else(|| \"\".to_string())}" }
                                span { class: "opacity-60", "{t.action}" }
                            }
                            if let Some(dpub) = t.dpub_id.as_ref() {
                                div { class: "text-xs text-gray-300 mt-1 truncate", "dpub: {dpub}" }
                            }
                            if let Some(edition) = t.edition_id.as_ref() {
                                div { class: "text-xs text-gray-300 truncate", "edition: {edition}" }
                            }
                            if let Some(outputs) = t.outputs.as_ref() {
                                div { class: "mt-2 text-[10px] text-green-400/80 font-mono overflow-x-auto",
                                    "<= {outputs}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
