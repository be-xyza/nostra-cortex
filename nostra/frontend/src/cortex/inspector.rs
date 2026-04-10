use crate::cortex::components::inspector_trace::InspectorTrace;
use dioxus::prelude::*;

#[component]
pub fn Inspector(on_close: EventHandler<MouseEvent>) -> Element {
    // This component will eventually host the A2UI Renderer
    // For now, it shows the "Architect's" view of the selected node

    rsx! {
        div { class: "flex flex-col h-full",
            // Header
            div { class: "h-14 border-b border-[#334155] flex items-center justify-between px-4 bg-[#0F172A]",
                div { class: "flex items-center gap-2",
                    div { class: "w-2 h-2 rounded-full bg-[#3B82F6]" }
                    span { class: "font-['Orbitron'] font-bold text-sm tracking-wider", "INSPECTOR" }
                }
                button {
                    class: "p-1 hover:bg-[#1E293B] rounded text-[#94A3B8] hover:text-white transition-colors",
                    onclick: on_close,
                    svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                        path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M6 18L18 6M6 6l12 12" }
                    }
                }
            }

            // Body (A2UI Container)
            div { class: "flex-1 overflow-y-auto p-4 space-y-6",

                // Mock A2UI Render Output
                div { class: "space-y-2",
                    label { class: "text-xs font-semibold text-[#64748B] uppercase tracking-wider", "Task Description" }
                    div { class: "text-sm text-[#E2E8F0] leading-relaxed",
                        "Please review the proposal for 'Project Genesis'. Requires > 60% approval."
                    }
                }

                div { class: "space-y-4 pt-4 border-t border-[#334155]",
                    // Mock Form
                    div { class: "space-y-2",
                        label { class: "text-xs font-semibold text-[#64748B] uppercase tracking-wider", "Your Vote" }
                        div { class: "flex gap-2",
                            button { class: "flex-1 py-2 bg-[#22C55E]/10 border border-[#22C55E]/20 text-[#22C55E] rounded hover:bg-[#22C55E]/20 transition-colors", "Approve" }
                            button { class: "flex-1 py-2 bg-[#EF4444]/10 border border-[#EF4444]/20 text-[#EF4444] rounded hover:bg-[#EF4444]/20 transition-colors", "Reject" }
                        }
                    }
                }

                // Constitutional Audit (Glass Box View)
                 div { class: "mt-8 p-3 bg-[#1E293B] rounded border border-[#334155]",
                    div { class: "flex items-center gap-2 mb-2",
                         svg { class: "w-4 h-4 text-[#F59E0B]", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" }
                        }
                        span { class: "text-xs font-bold text-[#F59E0B]", "CONSTITUTIONAL AUDIT" }
                    }
                    // Render the Trace
                    InspectorTrace {
                        agent_name: "Librarian".to_string(),
                    }
                }

                // Ingestion Monitor (Phase 3)
                div { class: "p-3 bg-black/20 rounded border border-[#334155]",
                    div { class: "flex items-center gap-2 mb-2",
                         span { class: "text-xs font-bold text-[#22C55E]", "INGESTION STREAM" }
                         div { class: "px-1.5 py-0.5 rounded text-[10px] bg-[#22C55E]/20 text-[#22C55E] border border-[#22C55E]/30", "COMPLIANCE MODE" }
                    }
                    div { class: "space-y-2",
                        div { class: "flex items-center justify-between text-xs font-mono text-[#94A3B8]",
                            span { "chunk_123" }
                            span { class: "text-[#22C55E]", "INDEXED" }
                        }
                        div { class: "w-full h-1 bg-[#334155] rounded-full overflow-hidden",
                            div { class: "w-full h-full bg-[#22C55E]" }
                        }
                    }
                }
            }
        }
    }
}
