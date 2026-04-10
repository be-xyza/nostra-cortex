use dioxus::prelude::*;

#[component]
pub fn IntelligenceSettings() -> Element {
    let mut use_custom_gateway = use_signal(|| false);
    let mut gateway_url = use_signal(|| "http://localhost:8080/v1".to_string());
    let mut safe_mode = use_signal(|| true); // Require tool approval
    let mut api_key = use_signal(|| "".to_string());

    rsx! {
        div { class: "space-y-6 animate-in fade-in slide-in-from-right-4",
            div { class: "border-b pb-4",
                h3 { class: "text-lg font-medium", "Intelligence & Gateways" }
                p { class: "text-sm text-muted-foreground",
                    "Configure how Nostra connects to AI agents and external tools."
                }
            }

            // Safe Mode Toggle
            div { class: "flex items-center justify-between p-4 border rounded-lg bg-muted/20",
                div { class: "space-y-0.5",
                    label { class: "text-base font-medium", "Safe Mode" }
                    p { class: "text-xs text-muted-foreground",
                        "Require manual approval for all agent tool executions (recommended)."
                    }
                }
                input {
                    type: "checkbox",
                    class: "toggle toggle-primary",
                    checked: "{safe_mode}",
                    oninput: move |evt| safe_mode.set(evt.value() == "true")
                }
            }

            // Gateway Configuration
            div { class: "space-y-4",
                 div { class: "flex items-center justify-between",
                    label { class: "text-sm font-medium", "Use Custom Gateway (OpenAI Compatible)" }
                    input {
                        type: "checkbox",
                         class: "h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary",
                        checked: "{use_custom_gateway}",
                        oninput: move |evt| use_custom_gateway.set(evt.value() == "true")
                    }
                }

                if use_custom_gateway() {
                    div { class: "space-y-3 p-4 border rounded-lg bg-card",
                        div {
                            label { class: "block text-xs font-medium mb-1", "Gateway Base URL" }
                            input {
                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm",
                                value: "{gateway_url}",
                                oninput: move |evt| gateway_url.set(evt.value()),
                                placeholder: "e.g. http://localhost:8080/v1"
                            }
                        }

                        div {
                            label { class: "block text-xs font-medium mb-1", "API Key (Optional)" }
                            input {
                                type: "password",
                                class: "w-full bg-background border border-input rounded-md px-3 py-2 text-sm",
                                value: "{api_key}",
                                oninput: move |evt| api_key.set(evt.value()),
                                placeholder: "sk-..."
                            }
                        }
                    }
                }
            }
        }
    }
}
