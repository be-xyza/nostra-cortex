use crate::types::{Institution, LifecyclePhase};
use dioxus::prelude::*;

#[component]
pub fn InstitutionCard(
    institution: Institution,
    commons_ruleset_version: Option<String>,
    commons_rule_count: Option<usize>,
    commons_enforcement_mode: Option<String>,
    on_edit: EventHandler<Institution>,
    on_fork: EventHandler<Institution>,
) -> Element {
    let phase_color = match institution.lifecycle_phase {
        LifecyclePhase::Emergent => "bg-yellow-100 text-yellow-800",
        LifecyclePhase::Provisional => "bg-blue-100 text-blue-800",
        LifecyclePhase::Formalized => "bg-purple-100 text-purple-800",
        LifecyclePhase::Operational => "bg-green-100 text-green-800",
        LifecyclePhase::Dormant => "bg-gray-100 text-gray-800",
        LifecyclePhase::Archived => "bg-gray-200 text-gray-500",
    };

    let phase_label = format!("{:?}", institution.lifecycle_phase);
    let is_commons = institution.scope.contains("commons");

    let inst_edit = institution.clone();
    let inst_fork = institution.clone();

    rsx! {
        div { class: "flex flex-col p-4 rounded-lg border bg-card hover:shadow-md transition-shadow relative group",
            div { class: "flex items-start justify-between mb-2",
                div { class: "flex items-center gap-2",
                    span { class: "text-2xl", if is_commons { "📜" } else { "🏛️" } }
                    div {
                        h3 { class: "font-semibold text-lg hover:underline cursor-pointer",
                            "{institution.title}"
                        }
                        span { class: "text-xs text-muted-foreground", "ID: {institution.id}" }
                    }
                }
                div { class: "flex items-center gap-1.5",
                    if is_commons {
                        span { class: "text-xs px-2 py-0.5 rounded-full font-medium bg-teal-100 text-teal-800",
                            "Commons"
                        }
                    }
                    span { class: "text-xs px-2 py-0.5 rounded-full font-medium {phase_color}",
                        "{phase_label}"
                    }
                }
            }

            p { class: "text-sm text-foreground/80 mb-3 line-clamp-2",
                "{institution.description}"
            }

            div { class: "text-xs text-muted-foreground space-y-1",
                div {
                    span { class: "font-semibold", "Intent: " }
                    "{institution.intent}"
                }
                 div {
                    span { class: "font-semibold", "Scope: " }
                    "{institution.scope}"
                }
                if is_commons {
                    div {
                        span { class: "font-semibold", "Ruleset: " }
                        if let Some(version) = commons_ruleset_version.as_ref() {
                            "{version}"
                        } else {
                            "No ruleset attached"
                        }
                    }
                    div {
                        span { class: "font-semibold", "Rule Count: " }
                        if let Some(count) = commons_rule_count {
                            "{count}"
                        } else {
                            "0"
                        }
                    }
                    div {
                        span { class: "font-semibold", "Enforcement: " }
                        if let Some(mode) = commons_enforcement_mode.as_ref() {
                            "{mode}"
                        } else {
                            "unknown"
                        }
                    }
                }
            }

            // Stewards Facepile (Placeholder)
            div { class: "mt-4 flex items-center justify-between border-t pt-3",
                 div { class: "flex items-center gap-2",
                    span { class: "text-xs font-medium text-muted-foreground", "Stewards" }
                    div { class: "flex -space-x-2",
                        for _steward in institution.stewards.iter().take(3) {
                             div { class: "w-6 h-6 rounded-full bg-primary/20 border border-background flex items-center justify-center text-[10px]",
                                 "👤"
                             }
                        }
                        if institution.stewards.len() > 3 {
                             div { class: "w-6 h-6 rounded-full bg-muted border border-background flex items-center justify-center text-[10px]",
                                 "+{institution.stewards.len() - 3}"
                             }
                        }
                    }
                }

                // Actions (Visible on hover or always appropriate for MVP)
                div { class: "flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity",
                    button {
                        class: "p-1.5 hover:bg-muted rounded text-xs",
                        title: "Edit Institution",
                        onclick: move |_| on_edit.call(inst_edit.clone()),
                        "✏️"
                    }
                    button {
                        class: "p-1.5 hover:bg-muted rounded text-xs",
                        title: "Fork Institution",
                        onclick: move |_| on_fork.call(inst_fork.clone()),
                        "🔱"
                    }
                }
            }
        }
    }
}
