use crate::a2ui::{A2UIRenderer, Surface};
use crate::a2ui_theme::A2UIThemeName;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct A2UILabProps {
    pub on_back: EventHandler<()>,
}

#[component]
pub fn A2UILab(props: A2UILabProps) -> Element {
    let mut show_example_menu = use_signal(|| false);

    // Default sample (Kanban Board Mockup)
    let default_json = r#"{
    "root": {
        "id": "kanban_board",
        "type": "Container",
        "children": [
            {
                "id": "header",
                "type": "Heading",
                "text": "Nostra Development Kanban"
            },
            {
                "id": "columns_row",
                "type": "Row",
                "children": [
                    {
                        "id": "col_todo",
                        "type": "Column",
                        "children": [
                            { "id": "h_todo", "type": "Text", "text": "To Do" },
                            {
                                "id": "card_1",
                                "type": "Card",
                                "child": { "id": "c1_text", "type": "Text", "text": "Implement A2UI Switch" }
                            }
                        ]
                    },
                    {
                        "id": "col_doing",
                        "type": "Column",
                        "children": [
                            { "id": "h_doing", "type": "Text", "text": "In Progress" },
                            {
                                "id": "card_2",
                                "type": "Card",
                                "child": { "id": "c2_text", "type": "Text", "text": "Wireframe Labs UI" }
                            }
                        ]
                    }
                ]
            }
        ]
    }
}"#;

    let mut json_input = use_signal(|| default_json.to_string());
    let mut parse_error = use_signal(|| None::<String>);

    // Derived state: parsed surface
    let _surface_result = serde_json::from_str::<Surface>(&json_input.read());

    // We update parse_error based on result, and keep old valid surface if error?
    // Or just show error in preview area.

    rsx! {
        div { class: "flex flex-col h-full bg-background text-foreground",
            // Header
            div { class: "border-b p-4 flex items-center justify-between bg-card",
                div { class: "flex items-center gap-4",
                    button {
                        class: "p-2 hover:bg-muted rounded-full transition-colors",
                        onclick: move |_| props.on_back.call(()),
                        "← Back"
                    }
                    div {
                        h1 { class: "font-semibold text-lg", "A2UI Workbench" }
                        p { class: "text-xs text-muted-foreground", "v0.8 Specification Preview" }
                    }
                }

                div { class: "flex items-center gap-2",
                    button {
                         class: "px-3 py-1.5 text-sm font-medium border rounded hover:bg-muted",
                         onclick: move |_| {},
                         "Reset"
                    }
                    div { class: "relative",
                        button {
                            class: "px-3 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded hover:bg-primary/90",
                            onclick: move |_| show_example_menu.set(!show_example_menu()),
                            "Load Sample ▾"
                        }
                        // Dropdown would go here
                    }
                }
            }

            // Main Split Pane
            div { class: "flex-1 flex overflow-hidden",
                // Editor Pane (Left)
                div { class: "w-1/2 border-r flex flex-col",
                    div { class: "bg-muted/30 px-4 py-2 text-xs font-mono text-muted-foreground border-b flex justify-between",
                        span { "JSON Input" }
                        if let Some(_err) = parse_error.read().as_ref() {
                            span { class: "text-destructive font-bold", "Invalid JSON" }
                        } else {
                            span { class: "text-green-600 font-bold", "Valid" }
                        }
                    }
                    textarea {
                        class: "flex-1 w-full p-4 font-mono text-sm resize-none focus:outline-none bg-background",
                        value: "{json_input}",
                        spellcheck: "false",
                        oninput: move |evt| {
                            let val = evt.value();
                            json_input.set(val.clone());
                            match serde_json::from_str::<Surface>(&val) {
                                Ok(_) => parse_error.set(None),
                                Err(e) => parse_error.set(Some(e.to_string())),
                            }
                        }
                    }
                    // Error Log
                    if let Some(err) = parse_error.read().as_ref() {
                         div { class: "bg-destructive/10 text-destructive p-2 text-xs font-mono border-t border-destructive/20 whitespace-pre-wrap",
                            "{err}"
                         }
                    }
                }

                // Preview Pane (Right)
                div { class: "w-1/2 flex flex-col bg-muted/10",
                    div { class: "bg-muted/30 px-4 py-2 text-xs font-mono text-muted-foreground border-b", "Live Preview" }
                    div { class: "flex-1 overflow-y-auto p-8",
                        div { class: "max-w-2xl mx-auto bg-background border rounded-xl shadow-sm min-h-[200px] p-6",
                             {
                                 if let Ok(parsed_surface) = serde_json::from_str::<Surface>(&json_input.read()) {
                                     rsx! {
                                         A2UIRenderer {
                                             surface: {parsed_surface},
                                             theme: Some(A2UIThemeName::Nostra),
                                             on_action: move |(action, payload)| {
                                                 web_sys::console::log_1(&format!("Action: {} Payload: {}", action, payload).into());
                                             }
                                         }
                                     }
                                 } else {
                                     rsx! {
                                        div { class: "flex items-center justify-center h-full text-muted-foreground italic",
                                            "Fix JSON errors to render preview"
                                        }
                                     }
                                 }
                             }
                        }
                    }
                }
            }
        }
    }
}
