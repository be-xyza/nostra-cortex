#[cfg(test)]
mod tests {
    use crate::workflows::engine_runner::WorkflowRunner;
    use nostra_workflow_core::builder::{WorkflowParser, WorkflowTemplates};
    use nostra_workflow_core::types::WorkflowStatus;
    // use std::collections::HashMap; // Unused for now as we pass None

    #[test]
    fn test_end_to_end_workflow_execution() {
        // 1. Setup Runner
        let runner = WorkflowRunner::new(None);

        // 2. Start Workflow (Approval Template)
        // Parse the YAML template first
        let yaml = WorkflowTemplates::approval();
        let def = WorkflowParser::from_yaml(yaml).expect("Failed to parse template");

        // Start with a specific ID
        let workflow_id = "doc-123";
        runner
            .start(workflow_id, def)
            .expect("Failed to start workflow");

        // 3. Tick - The runner auto-ticks on start, but let's check status
        let status = runner.get_status(workflow_id).expect("Workflow not found");

        // The approval workflow starts with 'submit' (UserTask), so it should be Paused
        assert!(
            matches!(status, WorkflowStatus::Paused),
            "Should be Paused at start, got {:?}",
            status
        );

        // 4. Verify Pending Tasks & Schema
        let pending = runner.get_pending_tasks(None, None);
        assert_eq!(pending.len(), 1, "Should have 1 pending task");
        assert_eq!(pending[0].instance_id, workflow_id);

        // Check direct schema access
        let schema = runner
            .get_a2ui_schema(workflow_id)
            .expect("Failed to get schema");
        assert!(
            schema.is_none(),
            "Submit step in approval template has no schema in default template"
        );

        // 5. Complete Task (Submit)
        // The first step 'submit' just needs completion
        let result = runner.complete_task(workflow_id, None);
        assert!(result.is_ok(), "Task completion failed: {:?}", result.err());

        // 6. Tick - Should move to 'approve'
        runner.tick(workflow_id).expect("Failed to tick");

        // Now we should be at 'approve' step
        let status = runner.get_status(workflow_id).unwrap();
        assert!(
            matches!(status, WorkflowStatus::Paused),
            "Should be paused at 'approve'"
        );

        let pending = runner.get_pending_tasks(None, None);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].step_id, "approve");

        // 7. Complete Approve Task
        runner
            .complete_task(workflow_id, None)
            .expect("Failed to approve");
        runner
            .tick(workflow_id)
            .expect("Failed to tick after approval");

        // 8. Should be at 'complete' step (which is Action::None -> triggers finish?)
        // Logic: Action::None -> transitions. If no transitions -> Workflow Completed.

        // We might need one more tick if the previous tick just moved US into 'complete' but didn't execute it yet?
        let status = runner.get_status(workflow_id).unwrap();

        // Helper to tick until completion or limit
        let mut ticks = 0;
        let mut current_status = status;
        while ticks < 5
            && !matches!(
                current_status,
                WorkflowStatus::Completed | WorkflowStatus::Failed(_)
            )
        {
            runner.tick(workflow_id).expect("Tick loop");
            current_status = runner.get_status(workflow_id).unwrap();
            ticks += 1;
        }

        assert!(
            matches!(current_status, WorkflowStatus::Completed),
            "Workflow should be completed, got {:?}",
            current_status
        );
    }
}
