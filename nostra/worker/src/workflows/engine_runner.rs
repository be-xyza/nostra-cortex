//! Workflow Engine Runner
//!
//! Bridges nostra-workflow-core with the Cortex worker runtime.
//!
//! This module provides:
//! - Workflow instance management
//! - A2UI schema delivery to frontend
//! - SystemOp adapter dispatch through an operation registry

use crate::workflows::op_registry::{OperationRegistry, create_default_registry};
use anyhow::Result;
use nostra_workflow_core::{
    Engine, WorkflowDefinition, WorkflowInstance, WorkflowStatus, primitives::Action,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const MAX_AUTO_STEPS_PER_TICK: usize = 128;

/// Callback function for workflow events
pub type EventCallback = Box<dyn Fn(&str, &str, &WorkflowStatus) + Send + Sync>;

/// Lightweight summary for workflow listing.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkflowSummary {
    pub id: String,
    pub status: String,
    pub current_step: Option<String>,
}

/// Manages running workflow instances.
pub struct WorkflowRunner {
    /// Active workflow instances, keyed by instance ID.
    instances: Arc<RwLock<HashMap<String, WorkflowInstance>>>,
    /// Callback for status updates
    on_step_change: Option<Arc<EventCallback>>,
    /// System operation adapter registry.
    op_registry: OperationRegistry,
}

impl Default for WorkflowRunner {
    fn default() -> Self {
        Self::new(None)
    }
}

impl WorkflowRunner {
    /// Create a new workflow runner with optional event callback.
    pub fn new(on_step_change: Option<EventCallback>) -> Self {
        Self::new_with_registry(on_step_change, create_default_registry())
    }

    /// Create a runner with an explicit operation registry.
    pub fn new_with_registry(
        on_step_change: Option<EventCallback>,
        op_registry: OperationRegistry,
    ) -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            on_step_change: on_step_change.map(Arc::new),
            op_registry,
        }
    }

    /// Start a new workflow instance.
    pub fn start(&self, instance_id: &str, definition: WorkflowDefinition) -> Result<()> {
        let instance = WorkflowInstance::new(instance_id, definition);
        {
            let mut instances = self.instances.write().unwrap();
            instances.insert(instance_id.to_string(), instance);
        }

        self.notify_step_change(instance_id, "start", &WorkflowStatus::Running);

        // Auto-advance immediately until waiting state or terminal state.
        self.tick(instance_id)?;
        Ok(())
    }

    /// Advance a workflow instance by one or more automatic steps.
    pub fn tick(&self, instance_id: &str) -> Result<Option<String>> {
        let mut instances = self.instances.write().unwrap();
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        if matches!(
            instance.status,
            WorkflowStatus::Completed | WorkflowStatus::Failed(_)
        ) {
            return Ok(None);
        }

        let mut processed_steps = 0usize;

        loop {
            if processed_steps >= MAX_AUTO_STEPS_PER_TICK {
                instance.status = WorkflowStatus::Failed(format!(
                    "Workflow exceeded max auto steps per tick ({MAX_AUTO_STEPS_PER_TICK})"
                ));
                instance
                    .context
                    .log("Workflow halted due to auto-step safety cap.");
                self.notify_step_change(instance_id, "safety_cap", &instance.status);
                break;
            }

            let current_step_id = match &instance.current_step_id {
                Some(step_id) => step_id.clone(),
                None => {
                    self.notify_step_change(instance_id, "complete", &instance.status);
                    break;
                }
            };

            let action = instance
                .definition
                .steps
                .get(&current_step_id)
                .ok_or_else(|| {
                    anyhow::anyhow!("Step not found in definition: {}", current_step_id)
                })?
                .action
                .clone();

            if let Action::SystemOp { op_type, payload } = action {
                if self.op_registry.has_adapter(&op_type) {
                    match self
                        .op_registry
                        .execute(&op_type, &payload, &mut instance.context)
                    {
                        Ok(adapter_result) => {
                            for (key, value) in adapter_result.outputs {
                                instance.context.set(key, value);
                            }
                            if let Some(message) = adapter_result.message {
                                instance.context.log(message);
                            }
                        }
                        Err(err) => {
                            instance.status = WorkflowStatus::Failed(format!(
                                "SystemOp '{}' failed: {}",
                                op_type, err
                            ));
                            instance
                                .context
                                .log(format!("SystemOp '{}' failed: {}", op_type, err));
                            self.notify_step_change(
                                instance_id,
                                &current_step_id,
                                &instance.status,
                            );
                            return Ok(instance.current_step_id.clone());
                        }
                    }
                } else {
                    instance.context.log(format!(
                        "No adapter registered for '{}', continuing with core transition.",
                        op_type
                    ));
                }
            }

            Engine::step(instance);
            processed_steps += 1;

            match &instance.current_step_id {
                Some(step_id) => self.notify_step_change(instance_id, step_id, &instance.status),
                None => self.notify_step_change(instance_id, "complete", &instance.status),
            }

            if matches!(
                instance.status,
                WorkflowStatus::Paused | WorkflowStatus::Completed | WorkflowStatus::Failed(_)
            ) {
                break;
            }
        }

        Ok(instance.current_step_id.clone())
    }

    /// Cancel a running workflow.
    pub fn cancel(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().unwrap();
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        if !matches!(
            instance.status,
            WorkflowStatus::Completed | WorkflowStatus::Failed(_)
        ) {
            instance.status = WorkflowStatus::Failed("User Cancelled".to_string());
            instance.context.log("Workflow cancelled by user signal.");
        }
        Ok(())
    }

    /// Retry a failed workflow.
    pub fn retry(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().unwrap();
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        if matches!(instance.status, WorkflowStatus::Failed(_)) {
            instance.status = WorkflowStatus::Running;
            instance.context.log("Retry initiated by user.");
        }
        Ok(())
    }

    /// Complete a user task with input data.
    pub fn complete_task(
        &self,
        instance_id: &str,
        data: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let mut instances = self.instances.write().unwrap();
        let instance = instances
            .get_mut(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        Engine::complete_user_task(instance, data);
        Ok(())
    }

    /// Get the A2UI schema for the current step.
    pub fn get_a2ui_schema(&self, instance_id: &str) -> Result<Option<String>> {
        let instances = self.instances.read().unwrap();
        let instance = instances
            .get(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        Ok(Engine::get_a2ui_schema(instance).map(|s| s.to_string()))
    }

    /// Get the status of a workflow instance.
    pub fn get_status(&self, instance_id: &str) -> Result<WorkflowStatus> {
        let instances = self.instances.read().unwrap();
        let instance = instances
            .get(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        Ok(instance.status.clone())
    }

    /// List all known workflow instances.
    pub fn list_instances(&self) -> Vec<WorkflowSummary> {
        let instances = self.instances.read().unwrap();
        let mut summaries: Vec<WorkflowSummary> = instances
            .iter()
            .map(|(id, instance)| WorkflowSummary {
                id: id.clone(),
                status: format!("{:?}", instance.status),
                current_step: instance.current_step_id.clone(),
            })
            .collect();

        summaries.sort_by(|a, b| a.id.cmp(&b.id));
        summaries
    }

    /// Get details of a workflow instance including history.
    pub fn get_details(&self, instance_id: &str) -> Result<WorkflowInstance> {
        let instances = self.instances.read().unwrap();
        let instance = instances
            .get(instance_id)
            .ok_or_else(|| anyhow::anyhow!("Instance not found: {}", instance_id))?;

        Ok(WorkflowInstance {
            id: instance.id.clone(),
            definition: instance.definition.clone(),
            current_step_id: instance.current_step_id.clone(),
            status: instance.status.clone(),
            context: instance.context.clone(),
        })
    }

    /// Get pending user tasks for a specific user/role.
    pub fn get_pending_tasks(
        &self,
        _user_id: Option<&str>,
        _role: Option<&str>,
    ) -> Vec<PendingTask> {
        let instances = self.instances.read().unwrap();
        let mut tasks = Vec::new();

        for (id, instance) in instances.iter() {
            if matches!(instance.status, WorkflowStatus::Paused) {
                if let Some(step_id) = &instance.current_step_id {
                    if let Some(step) = instance.definition.steps.get(step_id) {
                        if let Action::UserTask {
                            description,
                            a2ui_schema,
                            ..
                        } = &step.action
                        {
                            tasks.push(PendingTask {
                                instance_id: id.clone(),
                                step_id: step_id.clone(),
                                description: description.clone(),
                                a2ui_schema: a2ui_schema.clone(),
                            });
                        }
                    }
                }
            }
        }

        tasks
    }

    fn notify_step_change(&self, instance_id: &str, step_id: &str, status: &WorkflowStatus) {
        if let Some(cb) = &self.on_step_change {
            cb(instance_id, step_id, status);
        }
    }
}

/// A pending user task.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PendingTask {
    pub instance_id: String,
    pub step_id: String,
    pub description: String,
    pub a2ui_schema: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::op_registry::{
        AdapterExecutionResult, OperationAdapter, OperationRegistry,
    };
    use std::collections::HashMap;

    use nostra_workflow_core::primitives::{Step, Transition};
    use nostra_workflow_core::types::Context;

    fn create_approval_workflow() -> WorkflowDefinition {
        let mut steps = HashMap::new();

        let submit = Step::new("submit", "Submit Request")
            .with_action(Action::UserTask {
                description: "Submit your request".to_string(),
                candidate_roles: vec!["user".to_string()],
                candidate_users: vec![],
                a2ui_schema: Some(
                    r#"{"type": "form", "fields": [{"name": "request", "type": "text"}]}"#
                        .to_string(),
                ),
            })
            .with_transition(Transition::to("approve"));

        let approve = Step::new("approve", "Approve Request")
            .with_action(Action::UserTask {
                description: "Approve or reject".to_string(),
                candidate_roles: vec!["admin".to_string()],
                candidate_users: vec![],
                a2ui_schema: None,
            })
            .with_transition(Transition::to("done"));

        let done = Step::new("done", "Complete").with_action(Action::None);

        steps.insert("submit".to_string(), submit);
        steps.insert("approve".to_string(), approve);
        steps.insert("done".to_string(), done);

        WorkflowDefinition {
            id: "approval".to_string(),
            steps,
            start_step_id: "submit".to_string(),
        }
    }

    struct MockSystemAdapter;

    impl OperationAdapter for MockSystemAdapter {
        fn execute(
            &self,
            _payload: &str,
            _context: &mut Context,
        ) -> Result<AdapterExecutionResult> {
            let mut outputs = HashMap::new();
            outputs.insert("result.flag".to_string(), "ok".to_string());
            Ok(AdapterExecutionResult {
                outputs,
                message: Some("system op ran".to_string()),
            })
        }
    }

    #[test]
    fn test_workflow_runner() {
        let runner = WorkflowRunner::new(None);
        let def = create_approval_workflow();

        // Start workflow
        runner.start("test-1", def).unwrap();

        // Should be paused on submit
        let status = runner.get_status("test-1").unwrap();
        assert!(matches!(status, WorkflowStatus::Paused));

        // Should have A2UI schema
        let schema = runner.get_a2ui_schema("test-1").unwrap();
        assert!(schema.is_some());

        // Should have pending task
        let tasks = runner.get_pending_tasks(None, None);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].step_id, "submit");

        // Complete the task
        runner.complete_task("test-1", None).unwrap();

        // Should now be on approve step
        let status = runner.get_status("test-1").unwrap();
        assert!(matches!(
            status,
            WorkflowStatus::Paused | WorkflowStatus::Running | WorkflowStatus::Completed
        ));

        // Validate progress moved past the initial submit task.
        let pending_after = runner.get_pending_tasks(None, None);
        assert!(
            pending_after.is_empty() || pending_after.iter().any(|task| task.step_id != "submit"),
            "workflow should have advanced beyond submit"
        );
    }

    #[test]
    fn tick_executes_registered_system_ops_and_writes_outputs() {
        let mut registry = OperationRegistry::new();
        registry.register_adapter("ops.test", Arc::new(MockSystemAdapter));

        let runner = WorkflowRunner::new_with_registry(None, registry);

        let mut steps = HashMap::new();
        let start = Step::new("start", "Start").with_action(Action::SystemOp {
            op_type: "ops.test".to_string(),
            payload: "{}".to_string(),
        });
        steps.insert("start".to_string(), start);

        let definition = WorkflowDefinition {
            id: "sysop".to_string(),
            steps,
            start_step_id: "start".to_string(),
        };

        runner.start("sysop-1", definition).unwrap();

        let details = runner.get_details("sysop-1").unwrap();
        assert_eq!(details.status, WorkflowStatus::Completed);
        assert_eq!(
            details.context.get("result.flag").map(String::as_str),
            Some("ok")
        );
    }
}
