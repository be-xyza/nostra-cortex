use crate::RuntimeError;
use crate::gateway::state::GatewayRuntimeState;
use crate::gateway::types::{
    GatewayDispatchError, GatewayDispatchErrorClass, GatewayIdempotencyOutcome,
    GatewayIdempotencySemantics, GatewayRequestEnvelope, GatewayResponseEnvelope,
};
use crate::ports::GatewayHostAdapter;
use serde_json::to_string;

pub struct GatewayDispatcher<'a> {
    host: &'a dyn GatewayHostAdapter,
    state: &'a GatewayRuntimeState,
}

impl<'a> GatewayDispatcher<'a> {
    pub fn new(host: &'a dyn GatewayHostAdapter, state: &'a GatewayRuntimeState) -> Self {
        Self { host, state }
    }

    pub async fn handle_request(
        &self,
        mut request: GatewayRequestEnvelope,
    ) -> Result<GatewayResponseEnvelope, RuntimeError> {
        request.method = request.method.trim().to_ascii_uppercase();

        let Some(route) = self.host.resolve_route(&request.method, &request.path)? else {
            return Ok(GatewayResponseEnvelope::not_found(
                request.method,
                request.path,
            ));
        };
        request.path_template = Some(route.path_template.clone());
        request.path_params = route.path_params.clone();

        let semantics = route.idempotency_semantics;
        let cache_key = self.cache_key(&request, &semantics)?;

        if let Some(cache_key) = cache_key.as_ref() {
            if let Some(mut cached) = self.state.replay(cache_key) {
                cached.idempotency = GatewayIdempotencyOutcome {
                    semantics: semantics.clone(),
                    replayed: true,
                    cache_key: Some(cache_key.clone()),
                };
                return Ok(cached);
            }
        }

        let mut response = self.host.dispatch(&request).await?;
        response.route_template = Some(route.path_template.clone());

        if response.event_emissions.is_empty() {
            response.event_emissions = route.expected_event_emissions.clone();
        }

        response.transaction_boundary = route.transaction_boundary;

        response.idempotency = GatewayIdempotencyOutcome {
            semantics: semantics.clone(),
            replayed: false,
            cache_key: cache_key.clone(),
        };

        if response.dispatch_error.is_none() && response.status >= 500 {
            response.dispatch_error = Some(GatewayDispatchError {
                class: GatewayDispatchErrorClass::Upstream5xx,
                code: "legacy_upstream_5xx".to_string(),
                retryable: true,
                upstream_status: Some(response.status),
            });
        } else if response.dispatch_error.is_none() && response.status >= 400 {
            response.dispatch_error = Some(GatewayDispatchError {
                class: GatewayDispatchErrorClass::Upstream4xx,
                code: "legacy_upstream_4xx".to_string(),
                retryable: false,
                upstream_status: Some(response.status),
            });
        }

        if let Some(cache_key) = cache_key {
            self.state.store(cache_key, response.clone());
        }

        Ok(response)
    }

    fn cache_key(
        &self,
        request: &GatewayRequestEnvelope,
        semantics: &GatewayIdempotencySemantics,
    ) -> Result<Option<String>, RuntimeError> {
        let applies = matches!(
            semantics,
            GatewayIdempotencySemantics::OptionalHeader
                | GatewayIdempotencySemantics::RecommendedHeader
                | GatewayIdempotencySemantics::RequiredHeader
        );

        if !applies {
            return Ok(None);
        }

        let Some(idempotency_key) = request.idempotency_key.as_ref() else {
            return Ok(None);
        };

        let body_digest = match request.body.as_ref() {
            Some(body) => to_string(body)
                .map_err(|err| RuntimeError::Serialization(err.to_string()))?
                .len(),
            None => 0,
        };

        Ok(Some(format!(
            "{}:{}:{}:{}",
            request.method, request.path, idempotency_key, body_digest
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::types::{
        GatewayIdempotencySemantics, GatewayRequestEnvelope, GatewayResponseEnvelope,
        GatewayTransactionBoundary,
    };
    use crate::ports::GatewayHostAdapter;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::Mutex;

    struct MockGatewayHost {
        dispatch_count: Mutex<u64>,
    }

    impl Default for MockGatewayHost {
        fn default() -> Self {
            Self {
                dispatch_count: Mutex::new(0),
            }
        }
    }

    #[async_trait]
    impl GatewayHostAdapter for MockGatewayHost {
        fn route_exists(&self, _method: &str, path: &str) -> bool {
            !path.contains("unknown")
        }

        fn idempotency_semantics(&self, method: &str, path: &str) -> GatewayIdempotencySemantics {
            if method == "POST" && path.contains("/mutation") {
                GatewayIdempotencySemantics::RecommendedHeader
            } else {
                GatewayIdempotencySemantics::NotApplicable
            }
        }

        fn transaction_boundary(&self, method: &str, path: &str) -> GatewayTransactionBoundary {
            if method == "POST" && path.contains("/mutation") {
                GatewayTransactionBoundary::SingleRequestMutation
            } else {
                GatewayTransactionBoundary::ReadOnly
            }
        }

        fn expected_event_emissions(&self, _method: &str, path: &str) -> Vec<String> {
            if path.contains("/decision") {
                vec!["nostra.gateway.decision.recorded".to_string()]
            } else {
                Vec::new()
            }
        }

        async fn dispatch(
            &self,
            request: &GatewayRequestEnvelope,
        ) -> Result<GatewayResponseEnvelope, RuntimeError> {
            let mut guard = self.dispatch_count.lock().unwrap();
            *guard += 1;
            Ok(GatewayResponseEnvelope::ok(json!({
                "path": request.path,
                "dispatchCount": *guard,
            })))
        }
    }

    #[test]
    fn unknown_route_returns_normalized_404() {
        futures::executor::block_on(async {
            let host = MockGatewayHost::default();
            let state = GatewayRuntimeState::default();
            let dispatcher = GatewayDispatcher::new(&host, &state);

            let request = GatewayRequestEnvelope::new("GET", "/api/unknown");
            let response = dispatcher.handle_request(request).await.unwrap();

            assert_eq!(response.status, 404);
            assert_eq!(
                response.error.as_ref().map(|error| error.code.as_str()),
                Some("route_not_found")
            );
        });
    }

    #[test]
    fn idempotency_replay_is_deterministic() {
        futures::executor::block_on(async {
            let host = MockGatewayHost::default();
            let state = GatewayRuntimeState::default();
            let dispatcher = GatewayDispatcher::new(&host, &state);

            let mut first = GatewayRequestEnvelope::new("POST", "/api/system/mutation");
            first.idempotency_key = Some("k-123".to_string());
            first.body = Some(json!({"value": 1}));

            let second = first.clone();

            let response_1 = dispatcher.handle_request(first).await.unwrap();
            let response_2 = dispatcher.handle_request(second).await.unwrap();

            assert_eq!(response_1.status, 200);
            assert_eq!(response_2.status, 200);
            assert_eq!(response_1.body, response_2.body);
            assert!(!response_1.idempotency.replayed);
            assert!(response_2.idempotency.replayed);
        });
    }

    #[test]
    fn mutation_transaction_boundary_is_enforced() {
        futures::executor::block_on(async {
            let host = MockGatewayHost::default();
            let state = GatewayRuntimeState::default();
            let dispatcher = GatewayDispatcher::new(&host, &state);

            let request = GatewayRequestEnvelope::new("POST", "/api/system/mutation");
            let response = dispatcher.handle_request(request).await.unwrap();

            assert_eq!(
                response.transaction_boundary,
                GatewayTransactionBoundary::SingleRequestMutation
            );
        });
    }

    #[test]
    fn decision_event_emissions_are_filled_when_missing() {
        futures::executor::block_on(async {
            let host = MockGatewayHost::default();
            let state = GatewayRuntimeState::default();
            let dispatcher = GatewayDispatcher::new(&host, &state);

            let request = GatewayRequestEnvelope::new("POST", "/api/system/decision/ack");
            let response = dispatcher.handle_request(request).await.unwrap();

            assert!(
                response
                    .event_emissions
                    .iter()
                    .any(|entry| entry == "nostra.gateway.decision.recorded")
            );
        });
    }
}
