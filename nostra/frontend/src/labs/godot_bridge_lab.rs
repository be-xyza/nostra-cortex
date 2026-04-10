use dioxus::document::eval;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Option<Value>,
}

#[component]
pub fn GodotBridgeLab(on_back: EventHandler<()>) -> Element {
    let mut logs = use_signal(|| Vec::<String>::new());
    let mut iframe_loaded = use_signal(|| false);

    // Message Handler
    use_future(move || async move {
        // We use a simplified eval loop here.
        // In a production app, we would likely use a dedicated service or hook.
        let mut msg_eval = eval(
            r#"
            window.addEventListener('message', (event) => {
                // In a real app, check event.origin!
                // For this lab, we accept messages from our own origin (iframe is same origin)
                if (event.data && event.data.jsonrpc) {
                    dioxus.send(event.data);
                }
            });
            // Signal ready
            dioxus.send("READY");
        "#,
        );

        while let Ok(msg) = msg_eval.recv::<Value>().await {
            if let Some(str_val) = msg.as_str() {
                if str_val == "READY" {
                    continue;
                }
            }

            // Log the incoming message
            logs.write().push(format!("Received: {:?}", msg));

            // Parse JSON-RPC
            if let Ok(req) = serde_json::from_value::<JsonRpcRequest>(msg.clone()) {
                match req.method.as_str() {
                    "HANDSHAKE" => {
                        logs.write()
                            .push("Handshake requested. Sending ACK.".to_string());
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::json!({"status": "READY", "host": "Nostra"})),
                            error: None,
                            id: req.id.clone(),
                        };
                        // Send back to iframe
                        let _ = eval(&format!(
                            "let iframe = document.getElementById('godot-frame');
                              if (iframe) {{
                                  iframe.contentWindow.postMessage({}, '*');
                              }}",
                            serde_json::to_string(&response).unwrap()
                        ));
                    }
                    "DELEGATE_REQUEST" => {
                        logs.write()
                            .push("Delegation requested. Simulating auth...".to_string());
                        // Simulate Async Auth Delay
                        gloo_timers::future::TimeoutFuture::new(1000).await;

                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::json!({
                                "delegation": {
                                    "chain": "mock_chain_data",
                                    "expiration": "2026-01-01T00:00:00Z"
                                }
                            })),
                            error: None,
                            id: req.id,
                        };

                        let _ = eval(&format!(
                            "let iframe = document.getElementById('godot-frame');
                              if (iframe) {{
                                  iframe.contentWindow.postMessage({}, '*');
                              }}",
                            serde_json::to_string(&response).unwrap()
                        ));
                        logs.write().push("Delegation sent.".to_string());
                    }
                    _ => {
                        logs.write().push(format!("Unknown method: {}", req.method));
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "flex h-full w-full bg-background text-foreground flex-col",
            // Header
            div { class: "p-4 border-b flex items-center justify-between bg-muted/20",
                div { class: "flex items-center gap-4",
                    button {
                        class: "p-2 hover:bg-muted rounded-full transition-colors",
                        onclick: move |_| on_back.call(()),
                        svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M10 19l-7-7m0 0l7-7m-7 7h18" }
                        }
                    }
                    div {
                        h2 { class: "text-lg font-semibold", "Godot Bridge Lab" }
                        p { class: "text-xs text-muted-foreground", "Testing JSON-RPC over window.postMessage" }
                    }
                }
            }

            // Main Content
            div { class: "flex-1 flex overflow-hidden",
                // Left: Godot Game Container (Iframe)
                div { class: "flex-1 bg-black relative flex items-center justify-center",
                    iframe {
                        id: "godot-frame",
                        src: "/labs/godot-bridge/mock_client.html",
                        class: "w-full h-full border-none",
                        onload: move |_| iframe_loaded.set(true)
                    }
                }

                // Right: Host Debug Console
                div { class: "w-96 border-l bg-card flex flex-col",
                    div { class: "p-3 border-b font-mono text-sm font-semibold bg-muted/50", "Host Bridge Protocol Log" }
                    div { class: "flex-1 overflow-y-auto p-4 font-mono text-xs space-y-2",
                        for log in logs() {
                            div { class: "border-b border-border/50 pb-1 mb-1 break-all", "{log}" }
                        }
                        if logs().is_empty() {
                            div { class: "text-muted-foreground italic", "Waiting for messages..." }
                        }
                    }
                }
            }
        }
    }
}
