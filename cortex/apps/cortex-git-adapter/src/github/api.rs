use crate::config::AppConfig;
use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct GithubApi {
    base: String,
    client: reqwest::Client,
    auth: Arc<Mutex<AuthState>>,
}

#[derive(Clone, Debug)]
enum AuthState {
    StaticToken(String),
    App(AppAuthState),
    Missing,
}

#[derive(Clone, Debug)]
struct AppAuthState {
    app_id: String,
    installation_id: String,
    private_key_pem: String,
    cached_token: Option<CachedToken>,
}

#[derive(Clone, Debug)]
struct CachedToken {
    token: String,
    expires_at: DateTime<Utc>,
}

impl GithubApi {
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            "cortex-git-adapter".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("failed to build reqwest client")?;

        let auth = if let Some(token) = config.github_token.clone() {
            AuthState::StaticToken(token)
        } else if let (Some(app_id), Some(installation_id)) = (
            config.github_app_id.clone(),
            config.github_app_installation_id.clone(),
        ) {
            let pem = if let Some(pem) = config.github_app_private_key_pem.clone() {
                pem
            } else if let Some(path) = config.github_app_private_key_path.clone() {
                tokio::fs::read_to_string(&path)
                    .await
                    .with_context(|| format!("failed to read github app private key {}", path.display()))?
            } else {
                return Ok(Self {
                    base: config.github_api_base.clone(),
                    client,
                    auth: Arc::new(Mutex::new(AuthState::Missing)),
                });
            };

            AuthState::App(AppAuthState {
                app_id,
                installation_id,
                private_key_pem: pem,
                cached_token: None,
            })
        } else {
            AuthState::Missing
        };

        Ok(Self {
            base: config.github_api_base.clone(),
            client,
            auth: Arc::new(Mutex::new(auth)),
        })
    }

    async fn bearer_token(&self) -> anyhow::Result<String> {
        enum Decision {
            Token(String),
            RefreshApp {
                app_id: String,
                installation_id: String,
                private_key_pem: String,
            },
            Missing,
        }

        let decision = {
            let mut guard = self.auth.lock().await;
            match &mut *guard {
                AuthState::StaticToken(token) => Decision::Token(token.clone()),
                AuthState::App(state) => {
                    if let Some(cached) = &state.cached_token {
                        if cached.expires_at > Utc::now() + Duration::seconds(30) {
                            return Ok(cached.token.clone());
                        }
                    }
                    Decision::RefreshApp {
                        app_id: state.app_id.clone(),
                        installation_id: state.installation_id.clone(),
                        private_key_pem: state.private_key_pem.clone(),
                    }
                }
                AuthState::Missing => Decision::Missing,
            }
        };

        match decision {
            Decision::Token(token) => Ok(token),
            Decision::Missing => Err(anyhow::anyhow!(
                "missing GitHub API auth; set GITHUB_TOKEN or GitHub App env vars"
            )),
            Decision::RefreshApp {
                app_id,
                installation_id,
                private_key_pem,
            } => {
                let jwt = build_github_app_jwt(&app_id, &private_key_pem)?;
                let token = self
                    .create_installation_token(&installation_id, &jwt)
                    .await?;
                let mut guard = self.auth.lock().await;
                if let AuthState::App(state) = &mut *guard {
                    state.cached_token = Some(token.clone());
                }
                Ok(token.token)
            }
        }
    }

    async fn create_installation_token(
        &self,
        installation_id: &str,
        jwt: &str,
    ) -> anyhow::Result<CachedToken> {
        #[derive(Debug, Deserialize)]
        struct TokenResponse {
            token: String,
            expires_at: String,
        }

        let url = format!(
            "{}/app/installations/{}/access_tokens",
            self.base, installation_id
        );
        let resp = self
            .client
            .post(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", jwt))
            .send()
            .await
            .context("github installation token request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "github installation token request failed ({status}): {body}"
            ));
        }

        let decoded: TokenResponse = resp.json().await.context("decode token response failed")?;
        let expires_at = DateTime::parse_from_rfc3339(&decoded.expires_at)
            .context("invalid expires_at")?
            .with_timezone(&Utc);
        Ok(CachedToken {
            token: decoded.token,
            expires_at,
        })
    }

    pub async fn list_commits_since(
        &self,
        repo_full_name: &str,
        branch: &str,
        since: DateTime<Utc>,
        per_page: usize,
        page: usize,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let token = self.bearer_token().await?;
        let url = format!(
            "{}/repos/{}/commits?sha={}&since={}&per_page={}&page={}",
            self.base,
            repo_full_name,
            branch,
            urlencoding::encode(&since.to_rfc3339()),
            per_page,
            page
        );
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("list commits failed ({status}): {body}"));
        }
        Ok(resp.json::<Vec<serde_json::Value>>().await?)
    }

    pub async fn list_pulls_updated(
        &self,
        repo_full_name: &str,
        state: &str,
        per_page: usize,
        page: usize,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let token = self.bearer_token().await?;
        let url = format!(
            "{}/repos/{}/pulls?state={}&sort=updated&direction=desc&per_page={}&page={}",
            self.base, repo_full_name, state, per_page, page
        );
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("list pulls failed ({status}): {body}"));
        }
        Ok(resp.json::<Vec<serde_json::Value>>().await?)
    }

    pub async fn list_issues_since(
        &self,
        repo_full_name: &str,
        since: DateTime<Utc>,
        per_page: usize,
        page: usize,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let token = self.bearer_token().await?;
        let url = format!(
            "{}/repos/{}/issues?state=all&since={}&per_page={}&page={}",
            self.base,
            repo_full_name,
            urlencoding::encode(&since.to_rfc3339()),
            per_page,
            page
        );
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("list issues failed ({status}): {body}"));
        }
        Ok(resp.json::<Vec<serde_json::Value>>().await?)
    }
}

#[derive(Debug, Serialize)]
struct AppClaims<'a> {
    iat: usize,
    exp: usize,
    iss: &'a str,
}

fn build_github_app_jwt(app_id: &str, private_key_pem: &str) -> anyhow::Result<String> {
    let now = Utc::now();
    let iat = (now - Duration::seconds(10)).timestamp() as usize;
    let exp = (now + Duration::minutes(9)).timestamp() as usize;
    let claims = AppClaims { iat, exp, iss: app_id };
    let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .context("invalid github app private key pem")?;
    Ok(jsonwebtoken::encode(&Header::new(jsonwebtoken::Algorithm::RS256), &claims, &key)?)
}
