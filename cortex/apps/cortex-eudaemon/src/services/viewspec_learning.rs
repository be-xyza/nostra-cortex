use crate::services::viewspec::{ViewSpecConfidence, ViewSpecV1};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const VIEWSPEC_LEARNING_SCHEMA_VERSION: &str = "1.0.0";
pub const VIEWSPEC_LEARNING_SIGNAL_INDEX_KEY: &str =
    "/cortex/ux/viewspecs/learning/signals/index.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceLearningProfileV1 {
    pub schema_version: String,
    pub space_id: String,
    pub profile_version: u64,
    pub updated_at: String,
    pub signal_count: u64,
    pub confidence_model: SpaceConfidenceModelV1,
    #[serde(default)]
    pub feature_weights: BTreeMap<String, f32>,
    pub policy: LearningPolicyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceConfidenceModelV1 {
    pub base_confidence: f32,
    pub approval_boost: f32,
    pub rejection_penalty: f32,
    pub recency_decay: f32,
    pub min_confidence: f32,
    pub max_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecLearningSignal {
    pub signal_id: String,
    pub event_type: String,
    pub view_spec_id: String,
    pub space_id: String,
    pub actor: String,
    pub timestamp: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LearningReplayResult {
    pub run_id: String,
    pub space_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub input_signal_count: u64,
    pub applied_signal_count: u64,
    pub output_profile_version: u64,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LearningPolicyV1 {
    pub apply_mode: String,
    pub auto_apply_enabled: bool,
    pub global_merge_enabled: bool,
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn supported_learning_events() -> &'static [&'static str] {
    &[
        "candidate_staged",
        "viewspec_locked",
        "viewspec_proposed",
        "viewspec_forked",
        "proposal_ratified",
        "proposal_rejected",
        "manual_confidence_override",
    ]
}

pub fn normalize_event_type(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub fn is_supported_event_type(value: &str) -> bool {
    let normalized = normalize_event_type(value);
    supported_learning_events().contains(&normalized.as_str())
}

pub fn validate_learning_signal(signal: &ViewSpecLearningSignal) -> Result<(), String> {
    if signal.signal_id.trim().is_empty() {
        return Err("signal_id is required".to_string());
    }
    if signal.view_spec_id.trim().is_empty() {
        return Err("view_spec_id is required".to_string());
    }
    if signal.space_id.trim().is_empty() {
        return Err("space_id is required".to_string());
    }
    if signal.actor.trim().is_empty() {
        return Err("actor is required".to_string());
    }
    if signal.timestamp.trim().is_empty() {
        return Err("timestamp is required".to_string());
    }
    if !is_supported_event_type(&signal.event_type) {
        return Err(format!(
            "unsupported event_type '{}'",
            signal.event_type.trim()
        ));
    }
    Ok(())
}

pub fn default_learning_policy() -> LearningPolicyV1 {
    LearningPolicyV1 {
        apply_mode: "advisory".to_string(),
        auto_apply_enabled: false,
        global_merge_enabled: false,
    }
}

pub fn default_space_confidence_model() -> SpaceConfidenceModelV1 {
    SpaceConfidenceModelV1 {
        base_confidence: 0.50,
        approval_boost: 0.06,
        rejection_penalty: 0.09,
        recency_decay: 0.97,
        min_confidence: 0.05,
        max_confidence: 0.95,
    }
}

pub fn default_space_learning_profile(space_id: &str) -> SpaceLearningProfileV1 {
    let mut feature_weights = BTreeMap::new();
    feature_weights.insert("approved_outcomes".to_string(), 0.0);
    feature_weights.insert("rejection_outcomes".to_string(), 0.0);
    feature_weights.insert("manual_overrides".to_string(), 0.0);
    feature_weights.insert("learned_score".to_string(), 0.50);

    SpaceLearningProfileV1 {
        schema_version: VIEWSPEC_LEARNING_SCHEMA_VERSION.to_string(),
        space_id: sanitize_token(space_id),
        profile_version: 1,
        updated_at: now_iso(),
        signal_count: 0,
        confidence_model: default_space_confidence_model(),
        feature_weights,
        policy: default_learning_policy(),
    }
}

pub fn sanitize_token(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn learning_signals_key(date_yyyy_mm_dd: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/learning/signals/{}.jsonl",
        sanitize_token(date_yyyy_mm_dd)
    )
}

pub fn learning_profile_key(space_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/learning/profiles/{}.json",
        sanitize_token(space_id)
    )
}

pub fn learning_replay_key(space_id: &str, run_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/learning/replay/{}/{}.json",
        sanitize_token(space_id),
        sanitize_token(run_id)
    )
}

pub fn replay_run_id() -> String {
    format!("viewspec_learning_replay_{}", Utc::now().timestamp_millis())
}

fn clamp_score(score: f32, model: &SpaceConfidenceModelV1) -> f32 {
    score.clamp(model.min_confidence, model.max_confidence)
}

fn parse_override_score(payload: &Value) -> Option<f32> {
    payload
        .get("score")
        .and_then(|value| value.as_f64().map(|score| score as f32))
        .or_else(|| {
            payload
                .get("score")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse::<f32>().ok())
        })
}

pub fn replay_space_learning_profile(
    space_id: &str,
    signals: &[ViewSpecLearningSignal],
    base_profile_version: u64,
) -> (SpaceLearningProfileV1, LearningReplayResult) {
    let started_at = now_iso();
    let run_id = replay_run_id();

    let mut ordered = signals
        .iter()
        .filter(|signal| signal.space_id == space_id)
        .cloned()
        .collect::<Vec<_>>();

    ordered.sort_by(|a, b| {
        a.timestamp
            .cmp(&b.timestamp)
            .then_with(|| a.signal_id.cmp(&b.signal_id))
    });

    let mut profile = default_space_learning_profile(space_id);
    profile.profile_version = base_profile_version.saturating_add(1);

    let mut score = profile.confidence_model.base_confidence;
    let mut applied_signal_count = 0u64;
    let mut warnings = Vec::new();
    let mut approved_count = 0u64;
    let mut rejected_count = 0u64;
    let mut manual_overrides = 0u64;

    for signal in ordered.iter() {
        let event_type = normalize_event_type(&signal.event_type);
        if !is_supported_event_type(&event_type) {
            warnings.push(format!(
                "Skipped unsupported event '{}' for signal {}",
                signal.event_type, signal.signal_id
            ));
            continue;
        }

        let event_counter = profile
            .feature_weights
            .entry(event_type.clone())
            .or_insert(0.0);
        *event_counter += 1.0;

        if event_type == "manual_confidence_override" {
            if let Some(value) = parse_override_score(&signal.payload) {
                score = clamp_score(value, &profile.confidence_model);
                manual_overrides = manual_overrides.saturating_add(1);
            } else {
                warnings.push(format!(
                    "manual_confidence_override missing numeric score for signal {}",
                    signal.signal_id
                ));
            }
            applied_signal_count = applied_signal_count.saturating_add(1);
            continue;
        }

        let mut adjustment = 0.0f32;
        if matches!(
            event_type.as_str(),
            "candidate_staged" | "viewspec_locked" | "proposal_ratified"
        ) {
            adjustment += profile.confidence_model.approval_boost;
            approved_count = approved_count.saturating_add(1);
        }
        if event_type == "proposal_rejected" {
            adjustment -= profile.confidence_model.rejection_penalty;
            rejected_count = rejected_count.saturating_add(1);
        }

        score = clamp_score(
            (score * profile.confidence_model.recency_decay) + adjustment,
            &profile.confidence_model,
        );
        applied_signal_count = applied_signal_count.saturating_add(1);
    }

    profile.updated_at = now_iso();
    profile.signal_count = applied_signal_count;
    profile
        .feature_weights
        .insert("approved_outcomes".to_string(), approved_count as f32);
    profile
        .feature_weights
        .insert("rejection_outcomes".to_string(), rejected_count as f32);
    profile
        .feature_weights
        .insert("manual_overrides".to_string(), manual_overrides as f32);
    profile
        .feature_weights
        .insert("learned_score".to_string(), score);

    let replay = LearningReplayResult {
        run_id,
        space_id: space_id.to_string(),
        started_at,
        finished_at: now_iso(),
        input_signal_count: ordered.len() as u64,
        applied_signal_count,
        output_profile_version: profile.profile_version,
        warnings,
    };

    (profile, replay)
}

pub fn reset_space_learning_profile(
    space_id: &str,
    base_profile_version: u64,
    actor: &str,
    reason: Option<&str>,
) -> (SpaceLearningProfileV1, LearningReplayResult) {
    let started_at = now_iso();
    let run_id = replay_run_id();

    let mut profile = default_space_learning_profile(space_id);
    profile.profile_version = base_profile_version.saturating_add(1);
    profile.updated_at = now_iso();

    let replay = LearningReplayResult {
        run_id,
        space_id: space_id.to_string(),
        started_at,
        finished_at: now_iso(),
        input_signal_count: 0,
        applied_signal_count: 0,
        output_profile_version: profile.profile_version,
        warnings: vec![format!(
            "profile_reset actor={} reason={}",
            actor.trim(),
            reason.unwrap_or("none").trim()
        )],
    };

    (profile, replay)
}

pub fn recompute_viewspec_confidence(
    spec: &ViewSpecV1,
    profile: &SpaceLearningProfileV1,
) -> ViewSpecConfidence {
    let learned_score = profile
        .feature_weights
        .get("learned_score")
        .copied()
        .unwrap_or(profile.confidence_model.base_confidence);

    let approvals = profile
        .feature_weights
        .get("approved_outcomes")
        .copied()
        .unwrap_or(0.0);
    let rejections = profile
        .feature_weights
        .get("rejection_outcomes")
        .copied()
        .unwrap_or(0.0);
    let overrides = profile
        .feature_weights
        .get("manual_overrides")
        .copied()
        .unwrap_or(0.0);

    let score = clamp_score(learned_score, &profile.confidence_model);

    ViewSpecConfidence {
        score,
        rationale: format!(
            "Space-scoped advisory confidence for {} using profile v{}: signals={}, approvals={}, rejections={}, manualOverrides={}, applyMode={}, autoApply={}",
            spec.view_spec_id,
            profile.profile_version,
            profile.signal_count,
            approvals,
            rejections,
            overrides,
            profile.policy.apply_mode,
            profile.policy.auto_apply_enabled
        ),
    }
}

pub fn extract_space_id_from_payload(payload: &Value) -> Option<String> {
    payload
        .get("spaceId")
        .and_then(|value| value.as_str())
        .or_else(|| payload.get("space_id").and_then(|value| value.as_str()))
        .or_else(|| {
            payload
                .get("scope")
                .and_then(|scope| scope.get("spaceId"))
                .and_then(|value| value.as_str())
        })
        .or_else(|| {
            payload
                .get("scope")
                .and_then(|scope| scope.get("space_id"))
                .and_then(|value| value.as_str())
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signal(
        signal_id: &str,
        event_type: &str,
        timestamp: &str,
        payload: Value,
    ) -> ViewSpecLearningSignal {
        ViewSpecLearningSignal {
            signal_id: signal_id.to_string(),
            event_type: event_type.to_string(),
            view_spec_id: "viewspec_1".to_string(),
            space_id: "space-alpha".to_string(),
            actor: "tester".to_string(),
            timestamp: timestamp.to_string(),
            payload,
        }
    }

    fn test_spec() -> ViewSpecV1 {
        let mut spec = crate::services::viewspec::generate_candidate_viewspecs(
            crate::services::viewspec::ViewSpecScope {
                space_id: Some("space-alpha".to_string()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            },
            "Show project progress clearly",
            &[],
            1,
            "tester",
            "human",
        )
        .into_iter()
        .next()
        .expect("candidate");
        spec.view_spec_id = "viewspec_1".to_string();
        spec
    }

    #[test]
    fn replay_is_deterministic_for_same_signal_set() {
        let signals = vec![
            signal(
                "sig_1",
                "candidate_staged",
                "2026-02-09T00:00:01Z",
                Value::Null,
            ),
            signal(
                "sig_2",
                "viewspec_locked",
                "2026-02-09T00:00:02Z",
                Value::Null,
            ),
            signal(
                "sig_3",
                "proposal_rejected",
                "2026-02-09T00:00:03Z",
                Value::Null,
            ),
        ];

        let (a, _) = replay_space_learning_profile("space-alpha", &signals, 4);
        let (b, _) = replay_space_learning_profile("space-alpha", &signals, 4);

        assert_eq!(
            a.feature_weights.get("learned_score"),
            b.feature_weights.get("learned_score")
        );
        assert_eq!(a.signal_count, b.signal_count);
        assert_eq!(a.profile_version, b.profile_version);
    }

    #[test]
    fn confidence_increases_only_on_approved_outcomes() {
        let base_score = default_space_confidence_model().base_confidence;
        let signals = vec![
            signal(
                "sig_1",
                "viewspec_proposed",
                "2026-02-09T00:00:01Z",
                Value::Null,
            ),
            signal(
                "sig_2",
                "viewspec_forked",
                "2026-02-09T00:00:02Z",
                Value::Null,
            ),
            signal(
                "sig_3",
                "candidate_staged",
                "2026-02-09T00:00:03Z",
                Value::Null,
            ),
        ];
        let (profile, _) = replay_space_learning_profile("space-alpha", &signals, 0);
        let learned_score = profile
            .feature_weights
            .get("learned_score")
            .copied()
            .unwrap_or(0.0);

        assert!(learned_score > base_score);
        assert_eq!(
            profile
                .feature_weights
                .get("approved_outcomes")
                .copied()
                .unwrap_or(0.0),
            1.0
        );
    }

    #[test]
    fn rejection_penalty_is_applied_and_clamped() {
        let mut signals = Vec::new();
        for idx in 0..30 {
            signals.push(signal(
                format!("sig_{}", idx + 1).as_str(),
                "proposal_rejected",
                format!("2026-02-09T00:00:{:02}Z", idx + 1).as_str(),
                Value::Null,
            ));
        }
        let (profile, _) = replay_space_learning_profile("space-alpha", &signals, 0);
        let learned_score = profile
            .feature_weights
            .get("learned_score")
            .copied()
            .unwrap_or(0.0);

        assert_eq!(learned_score, profile.confidence_model.min_confidence);
    }

    #[test]
    fn reset_returns_baseline_profile() {
        let (profile, replay) =
            reset_space_learning_profile("space-alpha", 7, "systems-steward", Some("manual reset"));

        assert_eq!(profile.profile_version, 8);
        assert_eq!(profile.signal_count, 0);
        assert!(!profile.policy.auto_apply_enabled);
        assert!(!profile.policy.global_merge_enabled);
        assert_eq!(replay.output_profile_version, 8);
        assert_eq!(replay.input_signal_count, 0);
    }

    #[test]
    fn confidence_recompute_uses_profile_score() {
        let spec = test_spec();
        let signals = vec![
            signal(
                "sig_1",
                "candidate_staged",
                "2026-02-09T00:00:01Z",
                Value::Null,
            ),
            signal(
                "sig_2",
                "viewspec_locked",
                "2026-02-09T00:00:02Z",
                Value::Null,
            ),
        ];
        let (profile, _) = replay_space_learning_profile("space-alpha", &signals, 0);
        let confidence = recompute_viewspec_confidence(&spec, &profile);

        assert!(confidence.score >= profile.confidence_model.min_confidence);
        assert!(confidence.score <= profile.confidence_model.max_confidence);
        assert!(confidence.rationale.contains("profile v"));
    }
}
