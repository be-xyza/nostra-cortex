use anyhow::Result;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_agent::export::Principal;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
// Crypto
use nostra_shared::crypto::{ALG_HPKE_X25519_CHACHA20POLY1305, ENC_VERSION_V1};
use rand::rngs::OsRng;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

// New imports
use nostra_core::interceptor::{MiddlewareStack, TaskContext};
use nostra_core::telemetry::OtelInterceptor;

// Modules are now defined in lib.rs
use cortex_worker::agent_builder::build_agent_with_resolved_identity;
use cortex_worker::config_service::ConfigService;
use cortex_worker::embedding_provider::EmbeddingProvider;
use cortex_worker::kip_client::KipClient;
use cortex_worker::ollama_embedder::OllamaEmbedder;
use cortex_worker::provider_routing::{EmbeddingRoute, build_provider, resolve_embedding_route};
use cortex_worker::skills::{
    Skill, analyst::Analyst, architect::Architect, dev::Dev, gardener::Gardener,
    hrm_scheduler::HrmScheduler, librarian::Librarian, pm::Pm, qa::Qa,
};
use cortex_worker::vector_service::VectorService;
use cortex_worker::{api, config_service, workflows};

const WORKER_KEYS_PATH_ENV: &str = "NOSTRA_WORKER_KEYS_PATH";
const DEFAULT_WORKER_KEYS_PATH: &str = "worker_keys.json";

#[derive(Serialize, SerdeDeserialize)]
struct WorkerKeyStoreV1 {
    key_id: String,
    rsa_private_key_der: Vec<u8>,
    rsa_public_key_der: Vec<u8>,
}

#[derive(Serialize, SerdeDeserialize)]
struct WorkerKeyStoreV2 {
    rsa_key_id: String,
    rsa_private_key_der: Vec<u8>,
    rsa_public_key_der: Vec<u8>,
    hpke_key_id: String,
    hpke_private_key: Vec<u8>,
    hpke_public_key: Vec<u8>,
}

#[derive(Serialize, SerdeDeserialize)]
struct WorkerKeyStoreV3 {
    hpke_key_id: String,
    hpke_private_key: Vec<u8>,
    hpke_public_key: Vec<u8>,
}

struct WorkerKeys {
    hpke_private_key: Vec<u8>,
    hpke_public_key: Vec<u8>,
    hpke_key_id: String,
}

fn worker_keys_path() -> String {
    std::env::var(WORKER_KEYS_PATH_ENV).unwrap_or_else(|_| DEFAULT_WORKER_KEYS_PATH.to_string())
}

fn load_or_generate_keys() -> Result<WorkerKeys> {
    let path = worker_keys_path();
    if Path::new(&path).exists() {
        let data = fs::read_to_string(&path)?;
        if let Ok(stored) = serde_json::from_str::<WorkerKeyStoreV3>(&data) {
            return Ok(WorkerKeys {
                hpke_private_key: stored.hpke_private_key,
                hpke_public_key: stored.hpke_public_key,
                hpke_key_id: stored.hpke_key_id,
            });
        }

        if let Ok(stored) = serde_json::from_str::<WorkerKeyStoreV2>(&data) {
            let upgraded = WorkerKeyStoreV3 {
                hpke_key_id: stored.hpke_key_id.clone(),
                hpke_private_key: stored.hpke_private_key.clone(),
                hpke_public_key: stored.hpke_public_key.clone(),
            };
            fs::write(&path, serde_json::to_string_pretty(&upgraded)?)?;
            return Ok(WorkerKeys {
                hpke_private_key: stored.hpke_private_key,
                hpke_public_key: stored.hpke_public_key,
                hpke_key_id: stored.hpke_key_id,
            });
        }

        if let Ok(_stored) = serde_json::from_str::<WorkerKeyStoreV1>(&data) {
            println!("   ! Legacy RSA-only worker key store detected; generating HPKE keys.");
            let (hpke_private_key, hpke_public_key, hpke_key_id) = generate_hpke_keypair()?;
            let upgraded = WorkerKeyStoreV3 {
                hpke_key_id: hpke_key_id.clone(),
                hpke_private_key: hpke_private_key.clone(),
                hpke_public_key: hpke_public_key.clone(),
            };
            fs::write(&path, serde_json::to_string_pretty(&upgraded)?)?;
            return Ok(WorkerKeys {
                hpke_private_key,
                hpke_public_key,
                hpke_key_id,
            });
        }
    }

    println!("   > Generating HPKE (X25519) Keypair...");
    let (hpke_private_key, hpke_public_key, hpke_key_id) = generate_hpke_keypair()?;

    let stored = WorkerKeyStoreV3 {
        hpke_key_id: hpke_key_id.clone(),
        hpke_private_key: hpke_private_key.clone(),
        hpke_public_key: hpke_public_key.clone(),
    };
    fs::write(&path, serde_json::to_string_pretty(&stored)?)?;

    Ok(WorkerKeys {
        hpke_private_key,
        hpke_public_key,
        hpke_key_id,
    })
}

fn generate_hpke_keypair() -> Result<(Vec<u8>, Vec<u8>, String)> {
    let secret = X25519StaticSecret::random_from_rng(OsRng);
    let public = X25519PublicKey::from(&secret);
    let key_id = uuid::Uuid::new_v4().to_string();
    Ok((
        secret.to_bytes().to_vec(),
        public.as_bytes().to_vec(),
        key_id,
    ))
}

#[derive(CandidType, Deserialize, Debug)]
struct GeoLocation {
    latitude: f64,
    longitude: f64,
    precision: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(CandidType, Deserialize, Debug)]
struct Jurisdiction {
    countryCode: String,
    region: Option<String>,
    city: Option<String>,
}

#[allow(non_snake_case)]
#[derive(CandidType, Deserialize, Debug)]
struct ExternalRequest {
    requestId: String,
    instanceId: String,
    stepId: String,
    startedAt: candid::Int,
    payload: Vec<(String, String)>,
    geoLocation: Option<GeoLocation>,
    jurisdiction: Option<Jurisdiction>,
}

#[derive(CandidType, Deserialize, Debug)]
struct ChatTask {
    input: String,
    client_principal: Principal,
    conversation_id: Option<String>,
}

#[derive(CandidType, Deserialize, Debug)]
enum LogLevel {
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(CandidType, Deserialize, Debug)]
enum LogSource {
    Agent(String),
    Backend,
    Frontend,
}

#[derive(Debug)]
struct WorkerTask {
    trace_id: String,
    name: String,
}

impl TaskContext for WorkerTask {
    fn trace_id(&self) -> String {
        self.trace_id.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    // 0. Config Initialization
    let config = config_service::ConfigService::get();
    println!("🧠 Nostra Cortex Worker Starting (Glass Box Enabled)...");
    println!("   > Environment: {:?}", config.get_env());

    // 0. Crypto Setup (Sovereign Key)
    let keys = load_or_generate_keys()?;

    println!("   > Connecting to IC Replica...");

    let url = std::env::var("IC_URL").unwrap_or("http://127.0.0.1:4943".to_string());
    let (agent, identity_label) = build_agent_with_resolved_identity(&url)?;
    println!("   > Agent Identity: {}", identity_label);

    // Create Arc-wrapped agent for sharing
    let agent_arc = Arc::new(agent);

    // Fetch root key (essential for local dev)
    if let Err(e) = agent_arc.fetch_root_key().await {
        println!("   ! Failed to fetch root key: {}", e);
    }

    let config = ConfigService::get();
    let canister_id = config
        .get_canister_id("primary")
        .expect("CANISTER_ID (primary) is strictly required");
    println!("   > Target Canister: {}", canister_id);

    // Initialize KIP Client
    let kip = KipClient::new(agent_arc.clone(), canister_id);

    // Streaming Canister
    let streaming_id = config
        .get_canister_id("streaming")
        .expect("Streaming Canister ID (CANISTER_ID_NOSTRA_STREAMING) must be set");
    println!("   > Streaming Canister: {}", streaming_id);

    let backend_id = config
        .get_canister_id("backend")
        .expect("Backend Canister ID (CANISTER_ID_NOSTRA_BACKEND) must be set");
    println!("   > Backend Canister: {}", backend_id);

    // Register worker key (v2 metadata)
    let register_result = agent_arc
        .update(&backend_id, "registerWorkerV2")
        .with_arg(Encode!(
            &keys.hpke_public_key,
            &Some(keys.hpke_key_id.clone()),
            &Some(ALG_HPKE_X25519_CHACHA20POLY1305.to_string()),
            &Some(ENC_VERSION_V1 as u64)
        )?)
        .call_and_wait()
        .await;
    if let Err(e) = register_result {
        println!("   ! Failed to register worker key: {}", e);
    }

    // 2. Initialize Skills (Now using Arc for Middleware compatibility)
    let mut squad: HashMap<String, Arc<dyn Skill>> = HashMap::new();

    // Helper: Construct Arc directly
    fn to_skill<S: Skill + 'static>(s: S) -> Arc<dyn Skill> {
        Arc::new(s)
    }

    squad.insert("analyst".to_string(), to_skill(Analyst::new(kip.clone())));
    squad.insert("gardener".to_string(), to_skill(Gardener::new(kip.clone())));

    // For others
    squad.insert("pm".to_string(), to_skill(Pm::new()));
    squad.insert("architect".to_string(), to_skill(Architect::new()));
    squad.insert("dev".to_string(), to_skill(Dev::new()));
    squad.insert("qa".to_string(), to_skill(Qa::new()));

    // Vector Service (Local-First Embedding Strategy)
    // Env overrides:
    // - NOSTRA_EMBEDDING_PROVIDER: auto | ollama | openai | mock
    // - NOSTRA_LOCAL_EMBEDDING_MODEL: defaults to qwen3-embedding:0.6b
    // - NOSTRA_EMBEDDING_DIM: defaults to 384
    let provider = std::env::var("NOSTRA_EMBEDDING_PROVIDER")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase();
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let local_model = std::env::var("NOSTRA_LOCAL_EMBEDDING_MODEL")
        .unwrap_or_else(|_| "qwen3-embedding:0.6b".to_string());
    let local_dim = std::env::var("NOSTRA_EMBEDDING_DIM")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(384);
    let local_base = config
        .get_llm_config()
        .map(|c| c.api_base.clone())
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    let local_probe_ok = {
        let probe_embedder =
            OllamaEmbedder::with_config(local_base.clone(), local_model.clone(), local_dim);
        match tokio::time::timeout(
            Duration::from_secs(8),
            probe_embedder.embed("nostra-embedding-probe"),
        )
        .await
        {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                println!(
                    "   ! Local embedding probe failed: {}. Evaluating fallback route.",
                    e
                );
                false
            }
            Err(_) => {
                println!("   ! Local embedding probe timed out. Evaluating fallback route.");
                false
            }
        }
    };

    let route = resolve_embedding_route(&provider, !api_key.is_empty(), local_probe_ok);
    let embedding_provider = build_provider(
        route,
        local_base.clone(),
        local_model.clone(),
        local_dim,
        api_key.clone(),
    );

    match route {
        EmbeddingRoute::Ollama => println!(
            "   > Embeddings: Ollama model '{}' @ {} ({}D)",
            local_model, local_base, local_dim
        ),
        EmbeddingRoute::OpenAI => {
            println!("   > Embeddings: OpenAI text-embedding-3-small (384D)")
        }
        EmbeddingRoute::Mock => println!("   > Embeddings: Mock deterministic generator (384D)"),
    }

    // Vector Client is now internal to Service, driven by Config
    // VectorService is custom-cloneable (Arc internal), so we can create it and clone handles.
    let vector_service = VectorService::new(
        embedding_provider,
        agent_arc.clone(),
        "nostra_knowledge".to_string(),
    );

    squad.insert(
        "librarian".to_string(),
        to_skill(Librarian::new(
            agent_arc.clone(),
            streaming_id,
            backend_id,
            Some(keys.hpke_private_key.clone()),
            Some(kip.clone()),
            Some(vector_service.clone()), // Pass a clone to Librarian
        )),
    );
    squad.insert("hrm_scheduler".to_string(), to_skill(HrmScheduler::new()));

    println!("   > Squad Ready: {:?}", squad.keys());

    // 2c. Initialize Gateway Service (The Nervous System)
    let gateway_service = Arc::new(cortex_worker::GatewayService::new());
    let gateway_service_for_callback = gateway_service.clone();

    // Define Event Callback
    let on_step_change =
        move |instance_id: &str,
              step_id: &str,
              status: &nostra_workflow_core::types::WorkflowStatus| {
            let event = cortex_worker::gateway_service::GatewayEvent {
                topic: "workflow_update".to_string(),
                source: "workflow_engine".to_string(),
                payload: serde_json::json!({
                    "agent_id": "workflow_engine",
                    "type": "workflow_step",
                    "action": format!("Step {:?}", status),
                    "status": match status {
                        nostra_workflow_core::types::WorkflowStatus::Running => "running",
                        nostra_workflow_core::types::WorkflowStatus::Completed => "completed",
                        nostra_workflow_core::types::WorkflowStatus::Failed(_) => "failed",
                        _ => "started",
                    },
                    "instance_id": instance_id,
                    "step_id": step_id,
                }),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };
            gateway_service_for_callback.broadcast(event);
        };

    // 2b. Initialize Workflow Engine
    let workflow_runner = Arc::new(workflows::engine_runner::WorkflowRunner::new(Some(
        Box::new(on_step_change),
    )));

    // 2d. Initialize ACP native automation scheduler (feature-gated).
    let acp_scheduler = workflows::scheduler::AcpAutomationScheduler::from_env(
        workflow_runner.clone(),
        gateway_service.clone(),
    );
    if acp_scheduler.is_enabled() {
        println!("   > ACP automation scheduler enabled");
        acp_scheduler.clone().start();
    } else {
        println!("   > ACP automation scheduler disabled");
    }

    // Spawn API Server (Port 3003)
    let workflow_runner_clone = workflow_runner.clone();
    let acp_scheduler_clone = acp_scheduler.clone();
    let vector_service_clone = vector_service.clone(); // Clone for API
    let gateway_service_clone = gateway_service.clone();

    tokio::spawn(async move {
        let _ = api::start_server(
            workflow_runner_clone,
            Some(acp_scheduler_clone),
            Some(vector_service_clone),
            gateway_service_clone,
            3003,
        )
        .await;
    });

    // 2c. Initialize Middleware Stack (The Glass Box)
    let stack = MiddlewareStack::new().add(OtelInterceptor::new("NostraWorker"));

    // 3. Main Polling Loop
    loop {
        // Poll Agent Tasks (for each role)
        for (role, skill) in &squad {
            let response = agent_arc
                .query(&backend_id, "getPendingTasks")
                .with_arg(Encode!(&role)?)
                .call()
                .await;

            match response {
                Ok(data) => {
                    let tasks: Vec<ExternalRequest> =
                        Decode!(data.as_slice(), Vec<ExternalRequest>)?;
                    if !tasks.is_empty() {
                        println!("   [Found] {} tasks for {}", tasks.len(), role);
                        for task in tasks {
                            // Prepare Context
                            let ctx = Box::new(WorkerTask {
                                trace_id: task.requestId.clone(),
                                name: skill.name().to_string(),
                            });

                            // Prepare Logic Closure
                            let skill_clone = skill.clone();
                            let backend_id_clone = backend_id;
                            let agent_clone = agent_arc.clone();
                            let task_info = (task.instanceId.clone(), task.stepId.clone());
                            let skill_name = skill.name().to_string();

                            // Payload construction logic
                            let mut payload_map = HashMap::new();
                            for (k, v) in task.payload {
                                payload_map.insert(k, v);
                            }
                            if let Some(loc) = task.geoLocation {
                                payload_map
                                    .insert("geo_latitude".to_string(), loc.latitude.to_string());
                                payload_map
                                    .insert("geo_longitude".to_string(), loc.longitude.to_string());
                                if let Some(prec) = loc.precision {
                                    payload_map
                                        .insert("geo_precision".to_string(), prec.to_string());
                                }
                            }
                            if let Some(jur) = task.jurisdiction {
                                payload_map.insert("jur_country".to_string(), jur.countryCode);
                                if let Some(r) = jur.region {
                                    payload_map.insert("jur_region".to_string(), r);
                                }
                                if let Some(c) = jur.city {
                                    payload_map.insert("jur_city".to_string(), c);
                                }
                            }
                            let payload_json =
                                serde_json::to_string(&payload_map).unwrap_or_default();

                            // Execute via Middleware
                            let outcome = stack
                                .execute(ctx, move |_| async move {
                                    skill_clone
                                        .execute(payload_json)
                                        .await
                                        .map_err(|e| anyhow::anyhow!(e))
                                })
                                .await;

                            // Handle Outcome (Log or Submit)
                            match outcome {
                                Ok(result) => {
                                    let _submit = agent_clone
                                        .update(&backend_id_clone, "submitTaskOutcome")
                                        .with_arg(
                                            Encode!(&task_info.0, &task_info.1, &result).unwrap(),
                                        )
                                        .call_and_wait()
                                        .await;
                                    println!("    << Submitted: {:?}", _submit);
                                }
                                Err(e) => {
                                    let context =
                                        vec![("instance_id".to_string(), task_info.0.clone())];
                                    let log_source = LogSource::Agent(skill_name);
                                    let _log = agent_clone
                                        .update(&backend_id_clone, "submitLog")
                                        .with_arg(
                                            Encode!(
                                                &log_source,
                                                &LogLevel::Error,
                                                &e.to_string(),
                                                &Some(context)
                                            )
                                            .unwrap(),
                                        )
                                        .call_and_wait()
                                        .await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("   ! IC poll error for role '{}': {}", role, e);
                }
            }
        }

        // 4. Poll Streaming Canister for Chat
        let chat_response = agent_arc
            .update(&streaming_id, "pop_chat_task")
            .with_arg(Encode!()?)
            .call_and_wait()
            .await;

        match chat_response {
            Ok(data) => {
                let task_opt: Option<ChatTask> = Decode!(data.as_slice(), Option<ChatTask>)?;
                if let Some(task) = task_opt {
                    println!(
                        "   [Chat] Processing message from {}",
                        task.client_principal
                    );

                    if let Some(librarian) = squad.get("librarian") {
                        let mut payload_map = HashMap::new();
                        payload_map.insert("input".to_string(), task.input);
                        payload_map.insert(
                            "client_principal".to_string(),
                            task.client_principal.to_text(),
                        );
                        if let Some(cid) = task.conversation_id {
                            payload_map.insert("conversation_id".to_string(), cid);
                        }
                        let payload_json = serde_json::to_string(&payload_map).unwrap_or_default();

                        // Middleware Execution for Chat
                        let ctx = Box::new(WorkerTask {
                            trace_id: format!("chat-{}", ic_agent::export::Principal::anonymous()),
                            name: "LibrarianChat".to_string(),
                        });
                        let lib_clone = librarian.clone();

                        let _ = stack
                            .execute(ctx, move |_| async move {
                                let res = lib_clone
                                    .execute(payload_json)
                                    .await
                                    .map_err(|e| anyhow::anyhow!(e));
                                if let Err(e) = &res {
                                    println!("    !! Chat Skill Failed: {}", e);
                                }
                                res
                            })
                            .await;
                    }
                }
            }
            Err(e) => {
                eprintln!("   ! Chat task poll error: {}", e);
            }
        }

        sleep(Duration::from_secs(2)).await;
    }
}
