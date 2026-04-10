use candid::{CandidType, Deserialize, Principal};
#[cfg(not(test))]
use ic_cdk::call::Call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;
const ATTRIBUTION_WEIGHT_POLICY_MAGIC: &[u8; 4] = b"NGP1";
const ACTOR_ROLE_BINDING_MAGIC: &[u8; 4] = b"NGR1";

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Policy {
    pub id: String,
    pub rule_type: String,  // "Threshold", "Epistemic", "Role"
    pub parameters: String, // JSON payload
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Constitution {
    pub id: String,
    pub policies: Vec<Policy>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum ComplianceResult {
    Pass,
    Fail(String),
    RequiresReview(String),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum EpistemicMode {
    #[serde(rename = "observe")]
    Observe,
    #[serde(rename = "soft_gate")]
    SoftGate,
    #[serde(rename = "hard_gate")]
    HardGate,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DecisionClass {
    #[serde(rename = "governance")]
    Governance,
    #[serde(rename = "merge")]
    Merge,
    #[serde(rename = "high_impact")]
    HighImpact,
    #[serde(rename = "standard")]
    Standard,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct EpistemicPolicy {
    pub mode: EpistemicMode,
    pub min_evidence: u32,
    pub min_alternatives: u32,
    pub min_robustness: f64,
    pub max_confidence_drift: f64,
    pub max_fork_pressure: f64,
    pub max_correction_density: f64,
    pub simulation_ttl_days: u32,
    pub enforced_decision_classes: Vec<DecisionClass>,
    pub block_on_soft: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AttributionWeightPolicy {
    pub domain_mode: String,
    pub weight: f64,
    pub allow_binding: bool,
    pub rationale: String,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ActionScopeEvaluation {
    pub allowed: bool,
    pub reason: String,
    pub effective_weight: f64,
    pub requires_review: bool,
    pub gate_decision: String,
    pub required_actions: Vec<String>,
    pub policy_ref: Option<String>,
    pub policy_version: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ActorRoleBinding {
    pub space_id: String,
    pub principal: Principal,
    pub role: String,
    pub source_ref: Option<String>,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PolicySnapshot {
    pub policy_ref: String,
    pub domain_mode: String,
    pub weight: f64,
    pub allow_binding: bool,
    pub rationale: String,
    pub policy_version: u64,
    pub captured_at: u64,
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorablePolicyKey(String);

impl Storable for StorablePolicyKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorablePolicyKey(String::from_utf8(bytes.to_vec()).unwrap_or_default())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize)]
struct StorableAttributionWeightPolicy(AttributionWeightPolicy);

impl Storable for StorableAttributionWeightPolicy {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap_or_default();
        let mut bytes = Vec::with_capacity(ATTRIBUTION_WEIGHT_POLICY_MAGIC.len() + payload.len());
        bytes.extend_from_slice(ATTRIBUTION_WEIGHT_POLICY_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        if let Some(payload) = bytes.strip_prefix(ATTRIBUTION_WEIGHT_POLICY_MAGIC) {
            if let Ok(decoded) = postcard::from_bytes::<AttributionWeightPolicy>(payload) {
                return StorableAttributionWeightPolicy(decoded);
            }
        } else if let Ok(decoded) = postcard::from_bytes::<AttributionWeightPolicy>(bytes) {
            ic_cdk::println!(
                "Governance: legacy attribution weight policy decoded without magic prefix"
            );
            return StorableAttributionWeightPolicy(decoded);
        }

        ic_cdk::println!(
            "Governance: failed to decode attribution weight policy; applying fallback"
        );
        StorableAttributionWeightPolicy(AttributionWeightPolicy {
            domain_mode: "unknown".to_string(),
            weight: 0.5,
            allow_binding: true,
            rationale: "legacy decode fallback".to_string(),
            updated_at: 0,
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableActorBindingKey(String);

impl Storable for StorableActorBindingKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableActorBindingKey(String::from_utf8(bytes.to_vec()).unwrap_or_default())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 256,
            is_fixed_size: false,
        };
}

#[derive(Deserialize)]
struct StorableActorRoleBinding(ActorRoleBinding);

impl Storable for StorableActorRoleBinding {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap_or_default();
        let mut bytes = Vec::with_capacity(ACTOR_ROLE_BINDING_MAGIC.len() + payload.len());
        bytes.extend_from_slice(ACTOR_ROLE_BINDING_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        if let Some(payload) = bytes.strip_prefix(ACTOR_ROLE_BINDING_MAGIC) {
            if let Ok(decoded) = postcard::from_bytes::<ActorRoleBinding>(payload) {
                return StorableActorRoleBinding(decoded);
            }
        } else if let Ok(decoded) = postcard::from_bytes::<ActorRoleBinding>(bytes) {
            ic_cdk::println!("Governance: legacy actor role binding decoded without magic prefix");
            return StorableActorRoleBinding(decoded);
        }

        ic_cdk::println!("Governance: failed to decode actor role binding; applying fallback");
        StorableActorRoleBinding(ActorRoleBinding {
            space_id: "unknown".to_string(),
            principal: Principal::anonymous(),
            role: "observer".to_string(),
            source_ref: Some("legacy decode fallback".to_string()),
            updated_at: 0,
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

fn policy_ref(domain_mode: &str, updated_at: u64) -> String {
    format!("governance_policy:{}:{}", domain_mode, updated_at)
}

#[cfg(not(test))]
fn now_unix_secs() -> u64 {
    (ic_cdk::api::time() / 1_000_000_000) as u64
}

#[cfg(test)]
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn actor_binding_key(space_id: &str, principal: Principal) -> Option<StorableActorBindingKey> {
    let normalized_space = normalize_key(space_id)?;
    Some(StorableActorBindingKey(format!(
        "{}::{}",
        normalized_space,
        principal.to_text()
    )))
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ATTRIBUTION_WEIGHT_POLICIES: RefCell<StableBTreeMap<StorablePolicyKey, StorableAttributionWeightPolicy, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static ACTOR_ROLE_BINDINGS: RefCell<StableBTreeMap<StorableActorBindingKey, StorableActorRoleBinding, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    // In a real system, this would use StableBTreeMap
    static CONSTITUTIONS: RefCell<HashMap<String, Constitution>> = RefCell::new(HashMap::new());
    static EPISTEMIC_POLICY: RefCell<Option<EpistemicPolicy>> = RefCell::new(None);
    static EPISTEMIC_POLICY_ADMIN: RefCell<Option<Principal>> = RefCell::new(None);
    static WORKFLOW_ENGINE_TARGET: RefCell<Option<Principal>> = RefCell::new(None);
}

#[ic_cdk::update]
fn register_constitution(constitution: Constitution) {
    CONSTITUTIONS.with(|c| {
        c.borrow_mut().insert(constitution.id.clone(), constitution);
    });
}

#[ic_cdk::query]
fn check_compliance(constitution_id: String, context_json: String) -> ComplianceResult {
    let context: HashMap<String, Value> = serde_json::from_str(&context_json).unwrap_or_default();

    let constitution = match CONSTITUTIONS.with(|c| c.borrow().get(&constitution_id).cloned()) {
        Some(c) => c,
        None => return ComplianceResult::Fail("Constitution not found".to_string()),
    };

    for policy in constitution.policies {
        match evaluate_policy(&policy, &context) {
            ComplianceResult::Pass => continue,
            fail_or_review => return fail_or_review,
        }
    }

    ComplianceResult::Pass
}

fn validate_epistemic_policy_thresholds(policy: &EpistemicPolicy) -> Result<(), String> {
    if !(0.0..=1.0).contains(&policy.min_robustness)
        || !(0.0..=1.0).contains(&policy.max_confidence_drift)
        || !(0.0..=1.0).contains(&policy.max_fork_pressure)
        || !(0.0..=1.0).contains(&policy.max_correction_density)
    {
        return Err("policy threshold values must be within 0.0..=1.0".to_string());
    }
    Ok(())
}

fn authorize_epistemic_policy_admin(caller: Principal) -> Result<(), String> {
    EPISTEMIC_POLICY_ADMIN.with(|admin| {
        let mut admin = admin.borrow_mut();
        match *admin {
            Some(existing) if existing != caller => {
                Err("Only epistemic policy admin may update policy".to_string())
            }
            Some(_) => Ok(()),
            None => {
                *admin = Some(caller);
                Ok(())
            }
        }
    })
}

fn set_epistemic_policy_internal(caller: Principal, policy: EpistemicPolicy) -> Result<(), String> {
    validate_epistemic_policy_thresholds(&policy)?;
    authorize_epistemic_policy_admin(caller)?;

    EPISTEMIC_POLICY.with(|p| {
        *p.borrow_mut() = Some(policy);
    });
    Ok(())
}

fn set_workflow_engine_target_internal(caller: Principal, target: Principal) -> Result<(), String> {
    authorize_epistemic_policy_admin(caller)?;
    if target == Principal::anonymous() {
        return Err("workflow engine target must be a concrete canister principal".to_string());
    }

    WORKFLOW_ENGINE_TARGET.with(|slot| {
        *slot.borrow_mut() = Some(target);
    });
    Ok(())
}

fn sync_preconditions(caller: Principal) -> Result<(Principal, EpistemicPolicy), String> {
    authorize_epistemic_policy_admin(caller)?;

    let policy = EPISTEMIC_POLICY
        .with(|p| p.borrow().clone())
        .ok_or_else(|| "No epistemic policy configured".to_string())?;

    let target = WORKFLOW_ENGINE_TARGET
        .with(|slot| *slot.borrow())
        .ok_or_else(|| "Workflow engine target not configured".to_string())?;

    Ok((target, policy))
}

fn normalize_sync_outcome(outcome: Result<(), String>) -> Result<(), String> {
    outcome.map_err(|err| format!("workflow_engine sync failed: {err}"))
}

#[cfg(not(test))]
async fn push_policy_to_workflow_engine(
    target: Principal,
    policy: EpistemicPolicy,
) -> Result<(), String> {
    let response = Call::unbounded_wait(target, "set_epistemic_policy")
        .with_arg(policy)
        .await
        .map_err(|err| {
            format!("cross-canister call failed while syncing policy to {target}: {err}")
        })?;

    let (result,): (Result<(), String>,) = response
        .candid_tuple()
        .map_err(|err| format!("failed to decode workflow_engine response: {err}"))?;

    result.map_err(|err| format!("workflow engine rejected policy sync: {err}"))
}

#[cfg(test)]
async fn push_policy_to_workflow_engine(
    _target: Principal,
    _policy: EpistemicPolicy,
) -> Result<(), String> {
    Err("transport unavailable in unit tests".to_string())
}

#[ic_cdk::update]
fn set_epistemic_policy(policy: EpistemicPolicy) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    set_epistemic_policy_internal(caller, policy)
}

#[ic_cdk::query]
fn get_epistemic_policy() -> Option<EpistemicPolicy> {
    EPISTEMIC_POLICY.with(|p| p.borrow().clone())
}

#[ic_cdk::update]
fn set_workflow_engine_target(target: Principal) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    set_workflow_engine_target_internal(caller, target)
}

#[ic_cdk::query]
fn get_workflow_engine_target() -> Option<Principal> {
    WORKFLOW_ENGINE_TARGET.with(|slot| *slot.borrow())
}

#[ic_cdk::update]
async fn sync_epistemic_policy_to_workflow_engine() -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    let (target, policy) = sync_preconditions(caller)?;
    normalize_sync_outcome(push_policy_to_workflow_engine(target, policy).await)
}

fn normalize_key(raw: &str) -> Option<String> {
    let key = raw.trim().to_ascii_lowercase();
    if key.is_empty() { None } else { Some(key) }
}

fn default_attribution_weight(domain_mode: &str) -> f64 {
    match domain_mode {
        "attributed" => 1.0,
        "pseudonymous" => 0.85,
        "delayed" => 0.75,
        "anonymous" => 0.60,
        _ => 0.50,
    }
}

#[ic_cdk::update]
fn set_attribution_weight_policy(
    domain_mode: String,
    policy: AttributionWeightPolicy,
) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    authorize_epistemic_policy_admin(caller)?;
    let key = normalize_key(&domain_mode).ok_or_else(|| "domain_mode is required".to_string())?;
    if !(0.0..=1.0).contains(&policy.weight) {
        return Err("policy.weight must be within 0.0..=1.0".to_string());
    }
    let mut normalized_policy = policy;
    normalized_policy.domain_mode = key.clone();
    normalized_policy.updated_at = now_unix_secs();
    ATTRIBUTION_WEIGHT_POLICIES.with(|store| {
        store.borrow_mut().insert(
            StorablePolicyKey(key),
            StorableAttributionWeightPolicy(normalized_policy),
        );
    });
    Ok(())
}

#[ic_cdk::query]
fn get_attribution_weight_policy(domain_mode: String) -> Option<AttributionWeightPolicy> {
    let key = normalize_key(&domain_mode)?;
    ATTRIBUTION_WEIGHT_POLICIES.with(|store| {
        store
            .borrow()
            .get(&StorablePolicyKey(key))
            .map(|entry| entry.0.clone())
    })
}

fn parse_policy_ref(input: &str) -> Option<(String, u64)> {
    let normalized = input.trim();
    let without_prefix = normalized.strip_prefix("governance_policy:")?;
    let (domain_mode, version_raw) = without_prefix.rsplit_once(':')?;
    let domain_mode = normalize_key(domain_mode)?;
    let version = version_raw.trim().parse::<u64>().ok()?;
    Some((domain_mode, version))
}

fn find_actor_role_binding(space_id: &str, principal: Principal) -> Option<ActorRoleBinding> {
    let key = actor_binding_key(space_id, principal)?;
    ACTOR_ROLE_BINDINGS.with(|store| store.borrow().get(&key).map(|entry| entry.0.clone()))
}

#[ic_cdk::update]
fn upsert_actor_role_binding(
    space_id: String,
    principal: Principal,
    role: String,
    source_ref: Option<String>,
) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    authorize_epistemic_policy_admin(caller)?;
    if principal == Principal::anonymous() {
        return Err("principal must be a concrete identity".to_string());
    }
    let space = normalize_key(&space_id).ok_or_else(|| "space_id is required".to_string())?;
    let normalized_role = normalize_key(&role).ok_or_else(|| "role is required".to_string())?;
    let key = actor_binding_key(&space, principal).ok_or_else(|| "space_id is required".to_string())?;
    let binding = ActorRoleBinding {
        space_id: space,
        principal,
        role: normalized_role,
        source_ref,
        updated_at: now_unix_secs(),
    };
    ACTOR_ROLE_BINDINGS.with(|store| {
        store
            .borrow_mut()
            .insert(key, StorableActorRoleBinding(binding));
    });
    Ok(())
}

#[ic_cdk::query]
fn get_actor_role_binding(space_id: String, principal: Principal) -> Option<ActorRoleBinding> {
    find_actor_role_binding(&space_id, principal)
}

#[ic_cdk::query]
fn list_actor_role_bindings(
    space_id: String,
    limit: u32,
    cursor: Option<Principal>,
) -> Vec<ActorRoleBinding> {
    let Some(space) = normalize_key(&space_id) else {
        return Vec::new();
    };
    let prefix = format!("{space}::");
    let cursor_text = cursor.map(|principal| principal.to_text());
    let mut entries = ACTOR_ROLE_BINDINGS.with(|store| {
        store
            .borrow()
            .iter()
            .filter(|entry| entry.key().0.starts_with(&prefix))
            .map(|entry| entry.value().0.clone())
            .collect::<Vec<_>>()
    });
    entries.sort_by(|left, right| left.principal.to_text().cmp(&right.principal.to_text()));
    if let Some(cursor_text) = cursor_text {
        entries.retain(|entry| entry.principal.to_text() > cursor_text);
    }
    entries.truncate(limit as usize);
    entries
}

#[ic_cdk::query]
fn get_policy_snapshot(policy_ref_value: String) -> Option<PolicySnapshot> {
    let (domain_mode, version) = parse_policy_ref(&policy_ref_value)?;
    let policy = ATTRIBUTION_WEIGHT_POLICIES.with(|store| {
        store
            .borrow()
            .get(&StorablePolicyKey(domain_mode.clone()))
            .map(|entry| entry.0.clone())
    })?;
    if policy.updated_at != version {
        return None;
    }
    Some(PolicySnapshot {
        policy_ref: policy_ref_value,
        domain_mode: policy.domain_mode,
        weight: policy.weight,
        allow_binding: policy.allow_binding,
        rationale: policy.rationale,
        policy_version: policy.updated_at,
        captured_at: now_unix_secs(),
    })
}

#[ic_cdk::query]
fn evaluate_action_scope(
    space_id: String,
    action_target: String,
    domain_mode: String,
) -> ActionScopeEvaluation {
    evaluate_action_scope_with_gate(space_id, action_target, domain_mode, "standard".to_string())
}

#[ic_cdk::query]
fn evaluate_action_scope_with_gate(
    space_id: String,
    action_target: String,
    domain_mode: String,
    gate_level: String,
) -> ActionScopeEvaluation {
    let normalized_mode = normalize_key(&domain_mode).unwrap_or_else(|| "unknown".to_string());
    let policy = ATTRIBUTION_WEIGHT_POLICIES.with(|store| {
        store
            .borrow()
            .get(&StorablePolicyKey(normalized_mode.clone()))
            .map(|entry| entry.0.clone())
    });
    let effective_weight = policy
        .as_ref()
        .map(|entry| entry.weight)
        .unwrap_or_else(|| default_attribution_weight(&normalized_mode));
    let allow_binding = policy.as_ref().map(|entry| entry.allow_binding).unwrap_or(true);

    let target = action_target.trim().to_ascii_lowercase();
    let governance_sensitive = target.contains("governance")
        || target.contains("constitution")
        || target.contains("role")
        || target.contains("scope");
    let release_sensitive = target.contains("release") || target.contains("deploy");
    let requires_review =
        governance_sensitive || release_sensitive || normalized_mode == "anonymous";

    let hard_gate = gate_level.trim().eq_ignore_ascii_case("release_blocker")
        || gate_level.trim().eq_ignore_ascii_case("hard_gate")
        || gate_level.trim().eq_ignore_ascii_case("hard");
    let allowed = allow_binding && effective_weight >= 0.35;
    let reason = if !allow_binding {
        "Domain policy disallows binding".to_string()
    } else if effective_weight < 0.35 {
        format!(
            "Effective weight {:.2} below minimum governance threshold 0.35",
            effective_weight
        )
    } else if requires_review {
        "Action allowed with mandatory review due to scope sensitivity".to_string()
    } else {
        "Action allowed".to_string()
    };
    let gate_decision = if !allowed || (hard_gate && requires_review) {
        "block".to_string()
    } else if requires_review {
        "review".to_string()
    } else {
        "allow".to_string()
    };
    let mut required_actions = Vec::<String>::new();
    if gate_decision == "review" {
        required_actions.push("decision_escalate".to_string());
    }
    if gate_decision == "block" {
        required_actions.push("decision_escalate".to_string());
        required_actions.push("decision_ack".to_string());
    }
    let policy_ref = policy
        .as_ref()
        .map(|entry| policy_ref(&entry.domain_mode, entry.updated_at));
    let policy_version = policy.as_ref().map(|entry| entry.updated_at).unwrap_or(0);

    let normalized_space = normalize_key(&space_id).unwrap_or_else(|| "unknown".to_string());

    let mut evaluation = ActionScopeEvaluation {
        allowed: gate_decision != "block" && allowed,
        reason,
        effective_weight,
        requires_review,
        gate_decision,
        required_actions,
        policy_ref,
        policy_version,
    };
    if normalized_space == "unknown" {
        evaluation.requires_review = true;
        if evaluation.gate_decision == "allow" {
            evaluation.gate_decision = "review".to_string();
        }
        if !evaluation
            .required_actions
            .contains(&"set_space_context".to_string())
        {
            evaluation.required_actions.push("set_space_context".to_string());
        }
        evaluation.reason = format!("{}; missing normalized space context", evaluation.reason);
    }
    evaluation
}

#[ic_cdk::query]
fn evaluate_action_scope_with_actor(
    space_id: String,
    action_target: String,
    domain_mode: String,
    gate_level: String,
    actor_principal: Principal,
) -> ActionScopeEvaluation {
    let mut base = evaluate_action_scope_with_gate(
        space_id.clone(),
        action_target,
        domain_mode,
        gate_level.clone(),
    );
    if actor_principal == Principal::anonymous() {
        base.allowed = false;
        base.requires_review = true;
        base.gate_decision = "block".to_string();
        if !base
            .required_actions
            .contains(&"bind_actor_principal".to_string())
        {
            base.required_actions.push("bind_actor_principal".to_string());
        }
        base.reason = format!("{}; anonymous actor principal is not allowed", base.reason);
        return base;
    }

    let binding = find_actor_role_binding(&space_id, actor_principal);
    match binding {
        Some(binding) if matches!(binding.role.as_str(), "viewer" | "observer") => {
            base.allowed = false;
            base.requires_review = true;
            base.gate_decision = "block".to_string();
            if !base.required_actions.contains(&"decision_escalate".to_string()) {
                base.required_actions.push("decision_escalate".to_string());
            }
            base.reason = format!(
                "{}; role {} cannot execute high-impact decisions",
                base.reason, binding.role
            );
        }
        Some(_) => {}
        None => {
            let hard_gate = gate_level.trim().eq_ignore_ascii_case("release_blocker")
                || gate_level.trim().eq_ignore_ascii_case("hard_gate")
                || gate_level.trim().eq_ignore_ascii_case("hard");
            if hard_gate {
                base.allowed = false;
                base.requires_review = true;
                base.gate_decision = "review".to_string();
                if !base
                    .required_actions
                    .contains(&"upsert_actor_role_binding".to_string())
                {
                    base.required_actions
                        .push("upsert_actor_role_binding".to_string());
                }
                base.reason = format!(
                    "{}; actor role binding is missing for principal {}",
                    base.reason,
                    actor_principal.to_text()
                );
            }
        }
    }
    base.required_actions.sort();
    base.required_actions.dedup();
    base
}

fn evaluate_policy(policy: &Policy, context: &HashMap<String, Value>) -> ComplianceResult {
    match policy.rule_type.as_str() {
        "EpistemicConfidence" => {
            // Param: {"min_confidence": 0.5, "assumption_id_field": "assumption_id"}
            let params: Value = serde_json::from_str(&policy.parameters).unwrap_or(Value::Null);
            let min_conf = params
                .get("min_confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let current_conf = context
                .get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            if current_conf < min_conf {
                return ComplianceResult::RequiresReview(format!(
                    "Confidence {} below minimum {}",
                    current_conf, min_conf
                ));
            }
            ComplianceResult::Pass
        }
        "VoteThreshold" => {
            // Simple placeholder
            ComplianceResult::Pass
        }
        _ => ComplianceResult::Pass, // Unknown policies are permissive for now (or strictly fail?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_policy() -> EpistemicPolicy {
        EpistemicPolicy {
            mode: EpistemicMode::Observe,
            min_evidence: 2,
            min_alternatives: 2,
            min_robustness: 0.7,
            max_confidence_drift: 0.25,
            max_fork_pressure: 0.60,
            max_correction_density: 0.50,
            simulation_ttl_days: 30,
            enforced_decision_classes: vec![DecisionClass::Governance, DecisionClass::Merge],
            block_on_soft: false,
        }
    }

    fn reset_state() {
        EPISTEMIC_POLICY.with(|slot| *slot.borrow_mut() = None);
        ATTRIBUTION_WEIGHT_POLICIES.with(|slot| {
            let keys = slot
                .borrow()
                .iter()
                .map(|entry| entry.key().clone())
                .collect::<Vec<_>>();
            let mut store = slot.borrow_mut();
            for key in keys {
                store.remove(&key);
            }
        });
        ACTOR_ROLE_BINDINGS.with(|slot| {
            let keys = slot
                .borrow()
                .iter()
                .map(|entry| entry.key().clone())
                .collect::<Vec<_>>();
            let mut store = slot.borrow_mut();
            for key in keys {
                store.remove(&key);
            }
        });
        EPISTEMIC_POLICY_ADMIN.with(|slot| *slot.borrow_mut() = None);
        WORKFLOW_ENGINE_TARGET.with(|slot| *slot.borrow_mut() = None);
    }

    #[test]
    fn test_policy_threshold_validation() {
        let mut policy = sample_policy();
        policy.max_fork_pressure = 1.1;
        assert!(validate_epistemic_policy_thresholds(&policy).is_err());

        policy.max_fork_pressure = 0.5;
        assert!(validate_epistemic_policy_thresholds(&policy).is_ok());
    }

    #[test]
    fn test_admin_authorization_transition() {
        reset_state();
        let alice = Principal::from_slice(&[1u8; 29]);
        let bob = Principal::from_slice(&[2u8; 29]);

        assert!(set_epistemic_policy_internal(alice, sample_policy()).is_ok());
        assert!(set_epistemic_policy_internal(bob, sample_policy()).is_err());
    }

    #[test]
    fn test_workflow_target_validation() {
        reset_state();
        let alice = Principal::from_slice(&[3u8; 29]);
        assert!(set_workflow_engine_target_internal(alice, Principal::anonymous()).is_err());

        let target = Principal::from_slice(&[4u8; 29]);
        assert!(set_workflow_engine_target_internal(alice, target).is_ok());
        assert_eq!(get_workflow_engine_target(), Some(target));
    }

    #[test]
    fn test_sync_preconditions_require_target() {
        reset_state();
        let alice = Principal::from_slice(&[5u8; 29]);
        assert!(set_epistemic_policy_internal(alice, sample_policy()).is_ok());

        let preflight = sync_preconditions(alice);
        assert!(preflight.is_err());
        assert!(preflight.unwrap_err().contains("target"));
    }

    #[test]
    fn test_sync_error_normalization() {
        let normalized = normalize_sync_outcome(Err("downstream rejected".to_string()));
        assert!(normalized.is_err());
        assert!(normalized
            .unwrap_err()
            .contains("workflow_engine sync failed"));
    }

    #[test]
    fn evaluate_action_scope_defaults_for_anonymous_domain() {
        reset_state();
        let evaluation = evaluate_action_scope(
            "space-main".to_string(),
            "governance:approve".to_string(),
            "anonymous".to_string(),
        );
        assert!(evaluation.allowed);
        assert!(evaluation.requires_review);
        assert!(evaluation.effective_weight >= 0.5);
        assert_eq!(evaluation.gate_decision, "review");
    }

    #[test]
    fn evaluate_action_scope_blocks_when_policy_disallows_binding() {
        reset_state();
        ATTRIBUTION_WEIGHT_POLICIES.with(|store| {
            store.borrow_mut().insert(
                StorablePolicyKey("anonymous".to_string()),
                StorableAttributionWeightPolicy(AttributionWeightPolicy {
                    domain_mode: "anonymous".to_string(),
                    weight: 0.2,
                    allow_binding: false,
                    rationale: "fixture".to_string(),
                    updated_at: 0,
                }),
            );
        });
        let evaluation = evaluate_action_scope(
            "space-main".to_string(),
            "release:promote".to_string(),
            "anonymous".to_string(),
        );
        assert!(!evaluation.allowed);
        assert_eq!(evaluation.gate_decision, "block");
        assert!(evaluation.reason.contains("disallows binding"));
    }

    #[test]
    fn evaluate_action_scope_with_gate_blocks_review_only_in_hard_mode() {
        reset_state();
        let evaluation = evaluate_action_scope_with_gate(
            "space-main".to_string(),
            "governance:approve".to_string(),
            "anonymous".to_string(),
            "release_blocker".to_string(),
        );
        assert!(!evaluation.allowed);
        assert_eq!(evaluation.gate_decision, "block");
        assert!(evaluation.required_actions.contains(&"decision_escalate".to_string()));
    }

    #[test]
    fn evaluate_action_scope_with_actor_requires_binding_for_hard_gate() {
        reset_state();
        let actor = Principal::from_slice(&[9u8; 29]);
        let evaluation = evaluate_action_scope_with_actor(
            "space-main".to_string(),
            "governance:approve".to_string(),
            "attributed".to_string(),
            "release_blocker".to_string(),
            actor,
        );
        assert!(!evaluation.allowed);
        assert_eq!(evaluation.gate_decision, "review");
        assert!(evaluation
            .required_actions
            .contains(&"upsert_actor_role_binding".to_string()));
    }

    #[test]
    fn get_policy_snapshot_returns_matching_policy() {
        reset_state();
        ATTRIBUTION_WEIGHT_POLICIES.with(|store| {
            store.borrow_mut().insert(
                StorablePolicyKey("pseudonymous".to_string()),
                StorableAttributionWeightPolicy(AttributionWeightPolicy {
                    domain_mode: "pseudonymous".to_string(),
                    weight: 0.8,
                    allow_binding: true,
                    rationale: "fixture".to_string(),
                    updated_at: 42,
                }),
            );
        });

        let snapshot = get_policy_snapshot("governance_policy:pseudonymous:42".to_string())
            .expect("snapshot");
        assert_eq!(snapshot.policy_version, 42);
        assert_eq!(snapshot.domain_mode, "pseudonymous");
    }
}

ic_cdk::export_candid!();
