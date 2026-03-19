use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRequestEnvelope {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub path_template: Option<String>,
    #[serde(default)]
    pub path_params: BTreeMap<String, String>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<Value>,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
}

impl GatewayRequestEnvelope {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            path_template: None,
            path_params: BTreeMap::new(),
            query: None,
            headers: BTreeMap::new(),
            body: None,
            idempotency_key: None,
            actor_id: None,
            request_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRouteMetadata {
    pub path_template: String,
    #[serde(default)]
    pub path_params: BTreeMap<String, String>,
    pub idempotency_semantics: GatewayIdempotencySemantics,
    pub transaction_boundary: GatewayTransactionBoundary,
    #[serde(default)]
    pub expected_event_emissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayResponseEnvelope {
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub body: Value,
    #[serde(default)]
    pub route_template: Option<String>,
    #[serde(default)]
    pub error: Option<GatewayErrorEnvelope>,
    #[serde(default)]
    pub dispatch_error: Option<GatewayDispatchError>,
    pub transaction_boundary: GatewayTransactionBoundary,
    #[serde(default)]
    pub event_emissions: Vec<String>,
    pub idempotency: GatewayIdempotencyOutcome,
}

impl GatewayResponseEnvelope {
    pub fn ok(body: Value) -> Self {
        Self {
            status: 200,
            headers: BTreeMap::new(),
            body,
            route_template: None,
            error: None,
            dispatch_error: None,
            transaction_boundary: GatewayTransactionBoundary::ReadOnly,
            event_emissions: Vec::new(),
            idempotency: GatewayIdempotencyOutcome::default(),
        }
    }

    pub fn not_found(method: impl Into<String>, path: impl Into<String>) -> Self {
        let method = method.into();
        let path = path.into();
        Self {
            status: 404,
            headers: BTreeMap::new(),
            body: json!({
                "error": {
                    "code": "route_not_found",
                    "message": format!("no gateway route for {} {}", method, path),
                    "details": {"method": method, "path": path}
                }
            }),
            route_template: None,
            error: Some(GatewayErrorEnvelope {
                code: "route_not_found".to_string(),
                message: "gateway route was not found".to_string(),
                details: Some(json!({"method": method, "path": path})),
                retryable: false,
            }),
            dispatch_error: Some(GatewayDispatchError {
                class: GatewayDispatchErrorClass::RouteNotFound,
                code: "route_not_found".to_string(),
                retryable: false,
                upstream_status: None,
            }),
            transaction_boundary: GatewayTransactionBoundary::ReadOnly,
            event_emissions: Vec::new(),
            idempotency: GatewayIdempotencyOutcome::default(),
        }
    }

    pub fn not_implemented(
        method: impl Into<String>,
        path: impl Into<String>,
        details: Value,
    ) -> Self {
        let method = method.into();
        let path = path.into();
        Self {
            status: 501,
            headers: BTreeMap::new(),
            body: json!({
                "error": {
                    "code": "runtime_gateway_not_implemented",
                    "message": "runtime gateway handler is not implemented",
                    "details": {
                        "method": method,
                        "path": path,
                        "context": details
                    }
                }
            }),
            route_template: None,
            error: Some(GatewayErrorEnvelope {
                code: "runtime_gateway_not_implemented".to_string(),
                message: "runtime gateway handler is not implemented".to_string(),
                details: Some(details),
                retryable: false,
            }),
            dispatch_error: Some(GatewayDispatchError {
                class: GatewayDispatchErrorClass::RuntimeInternal,
                code: "runtime_gateway_not_implemented".to_string(),
                retryable: false,
                upstream_status: None,
            }),
            transaction_boundary: GatewayTransactionBoundary::HostManaged,
            event_emissions: Vec::new(),
            idempotency: GatewayIdempotencyOutcome::default(),
        }
    }

    pub fn passthrough_error(
        status: u16,
        class: GatewayDispatchErrorClass,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
        details: Option<Value>,
    ) -> Self {
        let code = code.into();
        let message = message.into();
        let details_for_body = details.clone();
        Self {
            status,
            headers: BTreeMap::new(),
            body: json!({
                "error": {
                    "code": code,
                    "message": message,
                    "details": details_for_body
                }
            }),
            route_template: None,
            error: Some(GatewayErrorEnvelope {
                code: code.clone(),
                message: message.clone(),
                details,
                retryable,
            }),
            dispatch_error: Some(GatewayDispatchError {
                class,
                code,
                retryable,
                upstream_status: Some(status),
            }),
            transaction_boundary: GatewayTransactionBoundary::HostManaged,
            event_emissions: Vec::new(),
            idempotency: GatewayIdempotencyOutcome::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayErrorEnvelope {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<Value>,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayDispatchError {
    pub class: GatewayDispatchErrorClass,
    pub code: String,
    pub retryable: bool,
    #[serde(default)]
    pub upstream_status: Option<u16>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayDispatchErrorClass {
    RouteNotFound,
    UpstreamTimeout,
    UpstreamNetwork,
    UpstreamInvalidBody,
    Upstream5xx,
    Upstream4xx,
    RuntimeInternal,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayTransactionBoundary {
    ReadOnly,
    SingleRequestMutation,
    MultiStepBestEffort,
    HostManaged,
    StreamingSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayIdempotencySemantics {
    NotApplicable,
    OptionalHeader,
    RecommendedHeader,
    RequiredHeader,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayIdempotencyOutcome {
    pub semantics: GatewayIdempotencySemantics,
    pub replayed: bool,
    #[serde(default)]
    pub cache_key: Option<String>,
}

impl Default for GatewayIdempotencyOutcome {
    fn default() -> Self {
        Self {
            semantics: GatewayIdempotencySemantics::NotApplicable,
            replayed: false,
            cache_key: None,
        }
    }
}
