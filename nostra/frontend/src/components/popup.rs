use crate::components::popup_config::{ActionStyle, PopupConfig, PopupSize};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PopupProps {
    config: PopupConfig,
    on_close: EventHandler<()>,
    on_action: EventHandler<String>,
}

pub fn Popup(props: PopupProps) -> Element {
    let config = props.config;
    let on_close = props.on_close;
    let on_action = props.on_action;

    let size_classes = match config.size {
        PopupSize::Small => "max_w_sm",
        PopupSize::Medium => "max_w_md",
        PopupSize::Large => "max_w_lg",
    };

    // Clone necessary fields for closures or to avoid move issues
    // Note: PopupConfig derives Clone, so this is cheap-ish.
    // Ideally we'd use Refs but for now cloning is safe.
    let dismissible = config.dismissible;
    let title = config.title.clone();
    let body = config.body_markdown.clone();
    let primary = config.primary_action.clone();
    let secondary = config.secondary_action.clone();

    // Handlers
    let close_handler = move |_| {
        if dismissible {
            on_close.call(());
        }
    };

    rsx! {
        div { class: "fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4 animate-in fade-in duration-200",
            // Use close_handler
            onclick: close_handler,

            // Modal Content (stop propagation to prevent closing when clicking inside)
            div {
                class: "relative w-full {size_classes} bg-background border border-border rounded-xl shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-200",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div { class: "flex items-center justify-between p-6 border-b bg-muted/10",
                    h2 { class: "text-lg font-semibold tracking-tight", "{title}" }
                    if dismissible {
                        button {
                            class: "text-muted-foreground hover:text-foreground transition-colors p-1 rounded-md hover:bg-muted",
                            onclick: move |_| on_close.call(()),
                            svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M6 18L18 6M6 6l12 12" }
                            }
                        }
                    }
                }

                // Body
                div { class: "p-6 overflow-y-auto max-h-[60vh]",
                    div { class: "prose prose-sm dark:prose-invert text-muted-foreground",
                        "{body}"
                    }
                }

                // Footer Actions
                if primary.is_some() || secondary.is_some() {
                    div { class: "flex items-center justify-end gap-3 p-6 border-t bg-muted/10",
                        if let Some(sec) = secondary {
                            ActionButton {
                                label: sec.label,
                                style: sec.style,
                                onclick: move |_| on_action.call(sec.action_id.clone()),
                            }
                        }
                        if let Some(pri) = primary {
                            ActionButton {
                                label: pri.label,
                                style: pri.style,
                                onclick: move |_| on_action.call(pri.action_id.clone()),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActionButton(label: String, style: ActionStyle, onclick: EventHandler<()>) -> Element {
    let base_classes = "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2";

    let variant_classes = match style {
        ActionStyle::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
        ActionStyle::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ActionStyle::Destructive => {
            "bg-destructive text-destructive-foreground hover:bg-destructive/90"
        }
        ActionStyle::Outline => {
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
        }
    };

    rsx! {
        button {
            class: "{base_classes} {variant_classes}",
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}
