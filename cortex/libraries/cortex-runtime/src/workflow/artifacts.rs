use crate::ports::{TimeProvider, UxContractStoreAdapter};
use crate::{RuntimeError, workflow::digest::workflow_digest_hex};
use cortex_domain::workflow::{
    WorkflowCandidateSet, WorkflowConstraintRule, WorkflowGenerationMode, WorkflowMotifKind,
    WorkflowScope, generate_candidate_set,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct WorkflowArtifactRuntime<'a> {
    store: &'a dyn UxContractStoreAdapter,
    time: &'a dyn TimeProvider,
}

impl<'a> WorkflowArtifactRuntime<'a> {
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

    #[allow(clippy::too_many_arguments)]
    pub fn generate_candidate_set(
        &self,
        scope: WorkflowScope,
        intent: &str,
        motif_kind: WorkflowMotifKind,
        constraints: &[WorkflowConstraintRule],
        count: usize,
        created_by: &str,
        source_mode: &str,
        mode: WorkflowGenerationMode,
        candidate_set_id: Option<String>,
    ) -> Result<WorkflowCandidateSet, RuntimeError> {
        let created_at = self.now_iso()?;
        let seed = self.deterministic_seed("workflow_candidate");
        let set_id = candidate_set_id.unwrap_or_else(|| self.deterministic_seed("workflow_set"));
        Ok(generate_candidate_set(
            scope,
            intent,
            motif_kind,
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
        workflow_digest_hex(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{TimeProvider, UxContractStoreAdapter};
    use async_trait::async_trait;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    struct FixedTime;

    impl TimeProvider for FixedTime {
        fn now_unix_secs(&self) -> u64 {
            1_741_651_200
        }

        fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
            Ok(format!("{unix_secs}Z"))
        }
    }

    #[derive(Default)]
    struct MemoryStore {
        values: Arc<Mutex<BTreeMap<String, String>>>,
    }

    #[async_trait]
    impl UxContractStoreAdapter for MemoryStore {
        async fn read_text(&self, key: &str) -> Result<Option<String>, RuntimeError> {
            Ok(self.values.lock().unwrap().get(key).cloned())
        }

        async fn write_text(
            &self,
            key: &str,
            content: &str,
            _mime_type: &str,
        ) -> Result<(), RuntimeError> {
            self.values
                .lock()
                .unwrap()
                .insert(key.to_string(), content.to_string());
            Ok(())
        }

        async fn append_line(
            &self,
            key: &str,
            line: &str,
            _mime_type: &str,
        ) -> Result<(), RuntimeError> {
            let mut values = self.values.lock().unwrap();
            let current = values.entry(key.to_string()).or_default();
            current.push_str(line);
            current.push('\n');
            Ok(())
        }

        async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, RuntimeError> {
            let values = self.values.lock().unwrap();
            Ok(values
                .keys()
                .filter(|key| key.starts_with(prefix))
                .cloned()
                .collect())
        }
    }

    #[tokio::test]
    async fn candidate_generation_and_write_are_deterministic() {
        let store = MemoryStore::default();
        let time = FixedTime;
        let runtime = WorkflowArtifactRuntime::new(&store, &time);
        let set = runtime
            .generate_candidate_set(
                WorkflowScope::default(),
                "parallel compare",
                WorkflowMotifKind::ParallelCompare,
                &[],
                1,
                "tester",
                "human",
                WorkflowGenerationMode::DeterministicScaffold,
                Some("candidate_set_1".to_string()),
            )
            .unwrap();
        assert_eq!(set.candidate_set_id, "candidate_set_1");
        assert_eq!(set.candidates.len(), 1);

        runtime
            .write_json("/tmp/workflow-candidate.json", &set, "application/json")
            .await
            .unwrap();
        let restored: WorkflowCandidateSet = runtime
            .read_json("/tmp/workflow-candidate.json")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(restored.candidate_set_id, set.candidate_set_id);
    }
}
