use std::collections::HashMap;

pub struct WorkflowRegistry {
    workflows: HashMap<String, String>, // ID -> YAML Content
}

impl Default for WorkflowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            workflows: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    pub fn register(&mut self, id: &str, content: &str) {
        self.workflows.insert(id.to_string(), content.to_string());
    }

    pub fn get(&self, id: &str) -> Option<&String> {
        self.workflows.get(id)
    }

    fn register_defaults(&mut self) {
        // Defaults removed because the target YAML files in research/030 and research/080 are missing from the repository.
    }
}
