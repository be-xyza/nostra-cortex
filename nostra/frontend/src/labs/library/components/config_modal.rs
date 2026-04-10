use crate::labs::library::types::{CortexOverlayMode, LabConfig, SyncTarget};
use dioxus::prelude::*;

#[component]
pub fn ConfigModal(show_config: Signal<bool>, config: Signal<LabConfig>) -> Element {
    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200",
            div { class: "bg-card w-full max-w-md rounded-2xl border shadow-2xl overflow-hidden animate-in zoom-in-95 duration-200",
                // Header
                div { class: "p-6 border-b flex justify-between items-center bg-muted/30",
                    div {
                        h2 { class: "text-lg font-bold", "Lab Configuration" }
                        p { class: "text-xs text-muted-foreground", "Configure experimental library behaviors" }
                    }
                    button {
                        class: "p-2 hover:bg-muted rounded-full transition-colors",
                        onclick: move |_| show_config.set(false),
                        "×"
                    }
                }

                // Body
                div { class: "p-6 space-y-6",
                    // Sync Target
                    div { class: "space-y-2",
                        label { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground", "Sync Target" }
                        div { class: "grid grid-cols-2 gap-2",
                            for target in [SyncTarget::Local, SyncTarget::Canister] {
                                button {
                                    class: format!("py-2 px-3 rounded-lg border text-sm transition-all {}",
                                        if config.read().sync_target == target { "bg-primary text-primary-foreground border-primary shadow-sm" } else { "bg-muted/50 hover:bg-muted" }
                                    ),
                                    onclick: {
                                        let target = target.clone();
                                        move |_| config.write().sync_target = target.clone()
                                    },
                                    "{target:?}"
                                }
                            }
                        }
                    }

                    // Telemetry Toggle
                    div { class: "flex items-center justify-between p-3 bg-muted/30 rounded-xl border border-dashed",
                        div {
                            p { class: "text-sm font-medium", "Enable Telemetry" }
                            p { class: "text-[10px] text-muted-foreground", "Track reading heatmap & progress (OTel 054)" }
                        }
                        button {
                            class: format!("w-10 h-5 rounded-full transition-colors relative {}", if config.read().telemetry_enabled { "bg-blue-500" } else { "bg-gray-400" }),
                            onclick: move |_| {
                                let current = config.read().telemetry_enabled;
                                config.write().telemetry_enabled = !current;
                            },
                            div { class: format!("absolute top-1 w-3 h-3 rounded-full bg-white transition-all shadow-sm {}", if config.read().telemetry_enabled { "left-6" } else { "left-1" }) }
                        }
                    }

                    // Cortex Mode
                    div { class: "space-y-2",
                        label { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground", "Cortex Overlay (037)" }
                        select {
                            class: "w-full p-2.5 bg-background border rounded-lg text-sm focus:ring-2 focus:ring-primary",
                            onchange: move |evt| {
                                config.write().cortex_overlay = match evt.value().as_str() {
                                    "Hidden" => CortexOverlayMode::Hidden,
                                    "Active" => CortexOverlayMode::Active,
                                    _ => CortexOverlayMode::Passive,
                                };
                            },
                            option { selected: config.read().cortex_overlay == CortexOverlayMode::Hidden, "Hidden" }
                            option { selected: config.read().cortex_overlay == CortexOverlayMode::Passive, "Passive" }
                            option { selected: config.read().cortex_overlay == CortexOverlayMode::Active, "Active" }
                        }
                    }

                    // Treaty Enforcement
                    div { class: "space-y-2",
                        label { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground", "Cross-Space Treaty" }
                        div { class: "flex items-center justify-between p-3 bg-muted/30 rounded-xl border border-dashed",
                            div {
                                p { class: "text-sm font-medium", "Enforce Treaty" }
                                p { class: "text-[10px] text-muted-foreground", "Block cross-space reads without token" }
                            }
                            button {
                                class: format!("w-10 h-5 rounded-full transition-colors relative {}", if config.read().enforce_treaty { "bg-blue-500" } else { "bg-gray-400" }),
                                onclick: move |_| {
                                    let current = config.read().enforce_treaty;
                                    config.write().enforce_treaty = !current;
                                },
                                div { class: format!("absolute top-1 w-3 h-3 rounded-full bg-white transition-all shadow-sm {}", if config.read().enforce_treaty { "left-6" } else { "left-1" }) }
                            }
                        }
                        input {
                            class: "w-full bg-background border border-input rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-ring",
                            placeholder: "Current Space DID (e.g. did:nostra:space:legacy)",
                            value: "{config.read().current_space_did.clone().unwrap_or_default()}",
                            oninput: move |evt| {
                                let v = evt.value();
                                let trimmed = v.trim().to_string();
                                if trimmed.is_empty() {
                                    config.write().current_space_did = None;
                                } else {
                                    config.write().current_space_did = Some(trimmed);
                                }
                            }
                        }
                        input {
                            class: "w-full bg-background border border-input rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-ring",
                            placeholder: "Treaty Token (optional)",
                            value: "{config.read().treaty_token.clone().unwrap_or_default()}",
                            oninput: move |evt| {
                                let v = evt.value();
                                let trimmed = v.trim().to_string();
                                if trimmed.is_empty() {
                                    config.write().treaty_token = None;
                                } else {
                                    config.write().treaty_token = Some(trimmed);
                                }
                            }
                        }
                    }
                }

                // Footer
                div { class: "p-4 bg-muted/50 border-t flex justify-end gap-2",
                    button {
                        class: "px-4 py-2 text-sm font-medium hover:bg-muted rounded-lg transition-colors",
                        onclick: move |_| show_config.set(false),
                        "Close"
                    }
                }
            }
        }
    }
}
