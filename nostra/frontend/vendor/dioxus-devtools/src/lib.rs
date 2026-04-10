use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadMsg {
    pub for_build_id: Option<u64>,
    pub assets: Vec<String>,
    pub ms_elapsed: u64,
    pub templates: Vec<TemplateStub>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStub {
    pub name: String,
    pub roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevserverMsg {
    HotReload(HotReloadMsg),
    Shutdown,
    FullReloadStart,
    HotPatchStart,
    FullReloadFailed,
    FullReloadCommand,
}

// Most permissive signature to eat whatever references are passed
// If they pass &mut VirtualDom, T ref match or T move match.
pub fn apply_changes<T, U>(_: T, _: U) {}

pub fn init() {}
