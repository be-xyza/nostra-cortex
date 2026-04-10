use candid::CandidType;
use nostra_workflow_core::WorkflowStatus;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, Default)]
pub struct SystemStatus {
    pub active_workflows: u32,
    pub completed_workflows: u32,
    pub failed_workflows: u32,
    pub simulation_runs: u32,
    pub memory_usage_bytes: u64,
    pub cycles_balance: u64,
}

thread_local! {
    static SYSTEM_STATUS: RefCell<SystemStatus> = RefCell::new(SystemStatus::default());
}

pub fn record_workflow_status_change(status: &WorkflowStatus) {
    SYSTEM_STATUS.with(|state| {
        let mut status_state = state.borrow_mut();
        match status {
            WorkflowStatus::Running => status_state.active_workflows += 1,
            WorkflowStatus::Completed => {
                status_state.active_workflows = status_state.active_workflows.saturating_sub(1);
                status_state.completed_workflows += 1;
            }
            WorkflowStatus::Failed(_) => {
                status_state.active_workflows = status_state.active_workflows.saturating_sub(1);
                status_state.failed_workflows += 1;
            }
            _ => {}
        }
    });
}

#[allow(dead_code)]
pub fn record_simulation_run() {
    SYSTEM_STATUS.with(|state| {
        state.borrow_mut().simulation_runs += 1;
    });
}

#[ic_cdk::query]
pub fn get_system_status() -> SystemStatus {
    SYSTEM_STATUS.with(|state| {
        #[cfg(target_arch = "wasm32")]
        let mut status = state.borrow().clone();
        #[cfg(not(target_arch = "wasm32"))]
        let status = state.borrow().clone();

        #[cfg(target_arch = "wasm32")]
        {
            status.memory_usage_bytes = (core::arch::wasm32::memory_size(0) as u64) * 65536;
            status.cycles_balance = ic_cdk::api::canister_balance();
        }

        status
    })
}
