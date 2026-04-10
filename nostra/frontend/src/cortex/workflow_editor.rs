use dioxus::prelude::*;

#[component]
pub fn WorkflowEditor(on_node_select: EventHandler<String>) -> Element {
    // Initial State: "The Void" with some nodes
    // In a real app, this would fetch from 'workflow-engine' canister

    rsx! {
        div { class: "w-full h-full relative overflow-auto p-10",

            // Connecting Lines (SVG Layer)
            // Absolute positioned behind nodes
            svg { class: "absolute top-0 left-0 w-full h-full pointer-events-none z-0",
                // Edge: Start -> Vote
                path {
                    d: "M250 150 C 350 150, 350 150, 450 150",
                    stroke: "#334155",
                    stroke_width: "2",
                    fill: "none",
                    class: "animate-[dash_1s_linear_infinite]" // "Ants" animation placeholder
                }
            }

            // Node 1: Start
            div {
                class: "absolute top-[120px] left-[100px] w-[150px] bg-[#1E293B] border border-[#334155] rounded-lg shadow-lg z-10 cursor-pointer hover:border-[#F8FAFC] transition-colors",
                onclick: move |_| on_node_select.call("start_node".to_string()),

                // Header (System Type)
                div { class: "h-2 w-full bg-[#A855F7] rounded-t-lg" } // Purple for System

                div { class: "p-3",
                    div { class: "font-['Fira_Code'] text-sm font-bold", "Start" }
                    div { class: "text-xs text-[#94A3B8] mt-1", "Manual Trigger" }
                }
            }

            // Node 2: Governance Vote (User Task)
            div {
                class: "absolute top-[120px] left-[450px] w-[180px] bg-[#1E293B] border border-[#22C55E]/30 rounded-lg shadow-lg z-10 cursor-pointer hover:border-[#22C55E] transition-colors",
                onclick: move |_| on_node_select.call("vote_node".to_string()),

                // Header (User Type)
                div { class: "h-2 w-full bg-[#3B82F6] rounded-t-lg" } // Blue for User

                div { class: "p-3",
                    div { class: "flex justify-between items-center",
                        div { class: "font-['Fira_Code'] text-sm font-bold", "Vote" }
                        div { class: "w-2 h-2 rounded-full bg-[#22C55E] animate-pulse" } // Active status
                    }
                    div { class: "text-xs text-[#94A3B8] mt-1", "Approve Proposal" }
                    div { class: "mt-2 inline-block px-2 py-0.5 bg-[#3B82F6]/10 text-[#3B82F6] text-[10px] rounded", "Waiting" }
                }
            }
        }
    }
}
