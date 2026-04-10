//! Workflow Task Panel Component
//!
//! Renders pending workflow tasks using the A2UI renderer.
//! Connects to the workflow engine for task completion.

use crate::a2ui::{A11yProperties, A2UIRenderer, Component, ComponentProperties, Surface};
use crate::a2ui_theme::A2UIThemeName;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// --- Types ---

/// A pending workflow task from the engine.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorkflowTask {
    pub instance_id: String,
    pub step_id: String,
    pub workflow_name: String,
    pub description: String,
    pub a2ui_schema: Option<String>,
    pub created_at: String,
}

/// Task completion request sent to the worker.
#[derive(Clone, Debug, Serialize)]
pub struct TaskCompletionRequest {
    pub instance_id: String,
    pub step_id: String,
    pub data: std::collections::HashMap<String, String>,
}

// --- Props ---

#[derive(Props, PartialEq, Clone)]
pub struct WorkflowTaskPanelProps {
    /// List of pending tasks to display.
    pub tasks: Vec<WorkflowTask>,
    /// Handler called when a task is completed.
    pub on_complete: EventHandler<TaskCompletionRequest>,
}

// --- Component ---

/// Main workflow task panel showing pending tasks.
#[component]
pub fn WorkflowTaskPanel(props: WorkflowTaskPanelProps) -> Element {
    let mut selected_task = use_signal::<Option<usize>>(|| None);
    let mut form_data = use_signal(std::collections::HashMap::<String, String>::new);

    // Clone tasks for closure
    let tasks_for_action = props.tasks.clone();
    let on_complete = props.on_complete.clone();

    // Handler for A2UI actions
    let on_action = move |(action, payload): (String, String)| {
        if action == "submit" || action == "complete" {
            // Submit the task
            if let Some(idx) = selected_task() {
                if let Some(task) = tasks_for_action.get(idx) {
                    on_complete.call(TaskCompletionRequest {
                        instance_id: task.instance_id.clone(),
                        step_id: task.step_id.clone(),
                        data: form_data(),
                    });
                    // Reset selection
                    selected_task.set(None);
                    form_data.set(std::collections::HashMap::new());
                }
            }
        } else if action == "update_input" || action.starts_with("field_") {
            // Store form field value
            form_data.with_mut(|data| {
                data.insert(action.clone(), payload);
            });
        }
    };

    rsx! {
        div { class: "workflow-task-panel flex flex-col h-full",
            // Header
            div { class: "flex items-center justify-between p-4 border-b border-border",
                h2 { class: "text-lg font-semibold", "My Tasks" }
                span { class: "text-sm text-muted-foreground",
                    "{props.tasks.len()} pending"
                }
            }

            if props.tasks.is_empty() {
                // Empty state
                div { class: "flex-1 flex items-center justify-center p-8",
                    div { class: "text-center text-muted-foreground",
                        div { class: "text-4xl mb-4", "✅" }
                        p { "No pending tasks" }
                        p { class: "text-sm mt-2", "Check back later or start a new workflow." }
                    }
                }
            } else {
                div { class: "flex-1 flex overflow-hidden",
                    // Task list sidebar
                    div { class: "w-64 border-r border-border overflow-y-auto",
                        for (idx, task) in props.tasks.iter().enumerate() {
                            TaskListItem {
                                task: task.clone(),
                                is_selected: selected_task() == Some(idx),
                                on_select: move |_| selected_task.set(Some(idx))
                            }
                        }
                    }

                    // Task detail / form area
                    div { class: "flex-1 overflow-y-auto p-6",
                        if let Some(idx) = selected_task() {
                            if let Some(task) = props.tasks.get(idx) {
                                TaskDetailView {
                                    task: task.clone(),
                                    on_action: on_action
                                }
                            }
                        } else {
                            div { class: "flex items-center justify-center h-full text-muted-foreground",
                                p { "Select a task to view details" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// --- Sub-components ---

#[derive(Props, PartialEq, Clone)]
struct TaskListItemProps {
    task: WorkflowTask,
    is_selected: bool,
    on_select: EventHandler<()>,
}

#[component]
fn TaskListItem(props: TaskListItemProps) -> Element {
    let selected_class = if props.is_selected {
        "bg-accent text-accent-foreground"
    } else {
        "hover:bg-muted/50"
    };

    rsx! {
        button {
            class: "w-full text-left p-4 border-b border-border transition-colors {selected_class}",
            onclick: move |_| props.on_select.call(()),

            div { class: "font-medium text-sm truncate",
                "{props.task.description}"
            }
            div { class: "text-xs text-muted-foreground mt-1 truncate",
                "{props.task.workflow_name}"
            }
            div { class: "text-xs text-muted-foreground/60 mt-1",
                "Step: {props.task.step_id}"
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct TaskDetailViewProps {
    task: WorkflowTask,
    on_action: EventHandler<(String, String)>,
}

#[component]
fn TaskDetailView(props: TaskDetailViewProps) -> Element {
    // Parse A2UI schema if present
    let surface = if let Some(schema) = &props.task.a2ui_schema {
        parse_a2ui_schema(schema)
    } else {
        // Default fallback form
        create_default_form(&props.task.description)
    };

    rsx! {
        div { class: "space-y-6",
            // Task header
            div { class: "space-y-2",
                h3 { class: "text-xl font-semibold", "{props.task.description}" }
                div { class: "flex gap-2 text-sm text-muted-foreground",
                    span { class: "px-2 py-0.5 bg-secondary rounded-full",
                        "{props.task.workflow_name}"
                    }
                    span { "•" }
                    span { "Instance: {props.task.instance_id}" }
                }
            }

            hr { class: "border-border" }

            // A2UI rendered form
            div { class: "min-h-[200px]",
                A2UIRenderer {
                    surface: surface,
                    theme: Some(A2UIThemeName::Nostra),
                    on_action: props.on_action
                }
            }
        }
    }
}

// --- Helpers ---

/// Parse an A2UI JSON schema string into a Surface.
fn parse_a2ui_schema(schema: &str) -> Surface {
    if let Ok(surface) = serde_json::from_str::<Surface>(schema) {
        return surface;
    }

    if let Some(surface) = crate::a2ui::surface_from_v1(schema) {
        return surface;
    }

    // Return a fallback error surface
    Surface {
        id: Some("error".to_string()),
        root: Component {
            id: "error_msg".to_string(),
            properties: ComponentProperties::Text {
                text: "Error parsing form schema: unsupported format".to_string(),
            },
            a11y: None,
            meta: None,
        },
        meta: None,
    }
}

/// Create a default form for tasks without A2UI schema.
fn create_default_form(description: &str) -> Surface {
    Surface {
        id: Some("default_form".to_string()),
        root: Component {
            id: "form_container".to_string(),
            properties: ComponentProperties::Column {
                children: vec![
                    Component {
                        id: "heading".to_string(),
                        properties: ComponentProperties::Heading {
                            text: description.to_string(),
                        },
                        a11y: None,
                        meta: None,
                    },
                    Component {
                        id: "notes".to_string(),
                        properties: ComponentProperties::TextField {
                            label: "Notes (optional)".to_string(),
                            value: String::new(),
                        },
                        a11y: Some(A11yProperties {
                            label: Some("Notes (optional)".to_string()),
                            ..Default::default()
                        }),
                        meta: None,
                    },
                    Component {
                        id: "submit".to_string(),
                        properties: ComponentProperties::Button {
                            label: "Complete Task".to_string(),
                            action: "complete".to_string(),
                        },
                        a11y: Some(A11yProperties {
                            label: Some("Complete Task".to_string()),
                            ..Default::default()
                        }),
                        meta: None,
                    },
                ],
            },
            a11y: None,
            meta: None,
        },
        meta: None,
    }
}

// --- Demo/Test Data ---

/// Create sample tasks for testing.
#[allow(dead_code)]
pub fn create_sample_tasks() -> Vec<WorkflowTask> {
    vec![
        WorkflowTask {
            instance_id: "wf-001".to_string(),
            step_id: "review".to_string(),
            workflow_name: "Document Approval".to_string(),
            description: "Review and approve the Q4 budget proposal".to_string(),
            a2ui_schema: None,
            created_at: "2026-01-24T10:30:00Z".to_string(),
        },
        WorkflowTask {
            instance_id: "wf-002".to_string(),
            step_id: "vote".to_string(),
            workflow_name: "Governance Proposal".to_string(),
            description: "Vote on DAO treasury allocation".to_string(),
            a2ui_schema: Some(r#"{
                "id": "vote_form",
                "root": {
                    "id": "container",
                    "type": "Column",
                    "children": [
                        {"id": "h1", "type": "Heading", "text": "Cast Your Vote"},
                        {"id": "info", "type": "Text", "text": "Please review the proposal before voting."},
                        {"id": "vote_yes", "type": "Button", "label": "Vote Yes ✓", "action": "submit"},
                        {"id": "vote_no", "type": "Button", "label": "Vote No ✗", "action": "reject"}
                    ]
                }
            }"#.to_string()),
            created_at: "2026-01-24T09:15:00Z".to_string(),
        },
    ]
}
