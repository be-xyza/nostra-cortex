use anyhow::Result;
use chrono::Utc;
use cortex_worker::agent_builder::build_agent_with_resolved_identity;
use cortex_worker::embedding_provider::EmbeddingProvider;
use cortex_worker::mock_embedding::MockEmbeddingGenerator;
use cortex_worker::ollama_embedder::OllamaEmbedder;
use cortex_worker::openai_embedder::OpenAIEmbedder;
use cortex_worker::vector_service::{
    CeiMetadataV1, HybridConfig, IndexDocument, RetrievalMode, SearchFilters, SearchOptions,
    ShadowReport, VectorService,
};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
struct QueryCase {
    query: String,
    relevant: Vec<String>,
}

#[derive(Clone)]
struct FilteredQueryCase {
    query: String,
    relevant: Vec<String>,
    filters: SearchFilters,
}

#[derive(Serialize)]
struct ModeReport {
    mode: String,
    queries: usize,
    recall_at_10: f32,
    ndcg_at_10: f32,
    p50_latency_ms: f32,
    p95_latency_ms: f32,
}

#[derive(Serialize)]
struct BenchmarkReport {
    generated_at: String,
    provider: String,
    backend: String,
    embedding_model: String,
    total_documents: usize,
    total_queries: usize,
    lexical: ModeReport,
    hybrid: ModeReport,
    metadata_filter_queries: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata_filtered_hybrid: Option<ModeReport>,
    delta_recall_at_10: f32,
    delta_ndcg_at_10: f32,
    shadow: ShadowReport,
}

fn percentile(values: &mut [u128], p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_unstable();
    let idx = ((values.len() - 1) as f32 * p).round() as usize;
    values[idx] as f32
}

fn recall_at_k(retrieved: &[String], relevant: &HashSet<String>, k: usize) -> f32 {
    if relevant.is_empty() {
        return 1.0;
    }
    let hits = retrieved
        .iter()
        .take(k)
        .filter(|id| relevant.contains(*id))
        .count();
    hits as f32 / relevant.len() as f32
}

fn ndcg_at_k(retrieved: &[String], relevant: &HashSet<String>, k: usize) -> f32 {
    let mut dcg = 0.0;
    for (idx, id) in retrieved.iter().take(k).enumerate() {
        let rel = if relevant.contains(id) { 1.0 } else { 0.0 };
        if rel > 0.0 {
            let denom = (idx as f32 + 2.0).log2();
            dcg += (2.0_f32.powf(rel) - 1.0) / denom;
        }
    }

    let ideal_hits = std::cmp::min(k, relevant.len());
    if ideal_hits == 0 {
        return 1.0;
    }

    let mut idcg = 0.0;
    for idx in 0..ideal_hits {
        let denom = (idx as f32 + 2.0).log2();
        idcg += (2.0_f32.powf(1.0) - 1.0) / denom;
    }

    if idcg == 0.0 { 0.0 } else { dcg / idcg }
}

async fn evaluate_mode(
    service: &VectorService,
    mode: RetrievalMode,
    cases: &[QueryCase],
) -> Result<ModeReport> {
    let mut recall_sum = 0.0;
    let mut ndcg_sum = 0.0;
    let mut latencies: Vec<u128> = Vec::with_capacity(cases.len());

    for case in cases {
        let options = SearchOptions {
            retrieval_mode: mode.clone(),
            filters: SearchFilters::default(),
            fusion_weights: HybridConfig {
                vector_weight: 0.65,
                lexical_weight: 0.35,
                rerank_enabled: true,
            },
            diagnostics: false,
        };

        let started = Instant::now();
        let results = service
            .search_with_options(&case.query, 10, options)
            .await?;
        latencies.push(started.elapsed().as_millis());

        let ids: Vec<String> = results.into_iter().map(|r| r.id).collect();
        let relevant: HashSet<String> = case.relevant.iter().cloned().collect();
        recall_sum += recall_at_k(&ids, &relevant, 10);
        ndcg_sum += ndcg_at_k(&ids, &relevant, 10);
    }

    let mut p50 = latencies.clone();
    let mut p95 = latencies;

    Ok(ModeReport {
        mode: format!("{:?}", mode).to_lowercase(),
        queries: cases.len(),
        recall_at_10: recall_sum / cases.len() as f32,
        ndcg_at_10: ndcg_sum / cases.len() as f32,
        p50_latency_ms: percentile(&mut p50, 0.50),
        p95_latency_ms: percentile(&mut p95, 0.95),
    })
}

async fn evaluate_mode_filtered(
    service: &VectorService,
    mode: RetrievalMode,
    cases: &[FilteredQueryCase],
) -> Result<ModeReport> {
    let mut recall_sum = 0.0;
    let mut ndcg_sum = 0.0;
    let mut latencies: Vec<u128> = Vec::with_capacity(cases.len());

    for case in cases {
        let options = SearchOptions {
            retrieval_mode: mode.clone(),
            filters: case.filters.clone(),
            fusion_weights: HybridConfig {
                vector_weight: 0.65,
                lexical_weight: 0.35,
                rerank_enabled: true,
            },
            diagnostics: false,
        };

        let started = Instant::now();
        let results = service
            .search_with_options(&case.query, 10, options)
            .await?;
        latencies.push(started.elapsed().as_millis());

        let ids: Vec<String> = results.into_iter().map(|r| r.id).collect();
        let relevant: HashSet<String> = case.relevant.iter().cloned().collect();
        recall_sum += recall_at_k(&ids, &relevant, 10);
        ndcg_sum += ndcg_at_k(&ids, &relevant, 10);
    }

    let mut p50 = latencies.clone();
    let mut p95 = latencies;

    Ok(ModeReport {
        mode: format!("{:?}", mode).to_lowercase(),
        queries: cases.len(),
        recall_at_10: recall_sum / cases.len() as f32,
        ndcg_at_10: ndcg_sum / cases.len() as f32,
        p50_latency_ms: percentile(&mut p50, 0.50),
        p95_latency_ms: percentile(&mut p95, 0.95),
    })
}

fn choose_embedding_provider() -> (String, Arc<dyn EmbeddingProvider>) {
    let provider = std::env::var("NOSTRA_EMBEDDING_PROVIDER")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase();

    let local_model = std::env::var("NOSTRA_LOCAL_EMBEDDING_MODEL")
        .unwrap_or_else(|_| "qwen3-embedding:0.6b".to_string());
    let local_dim = std::env::var("NOSTRA_EMBEDDING_DIM")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(384);
    let local_base = std::env::var("NOSTRA_LLM_API_BASE")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    match provider.as_str() {
        "ollama" | "local" => (
            "ollama".to_string(),
            Arc::new(OllamaEmbedder::with_config(
                local_base,
                local_model,
                local_dim,
            )),
        ),
        "openai" if !api_key.is_empty() => {
            ("openai".to_string(), Arc::new(OpenAIEmbedder::new(api_key)))
        }
        "openai" => (
            "openai->mock(no-key)".to_string(),
            Arc::new(MockEmbeddingGenerator::new()),
        ),
        "mock" => ("mock".to_string(), Arc::new(MockEmbeddingGenerator::new())),
        _ => (
            "auto->ollama".to_string(),
            Arc::new(OllamaEmbedder::with_config(
                local_base,
                local_model,
                local_dim,
            )),
        ),
    }
}

fn benchmark_documents() -> Vec<IndexDocument> {
    let data = vec![
        (
            "doc-gov-1",
            "Nostra governance uses constitutional stewardship and explicit escalation pathways.",
            "governance",
            "space-governance",
            vec!["governance", "constitution"],
        ),
        (
            "doc-gov-2",
            "Contribution lifecycle includes exploratory, deliberative, decisive and executable phases.",
            "governance",
            "space-governance",
            vec!["lifecycle", "contribution"],
        ),
        (
            "doc-vdb-1",
            "ELNA vector storage supports similarity search with strict dimension constraints.",
            "vector",
            "space-vector",
            vec!["elna", "vector"],
        ),
        (
            "doc-vdb-2",
            "Hybrid retrieval combines lexical ranking and semantic ranking using weighted fusion.",
            "vector",
            "space-vector",
            vec!["hybrid", "retrieval"],
        ),
        (
            "doc-ingest-1",
            "Ingestion requires deterministic chunk IDs and idempotent re-index behavior.",
            "ingestion",
            "space-ingestion",
            vec!["ingestion", "idempotency"],
        ),
        (
            "doc-ingest-2",
            "CEI metadata captures source URI, author, timestamp and lineage refs.",
            "ingestion",
            "space-ingestion",
            vec!["cei", "provenance"],
        ),
        (
            "doc-ops-1",
            "Rollback procedure switches VECTOR_BACKEND from elna to mock and validates health endpoints.",
            "operations",
            "space-ops",
            vec!["rollback", "ops"],
        ),
        (
            "doc-ops-2",
            "Shadow comparison tracks overlap and parity between vector and lexical rankings.",
            "operations",
            "space-ops",
            vec!["shadow", "parity"],
        ),
        (
            "doc-ui-1",
            "Knowledge Workbench shows grounded answers and citation provenance panels.",
            "ui",
            "space-labs",
            vec!["workbench", "provenance"],
        ),
        (
            "doc-ui-2",
            "Diagnostics mode exposes vector score, lexical score, fused score and rank reason.",
            "ui",
            "space-labs",
            vec!["diagnostics", "ranking"],
        ),
    ];

    data.into_iter()
        .enumerate()
        .map(
            |(idx, (id, text, source_type, space_id, tags))| IndexDocument {
                id: id.to_string(),
                text: text.to_string(),
                label: format!("bench:{}", source_type),
                space_id: space_id.to_string(),
                source_ref: format!("urn:bench:{}", id),
                source_type: source_type.to_string(),
                tags: tags.into_iter().map(|s| s.to_string()).collect(),
                timestamp_ms: Some(1_700_000_000_000 + idx as i64 * 1000),
                cei_metadata: Some(CeiMetadataV1 {
                    contribution_id: format!("contrib-{}", id),
                    source_uri: format!("urn:bench:{}", id),
                    author: "did:nostra:benchmark".to_string(),
                    timestamp: "2026-02-08T00:00:00Z".to_string(),
                    lineage_refs: vec![format!("lineage:{}", id)],
                    source_version_id: Some(format!("sv-{}", id)),
                    model_id: Some("qwen3-embedding:0.6b".to_string()),
                    perspective_scope: Some(space_id.to_string()),
                    produced_by_agent: Some(format!("agent://{}", source_type)),
                    confidence: Some(0.95),
                    purpose: Some("benchmark_retrieval".to_string()),
                    modality: Some("text".to_string()),
                }),
                modality: None,
            },
        )
        .collect()
}

fn benchmark_queries() -> Vec<QueryCase> {
    let targets: Vec<(&str, Vec<&str>)> = vec![
        (
            "doc-gov-1",
            vec![
                "How are disputes moved up the chain under founding rules?",
                "What guidance defines authority handoff when conflict appears?",
                "Where is escalation process described for stewardship decisions?",
                "Which note covers constitutional conflict routing?",
                "How does the system route governance deadlocks?",
            ],
        ),
        (
            "doc-gov-2",
            vec![
                "What are the stages from brainstorming to action?",
                "Where are planning-to-execution phases listed?",
                "How is idea maturation sequence documented?",
                "Which guidance lists decision progression steps?",
                "What process map defines proposal evolution?",
            ],
        ),
        (
            "doc-vdb-1",
            vec![
                "Which component enforces fixed embedding size during nearest-neighbor lookup?",
                "Where are shape checks for memory vectors defined?",
                "What store validates vector length before similarity queries?",
                "Which module guards embedding width consistency?",
                "How are malformed vector sizes rejected?",
            ],
        ),
        (
            "doc-vdb-2",
            vec![
                "How are keyword and meaning signals blended together?",
                "Where is rank blending with adjustable weights explained?",
                "Which path merges symbolic and embedding-based search?",
                "What method mixes sparse and dense relevance?",
                "How does combined ranking compute final score?",
            ],
        ),
        (
            "doc-ingest-1",
            vec![
                "How do we prevent duplicate indexing when replaying imports?",
                "Which logic keeps segment identifiers stable across reprocessing?",
                "Where is repeat-safe ingestion behavior described?",
                "How is chunk identity kept constant on re-run?",
                "What approach avoids double writes during refresh?",
            ],
        ),
        (
            "doc-ingest-2",
            vec![
                "Which fields preserve origin, writer, time, and ancestry links?",
                "Where is document lineage metadata captured?",
                "What schema stores provenance attributes for each chunk?",
                "How do we record source pointer and contributor details?",
                "Which metadata tracks artifact ancestry over time?",
            ],
        ),
        (
            "doc-ops-1",
            vec![
                "How do operators revert retrieval storage to safe mode?",
                "Which step flips backend from vector canister to in-memory fallback?",
                "What is the emergency downgrade path for search backend?",
                "How is service continuity maintained during backend revert?",
                "Where is backend failback procedure documented?",
            ],
        ),
        (
            "doc-ops-2",
            vec![
                "How is agreement between two ranking paths measured?",
                "Which metric tracks overlap between dense and sparse results?",
                "Where do we compute parity for side-by-side retrieval?",
                "How is ranking concordance monitored in shadow runs?",
                "What reports intersection over union for search outputs?",
            ],
        ),
        (
            "doc-ui-1",
            vec![
                "Where does the interface show citations for generated responses?",
                "Which screen exposes evidence links for answers?",
                "How can users inspect sources behind an answer?",
                "What UI element displays supporting references?",
                "Where is attribution panel in knowledge workspace?",
            ],
        ),
        (
            "doc-ui-2",
            vec![
                "Which debug view reveals component relevance scores?",
                "How can labs inspect why a result ranked first?",
                "Where are score breakdowns shown for retrieval?",
                "What output explains ranking rationale per item?",
                "How are dense/sparse contributions reported?",
            ],
        ),
    ];

    let mut out = Vec::new();
    for (doc_id, queries) in targets {
        for template in queries {
            out.push(QueryCase {
                query: template.to_string(),
                relevant: vec![doc_id.to_string()],
            });
        }
    }
    out
}

fn filters_for_doc(doc_id: &str) -> SearchFilters {
    let (scope, produced_by_agent) = if doc_id.starts_with("doc-gov") {
        ("space-governance", "agent://governance")
    } else if doc_id.starts_with("doc-vdb") {
        ("space-vector", "agent://vector")
    } else if doc_id.starts_with("doc-ingest") {
        ("space-ingestion", "agent://ingestion")
    } else if doc_id.starts_with("doc-ops") {
        ("space-ops", "agent://operations")
    } else if doc_id.starts_with("doc-ui") {
        ("space-labs", "agent://ui")
    } else {
        ("default", "agent://unknown")
    };

    SearchFilters {
        perspective_scope: Some(scope.to_string()),
        produced_by_agent: Some(produced_by_agent.to_string()),
        source_version_id: Some(format!("sv-{}", doc_id)),
        ..SearchFilters::default()
    }
}

fn benchmark_filtered_queries() -> Vec<FilteredQueryCase> {
    benchmark_queries()
        .into_iter()
        .map(|case| {
            let doc_id = case
                .relevant
                .first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let filters = filters_for_doc(&doc_id);
            FilteredQueryCase {
                query: case.query,
                relevant: case.relevant,
                filters,
            }
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::var("IC_URL").unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());
    let (agent, identity_label) = build_agent_with_resolved_identity(&url)?;
    println!(
        "benchmark_retrieval: IC_URL={} identity={}",
        url, identity_label
    );
    let _ = agent.fetch_root_key().await;
    let agent = Arc::new(agent);

    let (provider_name, provider) = choose_embedding_provider();
    let embedding_model = provider.model_id().to_string();

    let service = VectorService::new(provider, agent, "nostra_retrieval_benchmark".to_string());
    let _ = service.init_collection().await;

    let docs = benchmark_documents();
    let _ = service
        .index_documents(docs.clone(), Some("benchmark-retrieval-closeout-v1"))
        .await?;

    let cases = benchmark_queries();

    let lexical = evaluate_mode(&service, RetrievalMode::Lexical, &cases).await?;
    let hybrid = evaluate_mode(&service, RetrievalMode::Hybrid, &cases).await?;
    let metadata_filter_enabled = std::env::var("NOSTRA_BENCHMARK_METADATA_FILTERS")
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    let (metadata_filter_queries, metadata_filtered_hybrid) = if metadata_filter_enabled {
        let filtered_cases = benchmark_filtered_queries();
        let filtered_hybrid =
            evaluate_mode_filtered(&service, RetrievalMode::Hybrid, &filtered_cases).await?;
        (filtered_cases.len(), Some(filtered_hybrid))
    } else {
        (0usize, None)
    };

    let report = BenchmarkReport {
        generated_at: Utc::now().to_rfc3339(),
        provider: provider_name,
        backend: service.backend_name().to_string(),
        embedding_model,
        total_documents: docs.len(),
        total_queries: cases.len(),
        metadata_filter_queries,
        metadata_filtered_hybrid,
        delta_recall_at_10: hybrid.recall_at_10 - lexical.recall_at_10,
        delta_ndcg_at_10: hybrid.ndcg_at_10 - lexical.ndcg_at_10,
        lexical,
        hybrid,
        shadow: service.shadow_report(),
    };

    let output = serde_json::to_string_pretty(&report)?;
    println!("{}", output);

    let dir = Path::new("../../logs/knowledge");
    fs::create_dir_all(dir)?;
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let report_path = dir.join(format!("retrieval_benchmark_{}.json", timestamp));
    let latest_path = dir.join("retrieval_benchmark_latest.json");

    fs::write(&report_path, &output)?;
    fs::write(&latest_path, &output)?;

    println!(
        "saved report: {} and {}",
        report_path.display(),
        latest_path.display()
    );

    Ok(())
}
