use crate::components::chronicle_explorer::ChronicleExplorer;
use crate::components::log_explorer::LogExplorer;
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Copy)]
pub enum AdminTab {
    Logs,
    Chronicle,
    SystemStatus,
}

pub fn AdminPage() -> Element {
    let mut active_tab = use_signal(|| AdminTab::Logs);

    rsx! {
        div { class: "container mx-auto py-10 px-4 h-full flex flex-col gap-6",
            // Header
            div { class: "flex flex-col gap-2",
                h2 { class: "text-3xl font-bold tracking-tight", "Administration" }
                p { class: "text-muted-foreground", "System controls and monitoring." }
            }

            // Tabs
            div { class: "flex items-center gap-4 border-b",
                button {
                    class: format!(
                        "px-4 py-2 text-sm font-medium border-b-2 transition-colors hover:text-foreground {}",
                        if active_tab() == AdminTab::Logs { "border-primary text-foreground" } else { "border-transparent text-muted-foreground" }
                    ),
                    onclick: move |_| active_tab.set(AdminTab::Logs),
                    "System Logs"
                }
                button {
                    class: format!(
                        "px-4 py-2 text-sm font-medium border-b-2 transition-colors hover:text-foreground {}",
                        if active_tab() == AdminTab::Chronicle { "border-primary text-foreground" } else { "border-transparent text-muted-foreground" }
                    ),
                    onclick: move |_| active_tab.set(AdminTab::Chronicle),
                    "Chronicle"
                }
                button {
                    class: format!(
                        "px-4 py-2 text-sm font-medium border-b-2 transition-colors hover:text-foreground {}",
                        if active_tab() == AdminTab::SystemStatus { "border-primary text-foreground" } else { "border-transparent text-muted-foreground" }
                    ),
                    onclick: move |_| active_tab.set(AdminTab::SystemStatus),
                    "System Status"
                }
            }

            // Content
            div { class: "flex-1 overflow-hidden",
                match active_tab() {
                    AdminTab::Logs => rsx! { LogExplorer {} },
                    AdminTab::Chronicle => rsx! { ChronicleExplorer {} },
                    AdminTab::SystemStatus => rsx! {
                        div { class: "p-8 text-center text-muted-foreground bg-muted/10 rounded-lg border border-dashed",
                            p { "System Status checks coming soon..." }
                        }
                    },
                }
            }
        }
    }
}
