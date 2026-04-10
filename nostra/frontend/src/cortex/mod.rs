#![allow(dead_code)]

pub mod components;
pub mod inspector;
pub mod layout;
pub mod services;
pub mod ux_contract;
pub mod workflow_editor;
pub mod workbench;
// pub mod a2ui_renderer; // Will add later


#[derive(PartialEq, Clone, Copy)]
pub enum CortexMode {
    Universe, // Graph View
    Factory,  // Workflow Editor
    Workbench, // Flow Graph + Traces + Logs
    Library,  // Schema Registry
    Audit,    // Logs
}
