use crate::components::icons::{Icon, IconName};
use crate::types::*;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LibraryCardProps {
    pub library: LibraryManifest,
    pub is_enabled: bool,
    pub on_toggle: EventHandler<bool>,
}

#[component]
pub fn LibraryCard(props: LibraryCardProps) -> Element {
    let lib = props.library;
    let is_enabled = props.is_enabled;

    rsx! {
        div { class: "flex items-center justify-between p-4 rounded-lg border bg-muted/20 hover:bg-muted/40 transition-colors",
            div { class: "flex items-center gap-4",
                div { class: "w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center shrink-0",
                    Icon { name: IconName::Books, class: "w-5 h-5 text-primary" }
                }
                div {
                    div { class: "flex items-center gap-2",
                        h3 { class: "font-semibold", "{lib.id}" }
                        span { class: "text-xs px-2 py-0.5 rounded bg-secondary text-secondary-foreground", "v{lib.version}" }
                    }
                    p { class: "text-sm text-muted-foreground mt-1", "{lib.description}" }
                }
            }
            // Toggle switch
            button {
                class: format!(
                    "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {}",
                    if is_enabled { "bg-primary" } else { "bg-muted" }
                ),
                onclick: move |_| props.on_toggle.call(!is_enabled),
                span {
                    class: format!(
                        "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {}",
                        if is_enabled { "translate-x-5" } else { "translate-x-0" }
                    )
                }
            }
        }
    }
}
