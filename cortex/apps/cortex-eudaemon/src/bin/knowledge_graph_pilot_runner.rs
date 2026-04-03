use chrono::Utc;
use cortex_eudaemon::services::knowledge_graph_runtime::GlobalEventTripleRecord;
use cortex_eudaemon::services::knowledge_graph_service::{
    KnowledgeGraphService, LegacyRetrievalBenchmarkReport, LegacyRetrievalEvaluationReport,
};
use cortex_eudaemon::services::knowledge_graph_retrieval::{
    GraphRetrievalBenchmarkCase, VectorRetrievalHit,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct RetrievalBenchmarkFixture {
    vector_hits: Vec<VectorRetrievalHit>,
    cases: Vec<GraphRetrievalBenchmarkCase>,
}

#[derive(Debug, Serialize)]
struct OutputSummary {
    benchmark_latest: String,
    benchmark_timestamped: String,
    comparison_latest: String,
    comparison_timestamped: String,
    shared_evaluation_latest: String,
    shared_evaluation_timestamped: String,
    topology_latest: String,
    topology_timestamped: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/fixtures/knowledge_graph");
    let logs_dir = workspace_root.join("logs/knowledge");
    fs::create_dir_all(&logs_dir)?;

    let runtime_records: Vec<GlobalEventTripleRecord> =
        load_json(&fixture_dir.join("initiative_078_global_event_substrate_v1.json"))?;
    let benchmark_fixture: RetrievalBenchmarkFixture =
        load_json(&fixture_dir.join("initiative_078_retrieval_benchmark_v1.json"))?;
    let baseline: LegacyRetrievalBenchmarkReport =
        load_json(&workspace_root.join("logs/knowledge/retrieval_benchmark_latest.json"))?;
    let shared_evaluation_baseline: LegacyRetrievalEvaluationReport =
        load_json(&fixture_dir.join("legacy_037_shared_evaluation_v1.json"))?;

    let benchmark = KnowledgeGraphService::benchmark_runtime_records(
        "nostra://benchmark/initiative-078/phase-f-graph-pilot",
        &runtime_records,
        &benchmark_fixture.vector_hits,
        &benchmark_fixture.cases,
    )?;
    let comparison = KnowledgeGraphService::compare_with_037_baseline(
        &benchmark,
        "nostra://logs/knowledge/retrieval_benchmark_latest.json",
        &baseline,
    );
    let shared_evaluation = KnowledgeGraphService::compare_with_037_shared_evaluation(
        &benchmark,
        &shared_evaluation_baseline,
        "nostra://fixtures/knowledge/legacy_037_shared_evaluation_v1.json",
    );
    let topology = KnowledgeGraphService::derive_topology(
        "research",
        "nostra://graph/research/topology/phase-f",
        &runtime_records,
    )?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let benchmark_latest = logs_dir.join("graph_pilot_benchmark_latest.json");
    let benchmark_timestamped = logs_dir.join(format!("graph_pilot_benchmark_{timestamp}.json"));
    let comparison_latest = logs_dir.join("graph_pilot_comparison_latest.json");
    let comparison_timestamped = logs_dir.join(format!("graph_pilot_comparison_{timestamp}.json"));
    let shared_evaluation_latest = logs_dir.join("graph_pilot_shared_evaluation_latest.json");
    let shared_evaluation_timestamped =
        logs_dir.join(format!("graph_pilot_shared_evaluation_{timestamp}.json"));
    let topology_latest = logs_dir.join("graph_pilot_topology_latest.json");
    let topology_timestamped = logs_dir.join(format!("graph_pilot_topology_{timestamp}.json"));

    write_json(&benchmark_latest, &benchmark)?;
    write_json(&benchmark_timestamped, &benchmark)?;
    write_json(&comparison_latest, &comparison)?;
    write_json(&comparison_timestamped, &comparison)?;
    write_json(&shared_evaluation_latest, &shared_evaluation)?;
    write_json(&shared_evaluation_timestamped, &shared_evaluation)?;
    write_json(&topology_latest, &topology)?;
    write_json(&topology_timestamped, &topology)?;

    let summary = OutputSummary {
        benchmark_latest: display_path(&benchmark_latest),
        benchmark_timestamped: display_path(&benchmark_timestamped),
        comparison_latest: display_path(&comparison_latest),
        comparison_timestamped: display_path(&comparison_timestamped),
        shared_evaluation_latest: display_path(&shared_evaluation_latest),
        shared_evaluation_timestamped: display_path(&shared_evaluation_timestamped),
        topology_latest: display_path(&topology_latest),
        topology_timestamped: display_path(&topology_timestamped),
    };

    println!("{}", serde_json::to_string_pretty(&json!(summary))?);
    Ok(())
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(value) = env::var("NOSTRA_WORKSPACE_ROOT") {
        return Ok(PathBuf::from(value));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.join("../../..").canonicalize()?)
}

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), Box<dyn std::error::Error>> {
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{text}\n"))?;
    Ok(())
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}
