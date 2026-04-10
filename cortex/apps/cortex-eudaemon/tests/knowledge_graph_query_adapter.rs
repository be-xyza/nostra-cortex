use cortex_eudaemon::services::knowledge_graph_query::{
    InMemoryTriple, TripleQueryAdapter, TripleQueryRequest, TripleQueryResponse,
};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

fn load_json<T: DeserializeOwned>(path: &str) -> T {
    let full_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text = fs::read_to_string(&full_path).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

fn fixture_dataset() -> Vec<InMemoryTriple> {
    load_json("tests/fixtures/knowledge_graph/initiative_078_query_substrate_v1.json")
}

fn assert_fixture_pair(request_path: &str, response_path: &str) {
    let request: TripleQueryRequest = load_json(request_path);
    let expected: TripleQueryResponse = load_json(response_path);
    let actual = TripleQueryAdapter::execute(&fixture_dataset(), &request).expect("query result");
    assert_eq!(actual, expected);
}

#[test]
fn adapter_matches_actor_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/research_space_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/research_space_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_system_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/system_scope_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/system_scope_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_agent_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/agent_scope_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_any_scope_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/any_scope_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/any_scope_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_zero_result_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/zero_result_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/zero_result_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_provenance_disabled_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/provenance_disabled_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/provenance_disabled_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_scope_isolation_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/scope_isolation_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/scope_isolation_query_response_v1.json",
    );
}

#[test]
fn adapter_matches_multi_hop_planning_fixture() {
    assert_fixture_pair(
        "../../../shared/standards/knowledge_graphs/examples/triple_query/multi_hop_planning_query_request_v1.json",
        "../../../shared/standards/knowledge_graphs/examples/triple_query/multi_hop_planning_query_response_v1.json",
    );
}
