use super::types::{ActivityStatus, LabManifest, LabStatus};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct LabCardProps {
    pub manifest: LabManifest,
    pub is_enabled: bool,
    pub is_favorite: bool,
    pub on_toggle: EventHandler<bool>,
    pub on_favorite: EventHandler<()>,
    #[props(optional)]
    pub on_launch: Option<EventHandler<()>>,
    #[props(optional)]
    pub on_config: Option<EventHandler<()>>,
}

#[component]
pub fn LabCard(props: LabCardProps) -> Element {
    let manifest = &props.manifest;

    // Status Badge Color
    let status_color = match manifest.status {
        LabStatus::Alpha => "bg-orange-500/10 text-orange-500 border-orange-500/20",
        LabStatus::Beta => "bg-blue-500/10 text-blue-500 border-blue-500/20",
        LabStatus::Experimental => "bg-purple-500/10 text-purple-500 border-purple-500/20",
        LabStatus::Deprecated => "bg-red-500/10 text-red-500 border-red-500/20",
        LabStatus::Prototype => "bg-cyan-500/10 text-cyan-500 border-cyan-500/20",
    };

    let status_label = format!("{:?}", manifest.status);

    // Activity Indicator (UI/UX Principle 6: Time Must Be Legible)
    let (activity_icon, activity_color) = match manifest.activity_status {
        ActivityStatus::Active => ("🟢", "text-green-600 dark:text-green-400"),
        ActivityStatus::Dormant => ("🟡", "text-yellow-600 dark:text-yellow-400"),
        ActivityStatus::Archived => ("⚫", "text-gray-600 dark:text-gray-400"),
    };

    // Format last activity (simple relative time)
    let activity_text = {
        // In production, this would calculate relative time
        // For now, just showing the raw timestamp in a friendly format
        let parts: Vec<&str> = manifest.last_activity.split('T').collect();
        if parts.len() > 0 {
            parts[0].to_string()
        } else {
            manifest.last_activity.clone()
        }
    };

    rsx! {
        div { class: "group relative flex flex-col justify-between rounded-xl border bg-card text-card-foreground shadow transition-all hover:shadow-md",
            // Header
            div { class: "p-6 space-y-2",
                div { class: "flex items-start justify-between",
                    div { class: "flex items-center gap-3",
                        div { class: "w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center text-primary",
                            svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", d: "M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" }
                            }
                        }
                        div { class: "flex-1",
                            h3 { class: "font-semibold leading-none tracking-tight", "{manifest.name}" }
                            div { class: "flex items-center gap-2 mt-1.5",
                                span { class: "text-xs px-2 py-0.5 rounded border {status_color}", "{status_label}" }
                                span { class: "text-xs text-muted-foreground", "v{manifest.version}" }
                            }
                        }
                    }

                    button {
                        class: format!("text-muted-foreground hover:text-yellow-500 transition-colors {}", if props.is_favorite { "text-yellow-500" } else { "" }),
                        onclick: move |_| props.on_favorite.call(()),
                        svg { class: "w-5 h-5", fill: if props.is_favorite { "currentColor" } else { "none" }, stroke: "currentColor", stroke_width: "2", view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", d: "M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" }
                        }
                    }
                }

                p { class: "text-sm text-muted-foreground line-clamp-2 min-h-[2.5rem]", "{manifest.description}" }

                // Hypothesis (Constitution Principle 6)
                div { class: "mt-3 p-2 bg-blue-50 dark:bg-blue-950/30 rounded border border-blue-100 dark:border-blue-900",
                    p { class: "text-xs italic text-blue-700 dark:text-blue-300",
                        "Hypothesis: {manifest.hypothesis}"
                    }
                }

                // Activity Status (UI/UX Principle 6: Time Must Be Legible)
                div { class: "flex items-center gap-2 mt-2 text-xs {activity_color}",
                    span { "{activity_icon}" }
                    span { "Last activity: {activity_text}" }
                }
            }

            // Footer / Actions
            div { class: "flex items-center justify-between p-4 pt-0",
               div { class: "flex items-center gap-2",
                    button {
                        class: format!(
                            "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {}",
                            if props.is_enabled { "bg-primary" } else { "bg-muted" }
                        ),
                        onclick: move |_| props.on_toggle.call(!props.is_enabled),
                        span {
                            class: format!(
                                "pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {}",
                                if props.is_enabled { "translate-x-5" } else { "translate-x-0" }
                            )
                        }
                    }
                    span { class: "text-sm font-medium", if props.is_enabled { "Active" } else { "Inactive" } }
               }

               div { class: "flex gap-2",
                    if let Some(config_handler) = props.on_config.clone() {
                        button {
                            class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-3",
                            onclick: move |_| config_handler.call(()),
                            "Config"
                        }
                    }

                    if props.is_enabled {
                        if let Some(launch_handler) = props.on_launch.clone() {
                            button {
                                class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-3",
                                onclick: move |_| launch_handler.call(()),
                                "Launch"
                            }
                        }
                    }
               }
            }
        }
    }
}
