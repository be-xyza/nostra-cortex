use crate::RuntimeError;
use async_trait::async_trait;
use cortex_domain::workflow::{
    WorkflowCheckpointResultV1, WorkflowDefinitionV1, WorkflowExecutionBindingV1,
    WorkflowExecutionPlanV1, WorkflowInstanceV1, WorkflowSignalV1, WorkflowSnapshotV1,
};

#[async_trait]
pub trait WorkflowExecutionAdapter: Send + Sync {
    async fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, RuntimeError>;

    async fn start(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, RuntimeError>;

    async fn signal(
        &self,
        instance_id: &str,
        signal: WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError>;

    async fn snapshot(&self, instance_id: &str) -> Result<WorkflowSnapshotV1, RuntimeError>;

    async fn cancel(
        &self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError>;
}
