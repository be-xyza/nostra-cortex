use super::Skill;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use candid::{CandidType, Encode};
use nostra_shared::crypto::{
    ALG_HPKE_X25519_CHACHA20POLY1305, CryptoEnvelope, ENC_VERSION_V1,
    decrypt_hpke_x25519_chacha20poly1305,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::kip_client::KipClient;
use crate::vector_service::VectorService;

pub struct Librarian {
    agent: std::sync::Arc<ic_agent::Agent>,
    streaming_id: ic_agent::export::Principal,
    backend_id: ic_agent::export::Principal,
    hpke_private_key: Option<Vec<u8>>,
    kip: Option<KipClient>,
    vector_service: Option<VectorService>,
}

impl Librarian {
    pub fn new(
        agent: std::sync::Arc<ic_agent::Agent>,
        streaming_id: ic_agent::export::Principal,
        backend_id: ic_agent::export::Principal,
        hpke_private_key: Option<Vec<u8>>,
        kip: Option<KipClient>,
        vector_service: Option<VectorService>,
    ) -> Self {
        Self {
            agent,
            streaming_id,
            backend_id,
            hpke_private_key,
            kip,
            vector_service,
        }
    }
}

// ... (Structs same as before) ...
#[derive(serde::Serialize, candid::CandidType, Clone, Debug)]
struct ChatMessage {
    pub msg_type: String,
    pub content: String,
    pub conversation_id: Option<String>,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIDelta,
}

#[derive(Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}

#[derive(Clone, Debug, Deserialize, CandidType)]
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

fn log_to_file(msg: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    let path = "worker_debug.log";
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", msg);
    }
}

#[async_trait]
impl Skill for Librarian {
    fn name(&self) -> &str {
        "Librarian"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec![
            "knowledge_retrieval",
            "graph_query",
            "chat_interface",
            "llm-inference",
        ]
    }

    async fn execute(&self, payload: String) -> Result<String> {
        log_to_file(&format!(
            "[Librarian] Processing chat query... Payload len: {}",
            payload.len()
        ));
        use candid::Decode;
        use futures::StreamExt;

        // 1. Parse Payload
        let data: HashMap<String, String> = serde_json::from_str(&payload)
            .map_err(|e| anyhow!("Failed to parse payload JSON: {}", e))?;

        let input = data
            .get("input")
            .ok_or_else(|| anyhow!("Missing 'input' in payload"))?;
        let context = data.get("context").cloned().unwrap_or_default();

        // Extract Identity (Support both 'user_id' from Context and 'client_principal')
        let client_str = data
            .get("client_principal")
            .or_else(|| data.get("user_id"))
            .ok_or_else(|| anyhow!("Missing 'client_principal' or 'user_id'"))?;

        let client_principal = ic_agent::export::Principal::from_text(client_str)
            .map_err(|e| anyhow!("Invalid client principal: {}", e))?;
        let conversation_id = data.get("conversation_id").cloned();

        // Extract Space ID for Scoping
        let space_id = data.get("space_id").cloned();
        let scope_filter = space_id.map(|id| format!("space:{}", id));

        log_to_file(&format!(
            "[Librarian] Client Principal: {} (Raw: {}), Scope Filter: {:?}, Payload Keys: {:?}",
            client_principal,
            client_str,
            scope_filter,
            data.keys().collect::<Vec<_>>()
        ));

        // Check for special "Sync" command
        if let Some(cmd) = data.get("command") {
            if cmd == "sync_graph" {
                return self.sync_knowledge_graph().await;
            }
        }

        let mut full_response = String::new();

        // 2. Fetch User Keys (Priority 1)
        let mut api_key = String::new();
        let requested_model = "gpt-4o"; // Our default preference for now

        // Query Backend for Keys
        log_to_file(&format!(
            "[Librarian] Checking for Sovereign Keys for {} (Scope: {:?})...",
            client_principal, scope_filter
        ));
        let key_response = self
            .agent
            .query(&self.backend_id, "getUserKeys")
            .with_arg(Encode!(&client_principal, &scope_filter)?) // Updated Args
            .call()
            .await;

        match key_response {
            Ok(ref bytes) => {
                if let Ok(keys) = Decode!(bytes.as_slice(), Vec<KeyEntry>) {
                    log_to_file(&format!("[Librarian] Found {} keys.", keys.len()));

                    // Selection Logic: Find key with matching model, prioritizing Scoped over Global
                    let matching_model_keys: Vec<&KeyEntry> = keys
                        .iter()
                        .filter(|k| k.model.as_deref() == Some(requested_model))
                        .collect();

                    // Try to find one with a scope first
                    let best_key = matching_model_keys
                        .iter()
                        .find(|k| k.scope.is_some())
                        .or_else(|| matching_model_keys.first())
                        .copied(); // Deref

                    // If no matching model, fallback to ANY key (scoped preferred)
                    let fallback_key = if best_key.is_none() {
                        keys.iter().find(|k| k.scope.is_some()).or(keys.first())
                    } else {
                        best_key
                    };

                    if let Some(key_entry) = fallback_key {
                        log_to_file(&format!(
                            "[Librarian] Selected Key: '{}' (Model: {:?}, Scope: {:?}). Decrypting...",
                            key_entry.key_label, key_entry.model, key_entry.scope
                        ));
                        let alg = key_entry
                            .alg
                            .as_deref()
                            .unwrap_or(ALG_HPKE_X25519_CHACHA20POLY1305);
                        if alg != ALG_HPKE_X25519_CHACHA20POLY1305 {
                            log_to_file(&format!(
                                "[Librarian] Unsupported key alg '{}' (HPKE required).",
                                alg
                            ));
                        } else if let Some(hpke_private_key) = &self.hpke_private_key {
                            let envelope = CryptoEnvelope {
                                alg: alg.to_string(),
                                enc_version: key_entry.enc_version.unwrap_or(ENC_VERSION_V1 as u64)
                                    as u32,
                                key_id: key_entry.key_id.clone().unwrap_or_default(),
                                ephemeral_pub_key: key_entry
                                    .ephemeral_pub_key
                                    .clone()
                                    .unwrap_or_default(),
                                ciphertext: key_entry.encrypted_key.clone(),
                            };
                            match decrypt_hpke_x25519_chacha20poly1305(&envelope, hpke_private_key)
                            {
                                Ok(decrypted) => {
                                    if let Ok(key_str) = String::from_utf8(decrypted) {
                                        api_key = key_str;
                                        log_to_file("[Librarian] Decryption Successful (HPKE).");
                                    }
                                }
                                Err(e) => log_to_file(&format!(
                                    "[Librarian] Decryption Failed (HPKE): {}",
                                    e
                                )),
                            }
                        } else {
                            log_to_file("[Librarian] HPKE key unavailable; cannot decrypt.");
                        }
                    } else {
                        log_to_file("[Librarian] No keys available in list.");
                    }
                } else {
                    log_to_file("[Librarian] Failed to decode KeyEntry list.");
                }
            }
            Err(ref e) => log_to_file(&format!("[Librarian] Key Query Failed: {}", e)),
        };

        // Fallback to ENV KEY (Priority 2)
        if api_key.is_empty() {
            if let Ok(env_key) = std::env::var("OPENAI_API_KEY") {
                api_key = env_key;
                log_to_file("[Librarian] Using Environment Key.");
            }
        }

        // 3. Execution Logic
        if api_key.is_empty() {
            log_to_file("[Librarian] No API Key found. Simulating...");
            // Simulation Logic
            let debug_reason = if key_response.is_err() {
                format!(
                    "(Backend Call Failed: {})",
                    key_response.as_ref().err().unwrap()
                )
            } else if let Ok(bytes) = key_response.as_ref() {
                if let Ok(keys) = Decode!(bytes.as_slice(), Vec<KeyEntry>) {
                    if keys.is_empty() {
                        format!("(No keys found for {})", client_principal)
                    } else {
                        "(Decryption Failed or No Matching Key)".to_string()
                    }
                } else {
                    "(Decode Failed)".to_string()
                }
            } else {
                "(Unknown Error)".to_string()
            };

            let debug_msg = format!("(Simulated - {}) !", debug_reason);
            let tokens = vec!["Hello", " ", "from", " ", "Nostra", " ", &debug_msg];

            for token in tokens {
                full_response.push_str(token);
                let msg = ChatMessage {
                    msg_type: "ai_token".to_string(),
                    content: token.to_string(),
                    conversation_id: conversation_id.clone(),
                };
                let _ = self
                    .agent
                    .update(&self.streaming_id, "send_message_to_client")
                    .with_arg(Encode!(&client_principal, &msg)?)
                    .call_and_wait()
                    .await;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        } else {
            // Real LLM Call
            log_to_file("[Librarian] Calling OpenAI...");
            let client = reqwest::Client::new();

            // Determine System Prompt based on Intent
            let (system_prompt, clean_input) = if input.trim().starts_with("[AG-UI Request]") {
                let prompt = "You are an expert AG-UI Designer (Agent-Generated UI) for the Nostra Protocol.
Your goal is to generate a VALID AG-UI JSON array based on the user's request.
The AG-UI Protocol supports the following component types:
- Notification { variant: 'primary'|'success'|'warning'|'danger', message: String, icon: Option<String> }
- Details { summary: String, children: [Component] }
- Input { label: String, name: String, value: Option<String>, placeholder: Option<String> }
- Select { label: String, name: String, options: [{label: String, value: String}] }
- Action { label: String, actionId: String, variant: 'primary'|'neutral'|'danger'|'success' }
- Text { text: String }
- Row { children: [Component] }
- Column { children: [Component] }

Rules:
1. Return ONLY the JSON array. No markdown formatting (no ```json), no validatory text.
2. The root element must be an Array [ ... ] of components.
3. Be creative but strict with the schema.
4. If you aren't sure, allow the user to input data using Input components.
";
                let cleaned = input.replacen("[AG-UI Request]", "", 1);
                (prompt.to_string(), cleaned)
            } else {
                (
                    format!("You are Nostra Librarian. Context:\n{}", context),
                    input.clone(),
                )
            };

            let request_body = OpenAIRequest {
                model: "gpt-4-turbo-preview".to_string(),
                messages: vec![
                    OpenAIMessage {
                        role: "system".to_string(),
                        content: system_prompt,
                    },
                    OpenAIMessage {
                        role: "user".to_string(),
                        content: clean_input,
                    },
                ],
                temperature: 0.7,
                stream: true,
            };

            // ... Same streaming logic structure ...
            let res = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await;

            match res {
                Ok(response) => {
                    log_to_file(&format!(
                        "[Librarian] OpenAI Response Status: {}",
                        response.status()
                    ));
                    if !response.status().is_success() {
                        log_to_file(&format!(
                            "[Librarian] Error Text: {:?}",
                            response.text().await
                        ));
                    } else {
                        let mut stream = response.bytes_stream();
                        while let Some(item) = stream.next().await {
                            // ... (Existing Chunk Parsing Logic) ...
                            if let Ok(chunk) = item {
                                let s = String::from_utf8_lossy(&chunk);
                                for line in s.lines() {
                                    if let Some(json_str) = line.strip_prefix("data: ") {
                                        if json_str != "[DONE]" {
                                            if let Ok(resp) =
                                                serde_json::from_str::<OpenAIStreamChunk>(json_str)
                                            {
                                                if let Some(delta) = resp
                                                    .choices
                                                    .first()
                                                    .and_then(|c| c.delta.content.as_ref())
                                                {
                                                    full_response.push_str(delta);
                                                    let msg = ChatMessage {
                                                        msg_type: "ai_token".to_string(),
                                                        content: delta.clone(),
                                                        conversation_id: conversation_id.clone(),
                                                    };
                                                    let _ = self
                                                        .agent
                                                        .update(
                                                            &self.streaming_id,
                                                            "send_message_to_client",
                                                        )
                                                        .with_arg(Encode!(&client_principal, &msg)?)
                                                        .call_and_wait()
                                                        .await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log_to_file(&format!("[Librarian] OpenAI Request Error: {}", e));
                    let error_msg = ChatMessage {
                        msg_type: "ai_token".to_string(),
                        content: format!("Error: {}", e),
                        conversation_id: conversation_id.clone(),
                    };
                    let _ = self
                        .agent
                        .update(&self.streaming_id, "send_message_to_client")
                        .with_arg(Encode!(&client_principal, &error_msg)?)
                        .call_and_wait()
                        .await;
                }
            }
        }

        // Complete Signal
        let msg = ChatMessage {
            msg_type: "ai_complete".to_string(),
            content: "".to_string(),
            conversation_id: conversation_id.clone(),
        };
        let _ = self
            .agent
            .update(&self.streaming_id, "send_message_to_client")
            .with_arg(Encode!(&client_principal, &msg)?)
            .call_and_wait()
            .await;

        // Persist
        let _ = self
            .agent
            .update(&self.backend_id, "appendChatMessage")
            .with_arg(Encode!(&"assistant", &full_response)?)
            .call_and_wait()
            .await;

        Ok("Done".to_string())
    }
}

impl Librarian {
    async fn sync_knowledge_graph(&self) -> Result<String> {
        println!("   [Librarian] Syncing Knowledge Graph to Vector Store...");

        let kip = self
            .kip
            .as_ref()
            .ok_or_else(|| anyhow!("KIP Client not configured"))?;

        let vector_service = self
            .vector_service
            .as_ref()
            .ok_or_else(|| anyhow!("Vector Service not configured"))?;

        // 1. Fetch Concepts (Naive fetch all for prototype)
        // In real impl, we'd use a watermark/timestamp
        let query = "FIND ConceptNode";
        let _response = kip.query(query).await?;

        // Mock parsing the response
        // For the PROTOTYPE, we will just create a few "Synthetic" concepts to prove the pipeline.
        let synthetic_concepts = [
            ("concept_1", "Nostra Schema Standards"),
            ("concept_2", "Vector Database Integration"),
            ("concept_3", "Autonomous Coding Agent"),
            ("concept_4", "Slice-of-Day Resource Adequacy"),
            ("concept_5", "Temporal Workflow Engine"),
        ];

        println!(
            "   [Librarian] Found {} concepts to sync.",
            synthetic_concepts.len()
        );

        // 2. Prepare for VectorService (id, text, label)
        let batch: Vec<(&str, &str, &str)> = synthetic_concepts
            .iter()
            .map(|(id, text)| (*id, *text, "active_concept"))
            .collect();

        // 3. Initialize Collection (Idempotent)
        let _ = vector_service.init_collection().await;

        // 4. Index via Micro-Batching
        // This will automatically handle chunking and embedding generation
        vector_service
            .index_batch(batch)
            .await
            .map_err(|e| anyhow!("VectorService failure: {}", e))?;

        Ok(format!("Synced {} concepts", synthetic_concepts.len()))
    }
}

#[allow(dead_code)]
fn log_error(msg: &str) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Assuming worker runs in nostra/worker, relative path to logs/nostra is ../../logs/nostra
    let path = "../../logs/nostra/worker.log";

    // Ensure dir exists - best effort
    let _ = std::fs::create_dir_all("../../logs/nostra");

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    writeln!(file, "[{}] Error: {}", now, msg)?;
    Ok(())
}
