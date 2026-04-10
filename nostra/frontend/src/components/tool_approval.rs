use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ToolApprovalProps {
    tool_name: String,
    args_json: String,
    tool_call_id: String,
    on_approve: EventHandler<String>, // Returns tool_call_id
    on_deny: EventHandler<String>,    // Returns tool_call_id
}

pub fn ToolApprovalCard(props: ToolApprovalProps) -> Element {
    let tool_name = props.tool_name;
    let args = props.args_json;
    let call_id = props.tool_call_id;

    // Formatting args for readability
    let formatted_args = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&args) {
        serde_json::to_string_pretty(&parsed).unwrap_or(args.clone())
    } else {
        args.clone()
    };

    rsx! {
        div { class: "border border-yellow-500/50 bg-yellow-500/5 rounded-lg p-4 my-2 animate-in fade-in slide-in-from-bottom-2",
            // Header
            div { class: "flex items-center gap-2 mb-3 text-yellow-600 dark:text-yellow-400 font-semibold",
                svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" }
                }
                "Tool Approval Request"
            }

            // Body
            div { class: "text-sm text-foreground mb-1",
                "Agent wants to execute: "
                span { class: "font-mono font-bold bg-muted px-1.5 rounded", "{tool_name}" }
            }

            // Stats / Security Context (Auditability)
            div { class: "text-xs text-muted-foreground mb-3 flex gap-4",
                span { "Call ID: {call_id}" }
                span { "Risk Level: Medium" }
            }

            // Args Diff View
            div { class: "bg-muted/50 rounded-md p-3 mb-4 overflow-x-auto border border-border/50",
                pre { class: "text-xs font-mono text-foreground whitespace-pre-wrap",
                     "{formatted_args}"
                }
            }

            // Actions
            div { class: "flex gap-3 justify-end",
                {
                    let call_id_deny = call_id.clone();
                    rsx! {
                        button {
                            class: "px-4 py-2 text-sm font-medium text-red-600 hover:bg-red-500/10 border border-red-200 dark:border-red-900/50 rounded-md transition-colors",
                            onclick: move |_| props.on_deny.call(call_id_deny.clone()),
                            "Deny"
                        }
                    }
                }
                {
                    let call_id_approve = call_id.clone();
                    rsx! {
                        button {
                            class: "px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md shadow-sm transition-colors",
                            onclick: move |_| props.on_approve.call(call_id_approve.clone()),
                            "Approve Execution"
                        }
                    }
                }
            }
        }
    }
}
