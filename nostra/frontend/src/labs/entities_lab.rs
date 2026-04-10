use crate::components::icons::{Icon, IconName};
use crate::types::*;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum ViewMode {
    Entities,
    Relationships,
}
// ... (Render fn start) ...

#[component]
pub fn EntitiesLab(
    entities: Signal<Vec<Entity>>,
    relationships: Signal<Vec<Relationship>>,
) -> Element {
    let mut view_mode = use_signal(|| ViewMode::Entities);
    let mut selected_entity = use_signal(|| None::<String>);

    rsx! {
        div { class: "container mx-auto py-10 px-4 h-full overflow-y-auto custom-scrollbar",
            div { class: "flex flex-col gap-6",
                // Header Section
                div { class: "flex items-center justify-between",
                    div { class: "flex flex-col gap-2",
                        h2 { class: "text-3xl font-bold tracking-tight", "Semantic Entities" }
                        p { class: "text-muted-foreground", "Structured knowledge discovery for the ICP ecosystem." }
                    }
                    // Toggle Switch
                    div { class: "flex items-center bg-muted p-1 rounded-lg border border-border",
                        button {
                            class: format!(
                                "px-3 py-1.5 rounded-md text-sm font-medium transition-all {}",
                                if view_mode() == ViewMode::Entities { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }
                            ),
                            onclick: move |_| view_mode.set(ViewMode::Entities),
                            "Entities"
                        }
                        button {
                            class: format!(
                                "px-3 py-1.5 rounded-md text-sm font-medium transition-all {}",
                                if view_mode() == ViewMode::Relationships { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }
                            ),
                            onclick: move |_| view_mode.set(ViewMode::Relationships),
                            "Relationships"
                        }
                    }
                }

                match view_mode() {
                    ViewMode::Entities => rsx! {
                         // Main Content Split: Table + Details
                        div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                            // Left: Entity Table
                            div { class: "lg:col-span-2 rounded-xl border bg-card text-card-foreground shadow",
                                div { class: "p-6",
                                    div { class: "flex items-center justify-between mb-4",
                                        h3 { class: "text-lg font-semibold", "All Entities" }
                                        div { class: "text-sm text-muted-foreground bg-muted px-2.5 py-0.5 rounded-full",
                                            "Total: {entities().len()}"
                                        }
                                    }
                                    div { class: "relative w-full overflow-auto",
                                        table { class: "w-full caption-bottom text-sm text-left",
                                            thead { class: "[&_tr]:border-b",
                                                tr { class: "border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted",
                                                    th { class: "h-12 px-4 align-middle font-medium text-muted-foreground", "ID" }
                                                    th { class: "h-12 px-4 align-middle font-medium text-muted-foreground", "Name" }
                                                    th { class: "h-12 px-4 align-middle font-medium text-muted-foreground", "Type" }
                                                    th { class: "h-12 px-4 align-middle font-medium text-muted-foreground", "Tags" }
                                                    th { class: "h-12 px-4 align-middle font-medium text-muted-foreground text-right", "Actions" }
                                                }
                                            }
                                            tbody { class: "[&_tr:last-child]:border-0",
                                                for entity in entities() {
                                                    {
                                                        let row_id = entity.id.clone();
                                                        let btn_id = entity.id.clone();
                                                        rsx! {
                                                            tr {
                                                                class: format!("border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted cursor-pointer {}", if selected_entity() == Some(entity.id.clone()) { "bg-muted" } else { "" }),
                                                                onclick: move |_| selected_entity.set(Some(row_id.clone())),
                                                                td { class: "p-4 align-middle font-mono text-xs", "{entity.id}" }
                                                                td { class: "p-4 align-middle font-medium", "{entity.name}" }
                                                                td { class: "p-4 align-middle",
                                                                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
                                                                        "{entity.entity_type:?}"
                                                                    }
                                                                }
                                                                td { class: "p-4 align-middle",
                                                                    div { class: "flex flex-wrap gap-1",
                                                                        for tag in entity.tags.iter().take(2) {
                                                                            span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
                                                                                "{tag}"
                                                                            }
                                                                        }
                                                                        if entity.tags.len() > 2 {
                                                                            span { class: "text-xs text-muted-foreground", "+{entity.tags.len() - 2}" }
                                                                        }
                                                                    }
                                                                }
                                                                td { class: "p-4 align-middle text-right",
                                                                    button {
                                                                        class: "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-9 w-9",
                                                                        onclick: move |_| selected_entity.set(Some(btn_id.clone())),
                                                                        Icon { name: IconName::ChevronRight, class: "w-4 h-4" }
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

                            // Right: Entity Details (Sticky)
                            if let Some(sel_id) = selected_entity() {
                                if let Some(entity) = entities().iter().find(|e| e.id == sel_id) {
                                    div { class: "lg:col-span-1",
                                        div { class: "rounded-xl border bg-card text-card-foreground shadow sticky top-6",
                                            div { class: "flex flex-col space-y-4 p-6",
                                                div { class: "flex items-start justify-between",
                                                    div {
                                                        h3 { class: "text-2xl font-semibold leading-none tracking-tight flex items-center gap-2 mb-2",
                                                            Icon { name: IconName::File, class: "w-6 h-6 text-primary" }
                                                            "{entity.name}"
                                                        }
                                                        p { class: "text-sm text-muted-foreground font-mono", "ID: {entity.id}" }
                                                    }
                                                    button {
                                                        class: "text-muted-foreground hover:text-foreground p-1 rounded-md hover:bg-muted transition-colors",
                                                        onclick: move |_| selected_entity.set(None),
                                                        Icon { name: IconName::Close, class: "w-4 h-4" }
                                                    }
                                                }

                                                div { class: "flex flex-wrap gap-2 text-xs",
                                                        // Type Badge
                                                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 font-semibold transition-colors border-transparent bg-primary/10 text-primary gap-1",
                                                        Icon { name: IconName::Tag, class: "w-3 h-3" }
                                                        "{entity.entity_type:?}"
                                                    }
                                                    // Created Date Badge
                                                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 font-medium transition-colors border-transparent text-muted-foreground bg-muted gap-1",
                                                        Icon { name: IconName::Clock, class: "w-3 h-3" }
                                                        {
                                                            // Format timestamp
                                                            let nanos = entity.timestamp.0.to_string().parse::<u64>().unwrap_or(0);
                                                            let millis = (nanos / 1_000_000) as f64;
                                                            let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(millis));
                                                            // We use locale string
                                                            format!("Created: {}", date.to_locale_string("en-US", &wasm_bindgen::JsValue::undefined()).as_string().unwrap_or_default())
                                                        }
                                                    }
                                                }

                                                div { class: "p-6 pt-0 space-y-6",
                                                    // Description
                                                    div {
                                                        h4 { class: "text-sm font-semibold mb-2", "Description" }
                                                        p { class: "text-sm text-muted-foreground leading-relaxed", "{entity.description}" }
                                                    }

                                                    // Tags
                                                    if !entity.tags.is_empty() {
                                                        div {
                                                            h4 { class: "text-sm font-semibold mb-2", "Tags" }
                                                            div { class: "flex flex-wrap gap-1.5",
                                                                for tag in entity.tags.iter() {
                                                                    span { class: "inline-flex items-center rounded-md border border-border px-2 py-0.5 text-xs font-medium transition-colors bg-secondary/50 text-secondary-foreground",
                                                                        "{tag}"
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    // Relationships
                                                    div {
                                                        h4 { class: "text-sm font-semibold mb-2", "Relationships" }
                                                        div { class: "space-y-2 max-h-60 overflow-y-auto custom-scrollbar pr-2",
                                                            if relationships().iter().any(|r| r.from == entity.id || r.to == entity.id) {
                                                                for rel in relationships().iter().filter(|r| r.from == entity.id || r.to == entity.id) {
                                                                    {
                                                                        let target_id = if rel.from == entity.id { rel.to.clone() } else { rel.from.clone() };
                                                                        rsx! {
                                                                            div {
                                                                                class: "flex items-center gap-2 text-sm border border-border/50 p-2.5 rounded-lg bg-muted/20 cursor-pointer hover:bg-muted transition-colors group",
                                                                                onclick: move |_| selected_entity.set(Some(target_id.clone())),
                                                                                Icon { name: IconName::ArrowRight, class: "w-4 h-4 text-muted-foreground shrink-0 group-hover:text-primary transition-colors" }
                                                                                div { class: "flex-grow min-w-0",
                                                                                    div { class: "flex items-center justify-between gap-2",
                                                                                        span { class: "font-medium truncate group-hover:underline",
                                                                                            if rel.from == entity.id { "{rel.to}" } else { "{rel.from}" }
                                                                                        }
                                                                                        span { class: "text-[10px] uppercase tracking-wider text-muted-foreground border border-border px-1.5 py-0.5 rounded", "{rel.rel_type}" }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                p { class: "text-sm text-muted-foreground italic", "No relationships found. Add relationships using the Manage tab." }
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
                    },
                    ViewMode::Relationships => rsx! {
                        div { class: "rounded-xl border bg-card text-card-foreground shadow p-6",
                            h3 { class: "text-lg font-semibold mb-4", "All Relationships ({relationships().len()})" }
                             div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                                for rel in relationships() {
                                    div { class: "border rounded-lg p-4 bg-muted/10 flex flex-col gap-2 hover:bg-muted/20 transition-colors",
                                        div { class: "flex items-center justify-between mb-2",
                                            span { class: "text-xs font-mono bg-background px-2 py-0.5 rounded border", "{rel.rel_type}" }
                                            if rel.bidirectional {
                                                Icon { name: IconName::ArrowLeftRight, class: "w-4 h-4 text-muted-foreground" }
                                            } else {
                                                Icon { name: IconName::ArrowRight, class: "w-4 h-4 text-muted-foreground" }
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            div { class: "font-medium text-sm truncate", "{rel.from}" }
                                            Icon { name: IconName::ArrowRight, class: "w-4 h-4 text-muted-foreground shrink-0" }
                                            div { class: "font-medium text-sm truncate", "{rel.to}" }
                                        }
                                        if let Some(lib_id) = &rel.library_id {
                                            div { class: "mt-auto pt-2 text-[10px] text-muted-foreground text-right", "{lib_id}" }
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
