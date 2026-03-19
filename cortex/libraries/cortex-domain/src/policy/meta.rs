use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceContext {
    pub traceparent: Option<String>,
    pub tracestate: Option<String>,
    pub baggage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpMetaError {
    MetaMustBeObject,
    KeyNotNamespaced(String),
    ValueMustBeString(String),
}

impl std::fmt::Display for AcpMetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcpMetaError::MetaMustBeObject => write!(f, "_meta must be a JSON object"),
            AcpMetaError::KeyNotNamespaced(key) => {
                write!(
                    f,
                    "_meta key must be reserved or namespaced as nostra.*: {}",
                    key
                )
            }
            AcpMetaError::ValueMustBeString(key) => {
                write!(f, "_meta key value must be a string for key: {}", key)
            }
        }
    }
}

impl std::error::Error for AcpMetaError {}

pub fn validate_meta(
    meta: Option<&Value>,
) -> Result<(TraceContext, HashMap<String, Value>), AcpMetaError> {
    let mut trace = TraceContext::default();
    let mut passthrough = HashMap::new();

    let Some(meta) = meta else {
        return Ok((trace, passthrough));
    };

    let obj = meta.as_object().ok_or(AcpMetaError::MetaMustBeObject)?;

    for (key, value) in obj {
        match key.as_str() {
            "traceparent" => trace.traceparent = Some(expect_string(key, value)?),
            "tracestate" => trace.tracestate = Some(expect_string(key, value)?),
            "baggage" => trace.baggage = Some(expect_string(key, value)?),
            _ if key.starts_with("nostra.") => {
                passthrough.insert(key.clone(), value.clone());
            }
            _ => return Err(AcpMetaError::KeyNotNamespaced(key.clone())),
        }
    }

    Ok((trace, passthrough))
}

fn expect_string(key: &str, value: &Value) -> Result<String, AcpMetaError> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| AcpMetaError::ValueMustBeString(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn allows_reserved_and_nostra_namespaced_keys() {
        let meta = json!({
            "traceparent": "00-abc-123-01",
            "nostra.request_id": "req_1"
        });

        let (trace, passthrough) = validate_meta(Some(&meta)).unwrap();
        assert_eq!(trace.traceparent.as_deref(), Some("00-abc-123-01"));
        assert_eq!(passthrough.get("nostra.request_id").unwrap(), "req_1");
    }

    #[test]
    fn rejects_non_namespaced_custom_key() {
        let meta = json!({"foo": "bar"});
        let err = validate_meta(Some(&meta)).unwrap_err();
        assert!(matches!(err, AcpMetaError::KeyNotNamespaced(_)));
    }
}
