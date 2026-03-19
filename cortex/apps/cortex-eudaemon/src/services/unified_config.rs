use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InfrastructureMode {
    Hybrid,
    Sovereign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub capabilities: LabCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabCapabilities {
    pub uses_vector_store: bool,
    pub uses_fs_access: bool,
    pub uses_local_llm: bool,
}

pub trait InfrastructureProfile {
    fn mode(&self) -> InfrastructureMode;
    fn get_llm_endpoint(&self) -> String;
    fn get_vector_endpoint(&self) -> String;
    fn can_run_lab(&self, lab: &LabManifest) -> bool;
}

pub struct HybridProfile;
impl InfrastructureProfile for HybridProfile {
    fn mode(&self) -> InfrastructureMode {
        InfrastructureMode::Hybrid
    }
    fn get_llm_endpoint(&self) -> String {
        "https://api.nostra.ai/v1/llm".to_string()
    }
    fn get_vector_endpoint(&self) -> String {
        "https://api.nostra.ai/v1/vector".to_string()
    }
    fn can_run_lab(&self, lab: &LabManifest) -> bool {
        // Hybrid cannot run labs requiring local LLM or raw FS access
        !lab.capabilities.uses_local_llm && !lab.capabilities.uses_fs_access
    }
}

pub struct SovereignProfile;
impl InfrastructureProfile for SovereignProfile {
    fn mode(&self) -> InfrastructureMode {
        InfrastructureMode::Sovereign
    }
    fn get_llm_endpoint(&self) -> String {
        "http://localhost:11434".to_string() // Ollama
    }
    fn get_vector_endpoint(&self) -> String {
        "http://localhost:6333".to_string() // Qdrant
    }
    fn can_run_lab(&self, _lab: &LabManifest) -> bool {
        // Sovereign can run everything
        true
    }
}

pub struct UnifiedConfigFramework {
    pub current_mode: InfrastructureMode,
}

impl UnifiedConfigFramework {
    pub fn new(mode: InfrastructureMode) -> Self {
        Self { current_mode: mode }
    }

    pub fn get_active_profile(&self) -> Box<dyn InfrastructureProfile> {
        match self.current_mode {
            InfrastructureMode::Hybrid => Box::new(HybridProfile),
            InfrastructureMode::Sovereign => Box::new(SovereignProfile),
        }
    }

    pub fn switch_mode(&mut self, mode: InfrastructureMode) {
        println!("Switching infrastructure mode to: {:?}", mode);
        // In a real impl, this would trigger sync/archive processes
        self.current_mode = mode;
    }
}
