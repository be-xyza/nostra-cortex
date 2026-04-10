use crate::labs::ag_ui_types::AgComponent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pass,
    Fail(String),
    Warn(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assessment {
    pub status: ValidationStatus,
    pub details: Vec<ValidationResult>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub constraint: String,
    pub status: ValidationStatus,
}

pub struct A2UIValidator;

impl A2UIValidator {
    pub fn validate(ui: &[AgComponent], constraints: &[String]) -> Assessment {
        let mut results = Vec::new();
        let mut overall_fail = false;

        for constraint in constraints {
            let status = Self::check_constraint(ui, constraint);
            if let ValidationStatus::Fail(_) = status {
                overall_fail = true;
            }
            results.push(ValidationResult {
                constraint: constraint.clone(),
                status,
            });
        }

        // Run Integration Checks
        let integration_results = Self::check_integration_integrity(ui);
        for res in integration_results {
            if let ValidationStatus::Fail(_) = res.status {
                overall_fail = true;
            }
            results.push(res);
        }

        Assessment {
            status: if overall_fail {
                ValidationStatus::Fail("One or more constraints/checks failed.".into())
            } else {
                ValidationStatus::Pass
            },
            details: results,
        }
    }

    fn check_constraint(ui: &[AgComponent], constraint: &str) -> ValidationStatus {
        let c_lower = constraint.to_lowercase();

        // --- Heuristic 1: "Must have [Field]" ---
        if c_lower.contains("have") || c_lower.contains("collect") || c_lower.contains("input") {
            // naive keyword extraction: look for quoted words or just last word
            // "Must have 'Email'" -> look for label/placeholder "Email"
            let keywords: Vec<&str> = constraint
                .split_whitespace()
                .filter(|w| {
                    w.len() > 3
                        && !["must", "have", "collect", "input", "field"]
                            .contains(&w.to_lowercase().as_str())
                })
                .collect();

            for keyword in keywords {
                let found = Self::traverse_find(ui, &|comp| match comp {
                    AgComponent::Input(i) => {
                        i.label.to_lowercase().contains(&keyword.to_lowercase())
                    }
                    AgComponent::Text { text } => {
                        text.to_lowercase().contains(&keyword.to_lowercase())
                    }
                    _ => false,
                });

                if !found {
                    return ValidationStatus::Fail(format!(
                        "Could not find component matching '{}'",
                        keyword
                    ));
                }
            }
            return ValidationStatus::Pass;
        }

        // --- Heuristic 2: "Theme" ---
        if c_lower.contains("theme") {
            // We can't validate visual theme easily in JSON without style props.
            // But we can check for "variant" usage if constraint implies it.
            // e.g. "Danger theme" -> look for variant="danger"
            if c_lower.contains("danger") {
                let found = Self::traverse_find(ui, &|comp| match comp {
                    AgComponent::Action(a) => a.variant.as_deref() == Some("danger"),
                    AgComponent::Notification(n) => n.variant == "danger",
                    _ => false,
                });
                if !found {
                    return ValidationStatus::Fail(
                        "Constraint implies 'danger' variant, but none found.".into(),
                    );
                }
            }
            return ValidationStatus::Warn(
                "Theme constraints are visually verified manually.".into(),
            );
        }

        ValidationStatus::Warn(
            "Constraint too complex for auto-validation. Please verify manually.".into(),
        )
    }

    fn check_integration_integrity(ui: &[AgComponent]) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // Check 1: Action Validity
        // Traverse and find all Actions. Check if their IDs map to known system actions (Mocked).
        let known_actions = vec![
            "submit_vote",
            "confirm_tx",
            "cancel_tx",
            "update_profile",
            "login",
            "logout",
        ];

        Self::traverse_visit(ui, &mut |comp| {
            if let AgComponent::Action(a) = comp {
                if !known_actions.contains(&a.actionId.as_str()) {
                    results.push(ValidationResult {
                        constraint: format!("System Integrity: Action '{}'", a.actionId),
                        status: ValidationStatus::Warn(format!(
                            "Action ID '{}' is not in the standard registry (013-workflow-engine).",
                            a.actionId
                        )),
                    });
                } else {
                    results.push(ValidationResult {
                        constraint: format!("System Integrity: Action '{}'", a.actionId),
                        status: ValidationStatus::Pass,
                    });
                }
            }
        });

        results
    }

    // --- Helpers ---

    fn traverse_find(ui: &[AgComponent], predicate: &dyn Fn(&AgComponent) -> bool) -> bool {
        for comp in ui {
            if predicate(comp) {
                return true;
            }
            match comp {
                AgComponent::Row { children } | AgComponent::Column { children } => {
                    if Self::traverse_find(children, predicate) {
                        return true;
                    }
                }
                AgComponent::Details(d) => {
                    if Self::traverse_find(&d.children, predicate) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn traverse_visit(ui: &[AgComponent], visitor: &mut dyn FnMut(&AgComponent)) {
        for comp in ui {
            visitor(comp);
            match comp {
                AgComponent::Row { children } | AgComponent::Column { children } => {
                    Self::traverse_visit(children, visitor);
                }
                AgComponent::Details(d) => {
                    Self::traverse_visit(&d.children, visitor);
                }
                _ => {}
            }
        }
    }
}
