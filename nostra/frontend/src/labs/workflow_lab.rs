use crate::components::workflow_status::{
    StepHistoryEntry, StepStatus, WorkflowDetailView, WorkflowInstanceStatus, WorkflowStatusPanel,
};
use crate::components::workflow_task::WorkflowTaskPanel;
use crate::services::workflow_service::{PendingTask, WorkflowInstance, WorkflowService};
use dioxus::prelude::*;
use gloo_timers::future::sleep;
use std::time::Duration;

#[derive(Props, Clone, PartialEq)]
pub struct WorkflowLabProps {
    pub on_back: EventHandler<()>,
}

#[component]
pub fn WorkflowLab(props: WorkflowLabProps) -> Element {
    let service = use_signal(|| WorkflowService::new());
    let mut active_instances = use_signal(|| Vec::<WorkflowInstance>::new());
    let mut pending_tasks = use_signal(|| Vec::<PendingTask>::new());
    let mut error_msg = use_signal(|| None::<String>);
    let mut selected_instance_id = use_signal(|| None::<String>);

    // Modal state for AI generation
    let mut show_generate_modal = use_signal(|| false);
    let mut generate_intention = use_signal(|| String::new());
    let mut generated_preview = use_signal(|| None::<(String, String)>); // (yaml, preview text)

    // Polling loop for tasks
    use_future(move || async move {
        loop {
            match service.read().get_pending_tasks().await {
                Ok(tasks) => pending_tasks.set(tasks),
                Err(_) => {}
            }

            // Poll active instances for updates
            let current_ids: Vec<String> = active_instances
                .read()
                .iter()
                .map(|i| i.id.clone())
                .collect();
            let mut updated_instances = Vec::new();
            for id in current_ids {
                if let Ok(details) = service.read().get_workflow_details(&id).await {
                    updated_instances.push(details);
                }
            }
            if !updated_instances.is_empty() {
                active_instances.set(updated_instances);
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    let start_workflow = move |template: &str| {
        let template = template.to_string();
        async move {
            match service.read().start_workflow(&template).await {
                Ok(id) => {
                    let mut current = active_instances.read().clone();
                    current.push(WorkflowInstance {
                        id: id.clone(),
                        status: "Started".to_string(),
                        current_step: Some("start".to_string()),
                        history: vec![],
                    });
                    active_instances.set(current);
                    error_msg.set(None);
                }
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        }
    };

    let complete_task = move |instance_id: String, payload: String| async move {
        match service.read().complete_task(&instance_id, payload).await {
            Ok(_) => {
                // Tasks will refresh on next poll
                error_msg.set(None);
            }
            Err(e) => error_msg.set(Some(e.to_string())),
        }
    };

    rsx! {
        div { class: "flex flex-col h-full bg-background text-foreground",
            // Header
            div { class: "border-b p-4 flex items-center justify-between bg-card",
                div { class: "flex items-center gap-4",
                    button {
                        class: "p-2 hover:bg-muted rounded-full transition-colors",
                        onclick: move |_| props.on_back.call(()),
                        "← Back"
                    }
                    div {
                        h1 { class: "font-semibold text-lg", "Workflow Engine Lab" }
                        p { class: "text-xs text-muted-foreground", "Connects to local Nostra Worker (Port 3003)" }
                    }
                }
                div { class: "flex gap-2",
                    button {
                        class: "px-3 py-1.5 text-sm font-medium bg-primary text-primary-foreground rounded hover:bg-primary/90",
                        onclick: move |_| start_workflow("approval"),
                        "Start Approval Flow"
                    }
                    button {
                        class: "px-3 py-1.5 text-sm font-medium bg-secondary text-secondary-foreground rounded hover:bg-secondary/90",
                        onclick: move |_| start_workflow("governance"),
                        "Start Governance Flow"
                    }
                    button {
                        class: "px-3 py-1.5 text-sm font-medium bg-gradient-to-r from-purple-500 to-pink-500 text-white rounded hover:opacity-90",
                        onclick: move |_| {
                            show_generate_modal.set(true);
                            generate_intention.set(String::new());
                            generated_preview.set(None);
                        },
                        "✨ Generate with AI"
                    }
                }
            }

            // Error Banner
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "bg-destructive/10 text-destructive p-2 text-sm text-center",
                    "{err}"
                }
            }

            // Main Content
            div { class: "flex-1 overflow-auto p-6 grid grid-cols-1 lg:grid-cols-2 gap-8",

                // Left Column: Active Instances & Status
                div { class: "flex flex-col gap-6",
                    if let Some(selected_id) = selected_instance_id.read().clone() {
                        // DETAILED VIEW
                        div { class: "flex items-center justify-between mb-4",
                             div { class: "flex items-center gap-2",
                                 button {
                                     class: "text-sm text-muted-foreground hover:text-foreground underline",
                                     onclick: move |_| selected_instance_id.set(None),
                                     "← Back to List"
                                 }
                             }
                             // Cancel Action
                             button {
                                 class: "px-3 py-1.5 text-xs font-medium bg-destructive/10 text-destructive hover:bg-destructive/20 rounded border border-destructive/20 transition-colors",
                                 onclick: {
                                     let iid = selected_id.clone();
                                     let service = service.clone();
                                     move |_| {
                                         let iid = iid.clone();
                                         let service = service.clone();
                                         async move {
                                             let _ = service.read().cancel_workflow(&iid).await;
                                         }
                                     }
                                 },
                                 "Cancel Workflow"
                             }
                             // Retry Action (only show if failed - we check via status string)
                             button {
                                 class: "px-3 py-1.5 text-xs font-medium bg-amber-100 text-amber-700 hover:bg-amber-200 rounded border border-amber-200 transition-colors",
                                 onclick: {
                                     let iid = selected_id.clone();
                                     let service = service.clone();
                                     move |_| {
                                         let iid = iid.clone();
                                         let service = service.clone();
                                         async move {
                                             let _ = service.read().retry_workflow(&iid).await;
                                         }
                                     }
                                 },
                                 "↺ Retry"
                             }
                        }
                        if let Some(instance) = active_instances.read().iter().find(|i| i.id == selected_id) {
                            {
                                 // Map to summary
                                 let summary = crate::components::workflow_status::WorkflowInstanceSummary {
                                    id: instance.id.clone(),
                                    workflow_name: "Workflow".to_string(), // TODO: Get specific name if available
                                    status: match instance.status.as_str() {
                                        "Running" => WorkflowInstanceStatus::Running,
                                        "Completed" => WorkflowInstanceStatus::Completed,
                                        "Paused" => WorkflowInstanceStatus::Paused,
                                        _ => WorkflowInstanceStatus::Failed(instance.status.clone()),
                                    },
                                    current_step: instance.current_step.clone(),
                                    progress_percent: if instance.status == "Completed" { 100 } else { 50 },
                                    started_at: "-".to_string(),
                                    completed_at: None,
                                 };

                                 // Map history
                                 let history_entries: Vec<StepHistoryEntry> = instance.history.iter().map(|msg| {
                                     StepHistoryEntry {
                                         step_id: "log".to_string(),
                                         step_name: msg.clone(),
                                         status: StepStatus::Completed,
                                         timestamp: "-".to_string(),
                                         actor: None
                                     }
                                 }).collect();

                                 rsx! {
                                     WorkflowDetailView {
                                         instance: summary,
                                         history: history_entries
                                     }
                                 }
                            }
                        } else {
                            div { "Instance not found." }
                        }
                    } else {
                        // LIST VIEW
                        h2 { class: "text-xl font-semibold", "Active Workflows" },
                        if active_instances.read().is_empty() {
                            div { class: "text-muted-foreground italic", "No active workflows. Start one above." }
                        }
                        {
                            let summaries: Vec<crate::components::workflow_status::WorkflowInstanceSummary> = active_instances.read().iter().map(|i| {
                                crate::components::workflow_status::WorkflowInstanceSummary {
                                    id: i.id.clone(),
                                    workflow_name: "Workflow".to_string(),
                                    status: match i.status.as_str() {
                                        "Running" => WorkflowInstanceStatus::Running,
                                        "Completed" => WorkflowInstanceStatus::Completed,
                                        "Paused" => WorkflowInstanceStatus::Paused,
                                        _ => WorkflowInstanceStatus::Failed(i.status.clone()),
                                    },
                                    current_step: i.current_step.clone(),
                                    progress_percent: if i.status == "Completed" { 100 } else { 50 },
                                    started_at: "Just now".to_string(),
                                    completed_at: None,
                                }
                            }).collect();

                            rsx! {
                                WorkflowStatusPanel {
                                    instances: summaries,
                                    on_select: move |id| selected_instance_id.set(Some(id))
                                }
                            }
                        }
                    }
                }

                // Right Column: Pending Tasks
                div { class: "flex flex-col gap-6",
                    h2 { class: "text-xl font-semibold", "My Tasks" },
                    if pending_tasks.read().is_empty() {
                         div { class: "text-muted-foreground italic", "No pending tasks." }
                    }
                    for task in pending_tasks.read().iter() {
                        div { class: "border rounded-xl shadow-sm bg-card overflow-hidden",
                            div { class: "bg-muted px-4 py-2 border-b flex justify-between items-center",
                                span { class: "font-mono text-xs text-muted-foreground", "{task.instance_id}" }
                                span { class: "text-xs font-semibold bg-primary/10 text-primary px-2 py-0.5 rounded", "{task.step_id}" }
                            }
                            div { class: "p-4",
                                p { class: "mb-4 text-sm", "{task.description}" }

                                WorkflowTaskPanel {
                                    tasks: vec![crate::components::workflow_task::WorkflowTask {
                                        instance_id: task.instance_id.clone(),
                                        step_id: task.step_id.clone(),
                                        workflow_name: "Pending Task".to_string(),
                                        description: task.description.clone(),
                                        a2ui_schema: task.a2ui_schema.clone(),
                                        created_at: String::new(),
                                    }],
                                    on_complete: {
                                        let iid = task.instance_id.clone();
                                        move |req: crate::components::workflow_task::TaskCompletionRequest| {
                                           // Serialize the data map to JSON string for the legacy service
                                           let payload = serde_json::to_string(&req.data).unwrap_or("{}".to_string());
                                           complete_task(iid.clone(), payload)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

            }

            // AI Generation Modal
            if *show_generate_modal.read() {
                div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                    onclick: move |_| show_generate_modal.set(false),

                    div {
                        class: "bg-card border border-border rounded-xl shadow-2xl w-full max-w-lg p-6",
                        onclick: move |e| e.stop_propagation(),

                        // Header
                        div { class: "flex justify-between items-center mb-4",
                            h2 { class: "text-lg font-semibold", "✨ Generate Workflow with AI" }
                            button {
                                class: "text-muted-foreground hover:text-foreground",
                                onclick: move |_| show_generate_modal.set(false),
                                "✕"
                            }
                        }

                        // Input
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium mb-1", "Describe your workflow" }
                            textarea {
                                class: "w-full h-24 px-3 py-2 border border-input rounded-lg bg-background text-foreground resize-none focus:outline-none focus:ring-2 focus:ring-primary",
                                placeholder: "e.g., I need a 2-step approval process for expense reports...",
                                value: "{generate_intention}",
                                oninput: move |e| generate_intention.set(e.value())
                            }
                        }

                        // Generate Button
                        div { class: "mb-4",
                            button {
                                class: "w-full py-2 bg-gradient-to-r from-purple-500 to-pink-500 text-white rounded-lg hover:opacity-90",
                                onclick: {
                                    let service = service.clone();
                                    let intention = generate_intention.read().clone();
                                    move |_| {
                                        let service = service.clone();
                                        let intention = intention.clone();
                                        async move {
                                            match service.read().generate_workflow(&intention).await {
                                                Ok((yaml, preview)) => {
                                                    generated_preview.set(Some((yaml, preview)));
                                                }
                                                Err(e) => {
                                                    error_msg.set(Some(e.to_string()));
                                                }
                                            }
                                        }
                                    }
                                },
                                "Generate"
                            }
                        }

                        // Preview
                        if let Some((yaml, preview)) = generated_preview.read().as_ref() {
                            div { class: "mb-4",
                                p { class: "text-sm text-muted-foreground mb-2", "{preview}" }
                                pre { class: "bg-muted p-3 rounded-lg text-xs overflow-auto max-h-40 font-mono",
                                    "{yaml}"
                                }
                            }

                            // Deploy Button
                            button {
                                class: "w-full py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90",
                                onclick: {
                                    let service = service.clone();
                                    move |_| {
                                        let service = service.clone();
                                        async move {
                                            // For now, just start the approval template
                                            // In production: parse the YAML and register it
                                            match service.read().start_workflow("approval").await {
                                                Ok(id) => {
                                                    let mut current = active_instances.read().clone();
                                                    current.push(WorkflowInstance {
                                                        id: id.clone(),
                                                        status: "Started".to_string(),
                                                        current_step: Some("start".to_string()),
                                                        history: vec![],
                                                    });
                                                    active_instances.set(current);
                                                    show_generate_modal.set(false);
                                                }
                                                Err(e) => {
                                                    error_msg.set(Some(e.to_string()));
                                                }
                                            }
                                        }
                                    }
                                },
                                "🚀 Deploy Workflow"
                            }
                        }
                    }
                }
            }
        }
    }
}
