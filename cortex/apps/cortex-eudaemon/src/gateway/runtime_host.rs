use async_trait::async_trait;
use cortex_runtime::gateway::local::{
    LocalGatewayMutationRecord, LocalGatewayProbe, apply_queue_action, export_queue_json, probe,
    queue_snapshot,
};
#[cfg(feature = "service-scaffolds")]
use cortex_runtime::gateway::local::{
    LocalGatewayMutationSubmit, is_online, set_online, submit_mutation,
};
use cortex_runtime::gateway::types::{
    GatewayDispatchErrorClass, GatewayIdempotencySemantics, GatewayRequestEnvelope,
    GatewayResponseEnvelope, GatewayRouteMetadata, GatewayTransactionBoundary,
};
use cortex_runtime::gateway::{
    GatewayDispatcher, GatewayRouteDescriptor, GatewayRouteResolutionError, GatewayRuntimeState,
    resolve_route,
};
use cortex_runtime::ports::{GatewayHostAdapter, LocalGatewayOrchestrationAdapter};
use cortex_runtime::{GatewayLegacyDispatchMode, RuntimeError};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::LazyLock;

#[derive(Default)]
pub struct DesktopGatewayRuntimeHost;

#[derive(Debug, Deserialize)]
struct GatewayProtocolContract {
    endpoints: Vec<GatewayProtocolEndpoint>,
}

#[derive(Debug, Deserialize)]
struct GatewayProtocolEndpoint {
    method: String,
    path_template: String,
    transaction_boundary: String,
    #[serde(default)]
    event_emissions: Vec<String>,
    idempotency_semantics: GatewayProtocolIdempotency,
}

#[derive(Debug, Deserialize)]
struct GatewayProtocolIdempotency {
    mode: String,
}

static ROUTE_DESCRIPTORS: LazyLock<Vec<GatewayRouteDescriptor>> = LazyLock::new(|| {
    let contract_raw = include_str!(
        "../../../../../research/118-cortex-runtime-extraction/GATEWAY_PROTOCOL_CONTRACT_2026-02-16.json"
    );
    let contract: GatewayProtocolContract =
        serde_json::from_str(contract_raw).expect("invalid gateway protocol contract json");
    contract
        .endpoints
        .into_iter()
        .map(|endpoint| GatewayRouteDescriptor {
            method: endpoint.method.to_ascii_uppercase(),
            path_template: endpoint.path_template,
            idempotency_semantics: parse_idempotency_semantics(
                endpoint.idempotency_semantics.mode.as_str(),
            ),
            transaction_boundary: parse_transaction_boundary(
                endpoint.transaction_boundary.as_str(),
            ),
            expected_event_emissions: endpoint.event_emissions,
        })
        .collect()
});

static RUNTIME_STATE: LazyLock<GatewayRuntimeState> = LazyLock::new(GatewayRuntimeState::default);

fn local_gateway_adapter() -> &'static dyn LocalGatewayOrchestrationAdapter {
    crate::services::local_gateway_bridge::local_gateway_adapter()
}

fn parse_idempotency_semantics(raw: &str) -> GatewayIdempotencySemantics {
    match raw {
        "optional_header" => GatewayIdempotencySemantics::OptionalHeader,
        "recommended_header" => GatewayIdempotencySemantics::RecommendedHeader,
        "required_header" => GatewayIdempotencySemantics::RequiredHeader,
        _ => GatewayIdempotencySemantics::NotApplicable,
    }
}

fn parse_transaction_boundary(raw: &str) -> GatewayTransactionBoundary {
    match raw {
        "single_request_mutation" => GatewayTransactionBoundary::SingleRequestMutation,
        "multi_step_best_effort" => GatewayTransactionBoundary::MultiStepBestEffort,
        "host_managed" => GatewayTransactionBoundary::HostManaged,
        "streaming_session" => GatewayTransactionBoundary::StreamingSession,
        _ => GatewayTransactionBoundary::ReadOnly,
    }
}

#[async_trait]
impl GatewayHostAdapter for DesktopGatewayRuntimeHost {
    fn route_exists(&self, method: &str, path: &str) -> bool {
        self.resolve_route(method, path).ok().flatten().is_some()
    }

    fn resolve_route(
        &self,
        method: &str,
        path: &str,
    ) -> Result<Option<GatewayRouteMetadata>, RuntimeError> {
        match resolve_route(&ROUTE_DESCRIPTORS, method, path) {
            Ok(Some(resolved)) => Ok(Some(resolved.metadata)),
            Ok(None) => Ok(None),
            Err(GatewayRouteResolutionError::AmbiguousTemplateMatch) => Err(RuntimeError::Domain(
                format!("ambiguous gateway route template match for {method} {path}"),
            )),
        }
    }

    fn idempotency_semantics(&self, method: &str, path: &str) -> GatewayIdempotencySemantics {
        self.resolve_route(method, path)
            .ok()
            .flatten()
            .map(|route| route.idempotency_semantics)
            .unwrap_or(GatewayIdempotencySemantics::NotApplicable)
    }

    fn transaction_boundary(&self, method: &str, path: &str) -> GatewayTransactionBoundary {
        self.resolve_route(method, path)
            .ok()
            .flatten()
            .map(|route| route.transaction_boundary)
            .unwrap_or(GatewayTransactionBoundary::ReadOnly)
    }

    fn expected_event_emissions(&self, method: &str, path: &str) -> Vec<String> {
        self.resolve_route(method, path)
            .ok()
            .flatten()
            .map(|route| route.expected_event_emissions)
            .unwrap_or_default()
    }

    async fn dispatch(
        &self,
        request: &GatewayRequestEnvelope,
    ) -> Result<GatewayResponseEnvelope, RuntimeError> {
        match crate::services::gateway_config::gateway_legacy_dispatch_mode() {
            GatewayLegacyDispatchMode::InProcess => {
                match crate::gateway::server::dispatch_legacy_api_request_in_process(request).await
                {
                    Ok(response) => Ok(response),
                    Err(err)
                        if err
                            .to_string()
                            .contains("legacy in-process router unavailable") =>
                    {
                        dispatch_http_loopback(request).await
                    }
                    Err(err) => Err(err),
                }
            }
            GatewayLegacyDispatchMode::HttpLoopback => dispatch_http_loopback(request).await,
        }
    }
}

async fn dispatch_http_loopback(
    request: &GatewayRequestEnvelope,
) -> Result<GatewayResponseEnvelope, RuntimeError> {
    let client = reqwest::Client::new();
    let base = crate::services::gateway_config::gateway_base();
    let mut uri = format!("{}{}", base, request.path);
    if let Some(query) = request.query.as_ref() {
        if !query.trim().is_empty() {
            uri.push('?');
            uri.push_str(query);
        }
    }

    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|err| RuntimeError::Domain(format!("invalid request method: {err}")))?;
    let mut outbound = client
        .request(method, uri)
        .header(crate::gateway::server::LEGACY_BYPASS_HEADER, "1");

    for (key, value) in &request.headers {
        if !should_forward_request_header(key.as_str()) {
            continue;
        }
        outbound = outbound.header(key, value);
    }

    if let Some(payload) = request.body.as_ref() {
        outbound = outbound.json(payload);
    }

    let response = match outbound.send().await {
        Ok(response) => response,
        Err(err) => {
            return Ok(classify_reqwest_error(&err));
        }
    };

    let status = response.status().as_u16();
    let mut headers = std::collections::BTreeMap::new();
    for (key, value) in response.headers() {
        if !should_forward_response_header(key.as_str()) {
            continue;
        }
        if let Ok(value) = value.to_str() {
            headers.insert(key.as_str().to_string(), value.to_string());
        }
    }

    let raw_body = response.text().await.unwrap_or_default();
    let parsed_json = serde_json::from_str::<Value>(&raw_body).ok();
    if parsed_json.is_none() && status >= 400 {
        return Ok(GatewayResponseEnvelope::passthrough_error(
            status,
            GatewayDispatchErrorClass::UpstreamInvalidBody,
            "legacy_upstream_invalid_json",
            "legacy upstream returned non-json error payload",
            status >= 500,
            Some(json!({
                "status": status,
                "rawBody": raw_body,
            })),
        ));
    }

    Ok(GatewayResponseEnvelope {
        status,
        headers,
        body: parsed_json.unwrap_or(Value::Null),
        route_template: None,
        error: None,
        dispatch_error: None,
        transaction_boundary: GatewayTransactionBoundary::HostManaged,
        event_emissions: Vec::new(),
        idempotency: Default::default(),
    })
}

fn classify_reqwest_error(err: &reqwest::Error) -> GatewayResponseEnvelope {
    if err.is_timeout() {
        return GatewayResponseEnvelope::passthrough_error(
            504,
            GatewayDispatchErrorClass::UpstreamTimeout,
            "legacy_upstream_timeout",
            "legacy upstream request timed out",
            true,
            Some(json!({ "reason": err.to_string() })),
        );
    }

    GatewayResponseEnvelope::passthrough_error(
        502,
        GatewayDispatchErrorClass::UpstreamNetwork,
        "legacy_upstream_network",
        "legacy upstream request failed",
        true,
        Some(json!({ "reason": err.to_string() })),
    )
}

fn should_forward_request_header(header: &str) -> bool {
    !matches!(
        header.to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "host"
            | "content-length"
            | "x-cortex-runtime-legacy-bypass"
    )
}

fn should_forward_response_header(header: &str) -> bool {
    !matches!(
        header.to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "content-length"
    )
}

pub async fn dispatch_request(
    request: GatewayRequestEnvelope,
) -> Result<GatewayResponseEnvelope, RuntimeError> {
    let host = DesktopGatewayRuntimeHost;
    let dispatcher = GatewayDispatcher::new(&host, &RUNTIME_STATE);
    dispatcher.handle_request(request).await
}

pub fn local_gateway_queue_snapshot() -> Vec<LocalGatewayMutationRecord> {
    queue_snapshot(local_gateway_adapter()).unwrap_or_default()
}

pub fn local_gateway_export_queue_json() -> Result<String, String> {
    export_queue_json(local_gateway_adapter()).map_err(|err| err.to_string())
}

pub fn local_gateway_apply_queue_action(mutation_id: &str, action: &str) -> Result<(), String> {
    apply_queue_action(local_gateway_adapter(), mutation_id, action).map_err(|err| err.to_string())
}

pub fn local_gateway_probe() -> LocalGatewayProbe {
    probe(local_gateway_adapter()).unwrap_or(LocalGatewayProbe {
        queue_size: 0,
        queue_export_ok: false,
        gateway_online: false,
    })
}

#[cfg(feature = "service-scaffolds")]
pub fn set_local_gateway_online(status: bool) {
    let _ = set_online(local_gateway_adapter(), status);
}

#[cfg(feature = "service-scaffolds")]
pub fn local_gateway_is_online() -> bool {
    is_online(local_gateway_adapter()).unwrap_or(false)
}

#[cfg(feature = "service-scaffolds")]
pub fn submit_local_gateway_mutation(
    mutation: crate::services::local_gateway::Mutation,
) -> Result<String, String> {
    let request = LocalGatewayMutationSubmit {
        mutation_id: mutation.id,
        idempotency_key: mutation.idempotency_key,
        space_id: mutation.space_id,
        kip_command: mutation.kip_command,
        timestamp: mutation.timestamp,
        attempts: mutation.attempts,
        last_error: mutation.last_error,
        last_attempt_at: mutation.last_attempt_at,
    };
    submit_mutation(local_gateway_adapter(), request).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn contract_transaction_boundary_counts_match_baseline() {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for descriptor in ROUTE_DESCRIPTORS.iter() {
            let key = match descriptor.transaction_boundary {
                GatewayTransactionBoundary::ReadOnly => "read_only",
                GatewayTransactionBoundary::SingleRequestMutation => "single_request_mutation",
                GatewayTransactionBoundary::MultiStepBestEffort => "multi_step_best_effort",
                GatewayTransactionBoundary::HostManaged => "host_managed",
                GatewayTransactionBoundary::StreamingSession => "streaming_session",
            };
            *counts.entry(key.to_string()).or_default() += 1;
        }

        assert_eq!(counts.get("read_only"), Some(&105usize));
        assert_eq!(counts.get("single_request_mutation"), Some(&19usize));
        assert_eq!(counts.get("multi_step_best_effort"), Some(&2usize));
        assert_eq!(counts.get("host_managed"), Some(&53usize));
        assert_eq!(counts.get("streaming_session"), Some(&2usize));
    }

    #[test]
    fn inventory_templates_resolve_without_ambiguity() {
        let host = DesktopGatewayRuntimeHost;
        let inventory =
            include_str!("../../tests/fixtures/gateway_baseline/endpoint_inventory.tsv");
        for line in inventory.lines() {
            let mut parts = line.split('\t');
            let method = parts.next().expect("missing method");
            let path = parts.next().expect("missing path");
            let resolved = host
                .resolve_route(method, path)
                .expect("route resolution should not fail");
            assert!(
                resolved.is_some(),
                "inventory endpoint did not resolve: {method} {path}"
            );
        }
    }

    #[test]
    fn hop_by_hop_headers_are_not_forwarded() {
        assert!(!should_forward_request_header("Connection"));
        assert!(!should_forward_response_header("Transfer-Encoding"));
        assert!(should_forward_request_header("x-request-id"));
        assert!(should_forward_response_header("content-type"));
    }
}
