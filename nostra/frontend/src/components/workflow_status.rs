//! Workflow Status Visualization Component
//!
//! Displays workflow instance status, progress, and history.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// --- Types ---

/// Status of a workflow instance.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum WorkflowInstanceStatus {
    Running,
    Completed,
    Failed(String),
    Paused,
}

impl WorkflowInstanceStatus {
    fn display_name(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed(_) => "Failed",
            Self::Paused => "Waiting",
        }
    }

    fn color_class(&self) -> &'static str {
        match self {
            Self::Running => "text-blue-500",
            Self::Completed => "text-green-500",
            Self::Failed(_) => "text-red-500",
            Self::Paused => "text-amber-500",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Running => "⏳",
            Self::Completed => "✅",
            Self::Failed(_) => "❌",
            Self::Paused => "⏸️",
        }
    }
}

/// A workflow instance summary for display.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WorkflowInstanceSummary {
    pub id: String,
    pub workflow_name: String,
    pub status: WorkflowInstanceStatus,
    pub current_step: Option<String>,
    pub progress_percent: u8,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// A step in the workflow history.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StepHistoryEntry {
    pub step_id: String,
    pub step_name: String,
    pub status: StepStatus,
    pub timestamp: String,
    pub actor: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum StepStatus {
    Completed,
    Current,
    Pending,
    Skipped,
}

// --- Props ---

#[derive(Props, PartialEq, Clone)]
pub struct WorkflowStatusPanelProps {
    /// List of workflow instances to display.
    pub instances: Vec<WorkflowInstanceSummary>,
    /// Handler for selecting an instance.
    pub on_select: EventHandler<String>,
}

#[derive(Props, PartialEq, Clone)]
pub struct WorkflowDetailProps {
    /// The workflow instance to display.
    pub instance: WorkflowInstanceSummary,
    /// Step history for the timeline view.
    pub history: Vec<StepHistoryEntry>,
}

// --- Main Panel Component ---

/// Workflow status panel showing all active instances.
#[component]
pub fn WorkflowStatusPanel(props: WorkflowStatusPanelProps) -> Element {
    rsx! {
        div { class: "workflow-status-panel flex flex-col h-full bg-background",
            // Header
            div { class: "flex items-center justify-between p-4 border-b border-border",
                h2 { class: "text-lg font-semibold", "Workflow Dashboard" }
                span { class: "text-sm text-muted-foreground",
                    "{props.instances.len()} workflows"
                }
            }

            if props.instances.is_empty() {
                // Empty state
                div { class: "flex-1 flex items-center justify-center p-8",
                    div { class: "text-center text-muted-foreground",
                        div { class: "text-4xl mb-4", "📋" }
                        p { "No active workflows" }
                        p { class: "text-sm mt-2", "Start a new workflow to see it here." }
                    }
                }
            } else {
                // Instance grid
                div { class: "flex-1 overflow-y-auto p-4",
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                        for instance in props.instances.iter() {
                            WorkflowInstanceCard {
                                instance: instance.clone(),
                                on_click: props.on_select.clone()
                            }
                        }
                    }
                }
            }
        }
    }
}

// --- Instance Card ---

#[derive(Props, PartialEq, Clone)]
struct WorkflowInstanceCardProps {
    instance: WorkflowInstanceSummary,
    on_click: EventHandler<String>,
}

#[component]
fn WorkflowInstanceCard(props: WorkflowInstanceCardProps) -> Element {
    let instance = &props.instance;
    let instance_id = instance.id.clone();

    rsx! {
        button {
            class: "w-full text-left p-4 rounded-lg border border-border bg-card hover:bg-accent/50 transition-colors",
            onclick: move |_| props.on_click.call(instance_id.clone()),

            // Header: Name + Status
            div { class: "flex items-center justify-between mb-3",
                h3 { class: "font-semibold text-sm truncate", "{instance.workflow_name}" }
                span { class: "flex items-center gap-1 text-sm {instance.status.color_class()}",
                    "{instance.status.icon()}"
                    "{instance.status.display_name()}"
                }
            }

            // Progress bar
            div { class: "w-full h-2 bg-secondary rounded-full mb-3",
                div {
                    class: "h-full bg-primary rounded-full transition-all",
                    style: "width: {instance.progress_percent}%"
                }
            }

            // Current step
            if let Some(step) = &instance.current_step {
                div { class: "text-xs text-muted-foreground mb-2",
                    "Current: {step}"
                }
            }

            // Footer: ID + Time
            div { class: "flex justify-between text-xs text-muted-foreground/70",
                span { class: "font-mono", "{instance.id}" }
                span { "{instance.started_at}" }
            }
        }
    }
}

// --- Detail View with Timeline ---

/// Detailed workflow view with step timeline.
#[component]
pub fn WorkflowDetailView(props: WorkflowDetailProps) -> Element {
    let instance = &props.instance;

    rsx! {
        div { class: "workflow-detail flex flex-col h-full",
            // Header
            div { class: "p-6 border-b border-border",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-xl font-bold", "{instance.workflow_name}" }
                    span { class: "flex items-center gap-2 px-3 py-1 rounded-full bg-secondary {instance.status.color_class()}",
                        "{instance.status.icon()}"
                        "{instance.status.display_name()}"
                    }
                }
                div { class: "mt-2 text-sm text-muted-foreground",
                    span { class: "font-mono", "ID: {instance.id}" }
                    span { " • Started: {instance.started_at}" }
                    if let Some(completed) = &instance.completed_at {
                        span { " • Completed: {completed}" }
                    }
                }
            }

            // Progress overview
            div { class: "p-6 border-b border-border",
                div { class: "flex items-center gap-4",
                    span { class: "text-2xl font-bold", "{instance.progress_percent}%" }
                    div { class: "flex-1",
                        div { class: "w-full h-3 bg-secondary rounded-full",
                            div {
                                class: "h-full bg-primary rounded-full transition-all",
                                style: "width: {instance.progress_percent}%"
                            }
                        }
                    }
                }
            }

            // Timeline
            div { class: "flex-1 overflow-y-auto p-6",
                h3 { class: "text-lg font-semibold mb-4", "Execution Timeline" }
                div { class: "relative pl-6 border-l-2 border-border",
                    for entry in props.history.iter() {
                        TimelineEntry { entry: entry.clone() }
                    }
                }
            }
        }
    }
}

// --- Timeline Entry ---

#[derive(Props, PartialEq, Clone)]
struct TimelineEntryProps {
    entry: StepHistoryEntry,
}

#[component]
fn TimelineEntry(props: TimelineEntryProps) -> Element {
    let entry = &props.entry;

    let (dot_class, bg_class) = match entry.status {
        StepStatus::Completed => ("bg-green-500", ""),
        StepStatus::Current => ("bg-blue-500 animate-pulse", "bg-accent/30"),
        StepStatus::Pending => ("bg-muted-foreground/50", "opacity-50"),
        StepStatus::Skipped => ("bg-gray-400", "opacity-50 line-through"),
    };

    rsx! {
        div { class: "relative pb-6 {bg_class}",
            // Dot
            div { class: "absolute -left-[9px] w-4 h-4 rounded-full {dot_class}" }

            // Content
            div { class: "ml-4",
                div { class: "font-medium", "{entry.step_name}" }
                div { class: "text-sm text-muted-foreground", "{entry.step_id}" }
                div { class: "flex gap-2 text-xs text-muted-foreground/70 mt-1",
                    span { "{entry.timestamp}" }
                    if let Some(actor) = &entry.actor {
                        span { "• {actor}" }
                    }
                }
            }
        }
    }
}

// --- Demo Data ---

/// Create sample workflow instances for testing.
#[allow(dead_code)]
pub fn create_sample_instances() -> Vec<WorkflowInstanceSummary> {
    vec![
        WorkflowInstanceSummary {
            id: "wf-001".to_string(),
            workflow_name: "Document Approval".to_string(),
            status: WorkflowInstanceStatus::Paused,
            current_step: Some("Manager Review".to_string()),
            progress_percent: 50,
            started_at: "2026-01-24 10:30".to_string(),
            completed_at: None,
        },
        WorkflowInstanceSummary {
            id: "wf-002".to_string(),
            workflow_name: "Governance Proposal".to_string(),
            status: WorkflowInstanceStatus::Running,
            current_step: Some("Community Voting".to_string()),
            progress_percent: 75,
            started_at: "2026-01-24 09:15".to_string(),
            completed_at: None,
        },
        WorkflowInstanceSummary {
            id: "wf-003".to_string(),
            workflow_name: "Bug Triage".to_string(),
            status: WorkflowInstanceStatus::Completed,
            current_step: None,
            progress_percent: 100,
            started_at: "2026-01-23 14:00".to_string(),
            completed_at: Some("2026-01-23 16:30".to_string()),
        },
        WorkflowInstanceSummary {
            id: "wf-004".to_string(),
            workflow_name: "Treasury Transfer".to_string(),
            status: WorkflowInstanceStatus::Failed("Insufficient funds".to_string()),
            current_step: Some("Execute Transfer".to_string()),
            progress_percent: 80,
            started_at: "2026-01-24 08:00".to_string(),
            completed_at: None,
        },
    ]
}

/// Create sample history for a workflow.
#[allow(dead_code)]
pub fn create_sample_history() -> Vec<StepHistoryEntry> {
    vec![
        StepHistoryEntry {
            step_id: "submit".to_string(),
            step_name: "Submit Request".to_string(),
            status: StepStatus::Completed,
            timestamp: "10:30 AM".to_string(),
            actor: Some("alice.eth".to_string()),
        },
        StepHistoryEntry {
            step_id: "auto_validate".to_string(),
            step_name: "Auto-Validation".to_string(),
            status: StepStatus::Completed,
            timestamp: "10:31 AM".to_string(),
            actor: None,
        },
        StepHistoryEntry {
            step_id: "manager_review".to_string(),
            step_name: "Manager Review".to_string(),
            status: StepStatus::Current,
            timestamp: "10:32 AM".to_string(),
            actor: Some("bob.eth".to_string()),
        },
        StepHistoryEntry {
            step_id: "final_approval".to_string(),
            step_name: "Final Approval".to_string(),
            status: StepStatus::Pending,
            timestamp: "-".to_string(),
            actor: None,
        },
        StepHistoryEntry {
            step_id: "execute".to_string(),
            step_name: "Execute".to_string(),
            status: StepStatus::Pending,
            timestamp: "-".to_string(),
            actor: None,
        },
    ]
}
