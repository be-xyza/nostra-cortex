use crate::services::local_gateway::{Mutation, get_gateway};
use cortex_runtime::RuntimeError;
use cortex_runtime::gateway::local::{
    LocalGatewayMutationRecord, LocalGatewayMutationSubmit, LocalGatewayProbe,
    LocalGatewayQueueAction,
};
use cortex_runtime::ports::LocalGatewayOrchestrationAdapter;
use std::sync::LazyLock;

pub struct DesktopLocalGatewayAdapter;

static LOCAL_GATEWAY_ADAPTER: LazyLock<DesktopLocalGatewayAdapter> =
    LazyLock::new(|| DesktopLocalGatewayAdapter);

pub fn local_gateway_adapter() -> &'static DesktopLocalGatewayAdapter {
    &LOCAL_GATEWAY_ADAPTER
}

impl LocalGatewayOrchestrationAdapter for DesktopLocalGatewayAdapter {
    fn queue_snapshot(&self) -> Result<Vec<LocalGatewayMutationRecord>, RuntimeError> {
        let queue = get_gateway().queue_snapshot();
        Ok(queue.into_iter().map(map_mutation).collect())
    }

    fn export_queue_json(&self) -> Result<String, RuntimeError> {
        get_gateway()
            .export_queue_json()
            .map_err(RuntimeError::Storage)
    }

    fn apply_queue_action(
        &self,
        mutation_id: &str,
        action: LocalGatewayQueueAction,
    ) -> Result<(), RuntimeError> {
        let gateway = get_gateway();
        match action {
            LocalGatewayQueueAction::Retry => gateway.retry_mutation(mutation_id),
            LocalGatewayQueueAction::Discard => gateway.discard_mutation(mutation_id),
            LocalGatewayQueueAction::Fork => gateway.mark_fork_needed(mutation_id),
        }
        .map_err(RuntimeError::Domain)
    }

    fn probe(&self) -> Result<LocalGatewayProbe, RuntimeError> {
        let gateway = get_gateway();
        Ok(LocalGatewayProbe {
            queue_size: gateway.get_queue_size(),
            queue_export_ok: gateway.export_queue_json().is_ok(),
            gateway_online: gateway.is_network_online(),
        })
    }

    fn set_online(&self, status: bool) -> Result<(), RuntimeError> {
        get_gateway().set_online(status);
        Ok(())
    }

    fn is_online(&self) -> Result<bool, RuntimeError> {
        Ok(get_gateway().is_network_online())
    }

    fn submit_mutation(
        &self,
        mutation: LocalGatewayMutationSubmit,
    ) -> Result<String, RuntimeError> {
        let local = Mutation {
            id: mutation.mutation_id,
            idempotency_key: mutation.idempotency_key,
            space_id: mutation.space_id,
            kip_command: mutation.kip_command,
            timestamp: mutation.timestamp,
            attempts: mutation.attempts,
            last_error: mutation.last_error,
            last_attempt_at: mutation.last_attempt_at,
            preconditions: None,
        };
        get_gateway()
            .submit_mutation(local)
            .map_err(RuntimeError::Domain)
    }
}

fn map_mutation(mutation: Mutation) -> LocalGatewayMutationRecord {
    let conflict_state = mutation
        .last_error
        .as_ref()
        .map(|value| {
            let normalized = value.to_ascii_lowercase();
            normalized.contains("conflict")
                || normalized.contains("fork required")
                || normalized.contains("already executed")
        })
        .unwrap_or(false);

    LocalGatewayMutationRecord {
        mutation_id: mutation.id,
        idempotency_key: mutation.idempotency_key,
        space_id: mutation.space_id,
        kip_command: mutation.kip_command,
        timestamp: mutation.timestamp,
        attempts: mutation.attempts,
        last_error: mutation.last_error,
        last_attempt_at: mutation.last_attempt_at,
        conflict_state,
    }
}
