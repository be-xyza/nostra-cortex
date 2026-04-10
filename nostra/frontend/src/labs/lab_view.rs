use super::registry::get_lab_registry;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LabViewProps {
    pub lab_id: String,
    pub on_back: EventHandler<()>,
    pub children: Element,
}

#[component]
pub fn LabView(props: LabViewProps) -> Element {
    let registry = get_lab_registry();
    let lab = registry.iter().find(|l| l.id == props.lab_id);

    let title = lab
        .map(|l| l.name.clone())
        .unwrap_or_else(|| "Unknown Lab".to_string());

    rsx! {
        div { class: "flex flex-col h-full w-full bg-background",
            // Lab Toolbar
            div { class: "h-12 border-b bg-muted/30 flex items-center justify-between px-4 shrink-0",
                div { class: "flex items-center gap-4",
                    button {
                        class: "hover:bg-accent hover:text-accent-foreground rounded-full p-1.5 transition-colors",
                        onclick: move |_| props.on_back.call(()),
                        svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" }
                        }
                    }
                    div { class: "h-6 w-px bg-border" }
                    h2 { class: "font-semibold text-sm", "{title}" }
                    span { class: "text-xs px-2 py-0.5 rounded bg-orange-500/10 text-orange-500 border border-orange-500/20", "Experimental Mode" }
                }

                div { class: "flex items-center gap-2",
                    button {
                        class: "inline-flex items-center justify-center rounded-md text-xs font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 px-3",
                        "Configuration"
                    }
                }
            }

            // Lab Content Area
            div { class: "flex-grow relative overflow-hidden",
                {props.children}
            }
        }
    }
}
