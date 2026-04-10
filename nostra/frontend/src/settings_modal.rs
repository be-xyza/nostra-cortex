#![allow(dead_code)]

use crate::api::create_agent;
use candid::{CandidType, Decode, Deserialize, Encode};
use dioxus::prelude::*;
use ic_agent::Agent;
use nostra_shared::crypto::{
    encrypt_hpke_x25519_chacha20poly1305, ALG_HPKE_X25519_CHACHA20POLY1305, ENC_VERSION_V1,
};

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
struct KeyEntry {
    id: String,
    #[serde(rename = "keyLabel")]
    key_label: String,
    model: Option<String>,
    scope: Option<String>, // Added scope
    #[serde(rename = "encryptedKey")]
    encrypted_key: Vec<u8>,
    #[serde(rename = "ephemeralPubKey")]
    ephemeral_pub_key: Option<Vec<u8>>,
    alg: Option<String>,
    #[serde(rename = "encVersion")]
    enc_version: Option<u64>,
    #[serde(rename = "keyId")]
    key_id: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: candid::Int,
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
struct WorkerConfig {
    #[serde(rename = "workerId")]
    worker_id: String,
    #[serde(rename = "publicKey")]
    public_key: Vec<u8>,
    #[serde(rename = "keyId")]
    key_id: Option<String>,
    alg: Option<String>,
    #[serde(rename = "encVersion")]
    enc_version: Option<u64>,
    #[serde(rename = "registeredAt")]
    registered_at: candid::Int,
}

// ...

async fn fetch_user_keys() -> anyhow::Result<Vec<KeyEntry>> {
    let agent = create_agent().await;
    let backend_id = get_backend_id();
    let backend_principal = ic_agent::export::Principal::from_text(&backend_id).unwrap();

    let scope_filter: Option<String> = None; // Get all keys

    let response = agent
        .query(&backend_principal, "getMyKeys")
        .with_arg(Encode!(&scope_filter)?)
        .call()
        .await?;

    Ok(Decode!(response.as_slice(), Vec<KeyEntry>)?)
}

async fn add_key_v2(
    agent: &Agent,
    backend_id: ic_agent::export::Principal,
    id: String,
    label: String,
    model: Option<String>,
    scope: Option<String>,
    encrypted_key: Vec<u8>,
    ephemeral_pub_key: Option<Vec<u8>>,
    alg: Option<String>,
    enc_version: Option<u64>,
    key_id: Option<String>,
) -> anyhow::Result<()> {
    let _ = agent
        .update(&backend_id, "addKeyV2")
        .with_arg(Encode!(
            &id,
            &label,
            &model,
            &scope,
            &encrypted_key,
            &ephemeral_pub_key,
            &alg,
            &enc_version,
            &key_id
        )?)
        .call_and_wait()
        .await?;
    Ok(())
}

use crate::pages::settings::intelligence::IntelligenceSettings;

#[derive(Clone, PartialEq)]
enum SettingsTab {
    Identity,
    Intelligence,
}

#[component]
pub fn SettingsModal(on_close: EventHandler<()>) -> Element {
    let mut active_tab = use_signal(|| SettingsTab::Identity);

    // Identity Tab State
    let mut keys = use_signal(|| Vec::<KeyEntry>::new());
    let mut is_loading = use_signal(|| true);
    let mut status_msg = use_signal(|| "".to_string());
    let mut show_add_form = use_signal(|| false);

    // ... (Keep existing Identity State variables: form_label, form_key, etc.)
    let mut form_label = use_signal(|| "".to_string());
    let mut form_key = use_signal(|| "".to_string());
    let mut form_model = use_signal(|| "gpt-4o".to_string());
    let mut form_scope = use_signal(|| "global".to_string());
    let mut is_saving = use_signal(|| false);

    // Initial Fetch (Only if on Identity tab? Or just fetch once)
    use_effect(move || {
        spawn(async move {
            if let Ok(fetched_keys) = fetch_user_keys().await {
                keys.set(fetched_keys);
            }
            is_loading.set(false);
        });
    });

    // ... (Keep existing helpers locally or inside the component)
    let handle_delete = move |id: String| {
        spawn(async move {
            status_msg.set("Deleting...".to_string());
            if let Ok(_) = delete_key(id.clone()).await {
                let current_keys = keys();
                keys.set(current_keys.into_iter().filter(|k| k.id != id).collect());
                status_msg.set("Key Removed".to_string());
            } else {
                status_msg.set("Delete Failed".to_string());
            }
        });
    };

    let handle_save = move |_| {
        let label = form_label();
        let key = form_key();
        let model = form_model();
        let scope_val = form_scope();

        let scope =
            if scope_val.trim().eq_ignore_ascii_case("global") || scope_val.trim().is_empty() {
                Some("global".to_string())
            } else {
                Some(scope_val.trim().to_string())
            };

        if label.is_empty() || key.is_empty() {
            status_msg.set("Label and Key are required".to_string());
            return;
        }

        spawn(async move {
            is_saving.set(true);
            status_msg.set("Encrypting & Saving...".to_string());

            let agent = create_agent().await;
            let backend_id = get_backend_id();
            let backend_principal = ic_agent::export::Principal::from_text(&backend_id).unwrap();

            match get_worker_config(&agent, backend_principal).await {
                Ok(Some(worker_config)) => {
                    let alg = worker_config
                        .alg
                        .clone()
                        .unwrap_or_else(|| ALG_HPKE_X25519_CHACHA20POLY1305.to_string());
                    let enc_version = worker_config
                        .enc_version
                        .or(Some(ENC_VERSION_V1 as u64));
                    let key_id = worker_config.key_id.clone();

                    if alg != ALG_HPKE_X25519_CHACHA20POLY1305 {
                        status_msg.set("HPKE required (RSA disabled).".to_string());
                        is_saving.set(false);
                        return;
                    }

                    let encryption_result = encrypt_hpke_x25519_chacha20poly1305(
                        key.as_bytes(),
                        &worker_config.public_key,
                        key_id.as_deref().unwrap_or("unknown"),
                    )
                    .map(|envelope| (envelope.ciphertext, Some(envelope.ephemeral_pub_key)))
                    .map_err(|e| anyhow::anyhow!(e.to_string()));

                    match encryption_result {
                        Ok((encrypted_bytes, ephemeral_pub_key)) => {
                            let id = uuid::Uuid::new_v4().to_string();
                            match add_key_v2(
                                &agent,
                                backend_principal,
                                id.clone(),
                                label.clone(),
                                Some(model.clone()),
                                scope.clone(),
                                encrypted_bytes,
                                ephemeral_pub_key,
                                Some(alg),
                                enc_version,
                                key_id,
                            )
                            .await
                            {
                                Ok(_) => {
                                    if let Ok(fetched) = fetch_user_keys().await {
                                        keys.set(fetched);
                                    }
                                    form_label.set("".to_string());
                                    form_key.set("".to_string());
                                    form_scope.set("global".to_string());
                                    show_add_form.set(false);
                                    status_msg.set("Key Saved!".to_string());
                                }
                                Err(e) => status_msg.set(format!("Save Failed: {}", e)),
                            }
                        }
                        Err(e) => status_msg.set(format!("Encryption Failed: {}", e)),
                    }
                }
                Ok(None) => status_msg.set("No workers found to encrypt for.".to_string()),
                Err(e) => status_msg.set(format!("Fetch Worker Key Failed: {}", e)),
            }
            is_saving.set(false);
        });
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm",
            div { class: "bg-background border border-border rounded-lg shadow-lg w-full max-w-2xl h-[600px] flex overflow-hidden relative animate-in fade-in zoom-in duration-200",

                // Close Button
                button { class: "absolute top-4 right-4 z-50 text-muted-foreground hover:text-foreground",
                    onclick: move |_| on_close.call(()),
                    svg { class: "w-5 h-5", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                        path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M6 18L18 6M6 6l12 12" }
                    }
                }

                // Sidebar
                div { class: "w-48 border-r bg-muted/30 p-4 flex flex-col gap-2",
                    h2 { class: "font-semibold text-sm px-2 mb-2", "Settings" }

                    button {
                        class: format!("w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors {}",
                            if active_tab() == SettingsTab::Identity { "bg-muted text-foreground" } else { "text-muted-foreground hover:text-foreground hover:bg-muted/50" }),
                        onclick: move |_| active_tab.set(SettingsTab::Identity),
                        "Identity & Keys"
                    }
                    button {
                        class: format!("w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors {}",
                            if active_tab() == SettingsTab::Intelligence { "bg-muted text-foreground" } else { "text-muted-foreground hover:text-foreground hover:bg-muted/50" }),
                        onclick: move |_| active_tab.set(SettingsTab::Intelligence),
                        "Intelligence"
                    }
                }

                // Content Area
                div { class: "flex-1 p-6 overflow-y-auto",
                    match active_tab() {
                        SettingsTab::Identity => rsx! {
                            div {
                                h2 { class: "text-lg font-semibold mb-4", "Identity Settings" }
                                if !status_msg().is_empty() {
                                    div { class: "text-xs p-2 rounded bg-muted text-foreground mb-4", "{status_msg}" }
                                }

                                if show_add_form() {
                                    // ... (Existing Add Key Form)
                                    div { class: "space-y-4 border rounded p-4 bg-muted/20",
                                        div { class: "flex justify-between items-center",
                                             h3 { class: "font-medium text-sm", "Add New Key" }
                                             button { onclick: move |_| show_add_form.set(false), class: "text-xs text-muted-foreground hover:text-foreground", "Cancel" }
                                        }

                                        div {
                                            label { class: "block text-xs font-medium mb-1", "Label" }
                                            input {
                                                class: "w-full bg-muted/50 border border-input rounded-md px-3 py-2 text-sm",
                                                placeholder: "e.g. My Pro Key",
                                                value: "{form_label}",
                                                oninput: move |evt| form_label.set(evt.value()),
                                            }
                                        }

                                         div {
                                            label { class: "block text-xs font-medium mb-1", "Model Preference" }
                                            select {
                                                class: "w-full bg-muted/50 border border-input rounded-md px-3 py-2 text-sm",
                                                value: "{form_model}",
                                                oninput: move |evt| form_model.set(evt.value()),
                                                option { value: "gpt-4o", "GPT-4o" }
                                                option { value: "gpt-4-turbo", "GPT-4 Turbo" }
                                            }
                                        }

                                         div {
                                            label { class: "block text-xs font-medium mb-1", "Scope (Optional)" }
                                            input {
                                                class: "w-full bg-muted/50 border border-input rounded-md px-3 py-2 text-sm",
                                                placeholder: "global OR space:<uuid>",
                                                value: "{form_scope}",
                                                oninput: move |evt| form_scope.set(evt.value()),
                                            }
                                        }

                                         div {
                                            label { class: "block text-xs font-medium mb-1", "API Key" }
                                            input {
                                                class: "w-full bg-muted/50 border border-input rounded-md px-3 py-2 text-sm",
                                                type: "password",
                                                placeholder: "sk-...",
                                                value: "{form_key}",
                                                oninput: move |evt| form_key.set(evt.value()),
                                            }
                                        }

                                        button { class: "w-full px-4 py-2 text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 rounded-md transition-colors disabled:opacity-50",
                                            onclick: handle_save,
                                             disabled: "{is_saving}",
                                             if is_saving() { "Encrypting..." } else { "Save" }
                                        }
                                    }
                                } else {
                                     div { class: "space-y-4",
                                         div { class: "flex justify-between items-center",
                                              h3 { class: "text-sm font-medium", "My Keys" }
                                              button {
                                                  class: "text-xs bg-primary text-primary-foreground px-2 py-1 rounded hover:bg-primary/90",
                                                  onclick: move |_| show_add_form.set(true),
                                                  "+ Add Key"
                                              }
                                         }

                                         if is_loading() {
                                             div { class: "text-center text-sm text-muted-foreground", "Loading keys..." }
                                         } else if keys().is_empty() {
                                             div { class: "text-center text-sm text-muted-foreground py-4 border border-dashed rounded", "No keys stored." }
                                         } else {
                                             div { class: "space-y-2 max-h-[300px] overflow-y-auto",
                                                 for key in keys() {
                                                     div { class: "flex items-center justify-between p-3 bg-muted/30 rounded border",
                                                         div {
                                                             div { class: "font-medium text-sm", "{key.key_label}" }
                                                             div { class: "text-xs text-muted-foreground flex gap-2",
                                                                 span { if let Some(m) = &key.model { "{m}" } else { "Any" } }
                                                                 span { class: "text-muted-foreground/50", "|" }
                                                                 span { if let Some(s) = &key.scope { "{s}" } else { "global" } }
                                                             }
                                                         }
                                                         button {
                                                             class: "text-red-500 hover:text-red-700 p-1",
                                                             onclick: move |_| handle_delete(key.id.clone()),
                                                             svg { class: "w-4 h-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                                                 path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" }
                                                             }
                                                         }
                                                     }
                                                 }
                                             }
                                         }
                                     }
                                }
                            }
                        },
                        SettingsTab::Intelligence => rsx! {
                            IntelligenceSettings {}
                        }
                    }
                }
            }
        }
    }
}

// -- Helpers --

fn get_backend_id() -> String {
    std::env::var("CANISTER_ID_NOSTRA_BACKEND")
        .or_else(|_| std::env::var("CANISTER_ID"))
        .unwrap_or("uxrrr-q7777-77774-qaaaq-cai".to_string())
}

async fn delete_key(id: String) -> anyhow::Result<()> {
    let agent = create_agent().await;
    let backend_id = get_backend_id();
    let backend_principal = ic_agent::export::Principal::from_text(&backend_id).unwrap();

    let _ = agent
        .update(&backend_principal, "removeKey")
        .with_arg(Encode!(&id)?)
        .call_and_wait()
        .await?;
    Ok(())
}

async fn get_worker_config(
    agent: &Agent,
    backend_id: ic_agent::export::Principal,
) -> anyhow::Result<Option<WorkerConfig>> {
    let response = agent
        .query(&backend_id, "getWorkerConfig")
        .with_arg(Encode!()?)
        .call()
        .await?;
    Ok(Decode!(response.as_slice(), Option<WorkerConfig>).unwrap_or(None))
}
