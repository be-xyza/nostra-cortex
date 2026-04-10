use crate::v2::types::{
    BuildStatus, InstitutionSummary, KipEntity, SiqCoverage, SiqCoverageContribution,
    SiqDependencyClosure, SiqDependencyClosureRow, SiqGateCounts, SiqGateSummary, SiqHealth,
    SiqRuleResult, SiqRunArtifact, SiqSnapshot,
};
use candid::{CandidType, Decode, Encode, Principal};
use ic_agent::Agent;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

thread_local! {
    static AGENT_CACHE: RefCell<Option<Agent>> = RefCell::new(None);
}

pub fn health_snapshot() -> BuildStatus {
    BuildStatus {
        mode: "restoration",
        source: "v2-shell",
        last_verified: "2026-02-24",
    }
}

pub fn institutions_seed() -> Vec<InstitutionSummary> {
    vec![
        InstitutionSummary {
            id: "inst_foundation",
            name: "Nostra Foundation",
            stewardship_domain: "platform-governance",
            status: "active",
        },
        InstitutionSummary {
            id: "inst_runtime",
            name: "Cortex Runtime Council",
            stewardship_domain: "execution-integrity",
            status: "active",
        },
        InstitutionSummary {
            id: "inst_research",
            name: "Research Stewardship Circle",
            stewardship_domain: "portfolio-quality",
            status: "active",
        },
    ]
}

pub fn sample_siq_snapshot() -> SiqSnapshot {
    SiqSnapshot {
        health: SiqHealth {
            status: "ok".to_string(),
            siq_log_dir: "logs/siq".to_string(),
            schema_version: "siq.health.v1".to_string(),
            runs_count: 1,
            gate_exists: true,
            projection_exists: true,
            coverage_fresh: true,
            gate_fresh: true,
        },
        gates: SiqGateSummary {
            schema_version: "siq.gate.v1".to_string(),
            mode: "softgate".to_string(),
            latest_run_id: "sample_run".to_string(),
            overall_verdict: "ready".to_string(),
            counts: SiqGateCounts { pass: 3, fail: 0 },
        },
        coverage: SiqCoverage {
            schema_version: "1.0.0".to_string(),
            generated_at: "2026-02-24T00:00:00Z".to_string(),
            integrity_set: vec!["097".to_string(), "118".to_string(), "121".to_string()],
            contributions: vec![
                SiqCoverageContribution {
                    contribution_id: "118".to_string(),
                    directory: "118-cortex-runtime-extraction".to_string(),
                    status: "active".to_string(),
                    closure_state: "open".to_string(),
                    rules: vec![
                        "siq_governance_execution_contract".to_string(),
                        "siq_host_parity_contract".to_string(),
                    ],
                },
                SiqCoverageContribution {
                    contribution_id: "121".to_string(),
                    directory: "121-cortex-memory-fs".to_string(),
                    status: "draft".to_string(),
                    closure_state: "open".to_string(),
                    rules: vec![
                        "siq_governance_execution_contract".to_string(),
                        "siq_graph_projection_contract".to_string(),
                    ],
                },
            ],
        },
        dependency_closure: SiqDependencyClosure {
            schema_version: "1.0.0".to_string(),
            generated_at: "2026-02-24T00:00:00Z".to_string(),
            integrity_set: vec!["097".to_string(), "118".to_string(), "121".to_string()],
            overall_closure_state: "ready".to_string(),
            rows: vec![
                SiqDependencyClosureRow {
                    contribution_id: "118".to_string(),
                    required_dependencies: vec!["097".to_string()],
                    satisfied_dependencies: vec!["097".to_string()],
                    missing_dependencies: vec![],
                    closure_state: "ready".to_string(),
                },
                SiqDependencyClosureRow {
                    contribution_id: "121".to_string(),
                    required_dependencies: vec!["118".to_string()],
                    satisfied_dependencies: vec!["118".to_string()],
                    missing_dependencies: vec![],
                    closure_state: "ready".to_string(),
                },
            ],
        },
        runs: vec![SiqRunArtifact {
            schema_version: "1.0.0".to_string(),
            run_id: "siq_sample_run_001".to_string(),
            generated_at: "2026-02-24T00:00:00Z".to_string(),
            mode: "softgate".to_string(),
            policy_path: "shared/standards/alignment_contracts.toml".to_string(),
            policy_version: 1,
            overall_verdict: "ready".to_string(),
            required_gates_pass: true,
            counts: SiqGateCounts { pass: 3, fail: 0 },
            failures: vec![],
            results: vec![
                SiqRuleResult {
                    id: "siq_governance_execution_contract".to_string(),
                    severity: "P0".to_string(),
                    owner: "Systems Steward".to_string(),
                    source_standard: "research/121-cortex-memory-fs/INTEGRITY_DEPENDENCIES.md"
                        .to_string(),
                    status: "pass".to_string(),
                    failures: vec![],
                    notes: vec!["sample".to_string()],
                },
                SiqRuleResult {
                    id: "siq_host_parity_contract".to_string(),
                    severity: "P0".to_string(),
                    owner: "Systems Steward".to_string(),
                    source_standard:
                        "research/123-cortex-web-architecture/SPATIAL_PLANE_DESKTOP_PARITY_SPEC.md"
                            .to_string(),
                    status: "pass".to_string(),
                    failures: vec![],
                    notes: vec!["sample".to_string()],
                },
            ],
            git_commit: "sample".to_string(),
        }],
        latest_run: Some(SiqRunArtifact {
            schema_version: "1.0.0".to_string(),
            run_id: "siq_sample_run_001".to_string(),
            generated_at: "2026-02-24T00:00:00Z".to_string(),
            mode: "softgate".to_string(),
            policy_path: "shared/standards/alignment_contracts.toml".to_string(),
            policy_version: 1,
            overall_verdict: "ready".to_string(),
            required_gates_pass: true,
            counts: SiqGateCounts { pass: 3, fail: 0 },
            failures: vec![],
            results: vec![],
            git_commit: "sample".to_string(),
        }),
    }
}

fn gateway_base_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(origin) = window.location().origin() {
                return origin;
            }
        }
    }
    "http://127.0.0.1:3003".to_string()
}

async fn fetch_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", gateway_base_url(), path);
    let response = reqwest::get(url).await.map_err(|err| err.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "gateway request failed with status {}",
            response.status()
        ));
    }
    response.json::<T>().await.map_err(|err| err.to_string())
}

fn backend_canister_principal() -> Result<Principal, String> {
    let id = option_env!("BACKEND_CANISTER_ID")
        .ok_or_else(|| "Missing BACKEND_CANISTER_ID (build-time env)".to_string())?;
    Principal::from_text(id).map_err(|err| err.to_string())
}

pub async fn create_agent() -> Agent {
    if let Some(agent) = AGENT_CACHE.with(|cell| cell.borrow().clone()) {
        return agent;
    }

    let origin = {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|window| window.location().origin().ok())
                .unwrap_or_else(|| "http://127.0.0.1:3003".to_string())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            "http://127.0.0.1:3003".to_string()
        }
    };

    let url = format!("{}/ic-api", origin);
    let agent = Agent::builder()
        .with_url(url)
        .build()
        .expect("failed to build IC agent");

    #[cfg(target_arch = "wasm32")]
    {
        let _ = agent.fetch_root_key().await;
    }

    AGENT_CACHE.with(|cell| {
        *cell.borrow_mut() = Some(agent.clone());
    });

    agent
}

pub async fn execute_kip_command(agent: &Agent, command: String) -> Result<String, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, "executeKip")
        .with_arg(Encode!(&command).map_err(|err| err.to_string())?)
        .call_and_wait()
        .await
        .map_err(|err| err.to_string())?;

    Decode!(&response, Result<String, String>).map_err(|err| err.to_string())?
}

pub async fn execute_kip_query(agent: &Agent, command: String) -> Result<String, String> {
    execute_kip_command(agent, command).await
}

pub async fn execute_kip_mutation(agent: &Agent, command: String) -> Result<String, String> {
    execute_kip_command(agent, command).await
}

async fn execute_typed_kip_mutation<T: CandidType>(
    agent: &Agent,
    method: &str,
    arg: &T,
) -> Result<Result<String, String>, String> {
    let canister_id = backend_canister_principal()?;
    let response = agent
        .update(&canister_id, method)
        .with_arg(Encode!(arg).map_err(|err| err.to_string())?)
        .call_and_wait()
        .await
        .map_err(|err| err.to_string())?;

    Decode!(&response, Result<String, String>).map_err(|err| err.to_string())
}

pub async fn create_question(
    agent: &Agent,
    space_id: &str,
    title: &str,
    body: &str,
) -> Result<String, String> {
    match execute_typed_kip_mutation(agent, "createQuestion", &(space_id, title, body)).await {
        Ok(result) => result,
        Err(_) => {
            let command = build_question_upsert_command(space_id, title, body);
            execute_kip_mutation(agent, command).await
        }
    }
}

pub async fn create_comment(
    agent: &Agent,
    space_id: &str,
    parent_id: &str,
    parent_title: &str,
    body: &str,
) -> Result<String, String> {
    match execute_typed_kip_mutation(
        agent,
        "createComment",
        &(space_id, parent_id, parent_title, body),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            let command = build_comment_upsert_command(space_id, parent_id, body, parent_title);
            execute_kip_mutation(agent, command).await
        }
    }
}

pub async fn fetch_kip_entities(agent: &Agent, command: String) -> Result<Vec<KipEntity>, String> {
    let json = execute_kip_query(agent, command).await?;
    let payload: KipSearchPayload = serde_json::from_str(&json).map_err(|err| err.to_string())?;
    Ok(payload.results)
}

pub fn escape_kip_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

pub fn build_kip_upsert_command(
    entity_type: &str,
    name: &str,
    description: &str,
    props: &[(&str, &str)],
) -> String {
    let mut parts = vec![
        format!("@type: \"{}\"", entity_type),
        format!("name: \"{}\"", escape_kip_value(name)),
        format!("description: \"{}\"", escape_kip_value(description)),
    ];

    for &(key, value) in props {
        parts.push(format!("prop:{}: \"{}\"", key, escape_kip_value(value)));
    }

    format!("UPSERT {{ {} }}", parts.join(", "))
}

pub fn build_kip_find_by_space_and_type(entity_type: &str, space_id: &str) -> String {
    format!(
        "FIND {{ @type: \"{}\", prop:space_id: \"{}\" }}",
        entity_type,
        escape_kip_value(space_id)
    )
}

pub fn build_question_upsert_command(space_id: &str, title: &str, body: &str) -> String {
    build_kip_upsert_command(
        "Question",
        title,
        body,
        &[("space_id", space_id), ("title", title), ("status", "Open")],
    )
}

pub fn build_comment_upsert_command(
    space_id: &str,
    parent_id: &str,
    body: &str,
    question_name: &str,
) -> String {
    let comment_name = format!("Comment on {}", question_name);
    build_kip_upsert_command(
        "Comment",
        &comment_name,
        body,
        &[
            ("space_id", space_id),
            ("parent_id", parent_id),
            ("title", &comment_name),
            ("status", "Open"),
        ],
    )
}

pub fn build_comment_query(space_id: &str, parent_id: &str) -> String {
    format!(
        "FIND {{ @type: \"Comment\", prop:space_id: \"{}\", prop:parent_id: \"{}\" }}",
        escape_kip_value(space_id),
        escape_kip_value(parent_id)
    )
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct KipSearchPayload {
    results: Vec<KipEntity>,
}

pub async fn fetch_siq_health() -> Result<SiqHealth, String> {
    fetch_json("/api/system/siq/health").await
}

pub async fn fetch_siq_gates_latest() -> Result<SiqGateSummary, String> {
    fetch_json("/api/system/siq/gates/latest").await
}

pub async fn fetch_siq_coverage() -> Result<SiqCoverage, String> {
    fetch_json("/api/system/siq/coverage").await
}

pub async fn fetch_siq_dependency_closure() -> Result<SiqDependencyClosure, String> {
    fetch_json("/api/system/siq/dependency-closure").await
}

pub async fn fetch_siq_runs(limit: usize) -> Result<Vec<SiqRunArtifact>, String> {
    fetch_json(&format!("/api/system/siq/runs?limit={}", limit.min(100))).await
}

pub async fn fetch_siq_run(run_id: &str) -> Result<SiqRunArtifact, String> {
    fetch_json(&format!("/api/system/siq/runs/{}", run_id)).await
}

pub async fn fetch_siq_snapshot() -> Result<SiqSnapshot, String> {
    let health = fetch_siq_health().await?;
    let gates = fetch_siq_gates_latest().await?;
    let coverage = fetch_siq_coverage().await?;
    let dependency_closure = fetch_siq_dependency_closure().await?;
    let runs = fetch_siq_runs(8).await?;
    let latest_run = if let Some(first) = runs.first() {
        fetch_siq_run(&first.run_id).await.ok()
    } else {
        None
    };
    Ok(SiqSnapshot {
        health,
        gates,
        coverage,
        dependency_closure,
        runs,
        latest_run,
    })
}

#[cfg(test)]
mod kip_tests {
    use super::*;

    #[test]
    fn question_upsert_encodes_question_metadata() {
        let command = build_question_upsert_command("space_alpha", "Should we ship?", "Why now?");

        assert!(command.starts_with("UPSERT {"));
        assert!(command.contains("@type: \"Question\""));
        assert!(command.contains("name: \"Should we ship?\""));
        assert!(command.contains("prop:space_id: \"space_alpha\""));
        assert!(command.contains("prop:title: \"Should we ship?\""));
        assert!(command.contains("prop:status: \"Open\""));
    }

    #[test]
    fn comment_query_uses_parent_id_and_space_id() {
        let query = build_comment_query("space_alpha", "question_1");

        assert_eq!(
            query,
            "FIND { @type: \"Comment\", prop:space_id: \"space_alpha\", prop:parent_id: \"question_1\" }"
        );
    }

    #[test]
    fn comment_upsert_uses_selected_question_as_parent() {
        let command = build_comment_upsert_command(
            "space_alpha",
            "question_1",
            "I agree with this direction.",
            "Should we ship?",
        );

        assert!(command.contains("@type: \"Comment\""));
        assert!(command.contains("Comment on Should we ship?"));
        assert!(command.contains("prop:parent_id: \"question_1\""));
        assert!(command.contains("prop:title: \"Comment on Should we ship?\""));
    }

    #[test]
    fn find_builder_scopes_by_space() {
        let query = build_kip_find_by_space_and_type("Question", "space_alpha");

        assert_eq!(
            query,
            "FIND { @type: \"Question\", prop:space_id: \"space_alpha\" }"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn institutions_seed_has_unique_ids() {
        let institutions = institutions_seed();
        let mut ids = HashSet::new();
        for item in &institutions {
            assert!(ids.insert(item.id), "duplicate institution id: {}", item.id);
        }
    }

    #[test]
    fn siq_health_contract_deserializes() {
        let payload = r#"{
          "status": "ok",
          "siqLogDir": "/tmp/siq",
          "schemaVersion": "siq.health.v1",
          "runsCount": 9,
          "gateExists": true,
          "projectionExists": true,
          "coverageFresh": true,
          "gateFresh": true
        }"#;
        let parsed: SiqHealth = serde_json::from_str(payload).expect("parse siq health");
        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.runs_count, 9);
    }

    #[test]
    fn siq_gate_summary_contract_deserializes() {
        let payload = r#"{
          "schema_version": "siq.gate.v1",
          "mode": "softgate",
          "latest_run_id": "siq_20260224T090140632743Z",
          "overall_verdict": "ready",
          "counts": {"pass": 3, "fail": 0}
        }"#;
        let parsed: SiqGateSummary = serde_json::from_str(payload).expect("parse siq gate summary");
        assert_eq!(parsed.overall_verdict, "ready");
        assert_eq!(parsed.counts.pass, 3);
    }

    #[test]
    fn siq_coverage_contract_deserializes() {
        let payload = r#"{
          "schema_version": "1.0.0",
          "generated_at": "2026-02-24T10:00:00Z",
          "integrity_set": ["097", "121"],
          "contributions": [
            {
              "contribution_id": "121",
              "directory": "121-cortex-memory-fs",
              "status": "draft",
              "closure_state": "open",
              "rules": ["siq_governance_execution_contract"]
            }
          ]
        }"#;
        let parsed: SiqCoverage = serde_json::from_str(payload).expect("parse siq coverage");
        assert_eq!(parsed.contributions.len(), 1);
        assert_eq!(parsed.contributions[0].contribution_id, "121");
    }

    #[test]
    fn siq_dependency_closure_contract_deserializes() {
        let payload = r#"{
          "schema_version": "1.0.0",
          "generated_at": "2026-02-24T10:00:00Z",
          "integrity_set": ["097", "121"],
          "overall_closure_state": "ready",
          "rows": [
            {
              "contribution_id": "121",
              "required_dependencies": ["097"],
              "satisfied_dependencies": ["097"],
              "missing_dependencies": [],
              "closure_state": "ready"
            }
          ]
        }"#;
        let parsed: SiqDependencyClosure =
            serde_json::from_str(payload).expect("parse siq dependency closure");
        assert_eq!(parsed.overall_closure_state, "ready");
        assert_eq!(parsed.rows[0].contribution_id, "121");
    }

    #[test]
    fn siq_run_artifact_contract_deserializes() {
        let payload = r#"{
          "schema_version": "1.0.0",
          "run_id": "siq_20260224T102036225037Z",
          "generated_at": "2026-02-24T10:20:36Z",
          "mode": "softgate",
          "policy_path": "shared/standards/alignment_contracts.toml",
          "policy_version": 1,
          "overall_verdict": "ready",
          "required_gates_pass": true,
          "counts": {"pass": 3, "fail": 0},
          "failures": [],
          "results": [{
            "id": "siq_governance_execution_contract",
            "severity": "P0",
            "owner": "Systems Steward",
            "source_standard": "research/121-cortex-memory-fs/INTEGRITY_DEPENDENCIES.md",
            "status": "pass",
            "failures": [],
            "notes": ["ok"]
          }],
          "git_commit": "eda8a1b5"
        }"#;
        let parsed: SiqRunArtifact = serde_json::from_str(payload).expect("parse siq run artifact");
        assert_eq!(parsed.run_id, "siq_20260224T102036225037Z");
        assert_eq!(parsed.results.len(), 1);
    }
}
