use casbin::prelude::{CoreApi, DefaultModel, Enforcer, MemoryAdapter, MgmtApi};
use chrono::Utc;
use cortex_domain::capabilities::navigation_graph::{
    PlatformCapabilityCatalog, SpaceCapabilityGraph,
};
use cortex_domain::ux::clamp_required_role_floor;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

const AUTHZ_DECISION_VERSION: &str = "1.0.0";
const SHADOW_MISMATCH_REPORT_RELATIVE_PATH: &str =
    "logs/alignment/authz_shadow_mismatch_latest.json";
const SHADOW_MISMATCH_EVENTS_RELATIVE_PATH: &str =
    "logs/alignment/authz_shadow_mismatch_events.jsonl";
const CASBIN_MODEL_CONFIG_RELATIVE_PATH: &str =
    "cortex/apps/cortex-eudaemon/config/authz/nostra_cortex_rbac_abac_v1.conf";
const CASBIN_ENFORCER_CACHE_MAX_ENTRIES: usize = 64;
const CASBIN_MODEL_TEXT_FALLBACK: &str = r#"
[request_definition]
r = sub, dom, obj, act

[policy_definition]
p = sub, dom, obj, act, eft

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.dom == p.dom && r.obj == p.obj && r.act == p.act
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationEngineMode {
    Legacy,
    Shadow,
    Enforce,
}

impl AuthorizationEngineMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "legacy" => Some(Self::Legacy),
            "shadow" => Some(Self::Shadow),
            "enforce" => Some(Self::Enforce),
            _ => None,
        }
    }

    pub fn from_env() -> Self {
        env::var("NOSTRA_AUTHZ_ENGINE_MODE")
            .ok()
            .and_then(|raw| Self::parse(&raw))
            .unwrap_or(Self::Legacy)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Legacy => "legacy",
            Self::Shadow => "shadow",
            Self::Enforce => "enforce",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthorizationEndpointGroup {
    GroupA,
    GroupB,
    GroupC,
}

impl AuthorizationEndpointGroup {
    fn mode_env_var(self) -> &'static str {
        match self {
            Self::GroupA => "NOSTRA_AUTHZ_GROUP_A_MODE",
            Self::GroupB => "NOSTRA_AUTHZ_GROUP_B_MODE",
            Self::GroupC => "NOSTRA_AUTHZ_GROUP_C_MODE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DecisionMetricKey {
    engine: String,
    mode: String,
    endpoint: String,
    action: String,
    result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ShadowMismatchMetricKey {
    endpoint: String,
    action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IdentityUnverifiedMetricKey {
    endpoint: String,
}

#[derive(Debug, Default, Clone)]
struct AuthzMetricsStore {
    decision_total: BTreeMap<DecisionMetricKey, u64>,
    shadow_mismatch_total: BTreeMap<ShadowMismatchMetricKey, u64>,
    identity_unverified_total: BTreeMap<IdentityUnverifiedMetricKey, u64>,
    policy_compile_fail_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRequirement {
    pub domain: String,
    pub resource: String,
    pub action: String,
    pub required_role: String,
    #[serde(default)]
    pub required_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    pub actor_role: String,
    #[serde(default)]
    pub actor_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    pub resource: String,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_role: Option<String>,
    #[serde(default)]
    pub required_claims: Vec<String>,
    #[serde(default)]
    pub policy_requirements: Vec<PolicyRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthzDecision {
    pub allowed: bool,
    pub engine: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_role: Option<String>,
    #[serde(default)]
    pub required_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationOutcome {
    pub mode: String,
    pub decision: AuthzDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_decision: Option<AuthzDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub casbin_decision: Option<AuthzDecision>,
    pub mismatch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ShadowMismatchReport {
    schema_version: String,
    generated_at: String,
    mode: String,
    count: u64,
    latest: ShadowMismatchRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ShadowMismatchRecord {
    endpoint: String,
    principal: String,
    space_id: String,
    resource: String,
    action: String,
    actor_role: String,
    legacy_allowed: bool,
    casbin_allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    legacy_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    casbin_reason: Option<String>,
}

fn shadow_mismatch_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn enforcer_cache() -> &'static Mutex<BTreeMap<String, Arc<Enforcer>>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Arc<Enforcer>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn authz_metrics_store() -> &'static Mutex<AuthzMetricsStore> {
    static METRICS: OnceLock<Mutex<AuthzMetricsStore>> = OnceLock::new();
    METRICS.get_or_init(|| Mutex::new(AuthzMetricsStore::default()))
}

#[cfg(test)]
pub(crate) fn shared_testing_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn endpoint_group(endpoint: Option<&str>) -> Option<AuthorizationEndpointGroup> {
    let endpoint = endpoint?.trim();
    if endpoint.is_empty() {
        return None;
    }
    match endpoint {
        "put_space_capability_graph" | "get_space_navigation_plan" => {
            Some(AuthorizationEndpointGroup::GroupA)
        }
        "post_contribution_graph_pipeline_run"
        | "post_contribution_graph_steward_packet_export" => {
            Some(AuthorizationEndpointGroup::GroupC)
        }
        value
            if value.starts_with("post_cortex_heap_")
                || value.starts_with("post_cortex_artifact_") =>
        {
            Some(AuthorizationEndpointGroup::GroupB)
        }
        _ => None,
    }
}

fn resolve_group_mode_override(
    group: AuthorizationEndpointGroup,
) -> Option<AuthorizationEngineMode> {
    let raw = env::var(group.mode_env_var()).ok()?;
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "inherit" {
        return None;
    }
    AuthorizationEngineMode::parse(&normalized)
}

fn resolve_effective_mode(request: &AuthorizationRequest) -> AuthorizationEngineMode {
    let base = AuthorizationEngineMode::from_env();
    endpoint_group(request.endpoint.as_deref())
        .and_then(resolve_group_mode_override)
        .unwrap_or(base)
}

fn increment_metric<K>(map: &mut BTreeMap<K, u64>, key: K)
where
    K: Ord,
{
    let counter = map.entry(key).or_insert(0);
    *counter = counter.saturating_add(1);
}

fn record_decision_metric(
    decision: &AuthzDecision,
    mode: AuthorizationEngineMode,
    endpoint: &str,
    action: &str,
) {
    let Ok(mut metrics) = authz_metrics_store().lock() else {
        return;
    };
    let key = DecisionMetricKey {
        engine: decision.engine.clone(),
        mode: mode.as_str().to_string(),
        endpoint: endpoint.to_string(),
        action: action.to_string(),
        result: if decision.allowed {
            "allow".to_string()
        } else {
            "deny".to_string()
        },
    };
    increment_metric(&mut metrics.decision_total, key);
}

fn record_shadow_mismatch_metric(endpoint: &str, action: &str) {
    let Ok(mut metrics) = authz_metrics_store().lock() else {
        return;
    };
    increment_metric(
        &mut metrics.shadow_mismatch_total,
        ShadowMismatchMetricKey {
            endpoint: endpoint.to_string(),
            action: action.to_string(),
        },
    );
}

pub fn record_authz_identity_unverified(endpoint: &str) {
    let Ok(mut metrics) = authz_metrics_store().lock() else {
        return;
    };
    increment_metric(
        &mut metrics.identity_unverified_total,
        IdentityUnverifiedMetricKey {
            endpoint: endpoint.to_string(),
        },
    );
}

fn record_policy_compile_fail() {
    let Ok(mut metrics) = authz_metrics_store().lock() else {
        return;
    };
    metrics.policy_compile_fail_total = metrics.policy_compile_fail_total.saturating_add(1);
}

pub fn get_authz_metrics_snapshot() -> serde_json::Value {
    let snapshot = authz_metrics_store()
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    let decisions = snapshot
        .decision_total
        .into_iter()
        .map(|(key, count)| {
            json!({
                "engine": key.engine,
                "mode": key.mode,
                "endpoint": key.endpoint,
                "action": key.action,
                "result": key.result,
                "count": count
            })
        })
        .collect::<Vec<_>>();
    let mismatches = snapshot
        .shadow_mismatch_total
        .into_iter()
        .map(|(key, count)| {
            json!({
                "endpoint": key.endpoint,
                "action": key.action,
                "count": count
            })
        })
        .collect::<Vec<_>>();
    let identity_unverified = snapshot
        .identity_unverified_total
        .into_iter()
        .map(|(key, count)| {
            json!({
                "endpoint": key.endpoint,
                "count": count
            })
        })
        .collect::<Vec<_>>();
    json!({
        "schemaVersion": "1.0.0",
        "generatedAt": Utc::now().to_rfc3339(),
        "authz_decision_total": decisions,
        "authz_shadow_mismatch_total": mismatches,
        "authz_identity_unverified_total": identity_unverified,
        "authz_policy_compile_fail_total": snapshot.policy_compile_fail_total
    })
}

pub fn authz_decision_version() -> &'static str {
    AUTHZ_DECISION_VERSION
}

pub fn authz_dev_mode_enabled() -> bool {
    env::var("NOSTRA_AUTHZ_DEV_MODE")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub fn allow_unverified_role_header() -> bool {
    if !authz_dev_mode_enabled() {
        return false;
    }
    env::var("NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub fn normalize_role(role: &str) -> String {
    role.trim().to_ascii_lowercase()
}

pub fn role_rank(role: &str) -> u8 {
    match normalize_role(role).as_str() {
        "viewer" => 1,
        "editor" => 2,
        "operator" => 3,
        "steward" => 4,
        "admin" => 5,
        _ => 0,
    }
}

pub fn valid_role(role: &str) -> bool {
    role_rank(role) > 0
}

pub fn normalize_required_role(required_role: Option<&str>) -> String {
    required_role
        .map(normalize_role)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "viewer".to_string())
}

pub fn clamp_role_floor(base_required_role: &str, candidate_role: Option<&str>) -> String {
    clamp_required_role_floor(base_required_role, candidate_role)
}

pub fn build_navigation_policy_requirements(
    catalog: &PlatformCapabilityCatalog,
    space_graph: &SpaceCapabilityGraph,
) -> Vec<PolicyRequirement> {
    let override_by_id: BTreeMap<String, _> = space_graph
        .nodes
        .iter()
        .map(|item| (item.capability_id.0.clone(), item))
        .collect();
    let mut requirements = Vec::new();

    for node in &catalog.nodes {
        let route_id = match node.route_id.as_deref() {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => continue,
        };
        let override_node = override_by_id.get(&node.id.0).copied();
        if override_node.map(|item| item.is_active).unwrap_or(true) == false {
            continue;
        }

        let base_required_role = normalize_required_role(node.required_role.as_deref());
        let required_role = clamp_role_floor(
            &base_required_role,
            override_node
                .and_then(|item| item.local_required_role.as_deref())
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        );

        let mut required_claims: BTreeSet<String> = node
            .required_claims
            .iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
        if let Some(override_node) = override_node {
            for claim in &override_node.local_additional_required_claims {
                let trimmed = claim.trim();
                if !trimmed.is_empty() {
                    required_claims.insert(trimmed.to_string());
                }
            }
        }
        let required_claims: Vec<String> = required_claims.into_iter().collect();

        requirements.push(PolicyRequirement {
            domain: space_graph.space_id.clone(),
            resource: format!("route:{route_id}"),
            action: "navigate".to_string(),
            required_role: required_role.clone(),
            required_claims: required_claims.clone(),
        });
        requirements.push(PolicyRequirement {
            domain: space_graph.space_id.clone(),
            resource: format!("capability:{}", node.id.0),
            action: "navigate".to_string(),
            required_role,
            required_claims,
        });
    }

    requirements
}

pub async fn authorize(request: &AuthorizationRequest) -> AuthorizationOutcome {
    let mode = resolve_effective_mode(request);
    let legacy = evaluate_legacy(request);
    let casbin = evaluate_casbin(request).await;
    let mismatch = legacy.allowed != casbin.allowed;

    let mut decision = match mode {
        AuthorizationEngineMode::Legacy => legacy.clone(),
        AuthorizationEngineMode::Shadow => legacy.clone(),
        AuthorizationEngineMode::Enforce => casbin.clone(),
    };

    if mode == AuthorizationEngineMode::Enforce && casbin_engine_failure(&casbin) {
        decision = casbin_failure_fallback_decision(request, &casbin);
    }

    if mode == AuthorizationEngineMode::Shadow && mismatch {
        persist_shadow_mismatch(request, &legacy, &casbin);
        record_shadow_mismatch_metric(
            request.endpoint.as_deref().unwrap_or("unknown_endpoint"),
            &request.action,
        );
    }
    record_decision_metric(
        &decision,
        mode,
        request.endpoint.as_deref().unwrap_or("unknown_endpoint"),
        &request.action,
    );

    AuthorizationOutcome {
        mode: mode.as_str().to_string(),
        decision,
        legacy_decision: Some(legacy),
        casbin_decision: Some(casbin),
        mismatch,
    }
}

fn evaluate_legacy(request: &AuthorizationRequest) -> AuthzDecision {
    let actor_role = normalize_role(&request.actor_role);
    let required_role = normalize_required_role(request.required_role.as_deref());
    let claims_ok = required_claims_satisfied(&request.required_claims, &request.actor_claims);
    let allowed = role_rank(&actor_role) >= role_rank(&required_role) && claims_ok;

    AuthzDecision {
        allowed,
        engine: "legacy".to_string(),
        reason: if allowed {
            Some("allowed".to_string())
        } else if !claims_ok {
            Some("required claims not satisfied".to_string())
        } else {
            Some(format!(
                "role '{}' below required '{}'",
                actor_role, required_role
            ))
        },
        required_role: Some(required_role),
        required_claims: request.required_claims.clone(),
        principal: request.principal.clone(),
        space_id: request.space_id.clone(),
        resource: request.resource.clone(),
        action: request.action.clone(),
    }
}

fn casbin_error_decision(request: &AuthorizationRequest, reason: String) -> AuthzDecision {
    record_policy_compile_fail();
    AuthzDecision {
        allowed: false,
        engine: "casbin".to_string(),
        reason: Some(reason),
        required_role: request.required_role.clone(),
        required_claims: request.required_claims.clone(),
        principal: request.principal.clone(),
        space_id: request.space_id.clone(),
        resource: request.resource.clone(),
        action: request.action.clone(),
    }
}

fn casbin_engine_failure(decision: &AuthzDecision) -> bool {
    if decision.engine != "casbin" {
        return false;
    }
    decision
        .reason
        .as_deref()
        .map(|reason| {
            let normalized = reason.to_ascii_lowercase();
            normalized.contains("model compile failed")
                || normalized.contains("enforcer build failed")
                || normalized.contains("policy evaluation failed")
        })
        .unwrap_or(false)
}

fn action_is_mutating(action: &str) -> bool {
    matches!(
        action.trim().to_ascii_lowercase().as_str(),
        "mutate" | "create" | "publish" | "admin"
    )
}

fn casbin_failure_fallback_decision(
    request: &AuthorizationRequest,
    failed_decision: &AuthzDecision,
) -> AuthzDecision {
    let claims_ok = required_claims_satisfied(&request.required_claims, &request.actor_claims);
    if action_is_mutating(&request.action) {
        return AuthzDecision {
            allowed: false,
            engine: "casbin_fallback".to_string(),
            reason: Some(format!(
                "casbin unavailable: {}",
                failed_decision
                    .reason
                    .clone()
                    .unwrap_or_else(|| "unknown failure".to_string())
            )),
            required_role: request.required_role.clone(),
            required_claims: request.required_claims.clone(),
            principal: request.principal.clone(),
            space_id: request.space_id.clone(),
            resource: request.resource.clone(),
            action: request.action.clone(),
        };
    }
    let viewer_ok = role_rank(&request.actor_role) >= role_rank("viewer");
    let allowed = viewer_ok && claims_ok;
    AuthzDecision {
        allowed,
        engine: "casbin_fallback".to_string(),
        reason: if allowed {
            Some("casbin unavailable: viewer-floor fallback allowed".to_string())
        } else if !claims_ok {
            Some("casbin unavailable: required claims not satisfied".to_string())
        } else {
            Some("casbin unavailable: viewer-floor fallback denied".to_string())
        },
        required_role: Some("viewer".to_string()),
        required_claims: request.required_claims.clone(),
        principal: request.principal.clone(),
        space_id: request.space_id.clone(),
        resource: request.resource.clone(),
        action: request.action.clone(),
    }
}

fn casbin_model_path() -> PathBuf {
    workspace_root().join(CASBIN_MODEL_CONFIG_RELATIVE_PATH)
}

fn load_casbin_model_text() -> Result<String, String> {
    let path = casbin_model_path();
    let raw = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read model at {}: {err}", path.display()))?;
    if raw.trim().is_empty() {
        return Err(format!("model file at {} is empty", path.display()));
    }
    Ok(raw)
}

fn canonicalize_policy_requirements(requirements: &[PolicyRequirement]) -> Vec<PolicyRequirement> {
    let mut canonical = requirements.to_vec();
    for requirement in &mut canonical {
        requirement.domain = requirement.domain.trim().to_string();
        requirement.resource = requirement.resource.trim().to_string();
        requirement.action = requirement.action.trim().to_string();
        requirement.required_role =
            normalize_required_role(Some(requirement.required_role.as_str()));
        let claims: BTreeSet<String> = requirement
            .required_claims
            .iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
        requirement.required_claims = claims.into_iter().collect();
    }
    canonical.sort_by(|left, right| {
        (
            left.domain.as_str(),
            left.resource.as_str(),
            left.action.as_str(),
            left.required_role.as_str(),
            left.required_claims.join(","),
        )
            .cmp(&(
                right.domain.as_str(),
                right.resource.as_str(),
                right.action.as_str(),
                right.required_role.as_str(),
                right.required_claims.join(","),
            ))
    });
    canonical
}

fn policy_fingerprint(model_text: &str, requirements: &[PolicyRequirement]) -> String {
    let canonical = canonicalize_policy_requirements(requirements);
    let encoded = serde_json::to_vec(&json!({
        "model": model_text,
        "requirements": canonical
    }))
    .unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hex::encode(hasher.finalize())
}

async fn build_casbin_enforcer(
    model_text: &str,
    requirements: &[PolicyRequirement],
) -> Result<Arc<Enforcer>, String> {
    let model = DefaultModel::from_str(model_text)
        .await
        .map_err(|err| format!("model compile failed: {err}"))?;
    let adapter = MemoryAdapter::default();
    let mut enforcer = Enforcer::new(model, adapter)
        .await
        .map_err(|err| format!("enforcer build failed: {err}"))?;
    for requirement in canonicalize_policy_requirements(requirements) {
        let required_rank = role_rank(&requirement.required_role);
        for role in ["viewer", "editor", "operator", "steward", "admin"] {
            if role_rank(role) >= required_rank {
                enforcer
                    .add_policy(vec![
                        role.to_string(),
                        requirement.domain.clone(),
                        requirement.resource.clone(),
                        requirement.action.clone(),
                        "allow".to_string(),
                    ])
                    .await
                    .map_err(|err| format!("policy add failed: {err}"))?;
            }
        }
    }
    Ok(Arc::new(enforcer))
}

async fn resolve_casbin_enforcer(
    model_text: &str,
    requirements: &[PolicyRequirement],
) -> Result<Arc<Enforcer>, String> {
    let fingerprint = policy_fingerprint(model_text, requirements);
    if let Some(cached) = enforcer_cache()
        .lock()
        .ok()
        .and_then(|cache| cache.get(&fingerprint).cloned())
    {
        return Ok(cached);
    }

    let built = build_casbin_enforcer(model_text, requirements).await?;
    if let Ok(mut cache) = enforcer_cache().lock() {
        if let Some(existing) = cache.get(&fingerprint).cloned() {
            return Ok(existing);
        }
        if cache.len() >= CASBIN_ENFORCER_CACHE_MAX_ENTRIES {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }
        cache.insert(fingerprint, built.clone());
    }
    Ok(built)
}

async fn evaluate_casbin(request: &AuthorizationRequest) -> AuthzDecision {
    let actor_role = normalize_role(&request.actor_role);
    let requested_domain = request
        .space_id
        .clone()
        .unwrap_or_else(|| "global".to_string());
    let requirements = if request.policy_requirements.is_empty() {
        vec![PolicyRequirement {
            domain: requested_domain.clone(),
            resource: request.resource.clone(),
            action: request.action.clone(),
            required_role: normalize_required_role(request.required_role.as_deref()),
            required_claims: request.required_claims.clone(),
        }]
    } else {
        request.policy_requirements.clone()
    };
    let mut model_load_warning = None::<String>;
    let model_text = match load_casbin_model_text() {
        Ok(model_text) => model_text,
        Err(err) => {
            record_policy_compile_fail();
            model_load_warning = Some(format!("model load failed: {err}; using embedded fallback"));
            CASBIN_MODEL_TEXT_FALLBACK.to_string()
        }
    };
    let enforcer = match resolve_casbin_enforcer(&model_text, &requirements).await {
        Ok(enforcer) => enforcer,
        Err(err) => return casbin_error_decision(request, err),
    };

    let mut requirement_lookup: BTreeMap<(String, String, String), PolicyRequirement> =
        BTreeMap::new();
    for requirement in &requirements {
        requirement_lookup.insert(
            (
                requirement.domain.clone(),
                requirement.resource.clone(),
                requirement.action.clone(),
            ),
            requirement.clone(),
        );
    }

    let claims_requirement = requirement_lookup
        .get(&(
            requested_domain.clone(),
            request.resource.clone(),
            request.action.clone(),
        ))
        .map(|item| item.required_claims.clone())
        .unwrap_or_else(|| request.required_claims.clone());

    let claims_ok = required_claims_satisfied(&claims_requirement, &request.actor_claims);
    let allowed_by_policy = match enforcer.enforce((
        actor_role.as_str(),
        requested_domain.as_str(),
        request.resource.as_str(),
        request.action.as_str(),
    )) {
        Ok(allowed) => allowed,
        Err(err) => {
            return casbin_error_decision(request, format!("policy evaluation failed: {err}"));
        }
    };
    let allowed = allowed_by_policy && claims_ok;

    let required_role = requirement_lookup
        .get(&(
            requested_domain.clone(),
            request.resource.clone(),
            request.action.clone(),
        ))
        .map(|item| item.required_role.clone())
        .or_else(|| request.required_role.clone());

    AuthzDecision {
        allowed,
        engine: "casbin".to_string(),
        reason: if allowed {
            Some(
                model_load_warning
                    .map(|warning| format!("allowed ({warning})"))
                    .unwrap_or_else(|| "allowed".to_string()),
            )
        } else if !claims_ok {
            Some("required claims not satisfied".to_string())
        } else {
            Some(
                model_load_warning
                    .map(|warning| format!("policy denied ({warning})"))
                    .unwrap_or_else(|| "policy denied".to_string()),
            )
        },
        required_role,
        required_claims: claims_requirement,
        principal: request.principal.clone(),
        space_id: request.space_id.clone(),
        resource: request.resource.clone(),
        action: request.action.clone(),
    }
}

fn required_claims_satisfied(required_claims: &[String], actor_claims: &[String]) -> bool {
    if required_claims.is_empty() {
        return true;
    }
    let actor_claims: BTreeSet<String> = actor_claims
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    required_claims
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .all(|value| actor_claims.contains(&value))
}

fn workspace_root() -> PathBuf {
    if let Ok(value) = env::var("NOSTRA_WORKSPACE_ROOT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn persist_shadow_mismatch(
    request: &AuthorizationRequest,
    legacy: &AuthzDecision,
    casbin: &AuthzDecision,
) {
    let Ok(_guard) = shadow_mismatch_lock().lock() else {
        return;
    };
    let path = workspace_root().join(SHADOW_MISMATCH_REPORT_RELATIVE_PATH);
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let existing_count = fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<ShadowMismatchReport>(&raw).ok())
        .map(|report| report.count)
        .unwrap_or(0);
    let record = ShadowMismatchRecord {
        endpoint: request
            .endpoint
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        principal: request
            .principal
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        space_id: request
            .space_id
            .clone()
            .unwrap_or_else(|| "global".to_string()),
        resource: request.resource.clone(),
        action: request.action.clone(),
        actor_role: normalize_role(&request.actor_role),
        legacy_allowed: legacy.allowed,
        casbin_allowed: casbin.allowed,
        legacy_reason: legacy.reason.clone(),
        casbin_reason: casbin.reason.clone(),
    };
    let report = ShadowMismatchReport {
        schema_version: "1.0.0".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        mode: "shadow".to_string(),
        count: existing_count.saturating_add(1),
        latest: record.clone(),
    };
    let events_path = workspace_root().join(SHADOW_MISMATCH_EVENTS_RELATIVE_PATH);
    if let Some(parent) = events_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path)
    {
        let event = json!({
            "schemaVersion": "1.0.0",
            "generatedAt": Utc::now().to_rfc3339(),
            "mode": "shadow",
            "event": "authz_shadow_mismatch",
            "record": {
                "endpoint": record.endpoint,
                "principal": record.principal,
                "spaceId": record.space_id,
                "resource": record.resource,
                "action": record.action,
                "actorRole": record.actor_role,
                "legacyAllowed": record.legacy_allowed,
                "casbinAllowed": record.casbin_allowed,
                "legacyReason": record.legacy_reason,
                "casbinReason": record.casbin_reason
            }
        });
        if let Ok(encoded) = serde_json::to_string(&event) {
            let _ = writeln!(file, "{encoded}");
        }
    }
    if let Ok(encoded) = serde_json::to_string_pretty(&report) {
        let _ = fs::write(path, encoded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::capabilities::navigation_graph::{
        CapabilityId, CapabilityNode, IntentType, OperationalFrequency, PlatformCapabilityCatalog,
        SpaceCapabilityGraph, SpaceCapabilityNodeOverride, SurfacingHeuristic,
    };
    use std::path::{Path, PathBuf};

    fn reset_authz_test_state() {
        if let Ok(mut cache) = enforcer_cache().lock() {
            cache.clear();
        }
        if let Ok(mut metrics) = authz_metrics_store().lock() {
            *metrics = AuthzMetricsStore::default();
        }
    }

    fn acquire_test_env_lock() -> std::sync::MutexGuard<'static, ()> {
        crate::services::authz::shared_testing_env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = self.previous.as_ref() {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "cortex-authz-tests-{}-{}",
                std::process::id(),
                nonce
            ));
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn unverified_role_header_requires_dev_mode() {
        let _serial = acquire_test_env_lock();
        reset_authz_test_state();
        let _dev_mode = EnvVarGuard::unset("NOSTRA_AUTHZ_DEV_MODE");
        let _allow = EnvVarGuard::set("NOSTRA_AUTHZ_ALLOW_UNVERIFIED_ROLE_HEADER", "true");
        assert!(!allow_unverified_role_header());

        let _dev_mode_enabled = EnvVarGuard::set("NOSTRA_AUTHZ_DEV_MODE", "true");
        assert!(allow_unverified_role_header());
    }

    #[test]
    fn navigation_policy_requirements_clamp_role_floor_and_merge_claims() {
        let _serial = acquire_test_env_lock();
        reset_authz_test_state();
        let mut catalog = PlatformCapabilityCatalog::new();
        catalog.unverified_add_node(CapabilityNode {
            id: CapabilityId("route:/logs".to_string()),
            resource_ref: None,
            name: "Logs".to_string(),
            description: "Logs route".to_string(),
            intent_type: IntentType::Monitor,
            route_id: Some("/logs".to_string()),
            category: Some("core".to_string()),
            required_role: Some("operator".to_string()),
            required_claims: vec!["capability:mutate:logs".to_string()],
            icon: Some("logs".to_string()),
            surfacing_heuristic: SurfacingHeuristic::PrimaryCore,
            operational_frequency: OperationalFrequency::Continuous,
            domain_entities: vec![],
            placement_constraint: None,
            root_path: None,
            invariant_violations: vec![],
        });
        let graph = SpaceCapabilityGraph {
            schema_version: "1.0.0".to_string(),
            space_id: "space-a".to_string(),
            base_catalog_version: "v1".to_string(),
            base_catalog_hash: "hash".to_string(),
            nodes: vec![SpaceCapabilityNodeOverride {
                capability_id: CapabilityId("route:/logs".to_string()),
                local_alias: None,
                is_active: true,
                local_required_role: Some("viewer".to_string()),
                local_additional_required_claims: vec!["capability:approve:logs".to_string()],
                surfacing_heuristic: None,
                operational_frequency: None,
                placement_constraint: None,
            }],
            edges: vec![],
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            updated_by: "test".to_string(),
            lineage_ref: Some("decision:test".to_string()),
        };
        let requirements = build_navigation_policy_requirements(&catalog, &graph);
        let route_requirement = requirements
            .iter()
            .find(|item| item.resource == "route:/logs" && item.action == "navigate")
            .expect("route requirement");
        assert_eq!(route_requirement.required_role, "operator");
        assert_eq!(
            route_requirement.required_claims,
            vec![
                "capability:approve:logs".to_string(),
                "capability:mutate:logs".to_string()
            ]
        );
    }

    #[test]
    fn shadow_mode_persists_mismatch_report() {
        let _serial = acquire_test_env_lock();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        runtime.block_on(async {
            reset_authz_test_state();
            let temp = TempDir::new();
            let _workspace = EnvVarGuard::set(
                "NOSTRA_WORKSPACE_ROOT",
                temp.path().display().to_string().as_str(),
            );
            let _mode = EnvVarGuard::set("NOSTRA_AUTHZ_ENGINE_MODE", "shadow");
            let _ = EnvVarGuard::unset("NOSTRA_AUTHZ_DEV_MODE");
            let request = AuthorizationRequest {
                endpoint: Some("test-endpoint".to_string()),
                principal: Some("2vxsx-fae".to_string()),
                actor_role: "viewer".to_string(),
                actor_claims: vec![],
                space_id: Some("space-x".to_string()),
                resource: "capability:test".to_string(),
                action: "mutate".to_string(),
                required_role: Some("viewer".to_string()),
                required_claims: vec![],
                policy_requirements: vec![PolicyRequirement {
                    domain: "space-x".to_string(),
                    resource: "capability:test".to_string(),
                    action: "mutate".to_string(),
                    required_role: "admin".to_string(),
                    required_claims: vec![],
                }],
            };

            let outcome = authorize(&request).await;
            assert!(outcome.mismatch);
            assert_eq!(outcome.mode, "shadow");
            let report_path = temp.path().join(SHADOW_MISMATCH_REPORT_RELATIVE_PATH);
            let raw = std::fs::read_to_string(report_path).expect("read mismatch report");
            let report: serde_json::Value =
                serde_json::from_str(&raw).expect("parse mismatch report");
            assert_eq!(report["mode"], "shadow");
            assert_eq!(report["latest"]["endpoint"], "test-endpoint");
            assert_eq!(report["latest"]["legacyAllowed"], true);
            assert_eq!(report["latest"]["casbinAllowed"], false);
            let events_path = temp.path().join(SHADOW_MISMATCH_EVENTS_RELATIVE_PATH);
            let events_raw = std::fs::read_to_string(events_path).expect("read mismatch events");
            assert!(events_raw.lines().count() >= 1);
        });
    }

    #[test]
    fn group_mode_override_enforces_group_specific_mode() {
        let _serial = acquire_test_env_lock();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        runtime.block_on(async {
            reset_authz_test_state();
            let temp = TempDir::new();
            let _workspace = EnvVarGuard::set(
                "NOSTRA_WORKSPACE_ROOT",
                temp.path().display().to_string().as_str(),
            );
            let _global_mode = EnvVarGuard::set("NOSTRA_AUTHZ_ENGINE_MODE", "legacy");
            let _group_mode = EnvVarGuard::set("NOSTRA_AUTHZ_GROUP_B_MODE", "enforce");
            let request = AuthorizationRequest {
                endpoint: Some("post_cortex_heap_emit".to_string()),
                principal: Some("2vxsx-fae".to_string()),
                actor_role: "viewer".to_string(),
                actor_claims: vec![],
                space_id: Some("space-x".to_string()),
                resource: "capability:test".to_string(),
                action: "mutate".to_string(),
                required_role: Some("viewer".to_string()),
                required_claims: vec![],
                policy_requirements: vec![PolicyRequirement {
                    domain: "space-x".to_string(),
                    resource: "capability:test".to_string(),
                    action: "mutate".to_string(),
                    required_role: "admin".to_string(),
                    required_claims: vec![],
                }],
            };

            let outcome = authorize(&request).await;
            assert_eq!(outcome.mode, "enforce");
            assert!(!outcome.decision.allowed);
            assert_eq!(outcome.decision.engine, "casbin");
        });
    }

    #[test]
    fn authz_metrics_snapshot_tracks_identity_unverified_counts() {
        let _serial = acquire_test_env_lock();
        reset_authz_test_state();
        record_authz_identity_unverified("post_cortex_artifact_save");
        let snapshot = get_authz_metrics_snapshot();
        let totals = snapshot["authz_identity_unverified_total"]
            .as_array()
            .expect("identity totals array");
        assert!(
            totals.iter().any(|item| {
                item["endpoint"] == "post_cortex_artifact_save" && item["count"] == 1
            })
        );
    }
}
