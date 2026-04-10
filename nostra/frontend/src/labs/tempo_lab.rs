use crate::api::get_api_base_url;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessMetric {
    pub name: String,
    pub status: bool,
    pub score: f32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub version: String,
    pub phase: String,
    pub overall_score: f32,
    pub metrics: Vec<ReadinessMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    ReadinessAudit,
    PhaseTransition,
    SimulatedWork,
    SystemSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub activity_type: ActivityType,
    pub status: String,
    pub description: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[component]
pub fn TempoLab() -> Element {
    let mut report = use_signal::<Option<ReadinessReport>>(|| None);
    let _activities = use_signal::<Vec<Activity>>(|| Vec::new());

    let fetch_readiness = move || {
        spawn(async move {
            let url = format!("{}/activity/readiness", get_api_base_url());
            if let Ok(resp) = reqwest::get(&url).await {
                if let Ok(data) = resp.json::<ReadinessReport>().await {
                    report.set(Some(data));
                }
            }
        });
    };

    use_effect(move || {
        fetch_readiness();
    });

    rsx! {
        div { class: "p-6 max-w-5xl mx-auto space-y-8",
            div { class: "flex justify-between items-end border-b border-zinc-800 pb-4",
                div {
                    h1 { class: "text-3xl font-bold text-zinc-100", "Tempo Lab" }
                    p { class: "text-zinc-400 mt-1", "Temporal Strategy Control & Readiness Audit" }
                }
                button {
                    class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-md text-sm font-medium transition-colors",
                    onclick: move |_| fetch_readiness(),
                    "Run Audit"
                }
            }

            if let Some(r) = report() {
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                    // Global Score Card
                    div { class: "col-span-1 bg-zinc-900 border border-zinc-800 rounded-xl p-6 flex flex-col items-center justify-center space-y-2",
                        span { class: "text-zinc-500 text-sm font-medium", "Overall Readiness" }
                        span { class: "text-5xl font-bold text-indigo-400", "{r.overall_score * 100.0:.0}%" }
                        span { class: "text-xs text-zinc-600 px-2 py-1 bg-zinc-800 rounded-full", "{r.version}" }
                    }

                    // Status Info Card
                    div { class: "col-span-3 bg-zinc-900 border border-zinc-800 rounded-xl p-6",
                        h3 { class: "text-lg font-semibold text-zinc-100 mb-2", "{r.phase}" }
                        p { class: "text-sm text-zinc-400", "The system is currently auditing for V1 Ecosystem alignment. All core primitive must reach 80% sufficiency for Phase 2 transition." }

                        div { class: "mt-6 h-2 w-full bg-zinc-800 rounded-full overflow-hidden",
                            div {
                                class: "h-full bg-indigo-500 transition-all duration-1000",
                                style: "width: {r.overall_score * 100.0}%"
                            }
                        }
                    }
                }

                // Metrics List
                div { class: "space-y-4",
                    h3 { class: "text-xl font-semibold text-zinc-200", "Readiness Metrics" }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        for metric in r.metrics {
                            div { class: "bg-zinc-900/50 border border-zinc-800 rounded-lg p-4 space-y-2",
                                div { class: "flex justify-between items-start",
                                    span { class: "font-medium text-zinc-200", "{metric.name}" }
                                    span {
                                        class: if metric.status { "text-green-500" } else { "text-amber-500" },
                                        if metric.status { "PASSED" } else { "PENDING" }
                                    }
                                }
                                p { class: "text-xs text-zinc-400 h-8", "{metric.message}" }
                                div { class: "flex items-center space-x-2",
                                    div { class: "flex-1 h-1.5 bg-zinc-800 rounded-full overflow-hidden",
                                        div {
                                            class: "h-full bg-indigo-600/50",
                                            style: "width: {metric.score * 100.0}%"
                                        }
                                    }
                                    span { class: "text-xs text-zinc-500", "{metric.score * 100.0:.0}%" }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "py-20 flex flex-col items-center justify-center space-y-4 bg-zinc-900/30 rounded-2xl border border-dashed border-zinc-800",
                    div { class: "w-12 h-12 border-4 border-indigo-600 border-t-transparent rounded-full animate-spin" }
                    p { class: "text-zinc-500 font-medium", "Synchronizing with Temporal Scheduler..." }
                }
            }

            // Semantic Activity Stream
            div { class: "space-y-4 pb-12",
                div { class: "flex justify-between items-center",
                    h3 { class: "text-xl font-semibold text-zinc-200", "Activity Stream" }
                    span { class: "text-xs text-zinc-500", "Real-time semantic signals" }
                }

                div { class: "space-y-2",
                    // Placeholder for now
                    div { class: "group bg-zinc-900/40 border border-zinc-800/60 p-4 rounded-lg flex items-center space-x-4",
                        div { class: "w-2 h-2 rounded-full bg-indigo-500 animate-pulse" }
                        div { class: "flex-1",
                            p { class: "text-sm text-zinc-300", "System Initialized: Activity Service started." }
                            span { class: "text-[10px] text-zinc-600", "ID: act_init_001 • Just now" }
                        }
                    }
                }
            }
        }
    }
}
