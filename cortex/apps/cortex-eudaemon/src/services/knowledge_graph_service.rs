use crate::services::knowledge_graph_query::{
    InMemoryTriple, TripleQueryAdapter, TripleQueryRequest, TripleQueryResponse,
};
use crate::services::knowledge_graph_retrieval::{
    GraphRetrievalBenchmarkCase, GraphRetrievalHarness, GraphRetrievalRequest,
    GraphRetrievalResponse, VectorRetrievalHit,
};
use crate::services::knowledge_graph_runtime::{GlobalEventTripleRecord, RuntimeTripleProjector};
use crate::services::knowledge_graph_topology::{ExploreTopologyView, build_topology_view};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;

const SERVICE_SCHEMA_VERSION: &str = "1.0.0";
const CURRENT_037_BASELINE_LABEL: &str = "037_current_hybrid_retrieval";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotModeExecution {
    pub retrieval_mode: String,
    pub recall_score: f64,
    pub latency_ms: u64,
    pub citation_source_refs: Vec<String>,
    pub citation_count: usize,
    pub provenance_satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotCaseReport {
    pub case_id: String,
    pub query_class: String,
    pub query_id: String,
    pub query_text: String,
    pub relevant_source_refs: Vec<String>,
    pub results: Vec<GraphPilotModeExecution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotModeSummary {
    pub retrieval_mode: String,
    pub cases: usize,
    pub average_recall_score: f64,
    pub average_latency_ms: f64,
    pub provenance_pass_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotBenchmarkReport {
    pub schema_version: String,
    pub generated_at: String,
    pub dataset_ref: String,
    pub cases: Vec<GraphPilotCaseReport>,
    pub summaries: Vec<GraphPilotModeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyRetrievalBenchmarkMode {
    pub mode: String,
    pub queries: u64,
    #[serde(alias = "recall_at_10")]
    pub recall_at_10: f64,
    #[serde(alias = "ndcg_at_10")]
    pub ndcg_at_10: f64,
    #[serde(alias = "p50_latency_ms")]
    pub p50_latency_ms: f64,
    #[serde(alias = "p95_latency_ms")]
    pub p95_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyRetrievalBenchmarkReport {
    #[serde(alias = "generated_at")]
    pub generated_at: String,
    pub provider: String,
    pub backend: String,
    #[serde(alias = "embedding_model")]
    pub embedding_model: String,
    #[serde(alias = "total_documents")]
    pub total_documents: u64,
    #[serde(alias = "total_queries")]
    pub total_queries: u64,
    pub lexical: LegacyRetrievalBenchmarkMode,
    pub hybrid: LegacyRetrievalBenchmarkMode,
    #[serde(default)]
    #[serde(alias = "metadata_filter_queries")]
    pub metadata_filter_queries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyRetrievalEvaluationCase {
    pub case_id: String,
    pub query_class: String,
    pub query_id: String,
    pub query_text: String,
    pub mode: String,
    pub relevant_source_refs: Vec<String>,
    pub citation_source_refs: Vec<String>,
    pub citation_count: usize,
    pub recall_score: f64,
    pub latency_ms: u64,
    pub provenance_satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LegacyRetrievalEvaluationReport {
    pub schema_version: String,
    pub generated_at: String,
    pub baseline_source_ref: String,
    pub cases: Vec<LegacyRetrievalEvaluationCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotComparisonEntry {
    pub baseline_label: String,
    pub pilot_mode: String,
    pub baseline_recall_score: f64,
    pub pilot_recall_score: f64,
    pub recall_delta: f64,
    pub baseline_latency_ms: f64,
    pub pilot_latency_ms: f64,
    pub latency_comparable: bool,
    pub citation_ready: bool,
    pub beats_baseline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotComparisonReport {
    pub schema_version: String,
    pub generated_at: String,
    pub baseline_source_ref: String,
    pub graph_dataset_ref: String,
    pub entries: Vec<GraphPilotComparisonEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotSharedEvaluationEntry {
    pub case_id: String,
    pub query_class: String,
    pub query_id: String,
    pub query_text: String,
    pub mode: String,
    pub relevant_source_refs: Vec<String>,
    pub citation_source_refs: Vec<String>,
    pub citation_count: usize,
    pub recall_score: f64,
    pub latency_ms: u64,
    pub provenance_satisfied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphPilotSharedEvaluationReport {
    pub schema_version: String,
    pub generated_at: String,
    pub baseline_source_ref: String,
    pub graph_dataset_ref: String,
    pub entries: Vec<GraphPilotSharedEvaluationEntry>,
}

pub struct KnowledgeGraphService;

impl KnowledgeGraphService {
    pub fn project_runtime_records(
        records: &[GlobalEventTripleRecord],
    ) -> Result<Vec<InMemoryTriple>, String> {
        RuntimeTripleProjector::project(records)
    }

    pub fn execute_triple_query(
        records: &[GlobalEventTripleRecord],
        request: &TripleQueryRequest,
    ) -> Result<TripleQueryResponse, String> {
        let triples = Self::project_runtime_records(records)?;
        TripleQueryAdapter::execute(&triples, request)
    }

    pub fn execute_retrieval(
        records: &[GlobalEventTripleRecord],
        vector_hits: &[VectorRetrievalHit],
        request: &GraphRetrievalRequest,
    ) -> Result<GraphRetrievalResponse, String> {
        let triples = Self::project_runtime_records(records)?;
        GraphRetrievalHarness::execute(&triples, vector_hits, request)
    }

    pub fn derive_topology(
        space_id: &str,
        generated_from: &str,
        records: &[GlobalEventTripleRecord],
    ) -> Result<ExploreTopologyView, String> {
        let triples = Self::project_runtime_records(records)?;
        Ok(build_topology_view(space_id, generated_from, &triples))
    }

    pub fn benchmark_runtime_records(
        dataset_ref: &str,
        records: &[GlobalEventTripleRecord],
        vector_hits: &[VectorRetrievalHit],
        cases: &[GraphRetrievalBenchmarkCase],
    ) -> Result<GraphPilotBenchmarkReport, String> {
        let triples = Self::project_runtime_records(records)?;
        let mut case_reports = Vec::new();

        for case in cases {
            let mut results = Vec::new();
            for retrieval_mode in ["graph_only", "vector_only", "hybrid_graph_embedding"] {
                let request = GraphRetrievalRequest {
                    retrieval_mode: retrieval_mode.to_string(),
                    ..case.request.clone()
                };
                let start = Instant::now();
                let response = GraphRetrievalHarness::execute(&triples, vector_hits, &request)?;
                let latency_ms = elapsed_ms(start);
                let citation_source_refs = response
                    .citations
                    .iter()
                    .map(|item| item.source_ref.clone())
                    .collect::<Vec<_>>();
                let recall_score = recall_score(&citation_source_refs, &case.relevant_source_refs);
                let provenance_satisfied = provenance_satisfied(&response);
                results.push(GraphPilotModeExecution {
                    retrieval_mode: retrieval_mode.to_string(),
                    recall_score,
                    latency_ms,
                    citation_source_refs,
                    citation_count: response.citations.len(),
                    provenance_satisfied,
                });
            }

            case_reports.push(GraphPilotCaseReport {
                case_id: case.case_id.clone(),
                query_class: case.query_class.clone(),
                query_id: case.request.query_id.clone(),
                query_text: case.request.query_text.clone(),
                relevant_source_refs: case.relevant_source_refs.clone(),
                results,
            });
        }

        Ok(GraphPilotBenchmarkReport {
            schema_version: SERVICE_SCHEMA_VERSION.to_string(),
            generated_at: Utc::now().to_rfc3339(),
            dataset_ref: dataset_ref.to_string(),
            summaries: summarize_modes(&case_reports),
            cases: case_reports,
        })
    }

    pub fn compare_with_037_baseline(
        benchmark: &GraphPilotBenchmarkReport,
        baseline_source_ref: &str,
        baseline: &LegacyRetrievalBenchmarkReport,
    ) -> GraphPilotComparisonReport {
        let entries = benchmark
            .summaries
            .iter()
            .map(|summary| GraphPilotComparisonEntry {
                baseline_label: CURRENT_037_BASELINE_LABEL.to_string(),
                pilot_mode: summary.retrieval_mode.clone(),
                baseline_recall_score: baseline.hybrid.recall_at_10,
                pilot_recall_score: summary.average_recall_score,
                recall_delta: summary.average_recall_score - baseline.hybrid.recall_at_10,
                baseline_latency_ms: baseline.hybrid.p50_latency_ms,
                pilot_latency_ms: summary.average_latency_ms,
                latency_comparable: false,
                citation_ready: summary.retrieval_mode != "vector_only"
                    && summary.provenance_pass_rate >= 1.0,
                beats_baseline: summary.average_recall_score >= baseline.hybrid.recall_at_10,
            })
            .collect::<Vec<_>>();

        GraphPilotComparisonReport {
            schema_version: SERVICE_SCHEMA_VERSION.to_string(),
            generated_at: Utc::now().to_rfc3339(),
            baseline_source_ref: baseline_source_ref.to_string(),
            graph_dataset_ref: benchmark.dataset_ref.clone(),
            entries,
        }
    }

    pub fn compare_with_037_shared_evaluation(
        benchmark: &GraphPilotBenchmarkReport,
        baseline: &LegacyRetrievalEvaluationReport,
        baseline_source_ref: &str,
    ) -> GraphPilotSharedEvaluationReport {
        let mut baseline_cases = BTreeMap::new();
        for case in &baseline.cases {
            baseline_cases.insert(case.case_id.clone(), case.clone());
        }

        let mut entries = Vec::new();
        for case in &benchmark.cases {
            if let Some(baseline_case) = baseline_cases.get(&case.case_id) {
                entries.push(GraphPilotSharedEvaluationEntry {
                    case_id: baseline_case.case_id.clone(),
                    query_class: baseline_case.query_class.clone(),
                    query_id: baseline_case.query_id.clone(),
                    query_text: baseline_case.query_text.clone(),
                    mode: baseline_case.mode.clone(),
                    relevant_source_refs: baseline_case.relevant_source_refs.clone(),
                    citation_source_refs: baseline_case.citation_source_refs.clone(),
                    citation_count: baseline_case.citation_count,
                    recall_score: baseline_case.recall_score,
                    latency_ms: baseline_case.latency_ms,
                    provenance_satisfied: baseline_case.provenance_satisfied,
                });
            }

            for result in &case.results {
                entries.push(GraphPilotSharedEvaluationEntry {
                    case_id: case.case_id.clone(),
                    query_class: case.query_class.clone(),
                    query_id: case.query_id.clone(),
                    query_text: case.query_text.clone(),
                    mode: result.retrieval_mode.clone(),
                    relevant_source_refs: case.relevant_source_refs.clone(),
                    citation_source_refs: result.citation_source_refs.clone(),
                    citation_count: result.citation_count,
                    recall_score: result.recall_score,
                    latency_ms: result.latency_ms,
                    provenance_satisfied: result.provenance_satisfied,
                });
            }
        }

        GraphPilotSharedEvaluationReport {
            schema_version: SERVICE_SCHEMA_VERSION.to_string(),
            generated_at: Utc::now().to_rfc3339(),
            baseline_source_ref: baseline_source_ref.to_string(),
            graph_dataset_ref: benchmark.dataset_ref.clone(),
            entries,
        }
    }
}

fn summarize_modes(case_reports: &[GraphPilotCaseReport]) -> Vec<GraphPilotModeSummary> {
    let mut grouped: BTreeMap<String, Vec<&GraphPilotModeExecution>> = BTreeMap::new();

    for case in case_reports {
        for result in &case.results {
            grouped
                .entry(result.retrieval_mode.clone())
                .or_default()
                .push(result);
        }
    }

    grouped
        .into_iter()
        .map(|(retrieval_mode, results)| {
            let cases = results.len();
            let total_recall = results.iter().map(|item| item.recall_score).sum::<f64>();
            let total_latency = results.iter().map(|item| item.latency_ms as f64).sum::<f64>();
            let provenance_passes = results
                .iter()
                .filter(|item| item.provenance_satisfied)
                .count();

            GraphPilotModeSummary {
                retrieval_mode,
                cases,
                average_recall_score: if cases == 0 {
                    0.0
                } else {
                    total_recall / cases as f64
                },
                average_latency_ms: if cases == 0 {
                    0.0
                } else {
                    total_latency / cases as f64
                },
                provenance_pass_rate: if cases == 0 {
                    0.0
                } else {
                    provenance_passes as f64 / cases as f64
                },
            }
        })
        .collect()
}

fn recall_score(citation_source_refs: &[String], relevant_source_refs: &[String]) -> f64 {
    let relevant: std::collections::BTreeSet<&str> =
        relevant_source_refs.iter().map(String::as_str).collect();
    let matched = citation_source_refs
        .iter()
        .filter(|source_ref| relevant.contains(source_ref.as_str()))
        .count();

    if relevant.is_empty() {
        0.0
    } else {
        matched as f64 / relevant.len() as f64
    }
}

fn provenance_satisfied(response: &GraphRetrievalResponse) -> bool {
    if response.graph_matches.is_empty() {
        return false;
    }

    if response
        .citations
        .iter()
        .any(|citation| citation.source_ref.trim().is_empty())
    {
        return false;
    }

    !response
        .graph_matches
        .iter()
        .any(|item| item.source_ref.trim().is_empty() || item.provenance_scope.trim().is_empty())
}

fn elapsed_ms(start: Instant) -> u64 {
    let micros = start.elapsed().as_micros();
    let rounded_up_ms = micros.div_ceil(1000);
    rounded_up_ms.max(1) as u64
}
