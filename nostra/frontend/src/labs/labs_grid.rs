use super::lab_card::LabCard;
use super::registry::get_lab_registry;
use super::types::LabManifest;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LabsGridProps {
    pub enabled_labs: Vec<String>,
    pub is_global_enabled: bool,
    pub on_toggle_lab: EventHandler<(String, bool)>,
    pub on_toggle_global: EventHandler<bool>,
    pub on_launch_lab: EventHandler<String>,
}

#[component]
pub fn LabsGrid(props: LabsGridProps) -> Element {
    let registry = get_lab_registry();
    let mut search_query = use_signal(|| "".to_string());
    let mut active_filter = use_signal(|| "all".to_string());
    let mut favorites = use_signal(|| std::collections::HashSet::<String>::new());

    // Filter labs
    let query_str = search_query.read().to_lowercase();
    let filter_str = active_filter.read().clone();
    let favs = favorites.read().clone();
    let enabled_list = props.enabled_labs.clone();

    let filtered_labs: Vec<LabManifest> = registry
        .iter()
        .filter(|lab| {
            let matches_search = lab.name.to_lowercase().contains(&query_str)
                || lab.description.to_lowercase().contains(&query_str);

            let matches_filter = match filter_str.as_str() {
                "enabled" => enabled_list.contains(&lab.id),
                "favorites" => favs.contains(&lab.id),
                _ => true,
            };

            matches_search && matches_filter
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "container mx-auto py-10 px-4 h-full overflow-y-auto",
            div { class: "flex flex-col gap-6 max-w-6xl mx-auto",
                // Constitution Banner (Principle 6: Intentional Exploration)
                div { class: "bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-950/30 dark:to-purple-950/30 p-4 rounded-lg border border-blue-100 dark:border-blue-900",
                    div { class: "flex items-start gap-3",
                        span { class: "text-2xl shrink-0", "🧪" }
                        div { class: "flex-1",
                            h3 { class: "font-semibold text-blue-900 dark:text-blue-100 mb-1", "Nostra Labs Constitution" }
                            p { class: "text-sm text-blue-700 dark:text-blue-300 italic",
                                "\"Labs is where tomorrow's standards are allowed to be messy today.\""
                            }
                        }
                    }
                }

                // Header & Controls
                div { class: "flex flex-col gap-4",
                    div { class: "flex items-center justify-between",
                        div {
                            h2 { class: "text-3xl font-bold tracking-tight", "Labs Playground" }
                            p { class: "text-muted-foreground", "Explore experimental features, apps, and prototypes." }
                        }
                        div { class: "flex items-center gap-2",
                            span { class: "text-sm font-medium", "Enable Labs" }
                            button {
                                class: format!(
                                    "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 {}",
                                    if props.is_global_enabled { "bg-primary" } else { "bg-input" }
                                ),
                                onclick: move |_| props.on_toggle_global.call(!props.is_global_enabled),
                                span {
                                    class: format!(
                                        "pointer-events-none block h-5 w-5 rounded-full bg-background shadow-lg ring-0 transition-transform {}",
                                        if props.is_global_enabled { "translate-x-5" } else { "translate-x-0" }
                                    )
                                }
                            }
                        }
                    }

                    // Toolbar
                    div { class: "flex items-center justify-between gap-4 mt-2",
                         // Search
                         div { class: "relative max-w-sm w-full",
                            svg { class: "absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" }
                            }
                            input {
                                class: "flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 pl-9 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
                                placeholder: "Search labs...",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value())
                            }
                         }

                         // Filter Tabs
                         div { class: "flex items-center rounded-lg border bg-muted p-1",
                            for filter in ["all", "favorites", "enabled"] {
                                button {
                                    class: format!(
                                        "inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {}",
                                        if active_filter() == filter { "bg-background text-foreground shadow" } else { "text-muted-foreground hover:bg-background/50 hover:text-foreground" }
                                    ),
                                    onclick: move |_| active_filter.set(filter.to_string()),
                                    "{filter.to_uppercase()}"
                                }
                            }
                         }
                    }
                }

                // Grid
                div { class: "grid gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4",
                    {filtered_labs.into_iter().map(|lab| {
                        let id_toggle = lab.id.clone();
                        let id_fav = lab.id.clone();

                        rsx! {
                            LabCard {
                                key: "{lab.id}",
                                manifest: lab.clone(),
                                is_enabled: props.enabled_labs.contains(&lab.id),
                                is_favorite: favs.contains(&lab.id),
                                on_toggle: move |new_val| props.on_toggle_lab.call((id_toggle.clone(), new_val)),
                                on_favorite: move |_| {
                                    let mut new_favs = favorites.read().clone();
                                    if new_favs.contains(&id_fav) {
                                        new_favs.remove(&id_fav);
                                    } else {
                                        new_favs.insert(id_fav.clone());
                                    }
                                    favorites.set(new_favs);
                                },
                                on_launch: {
                                    if lab.route_path.is_some() {
                                        let handler = props.on_launch_lab.clone();
                                        let id = lab.id.clone();
                                        Some(EventHandler::new(move |_| handler.call(id.clone())))
                                    } else {
                                        None
                                    }
                                },
                                on_config: Some(EventHandler::new(move |_| {
                                    // TODO: Open Config Modal
                                }))
                            }
                        }
                    })}
                }
            }
        }
    }
}
