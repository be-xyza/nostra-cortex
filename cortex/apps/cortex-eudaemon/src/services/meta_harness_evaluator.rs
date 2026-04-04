use crate::services::meta_harness_store::HarnessExperienceStore;
use cortex_domain::agent::contracts::{
    AgentBenchmarkRecord, BenchmarkProjectionV1, EnvironmentBootstrapV1, HarnessCandidateMode,
    HarnessCandidateV1, HarnessEvaluationV1, HarnessEvaluationVerdict, HarnessRunStatus,
    HarnessRunV1,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessEvaluationRequest {
    pub evaluation_id: String,
    pub candidate_id: String,
    pub held_out_pack_id: String,
    #[serde(default)]
    pub execution_refs: Vec<String>,
    #[serde(default)]
    pub benchmark_records: Vec<AgentBenchmarkRecord>,
    #[serde(default)]
    pub evaluator_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordHarnessExperimentRequest {
    pub candidate: HarnessCandidateV1,
    pub bootstrap: EnvironmentBootstrapV1,
    pub run: HarnessRunV1,
    pub evaluation: HarnessEvaluationRequest,
}

pub fn build_environment_bootstrap(
    bootstrap_id: &str,
    cwd: &Path,
    provider_profile: Option<&str>,
    toolchain: &[&str],
    path_hints: &[&str],
    constraints: &[&str],
    prompt_override_verified: bool,
) -> EnvironmentBootstrapV1 {
    let provider_profile_owned = provider_profile.map(str::to_string);
    let toolchain_owned = toolchain
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let path_hints_owned = path_hints
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let constraints_owned = constraints
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();

    EnvironmentBootstrapV1 {
        schema_version: "1.0.0".to_string(),
        bootstrap_id: bootstrap_id.to_string(),
        captured_at: "2026-04-03T00:00:00Z".to_string(),
        cwd: cwd.display().to_string(),
        workspace_root: Some(
            crate::gateway::server::workspace_root()
                .display()
                .to_string(),
        ),
        provider_profile: provider_profile_owned.clone(),
        toolchain: toolchain_owned.clone(),
        path_hints: path_hints_owned.clone(),
        constraints: constraints_owned.clone(),
        summary: format!(
            "cwd={} provider={} tools={} constraints={}",
            cwd.display(),
            provider_profile_owned.unwrap_or_else(|| "default".to_string()),
            toolchain_owned.join(","),
            constraints_owned.join(",")
        ),
        prompt_override_verified,
    }
}

pub async fn evaluate_harness_candidate(
    request: &HarnessEvaluationRequest,
    store: &HarnessExperienceStore,
) -> Result<HarnessEvaluationV1, String> {
    let candidate = store
        .get_candidate(&request.candidate_id)?
        .ok_or_else(|| format!("unknown candidate '{}'", request.candidate_id))?;

    if candidate.mode != HarnessCandidateMode::RecommendationOnly
        || !candidate.search_space.recommendation_only
    {
        return Err("meta_harness_candidates_must_be_recommendation_only".to_string());
    }
    if candidate.search_space.prompt_variant_search_enabled {
        return Err("prompt_variant_search_requires_verified_prompt_override".to_string());
    }

    let aggregate = aggregate_benchmarks(&request.benchmark_records)?;
    Ok(HarnessEvaluationV1 {
        schema_version: "1.0.0".to_string(),
        evaluation_id: request.evaluation_id.clone(),
        candidate_id: request.candidate_id.clone(),
        held_out_pack_id: request.held_out_pack_id.clone(),
        verdict: HarnessEvaluationVerdict::Hold,
        rank: 0,
        total_score: aggregate.score,
        latency_ms: aggregate.projection.latency_ms,
        token_cost: aggregate.projection.token_cost,
        summary: format!(
            "Held-out pack {} produced {}.",
            request.held_out_pack_id, aggregate.projection.summary
        ),
        benchmark_projection: aggregate.projection,
        execution_refs: request.execution_refs.clone(),
        evaluator_ref: request.evaluator_ref.clone(),
        recommendation_only: true,
    })
}

pub async fn record_harness_experiment(
    store: &HarnessExperienceStore,
    request: &RecordHarnessExperimentRequest,
) -> Result<HarnessEvaluationV1, String> {
    if request.candidate.candidate_id != request.run.candidate_id
        || request.candidate.candidate_id != request.evaluation.candidate_id
    {
        return Err("harness_candidate_link_mismatch".to_string());
    }
    if request.run.status != HarnessRunStatus::Recorded
        && request.run.status != HarnessRunStatus::Evaluated
    {
        return Err("unsupported_harness_run_status".to_string());
    }

    store.put_candidate(&request.candidate)?;
    store.put_bootstrap(&request.bootstrap)?;

    let mut run = request.run.clone();
    if run.benchmark_projection.is_none() {
        run.benchmark_projection = run
            .benchmark
            .as_ref()
            .map(AgentBenchmarkRecord::to_projection_v1);
    }
    store.put_run(&run)?;

    let evaluation = evaluate_harness_candidate(&request.evaluation, store).await?;
    store.put_evaluation(&evaluation)?;
    Ok(evaluation)
}

pub fn rank_harness_evaluations(evaluations: &[HarnessEvaluationV1]) -> Vec<HarnessEvaluationV1> {
    let mut ranked = evaluations.to_vec();
    ranked.sort_by(|left, right| {
        right
            .total_score
            .partial_cmp(&left.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.latency_ms.cmp(&right.latency_ms))
            .then_with(|| left.token_cost.total_cmp(&right.token_cost))
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });

    for (index, evaluation) in ranked.iter_mut().enumerate() {
        evaluation.rank = (index + 1) as u32;
        evaluation.verdict = match index {
            0 => HarnessEvaluationVerdict::Winner,
            1 => HarnessEvaluationVerdict::RunnerUp,
            _ if evaluation.total_score >= 0.80 => HarnessEvaluationVerdict::Hold,
            _ => HarnessEvaluationVerdict::Rejected,
        };
        evaluation.recommendation_only = true;
    }

    ranked
}

struct BenchmarkAggregate {
    projection: BenchmarkProjectionV1,
    score: f64,
}

fn aggregate_benchmarks(records: &[AgentBenchmarkRecord]) -> Result<BenchmarkAggregate, String> {
    if records.is_empty() {
        return Err("at_least_one_benchmark_record_is_required".to_string());
    }

    let count = records.len() as f64;
    let pass_rate = records.iter().map(|record| record.pass_rate).sum::<f64>() / count;
    let latency_ms = (records
        .iter()
        .map(|record| record.latency_ms as f64)
        .sum::<f64>()
        / count)
        .round() as u64;
    let total_tokens = (records
        .iter()
        .map(|record| record.total_tokens as f64)
        .sum::<f64>()
        / count)
        .round() as u64;
    let assertions_passed = records
        .iter()
        .map(|record| record.assertions_passed)
        .sum::<usize>();
    let assertions_total = records
        .iter()
        .map(|record| record.assertions_total)
        .sum::<usize>();

    let projection = AgentBenchmarkRecord {
        pass_rate,
        latency_ms,
        total_tokens,
        assertions_passed,
        assertions_total,
        assertion_details: Vec::new(),
    }
    .to_projection_v1();

    let latency_penalty = (latency_ms as f64 / 10_000.0).min(0.2);
    let token_penalty = (total_tokens as f64 / 100_000.0).min(0.2);
    let score = (pass_rate - latency_penalty - token_penalty).clamp(0.0, 1.0);

    Ok(BenchmarkAggregate { projection, score })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::meta_harness_store::HarnessExperienceStore;
    use cortex_domain::agent::contracts::{
        HarnessCandidateMode, HarnessRunStatus, HarnessSearchSpaceV1,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn benchmark(pass_rate: f64, latency_ms: u64, total_tokens: u64) -> AgentBenchmarkRecord {
        AgentBenchmarkRecord {
            pass_rate,
            latency_ms,
            total_tokens,
            assertions_passed: (pass_rate * 10.0).round() as usize,
            assertions_total: 10,
            assertion_details: Vec::new(),
        }
    }

    fn candidate(candidate_id: &str, provider_profile: &str) -> HarnessCandidateV1 {
        HarnessCandidateV1 {
            schema_version: "1.0.0".to_string(),
            candidate_id: candidate_id.to_string(),
            created_at: "2026-04-03T00:00:00Z".to_string(),
            parent_candidate_id: None,
            mode: HarnessCandidateMode::RecommendationOnly,
            search_space: HarnessSearchSpaceV1::phase6_safe(),
            changed_knobs: BTreeMap::from([(
                "provider_profile".to_string(),
                json!(provider_profile),
            )]),
            provenance_refs: vec!["nostra://proposal/1".to_string()],
            workflow_snapshot_ref: Some("nostra://workflow/snapshots/wf-1".to_string()),
            heap_artifact_refs: vec!["nostra://heap/artifact-1".to_string()],
            replay_refs: vec!["nostra://replay/1".to_string()],
        }
    }

    fn run(run_id: &str, candidate_id: &str, benchmark: AgentBenchmarkRecord) -> HarnessRunV1 {
        HarnessRunV1 {
            schema_version: "1.0.0".to_string(),
            run_id: run_id.to_string(),
            candidate_id: candidate_id.to_string(),
            status: HarnessRunStatus::Recorded,
            started_at: "2026-04-03T00:00:00Z".to_string(),
            finished_at: Some("2026-04-03T00:01:00Z".to_string()),
            execution_ref: format!("nostra://workflow/wf-1/execution/{run_id}"),
            workflow_snapshot_ref: Some("nostra://workflow/snapshots/wf-1".to_string()),
            heap_artifact_refs: vec!["nostra://heap/artifact-1".to_string()],
            replay_refs: vec!["nostra://replay/1".to_string()],
            benchmark_projection: None,
            benchmark: Some(benchmark),
            notes: vec!["recommendation_only".to_string()],
        }
    }

    fn request(
        evaluation_id: &str,
        candidate_id: &str,
        records: Vec<AgentBenchmarkRecord>,
    ) -> HarnessEvaluationRequest {
        HarnessEvaluationRequest {
            evaluation_id: evaluation_id.to_string(),
            candidate_id: candidate_id.to_string(),
            held_out_pack_id: "pack-alpha".to_string(),
            execution_refs: vec![format!("nostra://workflow/wf-1/execution/{candidate_id}")],
            benchmark_records: records,
            evaluator_ref: Some("nostra://evaluator/meta-harness".to_string()),
        }
    }

    fn temp_store() -> HarnessExperienceStore {
        HarnessExperienceStore::new(
            PathBuf::from(std::env::temp_dir())
                .join(format!("meta_harness_eval_{}", uuid::Uuid::new_v4())),
        )
    }

    #[tokio::test]
    async fn record_harness_experiment_persists_end_to_end() {
        let store = temp_store();
        let candidate = candidate("candidate-alpha", "balanced");
        let bootstrap = build_environment_bootstrap(
            "bootstrap-alpha",
            Path::new("/Users/xaoj/ICP"),
            Some("balanced"),
            &["cargo", "node"],
            &["cortex/apps/cortex-eudaemon"],
            &["recommendation_only"],
            false,
        );
        let run = run("run-alpha", "candidate-alpha", benchmark(0.95, 1200, 2400));
        let evaluation = record_harness_experiment(
            &store,
            &RecordHarnessExperimentRequest {
                candidate: candidate.clone(),
                bootstrap: bootstrap.clone(),
                run,
                evaluation: request(
                    "evaluation-alpha",
                    "candidate-alpha",
                    vec![benchmark(0.95, 1200, 2400), benchmark(0.90, 1300, 2500)],
                ),
            },
        )
        .await
        .unwrap();

        assert_eq!(evaluation.candidate_id, "candidate-alpha");
        assert!(evaluation.recommendation_only);
        assert_eq!(
            store.get_candidate("candidate-alpha").unwrap().unwrap(),
            candidate
        );
        assert_eq!(
            store.get_bootstrap("bootstrap-alpha").unwrap().unwrap(),
            bootstrap
        );
        let stored_run = store.get_run("run-alpha").unwrap().unwrap();
        assert!(stored_run.benchmark_projection.is_some());
        assert_eq!(
            store
                .get_evaluation("evaluation-alpha")
                .unwrap()
                .unwrap()
                .evaluation_id,
            "evaluation-alpha"
        );
    }

    #[test]
    fn ranking_prefers_score_then_latency() {
        let ranked = rank_harness_evaluations(&[
            HarnessEvaluationV1 {
                schema_version: "1.0.0".to_string(),
                evaluation_id: "eval-1".to_string(),
                candidate_id: "candidate-a".to_string(),
                held_out_pack_id: "pack-alpha".to_string(),
                verdict: HarnessEvaluationVerdict::Hold,
                rank: 0,
                total_score: 0.93,
                latency_ms: 900,
                token_cost: 2000.0,
                summary: "a".to_string(),
                benchmark_projection: benchmark(0.95, 900, 2000).to_projection_v1(),
                execution_refs: vec![],
                evaluator_ref: None,
                recommendation_only: true,
            },
            HarnessEvaluationV1 {
                schema_version: "1.0.0".to_string(),
                evaluation_id: "eval-2".to_string(),
                candidate_id: "candidate-b".to_string(),
                held_out_pack_id: "pack-alpha".to_string(),
                verdict: HarnessEvaluationVerdict::Hold,
                rank: 0,
                total_score: 0.88,
                latency_ms: 700,
                token_cost: 1800.0,
                summary: "b".to_string(),
                benchmark_projection: benchmark(0.91, 700, 1800).to_projection_v1(),
                execution_refs: vec![],
                evaluator_ref: None,
                recommendation_only: true,
            },
            HarnessEvaluationV1 {
                schema_version: "1.0.0".to_string(),
                evaluation_id: "eval-3".to_string(),
                candidate_id: "candidate-c".to_string(),
                held_out_pack_id: "pack-alpha".to_string(),
                verdict: HarnessEvaluationVerdict::Hold,
                rank: 0,
                total_score: 0.62,
                latency_ms: 1600,
                token_cost: 2800.0,
                summary: "c".to_string(),
                benchmark_projection: benchmark(0.7, 1600, 2800).to_projection_v1(),
                execution_refs: vec![],
                evaluator_ref: None,
                recommendation_only: true,
            },
        ]);

        assert_eq!(ranked[0].candidate_id, "candidate-a");
        assert_eq!(ranked[0].verdict, HarnessEvaluationVerdict::Winner);
        assert_eq!(ranked[1].verdict, HarnessEvaluationVerdict::RunnerUp);
        assert_eq!(ranked[2].verdict, HarnessEvaluationVerdict::Rejected);
        assert!(ranked.iter().all(|entry| entry.recommendation_only));
    }

    #[test]
    fn environment_bootstrap_marks_prompt_override_as_unverified_by_default() {
        let bootstrap = build_environment_bootstrap(
            "bootstrap-test",
            Path::new("/Users/xaoj/ICP"),
            Some("balanced"),
            &["cargo", "node"],
            &["cortex/apps/cortex-eudaemon"],
            &["recommendation_only"],
            false,
        );

        assert!(!bootstrap.prompt_override_verified);
        assert!(bootstrap.summary.contains("provider=balanced"));
        assert!(bootstrap.summary.contains("tools=cargo,node"));
    }
}
