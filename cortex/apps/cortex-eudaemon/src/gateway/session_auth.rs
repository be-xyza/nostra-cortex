use crate::gateway::server::workspace_root;
use crate::services::authz::valid_role as authz_valid_role;
use axum::http::HeaderMap;
use cortex_domain::spaces::SpaceRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;

pub const GATEWAY_SESSION_COOKIE: &str = "cortex_session_id";
const SESSION_STORE_RELATIVE_PATH: &str = "_cortex/session_store.json";
const SESSION_SCHEMA_VERSION: &str = "1.0.0";
const ROLE_ORDER: [&str; 5] = ["viewer", "editor", "operator", "steward", "admin"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRoleGrant {
    pub space_id: String,
    pub roles: Vec<String>,
    #[serde(default)]
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthSession {
    pub schema_version: String,
    pub generated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    pub session_id: String,
    pub identity_verified: bool,
    pub identity_source: String,
    pub auth_mode: String,
    pub granted_roles: Vec<String>,
    pub active_role: String,
    #[serde(default)]
    pub global_claims: Vec<String>,
    #[serde(default)]
    pub space_grants: Vec<SpaceRoleGrant>,
    pub allow_role_switch: bool,
    pub allow_unverified_role_header: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySessionRecord {
    pub schema_version: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id_hint: Option<String>,
    pub identity_verified: bool,
    pub identity_source: String,
    pub auth_mode: String,
    pub granted_roles: Vec<String>,
    pub active_role: String,
    #[serde(default)]
    pub global_claims: Vec<String>,
    #[serde(default)]
    pub space_grants: Vec<SpaceRoleGrant>,
    pub allow_role_switch: bool,
    pub allow_unverified_role_header: bool,
    pub issued_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct GatewaySessionStore {
    schema_version: String,
    updated_at: String,
    sessions: BTreeMap<String, GatewaySessionRecord>,
}

impl Default for GatewaySessionStore {
    fn default() -> Self {
        Self {
            schema_version: SESSION_SCHEMA_VERSION.to_string(),
            updated_at: "1970-01-01T00:00:00Z".to_string(),
            sessions: BTreeMap::new(),
        }
    }
}

fn store_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn session_store_path() -> PathBuf {
    workspace_root().join(SESSION_STORE_RELATIVE_PATH)
}

fn load_store() -> GatewaySessionStore {
    let path = session_store_path();
    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(_) => return GatewaySessionStore::default(),
    };
    serde_json::from_str::<GatewaySessionStore>(&raw).unwrap_or_default()
}

fn save_store(store: &GatewaySessionStore) -> Result<(), String> {
    let path = session_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let body = serde_json::to_string_pretty(store).map_err(|err| err.to_string())?;
    fs::write(&path, body).map_err(|err| err.to_string())
}

pub fn role_rank(role: &str) -> usize {
    ROLE_ORDER
        .iter()
        .position(|candidate| candidate == &role)
        .unwrap_or(0)
}

pub fn canonicalize_roles<I>(roles: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut unique = BTreeSet::new();
    for role in roles {
        let normalized = role.trim().to_ascii_lowercase();
        if authz_valid_role(&normalized) {
            unique.insert(normalized);
        }
    }
    let mut ordered = unique.into_iter().collect::<Vec<_>>();
    ordered.sort_by_key(|role| role_rank(role));
    ordered
}

pub fn role_hierarchy(role: &str) -> Vec<String> {
    let normalized = role.trim().to_ascii_lowercase();
    let ceiling = role_rank(&normalized);
    ROLE_ORDER
        .iter()
        .enumerate()
        .filter(|(index, _)| *index <= ceiling)
        .map(|(_, role)| (*role).to_string())
        .collect()
}

pub fn auth_mode_from_identity_source(source: &str) -> String {
    let normalized = source.trim().to_ascii_lowercase();
    if normalized.contains("principal_binding") || normalized.contains("signed_principal_claim") {
        "principal_binding".to_string()
    } else if normalized.contains("session_claims") {
        "session_claims".to_string()
    } else if normalized.contains("dev_unverified_header") {
        "dev_override".to_string()
    } else {
        "read_fallback".to_string()
    }
}

fn load_registered_space_ids() -> Vec<String> {
    let mut ids = vec!["meta".to_string()];
    let registry_path = workspace_root().join("_spaces").join("registry.json");
    if let Ok(registry) = SpaceRegistry::load_from_path(&registry_path) {
        for space_id in registry.spaces.keys() {
            if !ids.contains(space_id) {
                ids.push(space_id.clone());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn parse_dev_session_roles() -> Vec<String> {
    let raw = match std::env::var("NOSTRA_AUTHZ_DEV_SESSION_ROLES") {
        Ok(value) => value,
        Err(_) => return ROLE_ORDER.iter().map(|role| (*role).to_string()).collect(),
    };
    let normalized = raw.trim();
    if normalized.is_empty() {
        return ROLE_ORDER.iter().map(|role| (*role).to_string()).collect();
    }

    let roles = if normalized.starts_with('[') {
        serde_json::from_str::<Vec<String>>(normalized).unwrap_or_default()
    } else {
        normalized
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect()
    };

    let canonical = canonicalize_roles(roles);
    if canonical.is_empty() {
        ROLE_ORDER.iter().map(|role| (*role).to_string()).collect()
    } else {
        canonical
    }
}

fn parse_space_grants_from_env(
    default_roles: &[String],
    default_claims: &[String],
) -> Option<Vec<SpaceRoleGrant>> {
    let raw = std::env::var("NOSTRA_AUTHZ_SPACE_ROLE_GRANTS").ok()?;
    let parsed = serde_json::from_str::<Value>(&raw).ok()?;
    let object = parsed.as_object()?;

    let mut grants = Vec::new();
    for (space_id, value) in object {
        if space_id.trim().is_empty() {
            continue;
        }

        let (roles, claims) = match value {
            Value::Array(items) => (
                canonicalize_roles(
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                ),
                default_claims.to_vec(),
            ),
            Value::Object(map) => {
                let roles = canonicalize_roles(
                    map.get("roles")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                );
                let claims = map
                    .get("claims")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                (roles, if claims.is_empty() { default_claims.to_vec() } else { claims })
            }
            _ => continue,
        };

        let roles = if roles.is_empty() {
            default_roles.to_vec()
        } else {
            roles
        };
        grants.push(SpaceRoleGrant {
            space_id: space_id.clone(),
            roles,
            claims,
        });
    }

    Some(grants)
}

fn build_space_grants(default_roles: &[String], default_claims: &[String]) -> Vec<SpaceRoleGrant> {
    if let Some(grants) = parse_space_grants_from_env(default_roles, default_claims) {
        return grants;
    }

    load_registered_space_ids()
        .into_iter()
        .map(|space_id| SpaceRoleGrant {
            space_id,
            roles: default_roles.to_vec(),
            claims: default_claims.to_vec(),
        })
        .collect()
}

pub fn resolve_session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for fragment in cookie_header.split(';') {
        let mut parts = fragment.trim().splitn(2, '=');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim();
        if key == GATEWAY_SESSION_COOKIE && !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

pub fn set_cookie_value(session_id: &str) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax",
        name = GATEWAY_SESSION_COOKIE,
        value = session_id
    )
}

pub fn load_session_from_headers(headers: &HeaderMap) -> Option<GatewaySessionRecord> {
    let session_id = resolve_session_cookie(headers)?;
    load_session_by_id(&session_id)
}

pub fn load_session_by_id(session_id: &str) -> Option<GatewaySessionRecord> {
    let _guard = store_lock().lock().ok()?;
    let store = load_store();
    store.sessions.get(session_id).cloned()
}

pub fn build_session_record(
    existing_session_id: Option<String>,
    principal: Option<String>,
    actor_id_hint: Option<String>,
    identity_verified: bool,
    identity_source: String,
    active_role_hint: String,
    global_claims: Vec<String>,
    allow_unverified_role_header: bool,
    generated_at: String,
) -> GatewaySessionRecord {
    let auth_mode = auth_mode_from_identity_source(&identity_source);
    let granted_roles = match auth_mode.as_str() {
        "dev_override" => parse_dev_session_roles(),
        "principal_binding" | "session_claims" => role_hierarchy(&active_role_hint),
        _ => vec!["viewer".to_string()],
    };
    let default_role = if granted_roles.iter().any(|role| role == &active_role_hint) {
        active_role_hint
    } else {
        granted_roles
            .last()
            .cloned()
            .unwrap_or_else(|| "viewer".to_string())
    };
    let space_grants = build_space_grants(&granted_roles, &global_claims);

    GatewaySessionRecord {
        schema_version: SESSION_SCHEMA_VERSION.to_string(),
        session_id: existing_session_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        principal,
        actor_id_hint,
        identity_verified,
        identity_source,
        auth_mode,
        granted_roles: granted_roles.clone(),
        active_role: default_role,
        global_claims,
        space_grants,
        allow_role_switch: granted_roles.len() > 1,
        allow_unverified_role_header,
        issued_at: generated_at.clone(),
        updated_at: generated_at,
        expires_at: None,
    }
}

pub fn upsert_session_record(mut record: GatewaySessionRecord) -> Result<GatewaySessionRecord, String> {
    let _guard = store_lock().lock().map_err(|err| err.to_string())?;
    let mut store = load_store();
    if let Some(existing) = store.sessions.get(&record.session_id) {
        record.issued_at = existing.issued_at.clone();
    }
    store.updated_at = record.updated_at.clone();
    store
        .sessions
        .insert(record.session_id.clone(), record.clone());
    save_store(&store)?;
    Ok(record)
}

fn role_is_granted(record: &GatewaySessionRecord, role: &str, space_id: Option<&str>) -> bool {
    let normalized = role.trim().to_ascii_lowercase();
    if let Some(space_id) = space_id {
        if let Some(grant) = record
            .space_grants
            .iter()
            .find(|grant| grant.space_id == space_id)
        {
            return grant.roles.iter().any(|candidate| candidate == &normalized);
        }
    }
    record
        .granted_roles
        .iter()
        .any(|candidate| candidate == &normalized)
}

pub fn switch_active_role(
    session_id: &str,
    role: &str,
    space_id: Option<&str>,
    updated_at: String,
) -> Result<GatewaySessionRecord, String> {
    let normalized = role.trim().to_ascii_lowercase();
    if !authz_valid_role(&normalized) {
        return Err("invalid_role".to_string());
    }

    let _guard = store_lock().lock().map_err(|err| err.to_string())?;
    let mut store = load_store();
    let record = store
        .sessions
        .get_mut(session_id)
        .ok_or_else(|| "session_not_found".to_string())?;

    if !role_is_granted(record, &normalized, space_id) {
        return Err("role_not_granted".to_string());
    }

    record.active_role = normalized;
    record.updated_at = updated_at.clone();
    store.updated_at = updated_at;
    let updated = record.clone();
    save_store(&store)?;
    Ok(updated)
}

pub fn project_auth_session(record: &GatewaySessionRecord, generated_at: String) -> AuthSession {
    AuthSession {
        schema_version: record.schema_version.clone(),
        generated_at,
        principal: record.principal.clone(),
        session_id: record.session_id.clone(),
        identity_verified: record.identity_verified,
        identity_source: record.identity_source.clone(),
        auth_mode: record.auth_mode.clone(),
        granted_roles: record.granted_roles.clone(),
        active_role: record.active_role.clone(),
        global_claims: record.global_claims.clone(),
        space_grants: record.space_grants.clone(),
        allow_role_switch: record.allow_role_switch,
        allow_unverified_role_header: record.allow_unverified_role_header,
        expires_at: record.expires_at.clone(),
    }
}
