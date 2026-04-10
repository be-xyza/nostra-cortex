use crate::api::{worker_get_extraction_status, worker_model_health, worker_submit_extraction};
use crate::services::knowledge_search::{
    SurfaceAskInput, SurfaceSearchInput, ask_knowledge_for_surface, parse_csv_field,
    search_knowledge_for_surface,
};
use crate::types::{
    ExtractionFallbackPolicyV1, ExtractionMode, ExtractionRequestV1, ExtractionResultV1,
    ExtractionStatus, KnowledgeAskResponse as AskResponse,
    KnowledgeModelHealthResponse as ModelHealthResponse, KnowledgeSearchResult as SearchResultItem,
    SearchFilters,
};
use dioxus::prelude::*;

fn parse_extraction_mode(raw: &str) -> ExtractionMode {
    match raw.trim().to_lowercase().as_str() {
        "cloud_fallback" => ExtractionMode::CloudFallback,
        "external_adapter" => ExtractionMode::ExternalAdapter,
        _ => ExtractionMode::Local,
    }
}

fn extraction_status_label(status: &ExtractionStatus) -> &'static str {
    match status {
        ExtractionStatus::Submitted => "submitted",
        ExtractionStatus::Running => "running",
        ExtractionStatus::Completed => "completed",
        ExtractionStatus::Failed => "failed",
        ExtractionStatus::NeedsReview => "needs_review",
    }
}

#[component]
pub fn KnowledgeWorkbenchLab() -> Element {
    let mut extraction_processing = use_signal(|| false);
    let mut extraction_feedback = use_signal(|| String::new());
    let mut extraction_result = use_signal(|| None::<ExtractionResultV1>);
    let mut extraction_mode = use_signal(|| "local".to_string());
    let mut extraction_source_ref = use_signal(|| "urn:nostra:lab:doc-001".to_string());
    let mut extraction_source_type = use_signal(|| "text".to_string());
    let mut extraction_schema_ref = use_signal(|| "nostra.project".to_string());
    let mut extraction_content = use_signal(|| {
        "Nostra Cortex local-first extraction can parse unstructured documents.\n\
         Zipstack teams use Rust and Motoko for worker and canister paths.\n\
         Project Atlas depends on Nostra workflow engine."
            .to_string()
    });
    let mut extraction_fallback_enabled = use_signal(|| false);
    let mut extraction_fallback_min_confidence = use_signal(|| "0.85".to_string());
    let mut extraction_allow_external_adapter = use_signal(|| false);
    let mut extraction_job_lookup = use_signal(|| String::new());

    let mut input_json = use_signal(|| {
        r#"{
    "@context": ["https://nostra.network/ns/v2"],
    "meta": {
        "id": "urn:nostra:book:frontend-test",
        "type": "Contribution::Book",
        "version_hash": "sha256:0",
        "phase": "Exploratory",
        "provenance": { "author_did": "user", "space_did": "space", "created_at": "2026-02-07T00:00:00Z" }
    },
    "structure": { "toc": [] },
    "content": [
      {
        "id": "ch-1",
        "type": "Chapter",
        "blocks": [
          { "type": "Paragraph", "content": "Nostra Cortex knowledge engine local-first retrieval baseline." }
        ]
      }
    ]
}"#
        .to_string()
    });
    let mut ingest_feedback = use_signal(|| String::new());

    let mut search_query = use_signal(|| String::new());
    let mut search_results = use_signal(|| Vec::<SearchResultItem>::new());
    let mut search_feedback = use_signal(|| String::new());
    let mut retrieval_mode = use_signal(|| "hybrid".to_string());
    let mut diagnostics = use_signal(|| true);
    let mut rerank_enabled = use_signal(|| false);
    let mut filter_perspective_scope = use_signal(|| String::new());
    let mut filter_produced_by_agent = use_signal(|| String::new());
    let mut filter_source_version_id = use_signal(|| String::new());
    let mut filter_modalities = use_signal(|| String::new());

    let mut ask_question = use_signal(|| String::new());
    let mut ask_feedback = use_signal(|| String::new());
    let mut ask_result = use_signal(|| None::<AskResponse>);

    let mut health_feedback = use_signal(|| String::new());
    let mut model_health = use_signal(|| None::<ModelHealthResponse>);

    let run_extraction = move |_| {
        let source_ref = extraction_source_ref.read().trim().to_string();
        let source_type = extraction_source_type.read().trim().to_string();
        let schema_ref_raw = extraction_schema_ref.read().trim().to_string();
        let content = extraction_content.read().clone();
        let mode = extraction_mode.read().clone();
        let fallback_enabled = *extraction_fallback_enabled.read();
        let fallback_allow_external = *extraction_allow_external_adapter.read();
        let min_confidence = extraction_fallback_min_confidence
            .read()
            .trim()
            .parse::<f32>()
            .unwrap_or(0.85)
            .clamp(0.0, 1.0);

        if source_ref.is_empty() || source_type.is_empty() {
            extraction_feedback.set("source_ref and source_type are required".to_string());
            return;
        }
        if content.trim().is_empty() {
            extraction_feedback.set("Extraction content cannot be empty".to_string());
            return;
        }

        extraction_processing.set(true);
        extraction_feedback.set("Submitting extraction job...".to_string());
        spawn(async move {
            let request = ExtractionRequestV1 {
                job_id: None,
                source_ref,
                source_type,
                schema_ref: if schema_ref_raw.is_empty() {
                    None
                } else {
                    Some(schema_ref_raw)
                },
                space_id: Some("space://knowledge-workbench".to_string()),
                content,
                content_ref: None,
                artifact_path: None,
                mime_type: Some("text/plain".to_string()),
                file_size: None,
                parser_profile: Some("docling".to_string()),
                extraction_mode: parse_extraction_mode(&mode),
                fallback_policy: ExtractionFallbackPolicyV1 {
                    enabled: fallback_enabled,
                    min_confidence,
                    provider: Some("azure_document_intelligence".to_string()),
                    allow_external_adapter: fallback_allow_external,
                },
                timeout_seconds: Some(60),
                index_to_knowledge: true,
                idempotency_key: None,
                provenance_hint: None,
            };

            match worker_submit_extraction(&request).await {
                Ok(result) => {
                    extraction_job_lookup.set(result.job_id.clone());
                    extraction_feedback.set(format!(
                        "Extraction {} (confidence {:.2})",
                        extraction_status_label(&result.status),
                        result.confidence
                    ));
                    extraction_result.set(Some(result));
                }
                Err(err) => extraction_feedback.set(format!("Extraction failed: {}", err)),
            }

            extraction_processing.set(false);
        });
    };

    let refresh_extraction_status = move |_| {
        let job_id = extraction_job_lookup.read().trim().to_string();
        if job_id.is_empty() {
            extraction_feedback.set("Enter a job id to refresh status".to_string());
            return;
        }

        extraction_processing.set(true);
        extraction_feedback.set(format!("Refreshing extraction status for {}...", job_id));
        spawn(async move {
            match worker_get_extraction_status(&job_id).await {
                Ok(result) => {
                    extraction_feedback.set(format!(
                        "Job {} is {} (confidence {:.2})",
                        result.job_id,
                        extraction_status_label(&result.status),
                        result.confidence
                    ));
                    extraction_result.set(Some(result));
                }
                Err(err) => extraction_feedback.set(format!("Status refresh failed: {}", err)),
            }
            extraction_processing.set(false);
        });
    };

    let run_ingest = move |_| {
        let json_str = input_json.read().clone();
        ingest_feedback.set("Ingesting...".to_string());

        spawn(async move {
            let client = reqwest::Client::new();
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                match client
                    .post("http://localhost:3003/knowledge/ingest/book")
                    .json(&json_val)
                    .send()
                    .await
                {
                    Ok(res) => {
                        if res.status().is_success() {
                            let text = res.text().await.unwrap_or_default();
                            ingest_feedback.set(format!("Success: {}", text));
                        } else {
                            let status = res.status();
                            let body = res.text().await.unwrap_or_default();
                            ingest_feedback.set(format!("API Error: {} {}", status, body));
                        }
                    }
                    Err(e) => {
                        ingest_feedback.set(format!("Network Error: {}", e));
                    }
                }
            } else {
                ingest_feedback.set("Invalid JSON syntax".to_string());
            }
        });
    };

    let run_search = move |_| {
        let query = search_query.read().clone();
        if query.trim().is_empty() {
            search_feedback.set("Enter a query".to_string());
            return;
        }

        let mode = retrieval_mode.read().clone();
        let diag = *diagnostics.read();
        let rerank = *rerank_enabled.read();
        let perspective_scope = filter_perspective_scope.read().trim().to_string();
        let produced_by_agent = filter_produced_by_agent.read().trim().to_string();
        let source_version_id = filter_source_version_id.read().trim().to_string();
        let modalities = parse_csv_field(filter_modalities.read().trim());
        let filters = if perspective_scope.is_empty()
            && produced_by_agent.is_empty()
            && source_version_id.is_empty()
            && modalities.is_empty()
        {
            None
        } else {
            Some(SearchFilters {
                perspective_scope: if perspective_scope.is_empty() {
                    None
                } else {
                    Some(perspective_scope)
                },
                produced_by_agent: if produced_by_agent.is_empty() {
                    None
                } else {
                    Some(produced_by_agent)
                },
                source_version_id: if source_version_id.is_empty() {
                    None
                } else {
                    Some(source_version_id)
                },
                modalities,
                ..SearchFilters::default()
            })
        };

        search_feedback.set("Searching...".to_string());
        search_results.set(vec![]);

        spawn(async move {
            let request = SurfaceSearchInput {
                query,
                limit: 8,
                retrieval_mode: mode,
                diagnostics: diag,
                rerank_enabled: rerank,
                filters,
                modalities: vec![],
            };

            match search_knowledge_for_surface("knowledge-workbench", request).await {
                Ok(items) => {
                    let count = items.len();
                    search_results.set(items);
                    search_feedback.set(format!("Found {} results", count));
                }
                Err(err) => {
                    search_feedback.set(format!("Search failed: {}", err));
                }
            }
        });
    };

    let run_ask = move |_| {
        let question = ask_question.read().clone();
        if question.trim().is_empty() {
            ask_feedback.set("Enter a question".to_string());
            return;
        }

        let mode = retrieval_mode.read().clone();
        let diag = *diagnostics.read();
        let rerank = *rerank_enabled.read();
        let perspective_scope = filter_perspective_scope.read().trim().to_string();
        let produced_by_agent = filter_produced_by_agent.read().trim().to_string();
        let source_version_id = filter_source_version_id.read().trim().to_string();
        let modalities = parse_csv_field(filter_modalities.read().trim());
        let filters = if perspective_scope.is_empty()
            && produced_by_agent.is_empty()
            && source_version_id.is_empty()
            && modalities.is_empty()
        {
            None
        } else {
            Some(SearchFilters {
                perspective_scope: if perspective_scope.is_empty() {
                    None
                } else {
                    Some(perspective_scope)
                },
                produced_by_agent: if produced_by_agent.is_empty() {
                    None
                } else {
                    Some(produced_by_agent)
                },
                source_version_id: if source_version_id.is_empty() {
                    None
                } else {
                    Some(source_version_id)
                },
                modalities,
                ..SearchFilters::default()
            })
        };

        ask_feedback.set("Generating grounded answer...".to_string());
        ask_result.set(None);

        spawn(async move {
            let request = SurfaceAskInput {
                question,
                limit: 8,
                retrieval_mode: mode,
                diagnostics: diag,
                rerank_enabled: rerank,
                filters,
                modalities: vec![],
                max_context_chunks: 4,
                require_provenance: true,
            };

            match ask_knowledge_for_surface("knowledge-workbench", request).await {
                Ok(payload) => {
                    ask_feedback.set("Grounded answer ready".to_string());
                    ask_result.set(Some(payload));
                }
                Err(err) => ask_feedback.set(format!("Ask failed: {}", err)),
            }
        });
    };

    let run_health = move |_| {
        health_feedback.set("Checking model/vector health...".to_string());
        model_health.set(None);

        spawn(async move {
            match worker_model_health().await {
                Ok(payload) => {
                    health_feedback.set(format!(
                        "Health: {} (model: {})",
                        payload.status, payload.generation_model
                    ));
                    model_health.set(Some(payload));
                }
                Err(err) => health_feedback.set(format!("Health check failed: {}", err)),
            }
        });
    };

    rsx! {
        div { class: "flex flex-col h-full bg-background text-foreground p-6 overflow-y-auto",
            h1 { class: "text-2xl font-bold mb-4", "Knowledge Workbench" }
            p { class: "text-muted-foreground mb-6", "Ingest documents, run hybrid retrieval, and generate grounded answers with provenance." }

            div { class: "mb-4 p-4 rounded-lg border border-gray-700 bg-gray-900/40",
                div { class: "flex flex-wrap gap-3 items-center",
                    label { class: "text-sm", "Retrieval Mode" }
                    select {
                        class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm",
                        value: "{retrieval_mode}",
                        onchange: move |evt| retrieval_mode.set(evt.value()),
                        option { value: "hybrid", "Hybrid" }
                        option { value: "vector", "Vector" }
                        option { value: "lexical", "Lexical" }
                    }
                    label { class: "text-sm flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: *diagnostics.read(),
                            onchange: move |evt| diagnostics.set(evt.value() == "true" || evt.value() == "on")
                        }
                        "Diagnostics"
                    }
                    label { class: "text-sm flex items-center gap-2",
                        input {
                            r#type: "checkbox",
                            checked: *rerank_enabled.read(),
                            onchange: move |evt| rerank_enabled.set(evt.value() == "true" || evt.value() == "on")
                        }
                        "Rerank"
                    }
                    button {
                        class: "px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded text-sm",
                        onclick: run_health,
                        "Check Health"
                    }
                }
                div { class: "text-xs text-gray-400 mt-2", "{health_feedback}" }
                if let Some(health) = model_health.read().clone() {
                    div { class: "text-xs text-gray-300 mt-2 font-mono",
                        "llm_base={health.llm_base} model={health.generation_model}"
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-2 mt-3",
                    input {
                        class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                        placeholder: "filter.perspective_scope",
                        value: "{filter_perspective_scope}",
                        oninput: move |evt| filter_perspective_scope.set(evt.value())
                    }
                    input {
                        class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                        placeholder: "filter.produced_by_agent",
                        value: "{filter_produced_by_agent}",
                        oninput: move |evt| filter_produced_by_agent.set(evt.value())
                    }
                    input {
                        class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                        placeholder: "filter.source_version_id",
                        value: "{filter_source_version_id}",
                        oninput: move |evt| filter_source_version_id.set(evt.value())
                    }
                }
                div { class: "grid grid-cols-1 gap-2 mt-2",
                    input {
                        class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                        placeholder: "filter.modalities (csv: text,image,audio,video)",
                        value: "{filter_modalities}",
                        oninput: move |evt| filter_modalities.set(evt.value())
                    }
                }
            }

            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                div { class: "flex flex-col gap-4",
                    h2 { class: "text-xl font-semibold", "Extraction Lab" }
                    input {
                        class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                        placeholder: "source_ref",
                        value: "{extraction_source_ref}",
                        oninput: move |evt| extraction_source_ref.set(evt.value())
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-2",
                        input {
                            class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                            placeholder: "source_type",
                            value: "{extraction_source_type}",
                            oninput: move |evt| extraction_source_type.set(evt.value())
                        }
                        input {
                            class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                            placeholder: "schema_ref (optional)",
                            value: "{extraction_schema_ref}",
                            oninput: move |evt| extraction_schema_ref.set(evt.value())
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-2",
                        select {
                            class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                            value: "{extraction_mode}",
                            onchange: move |evt| extraction_mode.set(evt.value()),
                            option { value: "local", "local" }
                            option { value: "cloud_fallback", "cloud_fallback" }
                            option { value: "external_adapter", "external_adapter" }
                        }
                        input {
                            class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                            placeholder: "min_confidence (0-1)",
                            value: "{extraction_fallback_min_confidence}",
                            oninput: move |evt| extraction_fallback_min_confidence.set(evt.value())
                        }
                    }
                    div { class: "flex flex-wrap gap-3",
                        label { class: "text-xs flex items-center gap-2",
                            input {
                                r#type: "checkbox",
                                checked: *extraction_fallback_enabled.read(),
                                onchange: move |evt| extraction_fallback_enabled.set(evt.value() == "true" || evt.value() == "on")
                            }
                            "Enable Cloud Fallback"
                        }
                        label { class: "text-xs flex items-center gap-2",
                            input {
                                r#type: "checkbox",
                                checked: *extraction_allow_external_adapter.read(),
                                onchange: move |evt| extraction_allow_external_adapter.set(evt.value() == "true" || evt.value() == "on")
                            }
                            "Allow External Adapter"
                        }
                    }
                    textarea {
                        class: "w-full h-48 bg-gray-800 text-white font-mono text-sm p-3 rounded-lg border border-gray-700 focus:border-blue-500 outline-none resize-none",
                        value: "{extraction_content}",
                        oninput: move |evt| extraction_content.set(evt.value())
                    }
                    div { class: "flex flex-wrap gap-2",
                        button {
                            class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-lg text-white font-bold transition-colors",
                            onclick: run_extraction,
                            "Run Extraction"
                        }
                        input {
                            class: "bg-gray-800 border border-gray-700 rounded px-3 py-2 text-xs font-mono",
                            placeholder: "job_id for status refresh",
                            value: "{extraction_job_lookup}",
                            oninput: move |evt| extraction_job_lookup.set(evt.value())
                        }
                        button {
                            class: "px-3 py-2 bg-slate-700 hover:bg-slate-600 rounded text-xs",
                            onclick: refresh_extraction_status,
                            "Refresh Status"
                        }
                    }
                    div { class: "text-sm text-gray-400", "{extraction_feedback}" }
                    if *extraction_processing.read() {
                        div { class: "text-yellow-400 font-mono text-sm", "Processing..." }
                    }
                    if let Some(result) = extraction_result.read().clone() {
                        div { class: "p-4 bg-gray-900 rounded-lg mt-2 border border-gray-700",
                            div { class: "text-xs text-gray-400 font-mono", "job_id={result.job_id}" }
                            div { class: "text-xs text-gray-400 font-mono", "status={extraction_status_label(&result.status)} confidence={result.confidence:.2}" }
                            div { class: "text-xs text-gray-500 font-mono", "backend={result.provenance.extraction_backend} model={result.provenance.model_id}" }
                            if !result.flags.is_empty() {
                                div { class: "mt-2 flex flex-wrap gap-2",
                                    for flag in result.flags.iter() {
                                        span { class: "px-2 py-1 rounded bg-amber-700/50 text-[11px] font-mono border border-amber-500/40", "{flag}" }
                                    }
                                }
                            }
                            div { class: "mt-3 text-sm font-semibold", "Candidate Entities" }
                            div { class: "mt-1 flex flex-col gap-1 max-h-32 overflow-y-auto",
                                for entity in result.candidate_entities.iter() {
                                    div { class: "text-xs font-mono text-gray-300", "{entity.id}: {entity.label} [{entity.entity_type}]" }
                                }
                            }
                            div { class: "mt-3 text-sm font-semibold", "Candidate Relations" }
                            div { class: "mt-1 flex flex-col gap-1 max-h-24 overflow-y-auto",
                                for rel in result.candidate_relations.iter() {
                                    div { class: "text-xs font-mono text-gray-300", "{rel.id}: {rel.source_id} -{rel.relation_type}-> {rel.target_id}" }
                                }
                            }
                        }
                    }
                }

                div { class: "flex flex-col gap-4",
                    h2 { class: "text-xl font-semibold", "Manual Ingestion" }
                    textarea {
                        class: "w-full h-64 bg-gray-800 text-white font-mono text-sm p-4 rounded-lg border border-gray-700 focus:border-blue-500 outline-none resize-none",
                        value: "{input_json}",
                        oninput: move |evt| input_json.set(evt.value())
                    }
                    button {
                        class: "px-4 py-2 bg-green-600 hover:bg-green-500 rounded-lg text-white font-bold transition-colors",
                        onclick: run_ingest,
                        "Ingest NostraBook JSON"
                    }
                    div { class: "p-4 bg-black/30 rounded-lg font-mono text-sm min-h-[3rem]",
                        "{ingest_feedback}"
                    }
                }

                div { class: "flex flex-col gap-4",
                    h2 { class: "text-xl font-semibold", "Knowledge Search" }
                    div { class: "flex gap-2",
                        input {
                            class: "flex-1 bg-gray-800 text-white px-4 py-2 rounded-lg border border-gray-700 focus:border-blue-500 outline-none",
                            placeholder: "Search query...",
                            value: "{search_query}",
                            oninput: move |evt| search_query.set(evt.value())
                        }
                        button {
                            class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-lg text-white font-bold transition-colors",
                            onclick: run_search,
                            "Search"
                        }
                    }
                    div { class: "text-sm text-gray-400", "{search_feedback}" }

                    div { class: "flex flex-col gap-2 mt-2 max-h-80 overflow-y-auto",
                        for item in search_results.read().iter() {
                            div { class: "p-3 bg-gray-800 rounded-lg border border-gray-700 hover:border-blue-500",
                                div { class: "font-mono text-xs text-blue-400 mb-1", "{item.id}" }
                                div { class: "text-xs text-gray-300", "score={item.score:.4}" }
                                if let Some(src) = &item.source_ref {
                                    div { class: "text-xs text-gray-500", "source={src}" }
                                }
                                if let Some(modality) = &item.modality {
                                    div { class: "text-xs text-gray-500", "modality={modality}" }
                                }
                                if let Some(content) = &item.content {
                                    div { class: "text-xs text-gray-400 mt-1", "{content}" }
                                }
                                if let Some(diag) = &item.diagnostic {
                                    div { class: "text-[11px] text-amber-300 mt-1 font-mono",
                                        "v={diag.vector_score:.4} l={diag.lexical_score:.4} f={diag.fused_score:.4} ({diag.rank_reason}) [{diag.backend} | {diag.embedding_model}]"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "mt-8 p-4 rounded-lg border border-gray-700 bg-gray-900/40",
                h2 { class: "text-xl font-semibold mb-3", "Grounded Ask" }
                div { class: "flex gap-2",
                    input {
                        class: "flex-1 bg-gray-800 text-white px-4 py-2 rounded-lg border border-gray-700 focus:border-blue-500 outline-none",
                        placeholder: "Ask a grounded question...",
                        value: "{ask_question}",
                        oninput: move |evt| ask_question.set(evt.value())
                    }
                    button {
                        class: "px-4 py-2 bg-purple-600 hover:bg-purple-500 rounded-lg text-white font-bold transition-colors",
                        onclick: run_ask,
                        "Ask"
                    }
                }
                div { class: "text-sm text-gray-400 mt-2", "{ask_feedback}" }

                if let Some(payload) = ask_result.read().clone() {
                    div { class: "mt-3 p-3 bg-gray-800 rounded border border-gray-700",
                        div { class: "text-xs text-gray-500 font-mono mb-2", "trace={payload.trace_id} model={payload.model}" }
                        div { class: "text-sm whitespace-pre-wrap", "{payload.answer}" }
                        div { class: "mt-3 text-xs font-semibold text-gray-300", "Citations" }
                        div { class: "mt-1 flex flex-col gap-2",
                            for citation in payload.citations.iter() {
                                div { class: "p-2 rounded bg-black/30 border border-gray-700",
                                    div { class: "font-mono text-xs text-blue-300", "{citation.id} (score={citation.score:.4})" }
                                    if let Some(src) = &citation.source_ref {
                                        div { class: "text-xs text-gray-500", "{src}" }
                                    }
                                    if let Some(text) = &citation.content {
                                        div { class: "text-xs text-gray-400 mt-1", "{text}" }
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
