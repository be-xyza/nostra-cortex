use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Props, Clone, PartialEq)]
pub struct ComplianceValidatorLabProps {
    pub on_back: EventHandler<()>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CompliancePolicy {
    id: String,
    name: String,
    rules: Vec<ComplianceRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ComplianceRule {
    rule_id: String,
    description: String,
    required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkflowConfig {
    id: String,
    steps: Vec<WorkflowStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkflowStep {
    id: String,
    action: String,
    encrypted: bool,
    logs: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ValidationResult {
    passed: bool,
    findings: Vec<String>,
}

#[component]
pub fn ComplianceValidatorLab(props: ComplianceValidatorLabProps) -> Element {
    // Default Policy: HIPAA-style
    let default_policy = r#"{
    "id": "policy_hipaa_v1",
    "name": "HIPAA Basic Privacy",
    "rules": [
        { "rule_id": "encyrption_at_rest", "description": "Data must be encrypted", "required": true },
        { "rule_id": "audit_logging", "description": "Access must be logged", "required": true }
    ]
}"#;

    // Default Workflow: Simple Patient Intake
    let default_workflow = r#"{
    "id": "wf_patient_intake",
    "steps": [
        { "id": "step_1", "action": "collect_info", "encrypted": false, "logs": true },
        { "id": "step_2", "action": "save_db", "encrypted": true, "logs": true }
    ]
}"#;

    let mut policy_json = use_signal(|| default_policy.to_string());
    let mut workflow_json = use_signal(|| default_workflow.to_string());
    let mut validation_result = use_signal(|| None::<ValidationResult>);

    let validate = move |_| {
        let policy: CompliancePolicy = match serde_json::from_str(&policy_json.read()) {
            Ok(p) => p,
            Err(_) => {
                validation_result.set(Some(ValidationResult {
                    passed: false,
                    findings: vec!["Invalid Policy JSON".to_string()],
                }));
                return;
            }
        };

        let workflow: WorkflowConfig = match serde_json::from_str(&workflow_json.read()) {
            Ok(w) => w,
            Err(_) => {
                validation_result.set(Some(ValidationResult {
                    passed: false,
                    findings: vec!["Invalid Workflow JSON".to_string()],
                }));
                return;
            }
        };

        let mut findings = vec![];
        let mut passed = true;

        for step in workflow.steps {
            // Check Encryption Rule
            if policy
                .rules
                .iter()
                .any(|r| r.rule_id == "encyrption_at_rest" && r.required)
            {
                if !step.encrypted {
                    findings.push(format!("Step '{}' violates Encryption Policy", step.id));
                    passed = false;
                }
            }
            // Check Logging Rule
            if policy
                .rules
                .iter()
                .any(|r| r.rule_id == "audit_logging" && r.required)
            {
                if !step.logs {
                    findings.push(format!("Step '{}' missing Audit Logs", step.id));
                    passed = false;
                }
            }
        }

        if passed {
            findings.push("All Checks Passed. Workflow is Compliant.".to_string());
        }

        validation_result.set(Some(ValidationResult { passed, findings }));
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
                        h1 { class: "font-semibold text-lg", "Compliance Validator" }
                        p { class: "text-xs text-muted-foreground", "Validate Workflows against Policies (GDPR, HIPAA)" }
                    }
                }
                button {
                    class: "px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90",
                    onclick: validate,
                    "Run Validation"
                }
            }

            // Body
            div { class: "flex-1 flex overflow-hidden",
                // Left: Inputs
                div { class: "w-1/3 border-r flex flex-col p-4 gap-4 bg-muted/5",
                    div { class: "flex flex-col gap-2 flex-1",
                        label { class: "text-sm font-medium", "Policy (JSON)" }
                        textarea {
                            class: "flex-1 p-2 font-mono text-sm border rounded resize-none focus:outline-none focus:ring-1 focus:ring-primary",
                            value: "{policy_json}",
                            oninput: move |e| policy_json.set(e.value())
                        }
                    }
                    div { class: "h-px bg-border", }
                    div { class: "flex flex-col gap-2 flex-1",
                        label { class: "text-sm font-medium", "Workflow Config (JSON)" }
                        textarea {
                            class: "flex-1 p-2 font-mono text-sm border rounded resize-none focus:outline-none focus:ring-1 focus:ring-primary",
                            value: "{workflow_json}",
                            oninput: move |e| workflow_json.set(e.value())
                        }
                    }
                }

                // Right: Results
                div { class: "flex-1 p-8 overflow-y-auto bg-muted/20",
                    if let Some(result) = validation_result.read().as_ref() {
                        div { class: "max-w-2xl mx-auto space-y-4",
                             div {
                                class: if result.passed {
                                    "p-4 rounded-lg bg-green-500/10 border border-green-500/20 text-green-600 flex items-center gap-3"
                                } else {
                                    "p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-600 flex items-center gap-3"
                                },
                                if result.passed {
                                    svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "2",
                                        path { d: "M5 13l4 4L19 7" }
                                    }
                                    span { class: "font-semibold", "Compliant" }
                                } else {
                                    svg { class: "w-6 h-6", fill: "none", view_box: "0 0 24 24", stroke: "currentColor", stroke_width: "2",
                                        path { d: "M6 18L18 6M6 6l12 12" }
                                    }
                                    span { class: "font-semibold", "Non-Compliant" }
                                }
                            }

                            div { class: "bg-card rounded-lg border shadow-sm p-4",
                                h3 { class: "text-sm font-medium mb-2", "Validation Findings" }
                                ul { class: "space-y-2",
                                    for finding in &result.findings {
                                        li { class: "text-sm text-foreground/80 flex items-start gap-2",
                                            span { class: "mt-1.5 w-1.5 h-1.5 rounded-full bg-slate-400 shrink-0" }
                                            "{finding}"
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "h-full flex flex-col items-center justify-center text-muted-foreground gap-2",
                            div { class: "text-4xl", "🛡️" }
                            p { "Configure Policy and Workflow, then click 'Run Validation'" }
                        }
                    }
                }
            }
        }
    }
}
