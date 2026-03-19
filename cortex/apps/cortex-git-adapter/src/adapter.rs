use crate::github::webhook::{GithubWebhookError, GithubWebhookHeaders, verify_github_signature};
use crate::projector::project_event_to_kip;
use crate::state::{IdempotencyStore, DeliveryRecord};
use axum::{
    extract::{State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
use serde_json::Value;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::AppConfig,
    pub registry: crate::config::Registry,
    pub sink: crate::nostra::NostraSinkHandle,
    pub github_api: crate::github::api::GithubApi,
    pub metrics: crate::metrics::Metrics,
}

pub async fn github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let timeout = std::time::Duration::from_secs(state.config.request_timeout_secs.max(1));
    let result = tokio::time::timeout(timeout, github_webhook_inner(&state, headers, body)).await;
    let result = match result {
        Ok(res) => res,
        Err(_) => Err(GithubWebhookError::Internal("request timeout".to_string())),
    };

    match result {
        Ok(_) => (StatusCode::OK, axum::Json(serde_json::json!({"ok": true}))).into_response(),
        Err(GithubWebhookError::Unauthorized(message)) => {
            state.metrics.inc_delivery_rejected();
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({"ok": false, "error": message})),
            )
                .into_response()
        }
        Err(GithubWebhookError::BadRequest(message)) => {
            state.metrics.inc_delivery_rejected();
            (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({"ok": false, "error": message})),
            )
                .into_response()
        }
        Err(GithubWebhookError::Internal(message)) => {
            state.metrics.inc_delivery_failed();
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({"ok": false, "error": message})),
            )
                .into_response()
        }
    }
}

async fn github_webhook_inner(
    state: &AppState,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(), GithubWebhookError> {
    let secret = state
        .config
        .webhook_secret
        .as_deref()
        .ok_or_else(|| GithubWebhookError::Internal("missing webhook secret".to_string()))?;

    let parsed_headers = GithubWebhookHeaders::from_headers(&headers)?;
    verify_github_signature(secret, &parsed_headers, &body)?;

    if parsed_headers.event.as_str() == "ping" {
        state.metrics.inc_delivery_accepted();
        return Ok(());
    }

    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| GithubWebhookError::BadRequest(format!("invalid json payload: {e}")))?;

    let repo_full_name = crate::github::webhook::repo_full_name_from_payload(&payload)
        .ok_or_else(|| GithubWebhookError::BadRequest("payload missing repository.full_name".to_string()))?;

    let repo_cfg = state
        .registry
        .repo_by_full_name(repo_full_name.as_str())
        .ok_or_else(|| {
            state.metrics.inc_delivery_rejected();
            GithubWebhookError::BadRequest("repo not registered".to_string())
        })?;

    if !repo_cfg.enabled {
        return Ok(());
    }

    let idempotency = IdempotencyStore::new(&state.config.state_dir);
    let record = DeliveryRecord::new(
        parsed_headers.delivery_id.as_str(),
        parsed_headers.event.as_str(),
        repo_full_name.as_str(),
        &body,
    );

    if idempotency
        .is_seen(parsed_headers.delivery_id.as_str())
        .map_err(GithubWebhookError::Internal)?
    {
        state.metrics.inc_delivery_idempotent();
        tracing::info!(
            delivery_id=%parsed_headers.delivery_id,
            event=%parsed_headers.event,
            repo=%repo_full_name,
            "webhook delivery already processed (idempotent)"
        );
        return Ok(());
    }

    let commands = project_event_to_kip(
        repo_cfg,
        parsed_headers.event.as_str(),
        &payload,
        &state.config.projector_settings(),
    )
        .map_err(|e| GithubWebhookError::BadRequest(format!("projection failed: {e}")))?;
    for cmd in commands {
        state.sink.execute_kip(cmd.as_str()).await.map_err(|e| {
            state.metrics.inc_execute_kip_failure();
            GithubWebhookError::Internal(format!("nostra executeKip failed: {e}"))
        })?;
    }

    idempotency
        .mark_seen(&record)
        .map_err(GithubWebhookError::Internal)?;
    state.metrics.inc_delivery_accepted();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, Registry, RepoConfig};
    use crate::github::api::GithubApi;
    use crate::nostra::sink::{NostraSink, NostraSinkHandle};
    use axum::body::Body;
    use axum::routing::post;
    use http::Request;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    #[derive(Default)]
    struct MockSink {
        commands: Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl NostraSink for MockSink {
        async fn execute_kip(&self, command: &str) -> Result<Value, String> {
            self.commands.lock().await.push(command.to_string());
            Ok(serde_json::json!({"ok": true}))
        }
    }

    fn signature(secret: &str, body: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let computed = mac.finalize().into_bytes();
        let bytes: &[u8] = computed.as_ref();
        format!("sha256={}", hex::encode(bytes))
    }

    fn base_config(state_dir: std::path::PathBuf, secret: &str) -> AppConfig {
        let registry_path = state_dir.join("registry.toml");
        let dfx_project_root = state_dir.join("nostra");
        AppConfig {
            bind: "127.0.0.1".to_string(),
            port: 8787,
            registry_path,
            state_dir,
            max_request_body_bytes: 1024 * 1024,
            request_timeout_secs: 10,
            delivery_retention_days: 30,
            reconcile_per_page: 50,
            reconcile_max_pages: 10,
            projector_emit_attributes: false,
            projector_store_author_email: false,
            webhook_secret: Some(secret.to_string()),
            github_api_base: "https://api.github.com".to_string(),
            github_token: Some("t".to_string()),
            github_app_id: None,
            github_app_installation_id: None,
            github_app_private_key_pem: None,
            github_app_private_key_path: None,
            nostra_ic_host: "http://127.0.0.1:4943".to_string(),
            nostra_kip_canister_id: None,
            kip_method: "execute_kip_mutation".to_string(),
            use_dfx: true,
            dfx_canister_name: "backend".to_string(),
            dfx_project_root: Some(dfx_project_root),
        }
    }

    #[tokio::test]
    async fn rejects_invalid_signature() {
        let tmp = tempfile::tempdir().unwrap();
        crate::state::ensure_state_dirs(tmp.path()).unwrap();

        let secret = "secret";
        let config = base_config(tmp.path().to_path_buf(), secret);
        let repo_cfg = RepoConfig {
            enabled: true,
            repo_full_name: "o/r".to_string(),
            branch: Some("main".to_string()),
            space_id: "s".to_string(),
            ingest_push: true,
            ingest_pull_request: true,
            ingest_issues: true,
            interval_secs: 3600,
            lookback_secs: 3600,
            tags: vec![],
        };
        let registry = Registry {
            repos: vec![repo_cfg],
        };

        let sink = NostraSinkHandle::new(Arc::new(MockSink::default()));
        let github_api = GithubApi::new(&config).await.unwrap();
        let state = AppState {
            config,
            registry,
            sink,
            github_api,
            metrics: crate::metrics::Metrics::new(),
        };

        let body = br#"{"repository":{"full_name":"o/r","html_url":"https://github.com/o/r","default_branch":"main","visibility":"public"},"commits":[]}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/webhooks/github")
            .header("x-github-event", "push")
            .header("x-github-delivery", "d1")
            .header("x-hub-signature-256", "sha256=deadbeef")
            .body(Body::from(body.as_slice()))
            .unwrap();

        let app = axum::Router::new()
            .route("/webhooks/github", post(github_webhook))
            .with_state(state);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn delivery_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        crate::state::ensure_state_dirs(tmp.path()).unwrap();

        let secret = "secret";
        let config = base_config(tmp.path().to_path_buf(), secret);
        let repo_cfg = RepoConfig {
            enabled: true,
            repo_full_name: "o/r".to_string(),
            branch: Some("main".to_string()),
            space_id: "s".to_string(),
            ingest_push: true,
            ingest_pull_request: true,
            ingest_issues: true,
            interval_secs: 3600,
            lookback_secs: 3600,
            tags: vec![],
        };
        let registry = Registry {
            repos: vec![repo_cfg],
        };

        let sink_impl = Arc::new(MockSink::default());
        let sink = NostraSinkHandle::new(sink_impl.clone());
        let github_api = GithubApi::new(&config).await.unwrap();
        let state = AppState {
            config,
            registry,
            sink,
            github_api,
            metrics: crate::metrics::Metrics::new(),
        };

        let body = br#"{"repository":{"full_name":"o/r","html_url":"https://github.com/o/r","default_branch":"main","visibility":"public"},"commits":[{"id":"abc","message":"m","url":"u","timestamp":"t","author":{"name":"n","email":"e"}}]}"#;
        let sig = signature(secret, body.as_slice());

        let build_req = || {
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("x-github-event", "push")
                .header("x-github-delivery", "d1")
                .header("x-hub-signature-256", sig.as_str())
                .body(Body::from(body.as_slice()))
                .unwrap()
        };

        let app = axum::Router::new()
            .route("/webhooks/github", post(github_webhook))
            .with_state(state);

        let resp1 = app.clone().oneshot(build_req()).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);
        let after_first = sink_impl.commands.lock().await.len();
        let resp2 = app.oneshot(build_req()).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);

        let after_second = sink_impl.commands.lock().await.len();
        assert!(after_first > 0);
        assert_eq!(after_first, after_second);
    }
}
