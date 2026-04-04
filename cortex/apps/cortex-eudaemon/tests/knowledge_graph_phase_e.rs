use cortex_eudaemon::services::knowledge_graph_query::{
    InMemoryTriple, TripleQueryRequest, TripleQueryResponse,
};
use cortex_eudaemon::services::knowledge_graph_retrieval::{
    GraphRetrievalBenchmarkCase, GraphRetrievalHarness, VectorRetrievalHit,
};
use cortex_eudaemon::services::knowledge_graph_runtime::{
    GlobalEventTripleRecord, RuntimeTripleProjector,
};
use cortex_eudaemon::services::knowledge_graph_service::{
    KnowledgeGraphService, LegacyRetrievalBenchmarkReport, LegacyRetrievalEvaluationReport,
};
use cortex_eudaemon::services::knowledge_graph_topology::{build_topology_view, ExploreTopologyView};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fs;
use std::path::Path;

fn load_json<T: DeserializeOwned>(path: &str) -> T {
    let full_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text = fs::read_to_string(&full_path).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

#[derive(Debug, Deserialize)]
struct RetrievalBenchmarkFixture {
    vector_hits: Vec<VectorRetrievalHit>,
    cases: Vec<GraphRetrievalBenchmarkCase>,
}

fn triples_from_response(response: TripleQueryResponse) -> Vec<InMemoryTriple> {
    response
        .triples
        .into_iter()
        .map(|triple| InMemoryTriple {
            subject: triple.subject,
            predicate: triple.predicate,
            object: triple.object,
            graph_scope: triple.graph_scope,
            scope_ref: response.scope.scope_ref.clone(),
            provenance_scope: triple.provenance_scope,
            source_ref: triple.source_ref,
            confidence: triple.confidence,
        })
        .collect()
}

#[test]
fn runtime_projection_matches_phase_d_triple_fixture() {
    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_global_event_substrate_v1.json");
    let expected: Vec<InMemoryTriple> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_query_substrate_v1.json");

    let projected = RuntimeTripleProjector::project(&runtime_records).expect("project triples");

    assert_eq!(projected, expected);
}

#[test]
fn hybrid_retrieval_benchmark_outperforms_vector_only_on_graph_native_case() {
    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_global_event_substrate_v1.json");
    let triples = RuntimeTripleProjector::project(&runtime_records).expect("project triples");
    let fixture: RetrievalBenchmarkFixture =
        load_json("tests/fixtures/knowledge_graph/initiative_078_retrieval_benchmark_v1.json");

    let results = GraphRetrievalHarness::benchmark(&triples, &fixture.vector_hits, &fixture.cases)
        .expect("benchmark results");

    let case = results
        .iter()
        .find(|item| item.case_id == "relation_traversal_runbook_dependency")
        .expect("graph-native relation traversal case");
    assert!(case.hybrid_graph_embedding_score > case.vector_only_score);
    assert!(case.hybrid_graph_embedding_score >= case.graph_only_score);
}

#[test]
fn derived_topology_matches_registered_fixture() {
    let triples: Vec<InMemoryTriple> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_query_substrate_v1.json");
    let expected: ExploreTopologyView = load_json(
        "../../../shared/standards/knowledge_graphs/examples/explore/research_space_topology_view_v1.json",
    );

    let actual = build_topology_view(
        "research",
        "nostra://graph/research/topology/phase-e",
        &triples,
    );

    assert_eq!(actual, expected);
}

#[test]
fn graph_service_query_matches_actor_fixture() {
    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_global_event_substrate_v1.json");
    let request: TripleQueryRequest = load_json(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/research_space_query_request_v1.json",
    );
    let expected: TripleQueryResponse = load_json(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/research_space_query_response_v1.json",
    );

    let actual =
        KnowledgeGraphService::execute_triple_query(&runtime_records, &request).expect("query");

    assert_eq!(actual, expected);
}

#[test]
fn graph_service_benchmark_summarizes_modes_and_comparison() {
    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_global_event_substrate_v1.json");
    let fixture: RetrievalBenchmarkFixture =
        load_json("tests/fixtures/knowledge_graph/initiative_078_retrieval_benchmark_v1.json");
    let baseline: LegacyRetrievalBenchmarkReport =
        load_json("tests/fixtures/knowledge_graph/legacy_037_retrieval_benchmark_latest.json");

    let benchmark = KnowledgeGraphService::benchmark_runtime_records(
        "nostra://benchmark/initiative-078/phase-f-graph-pilot",
        &runtime_records,
        &fixture.vector_hits,
        &fixture.cases,
    )
    .expect("benchmark");

    assert_eq!(benchmark.cases.len(), fixture.cases.len());
    assert_eq!(benchmark.summaries.len(), 3);

    let hybrid_summary = benchmark
        .summaries
        .iter()
        .find(|item| item.retrieval_mode == "hybrid_graph_embedding")
        .expect("hybrid summary");
    let vector_summary = benchmark
        .summaries
        .iter()
        .find(|item| item.retrieval_mode == "vector_only")
        .expect("vector summary");
    assert!(hybrid_summary.average_recall_score >= vector_summary.average_recall_score);
    assert!(hybrid_summary.provenance_pass_rate >= 1.0);

    let comparison = KnowledgeGraphService::compare_with_037_baseline(
        &benchmark,
        "nostra://logs/knowledge/retrieval_benchmark_latest.json",
        &baseline,
    );

    assert_eq!(comparison.entries.len(), 3);
    let hybrid_entry = comparison
        .entries
        .iter()
        .find(|item| item.pilot_mode == "hybrid_graph_embedding")
        .expect("hybrid comparison");
    let vector_entry = comparison
        .entries
        .iter()
        .find(|item| item.pilot_mode == "vector_only")
        .expect("vector comparison");
    assert!(hybrid_entry.citation_ready);
    assert!(!vector_entry.citation_ready);
    assert!(!hybrid_entry.latency_comparable);
    assert!(hybrid_entry.beats_baseline);
}

#[test]
fn graph_service_benchmark_covers_graph_native_case_matrix() {
    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_global_event_substrate_v1.json");
    let fixture: RetrievalBenchmarkFixture =
        load_json("tests/fixtures/knowledge_graph/initiative_078_retrieval_benchmark_v1.json");

    let benchmark = KnowledgeGraphService::benchmark_runtime_records(
        "nostra://benchmark/initiative-078/phase-f-graph-pilot",
        &runtime_records,
        &fixture.vector_hits,
        &fixture.cases,
    )
    .expect("benchmark");

    assert_eq!(benchmark.cases.len(), 4);
    assert!(
        benchmark
            .cases
            .iter()
            .any(|case| case.query_class == "graph_relation_traversal")
    );
    assert!(
        benchmark
            .cases
            .iter()
            .any(|case| case.query_class == "scope_constrained")
    );
    assert!(
        benchmark
            .cases
            .iter()
            .any(|case| case.query_class == "provenance_sensitive")
    );

    let relation_case = benchmark
        .cases
        .iter()
        .find(|case| case.case_id == "relation_traversal_runbook_dependency")
        .expect("relation traversal case");
    let relation_graph = relation_case
        .results
        .iter()
        .find(|result| result.retrieval_mode == "graph_only")
        .expect("graph only relation result");
    let relation_vector = relation_case
        .results
        .iter()
        .find(|result| result.retrieval_mode == "vector_only")
        .expect("vector only relation result");
    let relation_hybrid = relation_case
        .results
        .iter()
        .find(|result| result.retrieval_mode == "hybrid_graph_embedding")
        .expect("hybrid relation result");
    assert!(relation_hybrid.recall_score > relation_vector.recall_score);
    assert!(relation_graph.recall_score >= relation_vector.recall_score);
}

#[test]
fn graph_service_shared_eval_links_graph_pilot_to_037_cases() {
    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json("tests/fixtures/knowledge_graph/initiative_078_global_event_substrate_v1.json");
    let fixture: RetrievalBenchmarkFixture =
        load_json("tests/fixtures/knowledge_graph/initiative_078_retrieval_benchmark_v1.json");
    let baseline: LegacyRetrievalEvaluationReport = load_json(
        "tests/fixtures/knowledge_graph/legacy_037_shared_evaluation_v1.json",
    );

    let benchmark = KnowledgeGraphService::benchmark_runtime_records(
        "nostra://benchmark/initiative-078/phase-f-graph-pilot",
        &runtime_records,
        &fixture.vector_hits,
        &fixture.cases,
    )
    .expect("benchmark");

    let shared_eval = KnowledgeGraphService::compare_with_037_shared_evaluation(
        &benchmark,
        &baseline,
        "nostra://fixtures/knowledge/legacy_037_shared_evaluation_v1.json",
    );

    assert_eq!(shared_eval.entries.len(), fixture.cases.len() * 4);
    for case in &fixture.cases {
        let case_entries: Vec<_> = shared_eval
            .entries
            .iter()
            .filter(|entry| entry.case_id == case.case_id)
            .collect();
        assert_eq!(case_entries.len(), 4, "{}", case.case_id);
        assert_eq!(
            case_entries
                .iter()
                .filter(|entry| entry.mode == "037_current_hybrid_retrieval")
                .count(),
            1,
            "{}",
            case.case_id
        );
        assert_eq!(
            case_entries
                .iter()
                .filter(|entry| entry.mode != "037_current_hybrid_retrieval")
                .count(),
            3,
            "{}",
            case.case_id
        );
        for entry in case_entries {
            assert_eq!(entry.query_class, case.query_class, "{}", case.case_id);
            assert_eq!(entry.query_id, case.request.query_id, "{}", case.case_id);
            assert_eq!(entry.query_text, case.request.query_text, "{}", case.case_id);
        }
    }
}

#[test]
fn derived_agent_topology_matches_registered_fixture() {
    let response: TripleQueryResponse = load_json(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_response_v1.json",
    );
    let triples = triples_from_response(response);
    let expected: ExploreTopologyView = load_json(
        "../../../shared/standards/knowledge_graphs/examples/explore/research_agent_topology_view_v1.json",
    );

    let actual = build_topology_view(
        "research-agent",
        "nostra://graph/research/topology/agent-scope",
        &triples,
    );

    assert_eq!(actual, expected);
}
