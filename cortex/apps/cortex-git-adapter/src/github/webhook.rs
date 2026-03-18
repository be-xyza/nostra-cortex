use bytes::Bytes;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub enum GithubWebhookError {
    Unauthorized(String),
    BadRequest(String),
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct GithubWebhookHeaders {
    pub event: String,
    pub delivery_id: String,
    pub signature_256: String,
}

impl GithubWebhookHeaders {
    pub fn from_headers(headers: &HeaderMap) -> Result<Self, GithubWebhookError> {
        let event = header_required(headers, "x-github-event")?;
        let delivery_id = header_required(headers, "x-github-delivery")?;
        let signature_256 = header_required(headers, "x-hub-signature-256")?;
        Ok(Self {
            event,
            delivery_id,
            signature_256,
        })
    }
}

fn header_required(headers: &HeaderMap, name: &str) -> Result<String, GithubWebhookError> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| GithubWebhookError::BadRequest(format!("missing header {name}")))
}

pub fn verify_github_signature(
    secret: &str,
    headers: &GithubWebhookHeaders,
    body: &Bytes,
) -> Result<(), GithubWebhookError> {
    let sig = headers.signature_256.trim();
    let Some(hex_digest) = sig.strip_prefix("sha256=") else {
        return Err(GithubWebhookError::Unauthorized(
            "invalid X-Hub-Signature-256 format".to_string(),
        ));
    };

    let expected = hex::decode(hex_digest).map_err(|_| {
        GithubWebhookError::Unauthorized("invalid X-Hub-Signature-256 hex".to_string())
    })?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| GithubWebhookError::Internal(format!("hmac init failed: {e}")))?;
    mac.update(body);
    let computed = mac.finalize().into_bytes();

    if expected.ct_eq(computed.as_ref()).unwrap_u8() != 1 {
        return Err(GithubWebhookError::Unauthorized(
            "signature mismatch".to_string(),
        ));
    }

    Ok(())
}

pub fn repo_full_name_from_payload(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("repository")?
        .get("full_name")?
        .as_str()
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http::HeaderMap;

    fn signature(secret: &str, body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let computed = mac.finalize().into_bytes();
        let bytes: &[u8] = computed.as_ref();
        format!("sha256={}", hex::encode(bytes))
    }

    #[test]
    fn verify_signature_accepts_known_fixture() {
        let secret = "secret";
        let body = Bytes::from_static(br#"{"hello":"world"}"#);
        let mut headers = HeaderMap::new();
        headers.insert("x-github-event", "push".parse().unwrap());
        headers.insert("x-github-delivery", "d1".parse().unwrap());
        headers.insert(
            "x-hub-signature-256",
            signature(secret, body.as_ref()).parse().unwrap(),
        );

        let parsed = GithubWebhookHeaders::from_headers(&headers).unwrap();
        verify_github_signature(secret, &parsed, &body).unwrap();
    }

    #[test]
    fn verify_signature_rejects_modified_payload() {
        let secret = "secret";
        let body = Bytes::from_static(br#"{"hello":"world"}"#);
        let mut headers = HeaderMap::new();
        headers.insert("x-github-event", "push".parse().unwrap());
        headers.insert("x-github-delivery", "d1".parse().unwrap());
        headers.insert(
            "x-hub-signature-256",
            signature(secret, br#"{"hello":"WORLD"}"#).parse().unwrap(),
        );

        let parsed = GithubWebhookHeaders::from_headers(&headers).unwrap();
        let err = verify_github_signature(secret, &parsed, &body).unwrap_err();
        match err {
            GithubWebhookError::Unauthorized(_) => {}
            other => panic!("unexpected error: {:?}", other),
        }
    }
}
