//! Worker HTTP API
//!
//! Exposes a lightweight API to interact with the internal Workflow Engine.
//! This allows the frontend to query state and drive workflows during development/demos,
//! bridging the gap until full IC-based persistence is ready.

use crate::NostraBook;
use crate::activity_service::ActivityService;
use crate::book::BlockContent;
use crate::config_service::ConfigService;
use crate::gateway_service::GatewayService;
use crate::skills::extraction::{EXTRACTION_ASYNC_PAYLOAD_SCHEMA_V1, ExtractionOrchestrator};
use crate::temporal_governor::TemporalGovernor;
use crate::vector_service::{
    CeiMetadataV1, HybridConfig, IndexDocument, RetrievalMode, SearchFilters, SearchOptions,
    SearchResult, VectorService,
};
use crate::workflows::engine_runner::WorkflowRunner;
use crate::workflows::scheduler::{AcpAutomationScheduler, RunNowOutcome};
use axum::{
    Json, Router,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::DateTime;
use nostra_extraction::{
    ExtractionRequestV1, ExtractionResultV1, ExtractionStatus, validate_extraction_input_source,
};
use nostra_shared::types::provider_registry::ProviderRecord;
use nostra_workflow_core::builder::{WorkflowParser, WorkflowTemplates};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

/// Shared state for the API.
pub struct AppState {
    pub runner: Arc<WorkflowRunner>,
    pub acp_scheduler: Option<Arc<AcpAutomationScheduler>>,
    pub vector_service: Option<VectorService>,
    pub activity_service: ActivityService,
    pub temporal_governor: TemporalGovernor,
    pub gateway_service: Arc<GatewayService>,
    pub extraction_orchestrator: Arc<ExtractionOrchestrator>,
    pub extraction_jobs: Arc<RwLock<HashMap<String, ExtractionResultV1>>>,
}

pub async fn start_server(
    runner: Arc<WorkflowRunner>,
    acp_scheduler: Option<Arc<AcpAutomationScheduler>>,
    vector_service: Option<VectorService>,
    gateway_service: Arc<GatewayService>,
    port: u16,
) -> anyhow::Result<()> {
    let activity_service = ActivityService::new();
    let temporal_governor = TemporalGovernor::new(activity_service.clone());

    let state = Arc::new(AppState {
        runner,
        acp_scheduler,
        vector_service,
        activity_service,
        temporal_governor,
        gateway_service,
        extraction_orchestrator: Arc::new(ExtractionOrchestrator::new()),
        extraction_jobs: Arc::new(RwLock::new(HashMap::new())),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/model", get(get_model_health))
        .route("/workflows", get(list_workflows))
        .route("/workflows/:id", get(get_workflow_status))
        .route("/workflows/:id/cancel", post(cancel_workflow))
        .route("/workflows/:id/retry", post(retry_workflow))
        .route("/workflows/generate", post(generate_workflow))
        .route("/workflows/start/:template", post(start_workflow))
        .route("/automations/acp/run-now", post(run_acp_now))
        .route("/automations/acp/pause", post(pause_acp_automation))
        .route("/automations/acp/resume", post(resume_acp_automation))
        .route("/automations/acp/status", get(get_acp_automation_status))
        .route("/tasks", get(get_pending_tasks))
        .route("/tasks/:id/complete", post(complete_task))
        .route(
            "/extraction/payload-schema",
            get(get_extraction_payload_schema),
        )
        .route("/extraction/submit", post(submit_extraction))
        .route("/extraction/status/:job_id", get(get_extraction_status))
        .route("/extraction/callback", post(extraction_callback))
        .route("/knowledge/ingest/book", post(ingest_book))
        .route("/knowledge/index", post(index_knowledge))
        .route(
            "/knowledge/search",
            get(search_knowledge_get).post(search_knowledge_post),
        )
        .route("/knowledge/ask", post(ask_knowledge))
        .route("/knowledge/shadow/report", get(get_shadow_report))
        .route("/system/providers", get(get_system_providers))
        .route("/activity/readiness", get(get_readiness))
        .route("/gateway", get(ws_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("   > API Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    GatewayService::handle_ws_upgrade(ws, state.gateway_service.clone()).await
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[derive(Serialize)]
struct ModelHealthResponse {
    status: String,
    llm_base: String,
    generation_model: String,
    vector: serde_json::Value,
}

async fn get_model_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = ConfigService::get();
    let llm_base = resolve_generation_base(config);
    let generation_model = resolve_generation_model();

    if let Some(vs) = &state.vector_service {
        let vector_health = vs.health().await;
        (
            StatusCode::OK,
            Json(ModelHealthResponse {
                status: if vector_health.embedding_probe_ok {
                    "ok".to_string()
                } else {
                    "degraded".to_string()
                },
                llm_base,
                generation_model,
                vector: serde_json::to_value(vector_health).unwrap_or_else(|_| json!({})),
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ModelHealthResponse {
                status: "unavailable".to_string(),
                llm_base,
                generation_model,
                vector: json!({ "status": "vector_service_missing" }),
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct SystemProvidersResponse {
    providers: Vec<ProviderRecord>,
}

async fn get_system_providers() -> impl IntoResponse {
    let config = ConfigService::get();
    let providers = config.get_providers().clone();
    (StatusCode::OK, Json(SystemProvidersResponse { providers }))
}

// --- Workflow handlers ---

#[derive(Serialize)]
struct WorkflowListItem {
    id: String,
    status: String,
    current_step: Option<String>,
}

async fn list_workflows(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let items = state
        .runner
        .list_instances()
        .into_iter()
        .map(|summary| WorkflowListItem {
            id: summary.id,
            status: summary.status,
            current_step: summary.current_step,
        })
        .collect::<Vec<_>>();
    (StatusCode::OK, Json(items))
}

#[derive(Serialize)]
struct WorkflowStatusResponse {
    id: String,
    status: String,
    current_step: Option<String>,
    history: Vec<String>,
}

async fn get_workflow_status(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.runner.get_details(&id) {
        Ok(details) => (
            StatusCode::OK,
            Json(WorkflowStatusResponse {
                id,
                status: format!("{:?}", details.status),
                current_step: details.current_step_id,
                history: details.context.history,
            }),
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Workflow not found").into_response(),
    }
}

#[derive(Serialize)]
struct StartWorkflowResponse {
    workflow_id: String,
}

async fn start_workflow(
    Path(template): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let def = match template.as_str() {
        "approval" => match WorkflowParser::from_yaml(WorkflowTemplates::approval()) {
            Ok(parsed) => parsed,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        "governance" => match WorkflowParser::from_yaml(WorkflowTemplates::governance_vote()) {
            Ok(parsed) => parsed,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        "gap_closure" => crate::workflows::gap_closure::create_gap_closure_workflow(),
        "acp_pilot_ops" => crate::workflows::acp_pilot_ops::create_acp_pilot_ops_workflow(),
        _ => return (StatusCode::BAD_REQUEST, "Unknown template").into_response(),
    };

    let id = format!("{}-{}", template, uuid::Uuid::new_v4());

    match state.runner.start(&id, def) {
        Ok(_) => (
            StatusCode::OK,
            Json(StartWorkflowResponse { workflow_id: id }),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn cancel_workflow(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.runner.cancel(&id) {
        Ok(_) => (StatusCode::OK, "Workflow cancelled").into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn retry_workflow(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.runner.retry(&id) {
        Ok(_) => {
            let _ = state.runner.tick(&id);
            (StatusCode::OK, "Workflow retried").into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

#[derive(Serialize)]
struct AutomationActionResponse {
    status: String,
    workflow_id: Option<String>,
}

#[derive(Serialize)]
struct AutomationStatusResponse {
    automation_key: String,
    enabled: bool,
    paused: bool,
    interval_secs: u64,
    active_workflow_id: Option<String>,
    last_workflow_id: Option<String>,
    last_run_at: Option<String>,
    last_status: Option<String>,
}

fn requester_is_admin_or_steward(headers: &HeaderMap) -> bool {
    let Some(value) = headers.get("x-cortex-role") else {
        return false;
    };

    let Ok(role) = value.to_str() else {
        return false;
    };

    role.eq_ignore_ascii_case("admin") || role.eq_ignore_ascii_case("steward")
}

async fn run_acp_now(headers: HeaderMap, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if !requester_is_admin_or_steward(&headers) {
        return (StatusCode::FORBIDDEN, "admin or steward role required").into_response();
    }

    let Some(scheduler) = state.acp_scheduler.as_ref() else {
        return (
            StatusCode::NOT_FOUND,
            "ACP automation scheduler is not initialized",
        )
            .into_response();
    };

    match scheduler.run_now().await {
        Ok(RunNowOutcome::Started { workflow_id }) => (
            StatusCode::OK,
            Json(AutomationActionResponse {
                status: "started".to_string(),
                workflow_id: Some(workflow_id),
            }),
        )
            .into_response(),
        Ok(RunNowOutcome::AlreadyActive { workflow_id }) => (
            StatusCode::OK,
            Json(AutomationActionResponse {
                status: "already_active".to_string(),
                workflow_id: Some(workflow_id),
            }),
        )
            .into_response(),
        Ok(RunNowOutcome::Disabled) => (
            StatusCode::CONFLICT,
            Json(AutomationActionResponse {
                status: "disabled".to_string(),
                workflow_id: None,
            }),
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn pause_acp_automation(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if !requester_is_admin_or_steward(&headers) {
        return (StatusCode::FORBIDDEN, "admin or steward role required").into_response();
    }

    let Some(scheduler) = state.acp_scheduler.as_ref() else {
        return (
            StatusCode::NOT_FOUND,
            "ACP automation scheduler is not initialized",
        )
            .into_response();
    };

    match scheduler.pause().await {
        Ok(_) => (
            StatusCode::OK,
            Json(AutomationActionResponse {
                status: "paused".to_string(),
                workflow_id: None,
            }),
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn resume_acp_automation(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    if !requester_is_admin_or_steward(&headers) {
        return (StatusCode::FORBIDDEN, "admin or steward role required").into_response();
    }

    let Some(scheduler) = state.acp_scheduler.as_ref() else {
        return (
            StatusCode::NOT_FOUND,
            "ACP automation scheduler is not initialized",
        )
            .into_response();
    };

    match scheduler.resume().await {
        Ok(_) => (
            StatusCode::OK,
            Json(AutomationActionResponse {
                status: "resumed".to_string(),
                workflow_id: None,
            }),
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn get_acp_automation_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Some(scheduler) = state.acp_scheduler.as_ref() else {
        return (
            StatusCode::NOT_FOUND,
            "ACP automation scheduler is not initialized",
        )
            .into_response();
    };

    let snapshot = scheduler.snapshot().await;
    (
        StatusCode::OK,
        Json(AutomationStatusResponse {
            automation_key: snapshot.automation_key,
            enabled: snapshot.enabled,
            paused: snapshot.paused,
            interval_secs: snapshot.interval_secs,
            active_workflow_id: snapshot.active_workflow_id,
            last_workflow_id: snapshot.last_workflow_id,
            last_run_at: snapshot.last_run_at,
            last_status: snapshot.last_status,
        }),
    )
        .into_response()
}

#[derive(Deserialize)]
struct GenerateRequest {
    intention: String,
}

#[derive(Serialize)]
struct GenerateResponse {
    workflow_json: String,
    preview: String,
}

async fn generate_workflow(Json(body): Json<GenerateRequest>) -> impl IntoResponse {
    let workflow_json = if body.intention.to_lowercase().contains("approval") {
        nostra_workflow_core::builder::WorkflowTemplates::approval().to_string()
    } else if body.intention.to_lowercase().contains("vote")
        || body.intention.to_lowercase().contains("governance")
    {
        nostra_workflow_core::builder::WorkflowTemplates::governance_vote().to_string()
    } else {
        r#"
id: generated
name: Generated Workflow
start: step1
states:
  - name: step1
    type: operation
    transition: step2
  - name: step2
    type: operation
    end: true
"#
        .to_string()
    };

    let preview = format!("Generated workflow for: {}", body.intention);
    (
        StatusCode::OK,
        Json(GenerateResponse {
            workflow_json,
            preview,
        }),
    )
}

async fn get_pending_tasks(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tasks = state.runner.get_pending_tasks(None, None);
    (StatusCode::OK, Json(tasks))
}

#[derive(Deserialize)]
struct CompleteTaskRequest {
    payload: HashMap<String, String>,
}

async fn complete_task(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CompleteTaskRequest>,
) -> impl IntoResponse {
    match state.runner.complete_task(&id, Some(body.payload)) {
        Ok(_) => {
            let _ = state.runner.tick(&id);
            (StatusCode::OK, "Task completed").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// --- Extraction handlers ---

#[derive(Deserialize, Serialize)]
struct ExtractionCallbackRequest {
    result: ExtractionResultV1,
}

#[derive(Serialize)]
struct ExtractionCallbackResponse {
    status: String,
    job_id: String,
}

async fn get_extraction_payload_schema() -> impl IntoResponse {
    let schema: serde_json::Value = serde_json::from_str(EXTRACTION_ASYNC_PAYLOAD_SCHEMA_V1)
        .unwrap_or_else(|_| json!({ "type": "object" }));
    (StatusCode::OK, Json(schema))
}

async fn submit_extraction(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<ExtractionRequestV1>,
) -> impl IntoResponse {
    if request.source_ref.trim().is_empty() || request.source_type.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "source_ref and source_type are required",
        )
            .into_response();
    }
    if let Err(err) = validate_extraction_input_source(&request) {
        return (StatusCode::BAD_REQUEST, err).into_response();
    }
    if request.content.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "content must not be empty").into_response();
    }

    let trace_id = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();
    let job_id = request
        .job_id
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("extract-{}", uuid::Uuid::new_v4()));
    request.job_id = Some(job_id.clone());

    let mut result = state.extraction_orchestrator.extract(&request).await;
    if result.job_id.trim().is_empty() {
        result.job_id = job_id.clone();
    }

    if request.index_to_knowledge
        && matches!(result.status, ExtractionStatus::Completed)
        && state.vector_service.is_some()
    {
        if let Some(vs) = &state.vector_service {
            let content = build_extraction_summary(&result);
            let cei = extraction_to_cei_metadata(&request, &result);
            let doc = IndexDocument {
                id: format!("extract:{}", result.job_id),
                text: content,
                label: format!("extraction:{}", request.source_ref),
                space_id: request
                    .space_id
                    .clone()
                    .unwrap_or_else(|| "space://unknown".to_string()),
                source_ref: request.source_ref.clone(),
                source_type: request.source_type.clone(),
                tags: vec!["extraction".to_string()],
                timestamp_ms: iso_to_timestamp_ms(&cei.timestamp),
                cei_metadata: Some(cei),
                modality: Some("text".to_string()),
            };

            let idem = request
                .idempotency_key
                .clone()
                .unwrap_or_else(|| format!("extract-index:{}", result.job_id));
            if let Err(err) = vs.index_documents(vec![doc], Some(idem.as_str())).await {
                result
                    .flags
                    .push(format!("extraction_index_failed:{}", err));
                result.status = ExtractionStatus::NeedsReview;
            }
        }
    }

    state
        .extraction_jobs
        .write()
        .await
        .insert(result.job_id.clone(), result.clone());

    println!(
        "[extraction/submit][{}] job_id={} mode={:?} status={:?} confidence={:.2} duration_ms={}",
        trace_id,
        result.job_id,
        request.extraction_mode,
        result.status,
        result.confidence,
        started.elapsed().as_millis()
    );

    (StatusCode::OK, Json(result)).into_response()
}

async fn get_extraction_status(
    Path(job_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let jobs = state.extraction_jobs.read().await;
    match jobs.get(&job_id) {
        Some(result) => (StatusCode::OK, Json(result.clone())).into_response(),
        None => (StatusCode::NOT_FOUND, "extraction job not found").into_response(),
    }
}

async fn extraction_callback(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ExtractionCallbackRequest>,
) -> impl IntoResponse {
    if !callback_authorized(&headers) {
        return (StatusCode::FORBIDDEN, "invalid extraction callback token").into_response();
    }

    let job_id = body.result.job_id.clone();
    state
        .extraction_jobs
        .write()
        .await
        .insert(job_id.clone(), body.result);

    (
        StatusCode::OK,
        Json(ExtractionCallbackResponse {
            status: "accepted".to_string(),
            job_id,
        }),
    )
        .into_response()
}

fn callback_authorized(headers: &HeaderMap) -> bool {
    let Ok(expected) = std::env::var("NOSTRA_EXTRACTION_CALLBACK_TOKEN") else {
        return true;
    };
    if expected.trim().is_empty() {
        return true;
    }
    headers
        .get("x-extraction-callback-token")
        .and_then(|v| v.to_str().ok())
        .map(|provided| provided == expected)
        .unwrap_or(false)
}

fn build_extraction_summary(result: &ExtractionResultV1) -> String {
    let entities = result
        .candidate_entities
        .iter()
        .map(|e| format!("{} [{}]", e.label, e.entity_type))
        .collect::<Vec<_>>()
        .join(", ");
    let relations = result
        .candidate_relations
        .iter()
        .map(|r| format!("{} -{}-> {}", r.source_id, r.relation_type, r.target_id))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "ExtractionResultV1\njob_id={}\nsource_ref={}\nstatus={:?}\nconfidence={:.2}\nentities={}\nrelations={}",
        result.job_id, result.source_ref, result.status, result.confidence, entities, relations
    )
}

fn extraction_to_cei_metadata(
    request: &ExtractionRequestV1,
    result: &ExtractionResultV1,
) -> CeiMetadataV1 {
    CeiMetadataV1 {
        contribution_id: result.job_id.clone(),
        source_uri: request.source_ref.clone(),
        author: result.provenance.produced_by_agent.clone(),
        timestamp: result.provenance.timestamp.clone(),
        lineage_refs: vec![result.provenance.source_version_id.clone()],
        source_version_id: Some(result.provenance.source_version_id.clone()),
        model_id: Some(result.provenance.model_id.clone()),
        perspective_scope: Some(result.provenance.perspective_scope.clone()),
        produced_by_agent: Some(result.provenance.produced_by_agent.clone()),
        confidence: Some(result.provenance.confidence),
        purpose: Some(result.provenance.purpose.clone()),
        modality: Some("text".to_string()),
    }
}

// --- Knowledge handlers ---

#[derive(Deserialize)]
struct KnowledgeIndexDocument {
    id: String,
    text: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    source_ref: Option<String>,
    #[serde(default)]
    source_type: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    timestamp_ms: Option<i64>,
    #[serde(default)]
    cei_metadata: Option<CeiMetadataV1>,
    #[serde(default)]
    modality: Option<String>,
}

#[derive(Deserialize)]
struct KnowledgeIndexRequest {
    space_id: String,
    source_ref: String,
    #[serde(default)]
    source_type: Option<String>,
    #[serde(default)]
    cei_metadata: Option<CeiMetadataV1>,
    idempotency_key: String,
    documents: Vec<KnowledgeIndexDocument>,
}

#[derive(Serialize)]
struct KnowledgeIndexResponse {
    indexed_count: usize,
    skipped_duplicate: bool,
    idempotency_key: String,
}

async fn index_knowledge(
    State(state): State<Arc<AppState>>,
    Json(body): Json<KnowledgeIndexRequest>,
) -> impl IntoResponse {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();
    println!(
        "[knowledge/index][{}] docs={} source_ref={}",
        trace_id,
        body.documents.len(),
        body.source_ref
    );

    if body.space_id.trim().is_empty()
        || body.source_ref.trim().is_empty()
        || body.idempotency_key.trim().is_empty()
    {
        return (
            StatusCode::BAD_REQUEST,
            "space_id, source_ref, and idempotency_key are required",
        )
            .into_response();
    }

    if body.documents.is_empty() {
        return (StatusCode::BAD_REQUEST, "documents must not be empty").into_response();
    }

    let Some(vs) = &state.vector_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector Service not available",
        )
            .into_response();
    };

    let default_source_type = body
        .source_type
        .clone()
        .unwrap_or_else(|| "manual".to_string());

    let docs: Vec<IndexDocument> = body
        .documents
        .into_iter()
        .filter(|d| !d.text.trim().is_empty())
        .map(|d| IndexDocument {
            id: d.id,
            text: d.text,
            label: d
                .label
                .unwrap_or_else(|| format!("source:{}", body.source_ref)),
            space_id: d.space_id.unwrap_or_else(|| body.space_id.clone()),
            source_ref: d.source_ref.unwrap_or_else(|| body.source_ref.clone()),
            source_type: d.source_type.unwrap_or_else(|| default_source_type.clone()),
            tags: d.tags.unwrap_or_default(),
            timestamp_ms: d.timestamp_ms,
            cei_metadata: d.cei_metadata.or_else(|| body.cei_metadata.clone()),
            modality: d.modality,
        })
        .collect();

    match vs
        .index_documents(docs, Some(body.idempotency_key.as_str()))
        .await
    {
        Ok(outcome) => {
            println!(
                "[knowledge/index][{}] indexed={} duplicate={} duration_ms={}",
                trace_id,
                outcome.indexed_count,
                outcome.skipped_duplicate,
                started.elapsed().as_millis()
            );
            (
                StatusCode::OK,
                Json(KnowledgeIndexResponse {
                    indexed_count: outcome.indexed_count,
                    skipped_duplicate: outcome.skipped_duplicate,
                    idempotency_key: body.idempotency_key,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Indexing failed: {}", e),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    limit: Option<i32>,
    retrieval_mode: Option<String>,
    diagnostics: Option<bool>,
}

#[derive(Deserialize)]
struct KnowledgeSearchRequest {
    query: String,
    limit: Option<i32>,
    retrieval_mode: Option<String>,
    #[serde(default)]
    ui_surface: Option<String>,
    #[serde(default)]
    filters: Option<SearchFilters>,
    #[serde(default)]
    fusion_weights: Option<HybridConfig>,
    diagnostics: Option<bool>,
    rerank_enabled: Option<bool>,
}

fn parse_retrieval_mode(raw: Option<String>) -> RetrievalMode {
    match raw
        .unwrap_or_else(|| "hybrid".to_string())
        .to_lowercase()
        .as_str()
    {
        "vector" => RetrievalMode::Vector,
        "lexical" => RetrievalMode::Lexical,
        _ => RetrievalMode::Hybrid,
    }
}

fn build_search_options(
    retrieval_mode: Option<String>,
    filters: Option<SearchFilters>,
    fusion_weights: Option<HybridConfig>,
    diagnostics: Option<bool>,
    rerank_enabled: Option<bool>,
) -> SearchOptions {
    let mut weights = fusion_weights.unwrap_or_default();
    if let Some(flag) = rerank_enabled {
        weights.rerank_enabled = flag;
    }

    SearchOptions {
        retrieval_mode: parse_retrieval_mode(retrieval_mode),
        filters: filters.unwrap_or_default(),
        fusion_weights: weights,
        diagnostics: diagnostics.unwrap_or(false),
    }
}

async fn search_knowledge_get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();
    let Some(vs) = &state.vector_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector Service not available",
        )
            .into_response();
    };

    let limit = params.limit.unwrap_or(5);
    let options = build_search_options(params.retrieval_mode, None, None, params.diagnostics, None);

    match vs.search_with_options(&params.q, limit, options).await {
        Ok(results) => {
            println!(
                "[knowledge/search][{}] mode=get q_len={} limit={} results={} duration_ms={}",
                trace_id,
                params.q.len(),
                limit,
                results.len(),
                started.elapsed().as_millis()
            );
            (StatusCode::OK, Json(results)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Search failed: {}", e),
        )
            .into_response(),
    }
}

async fn search_knowledge_post(
    State(state): State<Arc<AppState>>,
    Json(body): Json<KnowledgeSearchRequest>,
) -> impl IntoResponse {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();
    let ui_surface = normalize_ui_surface(body.ui_surface.clone());
    let Some(vs) = &state.vector_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector Service not available",
        )
            .into_response();
    };

    if body.query.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "query must not be empty").into_response();
    }

    let limit = body.limit.unwrap_or(5);
    let mode = parse_retrieval_mode(body.retrieval_mode.clone());
    let options = build_search_options(
        body.retrieval_mode,
        body.filters,
        body.fusion_weights,
        body.diagnostics,
        body.rerank_enabled,
    );

    match vs.search_with_options(&body.query, limit, options).await {
        Ok(results) => {
            println!(
                "[knowledge/search][{}] mode=post surface={} retrieval_mode={:?} q_len={} limit={} results={} duration_ms={}",
                trace_id,
                ui_surface.as_deref().unwrap_or("unknown"),
                mode,
                body.query.len(),
                limit,
                results.len(),
                started.elapsed().as_millis()
            );
            (StatusCode::OK, Json(results)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Search failed: {}", e),
        )
            .into_response(),
    }
}

#[derive(Deserialize, Default)]
struct GroundingOptions {
    #[serde(default)]
    strict: Option<bool>,
    #[serde(default)]
    min_citations: Option<usize>,
}

#[derive(Deserialize)]
struct KnowledgeAskRequest {
    question: String,
    limit: Option<i32>,
    retrieval_mode: Option<String>,
    #[serde(default)]
    ui_surface: Option<String>,
    #[serde(default)]
    filters: Option<SearchFilters>,
    #[serde(default)]
    fusion_weights: Option<HybridConfig>,
    diagnostics: Option<bool>,
    rerank_enabled: Option<bool>,
    #[serde(default)]
    grounding: Option<GroundingOptions>,
    max_context_chunks: Option<usize>,
    require_provenance: Option<bool>,
}

#[derive(Serialize)]
struct AskCitation {
    id: String,
    score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<CeiMetadataV1>,
}

#[derive(Serialize)]
struct KnowledgeAskResponse {
    trace_id: String,
    model: String,
    answer: String,
    citations: Vec<AskCitation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui_surface: Option<String>,
}

fn normalize_ui_surface(surface: Option<String>) -> Option<String> {
    surface
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn ask_knowledge(
    State(state): State<Arc<AppState>>,
    Json(body): Json<KnowledgeAskRequest>,
) -> impl IntoResponse {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();
    let ui_surface = normalize_ui_surface(body.ui_surface.clone());

    if body.question.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "question must not be empty").into_response();
    }

    let Some(vs) = &state.vector_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector Service not available",
        )
            .into_response();
    };

    let search_options = build_search_options(
        body.retrieval_mode,
        body.filters,
        body.fusion_weights,
        body.diagnostics,
        body.rerank_enabled,
    );

    let limit = body.limit.unwrap_or(8);
    let search_results = match vs
        .search_with_options(&body.question, limit, search_options)
        .await
    {
        Ok(results) => results,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Retrieval failed: {}", e),
            )
                .into_response();
        }
    };

    if search_results.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(KnowledgeAskResponse {
                trace_id,
                model: "none".to_string(),
                answer: "No grounded context found for this question.".to_string(),
                citations: vec![],
                ui_surface,
            }),
        )
            .into_response();
    }

    let max_ctx = body.max_context_chunks.unwrap_or(4).max(1);
    let grounding = body.grounding.unwrap_or_default();
    let min_citations = grounding.min_citations.unwrap_or(1);

    let selected: Vec<SearchResult> = search_results.into_iter().take(max_ctx).collect();

    if body.require_provenance.unwrap_or(false)
        && selected.iter().any(|result| result.provenance.is_none())
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "require_provenance=true but one or more chunks are missing provenance",
        )
            .into_response();
    }

    if selected.len() < min_citations {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "grounding requirement not met: requested at least {} citations, got {}",
                min_citations,
                selected.len()
            ),
        )
            .into_response();
    }

    let contexts = selected
        .iter()
        .map(|item| {
            format!(
                "ID: {}\nSource: {}\nText: {}",
                item.id,
                item.source_ref
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                item.content.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>();

    let strict_grounding = grounding.strict.unwrap_or(true);
    let (answer, model) = generate_grounded_answer(&body.question, &contexts, strict_grounding)
        .await
        .unwrap_or_else(|fallback| {
            (
                format!(
                    "Grounded answer fallback. Best supporting snippets:\n{}\n\nGenerator error: {}",
                    contexts
                        .iter()
                        .enumerate()
                        .map(|(idx, c)| format!("[{}] {}", idx + 1, c))
                        .collect::<Vec<_>>()
                        .join("\n\n"),
                    fallback
                ),
                "fallback-extractive".to_string(),
            )
        });

    let citations: Vec<AskCitation> = selected
        .into_iter()
        .map(|item| AskCitation {
            id: item.id,
            score: item.score,
            source_ref: item.source_ref,
            content: item.content,
            provenance: item.provenance,
        })
        .collect();

    println!(
        "[knowledge/ask][{}] surface={} q_len={} citations={} duration_ms={}",
        trace_id,
        ui_surface.as_deref().unwrap_or("unknown"),
        body.question.len(),
        citations.len(),
        started.elapsed().as_millis()
    );

    (
        StatusCode::OK,
        Json(KnowledgeAskResponse {
            trace_id,
            model,
            answer,
            citations,
            ui_surface,
        }),
    )
        .into_response()
}

async fn get_shadow_report(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Some(vs) = &state.vector_service else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector Service not available",
        )
            .into_response();
    };

    (StatusCode::OK, Json(vs.shadow_report())).into_response()
}

async fn get_readiness(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let report = state.temporal_governor.audit_v1_readiness().await;
    Json(report)
}

async fn ingest_book(
    State(state): State<Arc<AppState>>,
    Json(book): Json<NostraBook>,
) -> impl IntoResponse {
    println!(
        " [Stats] Ingested Book: {} ({} chapters, {} entities)",
        book.meta.id,
        book.content.len(),
        book.knowledge_graph
            .as_ref()
            .map(|kg| kg.entities.len())
            .unwrap_or(0)
    );

    let Some(vs) = &state.vector_service else {
        println!("   [Warn] Vector Service not available. Indexing skipped.");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector Service not available",
        )
            .into_response();
    };

    let source_ref = format!("book:{}", book.meta.id);
    let source_type = "book_block".to_string();
    let space_id = book.meta.provenance.space_did.clone();
    let timestamp_ms = iso_to_timestamp_ms(&book.meta.provenance.created_at);
    let cei = CeiMetadataV1 {
        contribution_id: book.meta.id.clone(),
        source_uri: source_ref.clone(),
        author: book.meta.provenance.author_did.clone(),
        timestamp: book.meta.provenance.created_at.clone(),
        lineage_refs: vec![book.meta.version_hash.clone()],
        source_version_id: Some(book.meta.version_hash.clone()),
        model_id: None,
        perspective_scope: Some(space_id.clone()),
        produced_by_agent: None,
        confidence: None,
        purpose: Some("book_ingest".to_string()),
        modality: Some("text".to_string()),
    };

    let mut docs: Vec<IndexDocument> = Vec::new();

    for chapter in &book.content {
        for (i, block) in chapter.blocks.iter().enumerate() {
            let block_id = block
                .ref_id
                .clone()
                .unwrap_or_else(|| format!("{}_{}", chapter.id, i));
            let unique_id = format!("{}:{}", chapter.id, block_id);

            let text_content = match &block.content {
                Some(BlockContent::Text(s)) => s.clone(),
                Some(BlockContent::Inline(elements)) => elements
                    .iter()
                    .filter_map(|e| e.value.clone())
                    .collect::<Vec<_>>()
                    .join(" "),
                None => String::new(),
            };

            if !text_content.trim().is_empty() {
                docs.push(IndexDocument {
                    id: unique_id,
                    text: text_content,
                    label: source_ref.clone(),
                    space_id: space_id.clone(),
                    source_ref: source_ref.clone(),
                    source_type: source_type.clone(),
                    tags: vec!["book".to_string(), chapter.chapter_type.clone()],
                    timestamp_ms,
                    cei_metadata: Some(cei.clone()),
                    modality: Some("text".to_string()),
                });
            }
        }
    }

    let idempotency_key = format!("{}:{}", book.meta.id, book.meta.version_hash);

    match vs
        .index_documents(docs, Some(idempotency_key.as_str()))
        .await
    {
        Ok(outcome) => (
            StatusCode::OK,
            format!(
                "Book '{}' ingested. indexed={} duplicate={}",
                book.meta.id, outcome.indexed_count, outcome.skipped_duplicate
            ),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Indexing failed: {}", e),
        )
            .into_response(),
    }
}

fn iso_to_timestamp_ms(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

async fn generate_grounded_answer(
    question: &str,
    contexts: &[String],
    strict_grounding: bool,
) -> Result<(String, String), String> {
    let config = ConfigService::get();
    let llm_base = resolve_generation_base(config);
    let model = resolve_generation_model();

    let system_prompt = if strict_grounding {
        "You are a grounded knowledge assistant. Use only the provided context. If context is insufficient, explicitly say so. Cite chunk IDs in brackets like [chunk-id]."
    } else {
        "You are a knowledge assistant. Prefer provided context and cite chunk IDs when possible."
    };

    let user_prompt = format!(
        "Question:\n{}\n\nContext:\n{}\n\nReturn concise answer with citations.",
        question,
        contexts.join("\n\n---\n\n")
    );

    let payload = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "stream": false
    });

    let client = reqwest::Client::new();
    let url = generation_chat_url(&llm_base);
    let mut request = client.post(url).json(&payload);
    if let Some(api_key) = generation_api_key() {
        request = request.bearer_auth(api_key);
    }
    let timeout_seconds = env::var("NOSTRA_LLM_TIMEOUT_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(40);

    let response = tokio::time::timeout(
        Duration::from_secs(timeout_seconds),
        request.send(),
    )
    .await
    .map_err(|_| "generator timeout".to_string())
    .and_then(|r| r.map_err(|e| format!("generator request error: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("generator status {}: {}", status, body));
    }

    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("generator parse error: {}", e))?;

    let answer = extract_generation_answer(&value);

    if answer.is_empty() {
        return Err("generator returned empty answer".to_string());
    }

    Ok((
        answer,
        payload["model"].as_str().unwrap_or("unknown").to_string(),
    ))
}

fn resolve_generation_base(config: &ConfigService) -> String {
    env::var("NOSTRA_LLM_BASE_URL")
        .or_else(|_| env::var("NOSTRA_LLM_API_BASE"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
        .or_else(|| {
            config
                .get_llm_config()
                .map(|c| c.api_base.trim_end_matches('/').to_string())
        })
        .unwrap_or_else(|| "http://localhost:11434".to_string())
}

fn resolve_generation_model() -> String {
    generation_model_from_values(
        env::var("NOSTRA_LLM_MODEL").ok().as_deref(),
        env::var("NOSTRA_LOCAL_GENERATION_MODEL").ok().as_deref(),
    )
}

fn generation_model_from_values(llm_model: Option<&str>, local_model: Option<&str>) -> String {
    llm_model
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            local_model
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or("llama3.1:8b")
        .to_string()
}

fn generation_api_key() -> Option<String> {
    env::var("NOSTRA_LLM_API_KEY")
        .or_else(|_| env::var("OPENROUTER_API_KEY"))
        .or_else(|_| env::var("OPENAI_API_KEY"))
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn generation_chat_url(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") || trimmed.ends_with("/api/chat") {
        trimmed.to_string()
    } else if trimmed.contains("openrouter.ai") || trimmed.ends_with("/v1") {
        format!("{trimmed}/chat/completions")
    } else {
        format!("{trimmed}/api/chat")
    }
}

fn extract_generation_answer(value: &serde_json::Value) -> String {
    value
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .or_else(|| {
            value
                .get("message")
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
        })
        .or_else(|| value.get("response").and_then(|content| content.as_str()))
        .unwrap_or("")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_embedding::MockEmbeddingGenerator;
    use crate::skills::extraction::ExtractionOrchestrator;
    use axum::body::to_bytes;
    use axum::http::HeaderValue;
    use ic_agent::identity::AnonymousIdentity;
    use serde_json::Value;
    use tokio::time::Duration;

    fn make_state() -> Arc<AppState> {
        make_state_with_scheduler(None)
    }

    fn make_state_with_scheduler(
        acp_scheduler: Option<Arc<AcpAutomationScheduler>>,
    ) -> Arc<AppState> {
        let runner = Arc::new(WorkflowRunner::new(None));
        let activity_service = ActivityService::new();
        let temporal_governor = TemporalGovernor::new(activity_service.clone());
        let gateway_service = Arc::new(GatewayService::new());

        let agent = Arc::new(
            ic_agent::Agent::builder()
                .with_url("http://127.0.0.1:4943")
                .with_identity(AnonymousIdentity)
                .build()
                .expect("agent"),
        );
        let vector_service = VectorService::new(
            Arc::new(MockEmbeddingGenerator::new()),
            agent,
            "api_test_collection".to_string(),
        );

        Arc::new(AppState {
            runner,
            acp_scheduler,
            vector_service: Some(vector_service),
            activity_service,
            temporal_governor,
            gateway_service,
            extraction_orchestrator: Arc::new(ExtractionOrchestrator::new()),
            extraction_jobs: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    #[test]
    fn generation_model_prefers_live_provider_env() {
        assert_eq!(
            generation_model_from_values(Some(" ~moonshotai/kimi-latest "), Some("llama3.1:8b")),
            "~moonshotai/kimi-latest"
        );
        assert_eq!(
            generation_model_from_values(Some(""), Some("llama3.1:8b")),
            "llama3.1:8b"
        );
        assert_eq!(generation_model_from_values(None, None), "llama3.1:8b");
    }

    #[test]
    fn generation_chat_url_supports_openrouter_and_ollama_shapes() {
        assert_eq!(
            generation_chat_url("https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            generation_chat_url("https://openrouter.ai/api/v1/chat/completions"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            generation_chat_url("http://localhost:11434"),
            "http://localhost:11434/api/chat"
        );
    }

    #[test]
    fn extract_generation_answer_supports_openai_and_ollama_payloads() {
        let openai_like = json!({
            "choices": [
                {
                    "message": {
                        "content": " Kimi answer "
                    }
                }
            ]
        });
        let ollama_chat = json!({
            "message": {
                "content": " Ollama chat answer "
            }
        });
        let ollama_generate = json!({
            "response": " Ollama generate answer "
        });

        assert_eq!(extract_generation_answer(&openai_like), "Kimi answer");
        assert_eq!(extract_generation_answer(&ollama_chat), "Ollama chat answer");
        assert_eq!(
            extract_generation_answer(&ollama_generate),
            "Ollama generate answer"
        );
    }

    #[tokio::test]
    async fn test_search_diagnostics_include_backend_and_embedding_model() {
        let state = make_state();
        let index_req = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:source".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-1".to_string(),
                source_uri: "urn:test:source".to_string(),
                author: "did:example:alice".to_string(),
                timestamp: "2026-02-07T00:00:00Z".to_string(),
                lineage_refs: vec!["v1".to_string()],
                source_version_id: Some("sv-api-1".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("space-api".to_string()),
                produced_by_agent: Some("agent://api-test".to_string()),
                confidence: Some(0.88),
                purpose: Some("retrieval".to_string()),
                modality: Some("text".to_string()),
            }),
            idempotency_key: "idempo-api-1".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-api-1".to_string(),
                text: "Hybrid retrieval diagnostics include backend and embedding model."
                    .to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["diagnostics".to_string()]),
                timestamp_ms: Some(1_700_000_000_000),
                cei_metadata: None,
                modality: None,
            }],
        };

        let index_resp = index_knowledge(State(state.clone()), Json(index_req))
            .await
            .into_response();
        assert_eq!(index_resp.status(), StatusCode::OK);

        let search_req = KnowledgeSearchRequest {
            query: "backend embedding diagnostics".to_string(),
            limit: Some(5),
            retrieval_mode: Some("hybrid".to_string()),
            ui_surface: Some("knowledge-workbench".to_string()),
            filters: None,
            fusion_weights: None,
            diagnostics: Some(true),
            rerank_enabled: Some(true),
        };
        let search_resp = search_knowledge_post(State(state), Json(search_req))
            .await
            .into_response();
        assert_eq!(search_resp.status(), StatusCode::OK);

        let body = to_bytes(search_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&body).expect("json");
        let diagnostic = &payload[0]["diagnostic"];
        assert!(diagnostic["rank_reason"].as_str().unwrap_or("").len() > 0);
        assert!(diagnostic["backend"].as_str().unwrap_or("").len() > 0);
        assert!(diagnostic["embedding_model"].as_str().unwrap_or("").len() > 0);
    }

    #[tokio::test]
    async fn test_search_filters_include_metadata_scope_agent_and_source_version() {
        let state = make_state();
        let index_req = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:filters".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-filter-1".to_string(),
                source_uri: "urn:test:filters".to_string(),
                author: "did:example:alice".to_string(),
                timestamp: "2026-02-08T00:00:00Z".to_string(),
                lineage_refs: vec!["v1".to_string()],
                source_version_id: Some("sv-filter-1".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("space-api".to_string()),
                produced_by_agent: Some("agent://api-test".to_string()),
                confidence: Some(0.9),
                purpose: Some("retrieval".to_string()),
                modality: Some("text".to_string()),
            }),
            idempotency_key: "idempo-api-filter-1".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-api-filter-1".to_string(),
                text: "Metadata filter integration test document".to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["filters".to_string()]),
                timestamp_ms: Some(1_700_001_000_000),
                cei_metadata: None,
                modality: None,
            }],
        };

        let index_resp = index_knowledge(State(state.clone()), Json(index_req))
            .await
            .into_response();
        assert_eq!(index_resp.status(), StatusCode::OK);

        let search_req = KnowledgeSearchRequest {
            query: "metadata filter integration".to_string(),
            limit: Some(5),
            retrieval_mode: Some("hybrid".to_string()),
            ui_surface: Some("ideation".to_string()),
            filters: Some(SearchFilters {
                perspective_scope: Some("space-api".to_string()),
                produced_by_agent: Some("agent://api-test".to_string()),
                source_version_id: Some("sv-filter-1".to_string()),
                ..SearchFilters::default()
            }),
            fusion_weights: None,
            diagnostics: Some(true),
            rerank_enabled: Some(false),
        };

        let search_resp = search_knowledge_post(State(state), Json(search_req))
            .await
            .into_response();
        assert_eq!(search_resp.status(), StatusCode::OK);

        let body = to_bytes(search_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload.as_array().map(|v| v.len()), Some(1));
        assert_eq!(payload[0]["id"], "doc-api-filter-1");
    }

    #[tokio::test]
    async fn test_search_accepts_ui_surface_context() {
        let state = make_state();
        let index_req = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:search-surface".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-search-surface".to_string(),
                source_uri: "urn:test:search-surface".to_string(),
                author: "did:example:surface".to_string(),
                timestamp: "2026-02-08T12:03:00Z".to_string(),
                lineage_refs: vec!["v1".to_string()],
                source_version_id: Some("sv-search-surface".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("surface".to_string()),
                produced_by_agent: Some("agent://surface".to_string()),
                confidence: Some(0.9),
                purpose: Some("search-ui-surface-test".to_string()),
                modality: Some("text".to_string()),
            }),
            idempotency_key: "idempo-search-surface".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-search-surface".to_string(),
                text: "Surface tracing search request test document.".to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["surface".to_string()]),
                timestamp_ms: Some(1_700_002_300_000),
                cei_metadata: None,
                modality: None,
            }],
        };
        let index_resp = index_knowledge(State(state.clone()), Json(index_req))
            .await
            .into_response();
        assert_eq!(index_resp.status(), StatusCode::OK);

        let search_req = KnowledgeSearchRequest {
            query: "surface tracing search".to_string(),
            limit: Some(3),
            retrieval_mode: Some("hybrid".to_string()),
            ui_surface: Some("  projects  ".to_string()),
            filters: None,
            fusion_weights: None,
            diagnostics: Some(false),
            rerank_enabled: Some(false),
        };
        let search_resp = search_knowledge_post(State(state), Json(search_req))
            .await
            .into_response();
        assert_eq!(search_resp.status(), StatusCode::OK);
        let body = to_bytes(search_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload.as_array().map(|v| v.len()), Some(1));
        assert_eq!(payload[0]["id"], "doc-search-surface");
    }

    #[tokio::test]
    async fn test_ask_preserves_filter_intent_for_metadata_fields() {
        let state = make_state();

        let doc_with_scope_a = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:ask:scope-a".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-ask-a".to_string(),
                source_uri: "urn:test:ask:scope-a".to_string(),
                author: "did:example:alice".to_string(),
                timestamp: "2026-02-08T12:00:00Z".to_string(),
                lineage_refs: vec!["v1".to_string()],
                source_version_id: Some("sv-ask-a".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("scope-a".to_string()),
                produced_by_agent: Some("agent://scope-a".to_string()),
                confidence: Some(0.8),
                purpose: Some("ask-filter-test".to_string()),
                modality: Some("text".to_string()),
            }),
            idempotency_key: "idempo-ask-a".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-ask-a".to_string(),
                text: "Scoped ask answer for perspective scope A".to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["ask".to_string()]),
                timestamp_ms: Some(1_700_002_000_000),
                cei_metadata: None,
                modality: None,
            }],
        };
        let resp_a = index_knowledge(State(state.clone()), Json(doc_with_scope_a))
            .await
            .into_response();
        assert_eq!(resp_a.status(), StatusCode::OK);

        let doc_with_scope_b = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:ask:scope-b".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-ask-b".to_string(),
                source_uri: "urn:test:ask:scope-b".to_string(),
                author: "did:example:bob".to_string(),
                timestamp: "2026-02-08T12:01:00Z".to_string(),
                lineage_refs: vec!["v1".to_string()],
                source_version_id: Some("sv-ask-b".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("scope-b".to_string()),
                produced_by_agent: Some("agent://scope-b".to_string()),
                confidence: Some(0.8),
                purpose: Some("ask-filter-test".to_string()),
                modality: Some("text".to_string()),
            }),
            idempotency_key: "idempo-ask-b".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-ask-b".to_string(),
                text: "Scoped ask answer for perspective scope B".to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["ask".to_string()]),
                timestamp_ms: Some(1_700_002_100_000),
                cei_metadata: None,
                modality: None,
            }],
        };
        let resp_b = index_knowledge(State(state.clone()), Json(doc_with_scope_b))
            .await
            .into_response();
        assert_eq!(resp_b.status(), StatusCode::OK);

        let ask_req = KnowledgeAskRequest {
            question: "Which scoped ask answer should be used?".to_string(),
            limit: Some(5),
            retrieval_mode: Some("hybrid".to_string()),
            ui_surface: Some("projects".to_string()),
            filters: Some(SearchFilters {
                perspective_scope: Some("scope-a".to_string()),
                produced_by_agent: Some("agent://scope-a".to_string()),
                source_version_id: Some("sv-ask-a".to_string()),
                ..SearchFilters::default()
            }),
            fusion_weights: None,
            diagnostics: Some(true),
            rerank_enabled: Some(false),
            grounding: None,
            max_context_chunks: Some(2),
            require_provenance: Some(true),
        };
        let ask_resp = ask_knowledge(State(state), Json(ask_req))
            .await
            .into_response();
        assert_eq!(ask_resp.status(), StatusCode::OK);

        let body = to_bytes(ask_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&body).expect("json");
        let citations = payload["citations"].as_array().cloned().unwrap_or_default();
        assert!(!citations.is_empty());
        assert_eq!(citations[0]["id"], "doc-ask-a");
    }

    #[tokio::test]
    async fn test_ask_requires_provenance_when_flagged() {
        let state = make_state();
        let index_req = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:no-provenance".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: None,
            idempotency_key: "idempo-api-2".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-api-no-prov".to_string(),
                text: "This document has no explicit CEI metadata provenance.".to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["provenance".to_string()]),
                timestamp_ms: Some(1_700_000_000_500),
                cei_metadata: None,
                modality: None,
            }],
        };

        let index_resp = index_knowledge(State(state.clone()), Json(index_req))
            .await
            .into_response();
        assert_eq!(index_resp.status(), StatusCode::OK);

        let ask_req = KnowledgeAskRequest {
            question: "What provenance exists for this content?".to_string(),
            limit: Some(5),
            retrieval_mode: Some("hybrid".to_string()),
            ui_surface: Some("main-nav".to_string()),
            filters: None,
            fusion_weights: None,
            diagnostics: Some(true),
            rerank_enabled: Some(false),
            grounding: None,
            max_context_chunks: Some(3),
            require_provenance: Some(true),
        };
        let ask_resp = ask_knowledge(State(state), Json(ask_req))
            .await
            .into_response();
        assert_eq!(ask_resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_normalize_ui_surface_trims_and_drops_empty() {
        assert_eq!(
            normalize_ui_surface(Some("  ideation  ".to_string())),
            Some("ideation".to_string())
        );
        assert_eq!(normalize_ui_surface(Some("   ".to_string())), None);
        assert_eq!(normalize_ui_surface(None), None);
    }

    #[tokio::test]
    async fn test_submit_extraction_local_flow_returns_entities() {
        let state = make_state();
        let request = ExtractionRequestV1 {
            job_id: Some("extract-test-1".to_string()),
            source_ref: "urn:test:extract".to_string(),
            source_type: "text".to_string(),
            schema_ref: Some("nostra.project".to_string()),
            space_id: Some("space-api".to_string()),
            content: "Nostra Cortex uses Rust and Motoko. Alice works at Zipstack Labs."
                .to_string(),
            content_ref: None,
            artifact_path: None,
            mime_type: Some("text/plain".to_string()),
            file_size: None,
            parser_profile: Some("docling".to_string()),
            extraction_mode: nostra_extraction::ExtractionMode::Local,
            fallback_policy: nostra_extraction::ExtractionFallbackPolicyV1::default(),
            timeout_seconds: Some(60),
            index_to_knowledge: false,
            idempotency_key: Some("idem-extract-1".to_string()),
            provenance_hint: None,
        };

        let response = submit_extraction(State(state), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: ExtractionResultV1 = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload.job_id, "extract-test-1");
        assert!(!payload.candidate_entities.is_empty());
    }

    #[tokio::test]
    async fn test_get_extraction_status_round_trip() {
        let state = make_state();
        let request = ExtractionRequestV1 {
            job_id: Some("extract-test-2".to_string()),
            source_ref: "urn:test:extract-status".to_string(),
            source_type: "text".to_string(),
            schema_ref: None,
            space_id: Some("space-api".to_string()),
            content: "Project Atlas depends on Nostra and Cortex.".to_string(),
            content_ref: None,
            artifact_path: None,
            mime_type: Some("text/plain".to_string()),
            file_size: None,
            parser_profile: Some("docling".to_string()),
            extraction_mode: nostra_extraction::ExtractionMode::Local,
            fallback_policy: nostra_extraction::ExtractionFallbackPolicyV1::default(),
            timeout_seconds: Some(60),
            index_to_knowledge: false,
            idempotency_key: Some("idem-extract-2".to_string()),
            provenance_hint: None,
        };

        let submit = submit_extraction(State(state.clone()), Json(request))
            .await
            .into_response();
        assert_eq!(submit.status(), StatusCode::OK);

        let status = get_extraction_status(Path("extract-test-2".to_string()), State(state))
            .await
            .into_response();
        assert_eq!(status.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_extraction_callback_token_enforced() {
        let state = make_state();
        std::env::set_var("NOSTRA_EXTRACTION_CALLBACK_TOKEN", "secret-token");
        let callback_body = ExtractionCallbackRequest {
            result: ExtractionResultV1 {
                job_id: "extract-callback-1".to_string(),
                source_ref: "urn:test:cb".to_string(),
                source_type: "text".to_string(),
                schema_ref: None,
                status: ExtractionStatus::Completed,
                flags: vec![],
                confidence: 0.9,
                candidate_entities: vec![],
                candidate_relations: vec![],
                provenance: Default::default(),
                attempted_backends: vec!["external".to_string()],
                fallback_reason: None,
                normalized_document: None,
                result_ref: None,
            },
        };
        let mut headers = HeaderMap::new();
        let denied =
            extraction_callback(headers.clone(), State(state.clone()), Json(callback_body))
                .await
                .into_response();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);

        let callback_body_ok = ExtractionCallbackRequest {
            result: ExtractionResultV1 {
                job_id: "extract-callback-2".to_string(),
                source_ref: "urn:test:cb2".to_string(),
                source_type: "text".to_string(),
                schema_ref: None,
                status: ExtractionStatus::Completed,
                flags: vec![],
                confidence: 0.95,
                candidate_entities: vec![],
                candidate_relations: vec![],
                provenance: Default::default(),
                attempted_backends: vec!["external".to_string()],
                fallback_reason: None,
                normalized_document: None,
                result_ref: None,
            },
        };
        headers.insert(
            "x-extraction-callback-token",
            HeaderValue::from_static("secret-token"),
        );
        let ok = extraction_callback(headers, State(state), Json(callback_body_ok))
            .await
            .into_response();
        assert_eq!(ok.status(), StatusCode::OK);
        std::env::remove_var("NOSTRA_EXTRACTION_CALLBACK_TOKEN");
    }

    #[tokio::test]
    async fn test_ask_response_includes_ui_surface_when_provided() {
        let state = make_state();
        let index_req = KnowledgeIndexRequest {
            space_id: "space-api".to_string(),
            source_ref: "urn:test:ask-surface".to_string(),
            source_type: Some("note".to_string()),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-ask-surface".to_string(),
                source_uri: "urn:test:ask-surface".to_string(),
                author: "did:example:surface".to_string(),
                timestamp: "2026-02-08T12:02:00Z".to_string(),
                lineage_refs: vec!["v1".to_string()],
                source_version_id: Some("sv-ask-surface".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("surface".to_string()),
                produced_by_agent: Some("agent://surface".to_string()),
                confidence: Some(0.9),
                purpose: Some("ask-ui-surface-test".to_string()),
                modality: Some("text".to_string()),
            }),
            idempotency_key: "idempo-ask-surface".to_string(),
            documents: vec![KnowledgeIndexDocument {
                id: "doc-ask-surface".to_string(),
                text: "Surface tracing response metadata test document.".to_string(),
                label: Some("test".to_string()),
                space_id: None,
                source_ref: None,
                source_type: None,
                tags: Some(vec!["surface".to_string()]),
                timestamp_ms: Some(1_700_002_200_000),
                cei_metadata: None,
                modality: None,
            }],
        };

        let index_resp = index_knowledge(State(state.clone()), Json(index_req))
            .await
            .into_response();
        assert_eq!(index_resp.status(), StatusCode::OK);

        let ask_req = KnowledgeAskRequest {
            question: "What does this surface tracing document cover?".to_string(),
            limit: Some(3),
            retrieval_mode: Some("hybrid".to_string()),
            ui_surface: Some("  ideation  ".to_string()),
            filters: None,
            fusion_weights: None,
            diagnostics: Some(false),
            rerank_enabled: Some(false),
            grounding: None,
            max_context_chunks: Some(2),
            require_provenance: Some(true),
        };
        let ask_resp = ask_knowledge(State(state), Json(ask_req))
            .await
            .into_response();
        assert_eq!(ask_resp.status(), StatusCode::OK);

        let body = to_bytes(ask_resp.into_body(), usize::MAX)
            .await
            .expect("body");
        let payload: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(payload["ui_surface"], "ideation");
    }

    #[tokio::test]
    async fn test_run_acp_now_requires_admin_or_steward_role() {
        let state = make_state();
        let response = run_acp_now(HeaderMap::new(), State(state))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_run_acp_now_returns_disabled_when_scheduler_off() {
        let runner = Arc::new(WorkflowRunner::new(None));
        let gateway_service = Arc::new(GatewayService::new());
        let scheduler = Arc::new(AcpAutomationScheduler::new(
            runner,
            gateway_service,
            false,
            Duration::from_secs(60),
            std::env::temp_dir().join(format!("acp_api_test_{}.json", uuid::Uuid::new_v4())),
        ));

        let state = make_state_with_scheduler(Some(scheduler));
        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", HeaderValue::from_static("admin"));

        let response = run_acp_now(headers, State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }
}
