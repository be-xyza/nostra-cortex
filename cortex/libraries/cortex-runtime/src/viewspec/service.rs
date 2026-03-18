use crate::ports::{TimeProvider, UxContractStoreAdapter};
use crate::{RuntimeError, viewspec::digest::viewspec_digest_hex};
use cortex_domain::viewspec::{
    ConstraintRule, ViewSpecCandidateSet, ViewSpecGenerationMode, ViewSpecScope,
    generate_candidate_set,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct ViewSpecRuntime<'a> {
    store: &'a dyn UxContractStoreAdapter,
    time: &'a dyn TimeProvider,
}

impl<'a> ViewSpecRuntime<'a> {
    pub fn new(store: &'a dyn UxContractStoreAdapter, time: &'a dyn TimeProvider) -> Self {
        Self { store, time }
    }

    pub fn now_iso(&self) -> Result<String, RuntimeError> {
        let now = self.time.now_unix_secs();
        self.time.to_rfc3339(now)
    }

    pub fn deterministic_seed(&self, prefix: &str) -> String {
        format!("{}_{}", prefix, self.time.now_unix_secs())
    }

    pub fn generate_candidate_set(
        &self,
        scope: ViewSpecScope,
        intent: &str,
        constraints: &[ConstraintRule],
        count: usize,
        created_by: &str,
        source_mode: &str,
        mode: ViewSpecGenerationMode,
        candidate_set_id: Option<String>,
    ) -> Result<ViewSpecCandidateSet, RuntimeError> {
        let created_at = self.now_iso()?;
        let seed = self.deterministic_seed("viewspec_candidate");
        let set_id = candidate_set_id.unwrap_or_else(|| self.deterministic_seed("viewspec_set"));
        Ok(generate_candidate_set(
            scope,
            intent,
            constraints,
            count,
            created_by,
            source_mode,
            mode,
            &set_id,
            &created_at,
            &seed,
        ))
    }

    pub async fn write_json<T: Serialize>(
        &self,
        key: &str,
        payload: &T,
        mime_type: &str,
    ) -> Result<(), RuntimeError> {
        let encoded = serde_json::to_string(payload)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        self.store.write_text(key, &encoded, mime_type).await
    }

    pub async fn read_json<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, RuntimeError> {
        let raw = self.store.read_text(key).await?;
        match raw {
            Some(value) => serde_json::from_str::<T>(&value)
                .map(Some)
                .map_err(|err| RuntimeError::Serialization(err.to_string())),
            None => Ok(None),
        }
    }

    pub fn digest_payload(&self, payload: &Value) -> String {
        viewspec_digest_hex(payload)
    }
}
