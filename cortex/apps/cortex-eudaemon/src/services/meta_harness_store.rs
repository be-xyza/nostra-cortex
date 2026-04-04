use crate::gateway::server::workspace_root;
use cortex_domain::agent::contracts::{
    EnvironmentBootstrapV1, HarnessCandidateV1, HarnessEvaluationV1, HarnessRunV1,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessKnobDiff {
    pub left: Option<Value>,
    pub right: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct HarnessExperienceStore {
    root_dir: PathBuf,
}

impl HarnessExperienceStore {
    pub fn from_env() -> Self {
        Self::new(resolve_meta_harness_log_dir())
    }

    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn put_candidate(&self, candidate: &HarnessCandidateV1) -> Result<(), String> {
        write_json(
            &self.root_dir.join("candidates").join(format!(
                "{}.json",
                sanitize_fs_component(&candidate.candidate_id)
            )),
            candidate,
        )
    }

    pub fn get_candidate(&self, candidate_id: &str) -> Result<Option<HarnessCandidateV1>, String> {
        read_json(
            &self
                .root_dir
                .join("candidates")
                .join(format!("{}.json", sanitize_fs_component(candidate_id))),
        )
    }

    pub fn list_candidates(&self) -> Result<Vec<HarnessCandidateV1>, String> {
        let mut candidates =
            list_json_dir::<HarnessCandidateV1>(&self.root_dir.join("candidates"))?;
        candidates.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.candidate_id.cmp(&a.candidate_id))
        });
        Ok(candidates)
    }

    pub fn put_run(&self, run: &HarnessRunV1) -> Result<(), String> {
        write_json(
            &self
                .root_dir
                .join("runs")
                .join(format!("{}.json", sanitize_fs_component(&run.run_id))),
            run,
        )
    }

    pub fn get_run(&self, run_id: &str) -> Result<Option<HarnessRunV1>, String> {
        read_json(
            &self
                .root_dir
                .join("runs")
                .join(format!("{}.json", sanitize_fs_component(run_id))),
        )
    }

    pub fn latest_runs(&self, limit: usize) -> Result<Vec<HarnessRunV1>, String> {
        let mut runs = list_json_dir::<HarnessRunV1>(&self.root_dir.join("runs"))?;
        runs.sort_by(|a, b| {
            b.started_at
                .cmp(&a.started_at)
                .then_with(|| b.run_id.cmp(&a.run_id))
        });
        runs.truncate(limit.clamp(1, 200));
        Ok(runs)
    }

    pub fn put_evaluation(&self, evaluation: &HarnessEvaluationV1) -> Result<(), String> {
        write_json(
            &self.root_dir.join("evaluations").join(format!(
                "{}.json",
                sanitize_fs_component(&evaluation.evaluation_id)
            )),
            evaluation,
        )
    }

    pub fn get_evaluation(
        &self,
        evaluation_id: &str,
    ) -> Result<Option<HarnessEvaluationV1>, String> {
        read_json(
            &self
                .root_dir
                .join("evaluations")
                .join(format!("{}.json", sanitize_fs_component(evaluation_id))),
        )
    }

    pub fn list_evaluations(&self) -> Result<Vec<HarnessEvaluationV1>, String> {
        let mut evaluations =
            list_json_dir::<HarnessEvaluationV1>(&self.root_dir.join("evaluations"))?;
        evaluations.sort_by(|a, b| {
            b.total_score
                .partial_cmp(&a.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.latency_ms.cmp(&b.latency_ms))
                .then_with(|| a.evaluation_id.cmp(&b.evaluation_id))
        });
        Ok(evaluations)
    }

    pub fn best_candidate_by_evaluator(
        &self,
        held_out_pack_id: &str,
    ) -> Result<Option<HarnessEvaluationV1>, String> {
        let mut evaluations = self
            .list_evaluations()?
            .into_iter()
            .filter(|evaluation| evaluation.held_out_pack_id == held_out_pack_id)
            .collect::<Vec<_>>();
        evaluations.sort_by(|a, b| {
            b.total_score
                .partial_cmp(&a.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.latency_ms.cmp(&b.latency_ms))
                .then_with(|| a.candidate_id.cmp(&b.candidate_id))
        });
        Ok(evaluations.into_iter().next())
    }

    pub fn put_bootstrap(&self, bootstrap: &EnvironmentBootstrapV1) -> Result<(), String> {
        write_json(
            &self.root_dir.join("bootstrap").join(format!(
                "{}.json",
                sanitize_fs_component(&bootstrap.bootstrap_id)
            )),
            bootstrap,
        )
    }

    pub fn get_bootstrap(
        &self,
        bootstrap_id: &str,
    ) -> Result<Option<EnvironmentBootstrapV1>, String> {
        read_json(
            &self
                .root_dir
                .join("bootstrap")
                .join(format!("{}.json", sanitize_fs_component(bootstrap_id))),
        )
    }

    pub fn diff_candidates(
        &self,
        left_id: &str,
        right_id: &str,
    ) -> Result<BTreeMap<String, HarnessKnobDiff>, String> {
        let left = self
            .get_candidate(left_id)?
            .ok_or_else(|| format!("unknown candidate '{left_id}'"))?;
        let right = self
            .get_candidate(right_id)?
            .ok_or_else(|| format!("unknown candidate '{right_id}'"))?;

        let all_keys = left
            .changed_knobs
            .keys()
            .chain(right.changed_knobs.keys())
            .cloned()
            .collect::<BTreeSet<_>>();

        let mut diff = BTreeMap::new();
        for key in all_keys {
            let left_value = left.changed_knobs.get(&key).cloned();
            let right_value = right.changed_knobs.get(&key).cloned();
            if left_value != right_value {
                diff.insert(
                    key,
                    HarnessKnobDiff {
                        left: left_value,
                        right: right_value,
                    },
                );
            }
        }
        Ok(diff)
    }
}

pub fn resolve_meta_harness_log_dir() -> PathBuf {
    std::env::var("NOSTRA_META_HARNESS_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root()
                .join("logs")
                .join("cortex")
                .join("meta_harness")
        })
}

fn sanitize_fs_component(raw: &str) -> String {
    raw.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let encoded = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(path, encoded).map_err(|err| err.to_string())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str::<T>(&raw)
        .map(Some)
        .map_err(|err| err.to_string())
}

fn list_json_dir<T: DeserializeOwned>(dir: &Path) -> Result<Vec<T>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut values = Vec::new();
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(_) => continue,
        };
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        match serde_json::from_str::<T>(&raw) {
            Ok(value) => values.push(value),
            Err(_) => continue,
        }
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::agent::contracts::{
        AgentBenchmarkRecord, BenchmarkProjectionV1, HarnessCandidateMode,
        HarnessEvaluationVerdict, HarnessRunStatus, HarnessSearchSpaceV1,
    };
    use serde_json::json;

    fn temp_store() -> HarnessExperienceStore {
        let root =
            std::env::temp_dir().join(format!("meta_harness_store_{}", uuid::Uuid::new_v4()));
        HarnessExperienceStore::new(root)
    }

    fn candidate(candidate_id: &str, knob_value: &str) -> HarnessCandidateV1 {
        HarnessCandidateV1 {
            schema_version: "1.0.0".to_string(),
            candidate_id: candidate_id.to_string(),
            created_at: format!(
                "2026-04-03T00:00:0{}Z",
                &candidate_id[candidate_id.len() - 1..]
            ),
            parent_candidate_id: None,
            mode: HarnessCandidateMode::RecommendationOnly,
            search_space: HarnessSearchSpaceV1::phase6_safe(),
            changed_knobs: BTreeMap::from([("provider_profile".to_string(), json!(knob_value))]),
            provenance_refs: vec!["nostra://proposal/alpha".to_string()],
            workflow_snapshot_ref: Some("nostra://workflow/snapshots/wf-1".to_string()),
            heap_artifact_refs: vec!["nostra://heap/artifact-1".to_string()],
            replay_refs: vec!["nostra://replay/1".to_string()],
        }
    }

    fn run(run_id: &str, candidate_id: &str) -> HarnessRunV1 {
        let benchmark = AgentBenchmarkRecord {
            pass_rate: 0.95,
            latency_ms: 1200,
            total_tokens: 2400,
            assertions_passed: 19,
            assertions_total: 20,
            assertion_details: Vec::new(),
        };
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
            benchmark_projection: Some(benchmark.to_projection_v1()),
            benchmark: Some(benchmark),
            notes: vec!["recommendation_only".to_string()],
        }
    }

    fn evaluation(
        evaluation_id: &str,
        candidate_id: &str,
        total_score: f64,
    ) -> HarnessEvaluationV1 {
        HarnessEvaluationV1 {
            schema_version: "1.0.0".to_string(),
            evaluation_id: evaluation_id.to_string(),
            candidate_id: candidate_id.to_string(),
            held_out_pack_id: "pack-alpha".to_string(),
            verdict: HarnessEvaluationVerdict::Hold,
            rank: 0,
            total_score,
            latency_ms: 1000,
            token_cost: 2200.0,
            summary: "held-out ranking".to_string(),
            benchmark_projection: BenchmarkProjectionV1 {
                grade: "PASS".to_string(),
                latency_ms: 1000,
                token_cost: 2200.0,
                summary: "95% pass rate".to_string(),
                assertions_passed: 19,
                assertions_total: 20,
            },
            execution_refs: vec!["nostra://workflow/wf-1/execution/run-1".to_string()],
            evaluator_ref: Some("nostra://evaluator/meta-harness".to_string()),
            recommendation_only: true,
        }
    }

    fn bootstrap(bootstrap_id: &str) -> EnvironmentBootstrapV1 {
        EnvironmentBootstrapV1 {
            schema_version: "1.0.0".to_string(),
            bootstrap_id: bootstrap_id.to_string(),
            captured_at: "2026-04-03T00:00:00Z".to_string(),
            cwd: "/Users/xaoj/ICP".to_string(),
            workspace_root: Some("/Users/xaoj/ICP".to_string()),
            provider_profile: Some("codex_subscription".to_string()),
            toolchain: vec!["cargo".to_string(), "node".to_string()],
            path_hints: vec!["cortex/apps/cortex-eudaemon".to_string()],
            constraints: vec!["recommendation_only".to_string()],
            summary: "cargo,node available".to_string(),
            prompt_override_verified: false,
        }
    }

    #[test]
    fn store_roundtrips_candidate_run_evaluation_and_bootstrap() {
        let store = temp_store();
        let candidate = candidate("candidate-1", "balanced");
        let run = run("run-1", "candidate-1");
        let evaluation = evaluation("evaluation-1", "candidate-1", 0.91);
        let bootstrap = bootstrap("bootstrap-1");

        store.put_candidate(&candidate).unwrap();
        store.put_run(&run).unwrap();
        store.put_evaluation(&evaluation).unwrap();
        store.put_bootstrap(&bootstrap).unwrap();

        assert_eq!(
            store.get_candidate("candidate-1").unwrap().unwrap(),
            candidate
        );
        assert_eq!(store.get_run("run-1").unwrap().unwrap(), run);
        assert_eq!(
            store.get_evaluation("evaluation-1").unwrap().unwrap(),
            evaluation
        );
        assert_eq!(
            store.get_bootstrap("bootstrap-1").unwrap().unwrap(),
            bootstrap
        );
    }

    #[test]
    fn store_supports_latest_best_and_diff_queries() {
        let store = temp_store();
        let candidate_one = candidate("candidate-1", "balanced");
        let candidate_two = candidate("candidate-2", "strict");
        store.put_candidate(&candidate_one).unwrap();
        store.put_candidate(&candidate_two).unwrap();
        store.put_run(&run("run-1", "candidate-1")).unwrap();
        store.put_run(&run("run-2", "candidate-2")).unwrap();
        store
            .put_evaluation(&evaluation("evaluation-1", "candidate-1", 0.82))
            .unwrap();
        store
            .put_evaluation(&evaluation("evaluation-2", "candidate-2", 0.94))
            .unwrap();

        let latest_runs = store.latest_runs(1).unwrap();
        assert_eq!(latest_runs.len(), 1);
        assert_eq!(latest_runs[0].run_id, "run-2");

        let best = store
            .best_candidate_by_evaluator("pack-alpha")
            .unwrap()
            .unwrap();
        assert_eq!(best.candidate_id, "candidate-2");

        let diff = store.diff_candidates("candidate-1", "candidate-2").unwrap();
        assert_eq!(
            diff.get("provider_profile").unwrap(),
            &HarnessKnobDiff {
                left: Some(json!("balanced")),
                right: Some(json!("strict")),
            }
        );
    }
}
