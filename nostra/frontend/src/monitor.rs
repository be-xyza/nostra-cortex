use crate::components::icons::IconName;
use crate::components::metric_card::MetricCard;
use crate::services::gateway_service::GATEWAY_CONNECTED;
use crate::types::*;
use dioxus::prelude::*;

#[component]
pub fn MonitorDashboard(status: Signal<Option<SystemStatus>>) -> Element {
    let current_status = status.read();
    let gateway_connected = GATEWAY_CONNECTED.read();

    // Local state for tracking manual test run
    let mut workflow_log = use_signal(|| Vec::<String>::new());

    // Listen for gateway events (hacky polling of valid signal or using a global listener?)
    // Ideally GATEWAY_SERVICE would have a listener we hook into.
    // For this prototype, we'll just log "Simulated" or if we can hook the global signal...
    // Actually, `GATEWAY_CONNECTED` is just a bool. `gateway_service.rs` prints to console.
    // To show in UI, we should probably update a global `Signal<Vec<Event>>`.
    // Let's modify `monitor.rs` to just trigger the start and show a "Running..." state.

    let start_gap_closure = move |_| {
        spawn(async move {
            workflow_log
                .write()
                .push("Starting 'Gap Closure' Workflow...".to_string());
            let client = reqwest::Client::new();
            match client
                .post("http://localhost:3003/workflows/start/gap_closure")
                .send()
                .await
            {
                Ok(resp) => {
                    if let Ok(txt) = resp.text().await {
                        workflow_log.write().push(format!("Started: {}", txt));
                    }
                }
                Err(e) => {
                    workflow_log.write().push(format!("Error: {}", e));
                }
            }
        });
    };

    use_effect(move || {
        let event_opt = crate::services::gateway_service::LATEST_EVENT.read();
        if let Some(event) = &*event_opt {
            if event.topic == "workflow_update" {
                let payload_str = event.payload.to_string();
                // Pretty print if possible
                workflow_log
                    .write()
                    .insert(0, format!("[{}] {}", event.timestamp, payload_str));
            }
        }
    });

    rsx! {
        div { class: "flex flex-col h-full bg-background p-6 overflow-y-auto",
            div { class: "max-w-6xl mx-auto w-full space-y-8",
                // Header & Status
                div { class: "flex items-center justify-between",
                    div {
                        h1 { class: "text-3xl font-bold tracking-tight", "System Monitor" }
                        p { class: "text-muted-foreground", "Real-time diagnostics and fleet health." }
                    }
                    if let Some(s) = &*current_status {
                        div { class: format!("flex items-center gap-2 px-4 py-2 rounded-full border {}", match s.status {
                            MonitorLevel::Healthy => "bg-green-500/10 border-green-500/20 text-green-500",
                            MonitorLevel::Warning => "bg-yellow-500/10 border-yellow-500/20 text-yellow-500",
                            MonitorLevel::Critical => "bg-red-500/10 border-red-500/20 text-red-500",
                        }),
                            div { class: format!("w-2.5 h-2.5 rounded-full animate-pulse {}", match s.status {
                                MonitorLevel::Healthy => "bg-green-500",
                                MonitorLevel::Warning => "bg-yellow-500",
                                MonitorLevel::Critical => "bg-red-500",
                            })}
                            span { class: "font-semibold", "{s.status:?}" }
                        }
                    } else {
                        div { class: "flex items-center gap-2 px-4 py-2 rounded-full border bg-muted/50 text-muted-foreground",
                            div { class: "w-2.5 h-2.5 rounded-full bg-muted-foreground/50" }
                            "Connecting..."
                        }
                    }
                }

                if let Some(s) = &*current_status {
                    // Metrics Grid (Refactored)
                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-4",
                        MetricCard {
                            title: "Active Workflows".to_string(),
                            value: s.metrics.active_workflows.to_string(),
                            subtitle: "Running processes".to_string(),
                            icon: IconName::Play
                        }

                        MetricCard {
                            title: "Error Rate (24h)".to_string(),
                            value: s.metrics.error_count_24h.to_string(),
                            subtitle: "Exceptions logged".to_string(),
                            icon: IconName::Alert
                        }

                        MetricCard {
                            title: "System Uptime".to_string(),
                            value: "99.9%".to_string(),
                            subtitle: "Since last upgrade".to_string(),
                            icon: IconName::Check
                        }

                        MetricCard {
                            title: "Version".to_string(),
                            value: s.version.clone(),
                            subtitle: "Current Release".to_string(),
                            icon: IconName::Info
                        }
                    }

                    // Fleet Status (Placeholder for Phase 2.5)
                    div { class: "rounded-xl border bg-card text-card-foreground shadow-sm",
                         div { class: "flex flex-col space-y-1.5 p-6",
                            h3 { class: "font-semibold leading-none tracking-tight", "Canister Fleet" }
                            p { class: "text-sm text-muted-foreground", "Status of all managed canisters." }
                        }
                        div { class: "p-6 pt-0",
                            div { class: "rounded-md border",
                                table { class: "w-full text-sm",
                                    thead { class: "[&_tr]:border-b",
                                        tr { class: "border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted",
                                            th { class: "h-12 px-4 text-left align-middle font-medium text-muted-foreground", "Canister" }
                                            th { class: "h-12 px-4 text-left align-middle font-medium text-muted-foreground", "Cycles" }
                                            th { class: "h-12 px-4 text-left align-middle font-medium text-muted-foreground", "Status" }
                                        }
                                    }
                                    tbody { class: "[&_tr:last-child]:border-0",
                                        tr { class: "border-b transition-colors hover:bg-muted/50",
                                            td { class: "p-4 align-middle font-medium", "Backend (Core)" }
                                            td { class: "p-4 align-middle", "Healthy (2.4T)" }
                                            td { class: "p-4 align-middle",
                                                span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-green-500/10 text-green-500", "Running" }
                                            }
                                        }
                                         tr { class: "border-b transition-colors hover:bg-muted/50",
                                            td { class: "p-4 align-middle font-medium", "Frontend (Asset)" }
                                            td { class: "p-4 align-middle", "Healthy (1.8T)" }
                                            td { class: "p-4 align-middle",
                                                 span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-green-500/10 text-green-500", "Running" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                     // Loading State
                     div { class: "flex items-center justify-center h-64",
                        div { class: "flex flex-col items-center gap-4",
                            div { class: "w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin" }
                            p { class: "text-muted-foreground", "Fetching system telemetry..." }
                        }
                    }
                }

                // Gap Closure Diagnostic Panel
                div { class: "rounded-xl border bg-card text-card-foreground shadow-sm",
                    div { class: "flex flex-col space-y-1.5 p-6",
                        div { class: "flex items-center justify-between",
                            div {
                                h3 { class: "font-semibold leading-none tracking-tight", "Orchestration Diagnostics" }
                                p { class: "text-sm text-muted-foreground", "Manual trigger for workflow validation." }
                            }
                             button {
                                class: "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2",
                                onclick: start_gap_closure,
                                "Test Gap Closure"
                            }
                        }
                    }
                    div { class: "p-6 pt-0",
                        div { class: "rounded-md border bg-muted/50 p-4 font-mono text-xs overflow-hidden",
                            div { class: "flex items-center justify-between mb-2",
                                span { class: "font-semibold text-muted-foreground", "Live Logs" }
                                if *gateway_connected {
                                    span { class: "text-green-500", "● Gateway Connected" }
                                } else {
                                     span { class: "text-yellow-500", "○ Gateway Connecting..." }
                                }
                            }
                            div { class: "space-y-1 h-32 overflow-y-auto",
                                for log in workflow_log.read().iter() {
                                    div { "{log}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
