//! # Nostra Workflow Core
//!
//! Core traits and types for durable workflow execution on ICP.
//!
//! Implements patterns from:
//! - Temporal.io (Durable Execution, History-First)
//! - n8n (Node Execution Stack, Waiting Registry)
//! - CNCF Serverless Workflow Specification
//!
//! ## Key Traits
//!
//! - [`DurableActivity`]: Defines executable units with retry/compensation
//! - [`WorkflowExecutor`]: Core state machine driver
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nostra_workflow_core::{WorkflowDefinition, WorkflowInstance, Engine};
//!
//! let def = WorkflowDefinition::from_json(json)?;
//! let mut instance = WorkflowInstance::new("inst-1", def);
//! Engine::tick(&mut instance);
//! ```

pub mod alignment;
pub mod batching;
pub mod builder;
pub mod debates;
pub mod engine;
pub mod primitives;
pub mod saga;
pub mod system_ops;
pub mod traits;
pub mod types;

pub use engine::Engine;
pub use primitives::{
    Action, AsyncProviderStrategy, AsyncRetryPolicy, AsyncRetryStrategy, Step, Transition,
};
pub use system_ops::{SystemOpHandler, SystemOpResult};
pub use traits::{DurableActivity, WorkflowExecutor};
pub use types::{
    Context, RoleId, SimulationReport, StepId, UserId, WorkflowDefinition, WorkflowId,
    WorkflowInstance, WorkflowStatus,
};
