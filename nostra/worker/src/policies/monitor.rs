use crate::policies::{Policy, get_policy};
use anyhow::Result;
use nostra_shared::types::benchmark::PolicyConstraint;

pub struct AdversarialMonitor {
    policies: Vec<Box<dyn Policy>>,
    violations: Vec<String>,
}

impl AdversarialMonitor {
    pub fn new(constraints: &[PolicyConstraint]) -> Self {
        let policies = constraints.iter().map(get_policy).collect();
        Self {
            policies,
            violations: Vec::new(),
        }
    }

    pub fn get_system_prompt_additions(&self) -> String {
        self.policies
            .iter()
            .map(|p| p.system_prompt_injection())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn validate_action(&mut self, action: &str, args: &[String]) -> Result<bool> {
        let mut permitted = true;
        for policy in &self.policies {
            if !policy.validate_action(action, args)? {
                let msg = format!(
                    "Policy Violation [{}]: Action '{}' is forbidden.",
                    policy.name(),
                    action
                );
                self.violations.push(msg);
                permitted = false;
            }
        }
        Ok(permitted)
    }

    pub fn get_violations(&self) -> Vec<String> {
        self.violations.clone()
    }
}
