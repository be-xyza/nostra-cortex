use cortex_ic_adapter::workflow::WorkflowCanisterClient;
pub use cortex_runtime::ports::{
    AttributionDomain, DecisionLineage, EpistemicAssessment, ExecutionProfile, FileMetadata,
    GateOutcome, ReplayContract, WorkflowAdapter,
};

#[derive(Clone, Debug)]
pub struct WorkflowEngineClient {
    inner: WorkflowCanisterClient,
}

impl WorkflowEngineClient {
    pub async fn from_env() -> Result<Self, String> {
        let inner = WorkflowCanisterClient::from_env().await.map_err(|err| err.to_string())?;
        Ok(Self { inner })
    }

    pub async fn get_space_execution_profile(
        &self,
        space_id: &str,
    ) -> Result<Option<ExecutionProfile>, String> {
        self.inner
            .get_space_execution_profile(space_id)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn get_attribution_domains(
        &self,
        space_id: &str,
    ) -> Result<Vec<AttributionDomain>, String> {
        self.inner
            .get_attribution_domains(space_id)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn get_replay_contract(
        &self,
        mutation_id: &str,
    ) -> Result<Option<ReplayContract>, String> {
        self.inner
            .get_replay_contract(mutation_id)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn get_epistemic_assessment_by_mutation(
        &self,
        mutation_id: &str,
    ) -> Result<Option<EpistemicAssessment>, String> {
        self.inner
            .get_epistemic_assessment_by_mutation(mutation_id)
            .await
            .map_err(|err| err.to_string())
    }

    #[cfg(feature = "service-scaffolds")]
    pub async fn get_contribution_attribution_binding(
        &self,
        contribution_id: &str,
    ) -> Result<Option<cortex_runtime::ports::ContributionAttributionBinding>, String> {
        self.inner
            .get_contribution_attribution_binding(contribution_id)
            .await
            .map_err(|err| err.to_string())
    }

    #[cfg(feature = "service-scaffolds")]
    pub async fn list_space_replay_contracts(
        &self,
        space_id: &str,
        limit: u32,
    ) -> Result<Vec<ReplayContract>, String> {
        self.inner
            .list_space_replay_contracts(space_id, limit)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn get_decision_lineage_by_mutation(
        &self,
        mutation_id: &str,
    ) -> Result<Option<DecisionLineage>, String> {
        self.inner
            .get_decision_lineage_by_mutation(mutation_id)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn list_space_decision_lineage(
        &self,
        space_id: &str,
        limit: u32,
    ) -> Result<Vec<DecisionLineage>, String> {
        self.inner
            .list_space_decision_lineage(space_id, limit)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
        mime_type: &str,
    ) -> Result<(), String> {
        self.inner
            .write_file(path, content, mime_type)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        self.inner
            .read_file(path)
            .await
            .map_err(|err| err.to_string())
    }

    pub async fn list_files(&self, prefix: &str) -> Result<Vec<(String, FileMetadata)>, String> {
        self.inner
            .list_files(prefix)
            .await
            .map_err(|err| err.to_string())
    }
}
