pub mod dispatch;
pub mod local;
pub mod routes;
pub mod state;
pub mod types;

pub use dispatch::GatewayDispatcher;
pub use routes::{
    GatewayRouteDescriptor, GatewayRouteMatch, GatewayRouteResolutionError, resolve_route,
};
pub use state::GatewayRuntimeState;
pub use types::{
    GatewayDispatchError, GatewayDispatchErrorClass, GatewayErrorEnvelope,
    GatewayIdempotencyOutcome, GatewayIdempotencySemantics, GatewayRequestEnvelope,
    GatewayResponseEnvelope, GatewayRouteMetadata, GatewayTransactionBoundary,
};
