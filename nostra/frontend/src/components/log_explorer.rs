use crate::api;
use crate::types::{LogEntry, LogLevel, LogSource};
use dioxus::prelude::*;

pub fn LogExplorer() -> Element {
    let mut logs = use_signal(|| Vec::<LogEntry>::new());
    let mut is_loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);
    let fetch_logs = move || {
        spawn(async move {
            is_loading.set(true);
            error.set(None);

            // We need to create agent locally or use a service
            // Creating agent here similar to main.rs pattern
            let agent = api::create_agent().await;

            match api::get_logs(&agent, 100).await {
                Ok(fetched_logs) => {
                    logs.set(fetched_logs);
                    is_loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    };

    // Initial load
    use_effect(move || {
        fetch_logs();
    });

    rsx! {
        div { class: "flex flex-col gap-4 h-full",
            // Header / Controls
            div { class: "flex items-center justify-between",
                h2 { class: "text-2xl font-bold tracking-tight", "System Logs" }
                button {
                    class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2",
                    onclick: move |_| fetch_logs(),
                    "Refresh Logs"
                }
            }

            // Error Message
            if let Some(err_msg) = error() {
                div { class: "p-4 rounded-md bg-red-500/10 border border-red-500/20 text-red-500",
                    "Error loading logs: {err_msg}"
                }
            }

            // Logs Table
            div { class: "rounded-md border",
                div { class: "relative w-full overflow-auto",
                    table { class: "w-full caption-bottom text-sm text-left",
                        thead { class: "[&_tr]:border-b sticky top-0 bg-card z-10 shadow-sm",
                            tr { class: "border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted",
                                th { class: "h-12 px-4 align-middle font-medium text-muted-foreground w-28", "Time" }
                                th { class: "h-12 px-4 align-middle font-medium text-muted-foreground w-24", "Level" }
                                th { class: "h-12 px-4 align-middle font-medium text-muted-foreground w-32", "Source" }
                                th { class: "h-12 px-4 align-middle font-medium text-muted-foreground", "Message" }
                                th { class: "h-12 px-4 align-middle font-medium text-muted-foreground w-1/3", "Context" }
                            }
                        }
                        tbody { class: "[&_tr:last-child]:border-0",
                            if is_loading() {
                                tr {
                                    td { colspan: "5", class: "p-8 text-center text-muted-foreground",
                                        "Loading system logs..."
                                    }
                                }
                            } else if logs().is_empty() {
                                tr {
                                    td { colspan: "5", class: "p-8 text-center text-muted-foreground",
                                        "No logs found."
                                    }
                                }
                            } else {
                                for log in logs() {
                                    tr { class: "border-b transition-colors hover:bg-muted/50",
                                        // Time
                                        td { class: "p-4 align-middle font-mono text-xs text-muted-foreground whitespace-nowrap",
                                            {
                                                let nanos = log.timestamp.0.to_string().parse::<u64>().unwrap_or(0);
                                                let millis = (nanos / 1_000_000) as f64;
                                                let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(millis));
                                                format!("{:02}:{:02}:{:02}", date.get_hours(), date.get_minutes(), date.get_seconds())
                                            }
                                        }
                                        // Level
                                        td { class: "p-4 align-middle",
                                            match log.level {
                                                LogLevel::Info => rsx! { span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold border-transparent bg-blue-500/10 text-blue-500", "INFO" } },
                                                LogLevel::Warn => rsx! { span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold border-transparent bg-yellow-500/10 text-yellow-500", "WARN" } },
                                                LogLevel::Error => rsx! { span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold border-transparent bg-red-500/10 text-red-500", "ERROR" } },
                                                LogLevel::Critical => rsx! { span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-bold border-transparent bg-red-500/20 text-red-600 animate-pulse", "CRITICAL" } },
                                            }
                                        }
                                        // Source
                                        td { class: "p-4 align-middle",
                                            match &log.source {
                                                LogSource::Backend => rsx! { span { class: "font-medium", "Backend" } },
                                                LogSource::Frontend => rsx! { span { class: "font-medium", "Frontend" } },
                                                LogSource::Agent(name) => rsx! { span { class: "font-medium text-purple-600", "{name}" } },
                                            }
                                        }
                                        // Message
                                        td { class: "p-4 align-middle font-medium", "{log.message}" }
                                        // Context
                                        td { class: "p-4 align-middle",
                                            if let Some(ctx) = &log.context {
                                                div { class: "flex flex-wrap gap-1.5",
                                                    for (k, v) in ctx {
                                                        span { class: "inline-flex items-center gap-1 rounded bg-muted/50 px-1.5 py-0.5 text-[10px] border border-border/50 font-mono text-muted-foreground",
                                                            span { class: "font-semibold text-foreground/70", "{k}:" }
                                                            span { class: "truncate max-w-[150px]", "{v}" }
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
            }
        }
    }
}
