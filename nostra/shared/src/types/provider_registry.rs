use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    Llm,
    Embedding,
    Vector,
    Batch,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LlmProviderType {
    OpenAI,
    Anthropic,
    Ollama,
    Ignition,
    OpenRouter,
    DoubleWord,
    Mock,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderBatchCadenceKind {
    Immediate,
    Interval,
    TimeWindow,
    Scoped,
    Manual,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderBatchScopeKind {
    ProviderFamily,
    ProviderProfile,
    Space,
    Agent,
    Session,
    RequestGroup,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderBatchFlushPolicy {
    OnInterval,
    OnWindowClose,
    OnThreshold,
    OnIdle,
    Manual,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBatchWindow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBatchPolicy {
    pub provider_family_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_profile_id: Option<String>,
    pub cadence_kind: ProviderBatchCadenceKind,
    pub scope_kind: ProviderBatchScopeKind,
    pub flush_policy: ProviderBatchFlushPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordering_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedupe_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_window: Option<ProviderBatchWindow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub llm_type: Option<LlmProviderType>,
    pub endpoint: String,
    pub is_active: bool,
    pub priority: u32,
    pub config_json: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_policy: Option<ProviderBatchPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl ProviderRecord {
    pub fn new_llm(id: &str, name: &str, llm_type: LlmProviderType, endpoint: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            provider_type: ProviderType::Llm,
            llm_type: Some(llm_type),
            endpoint: endpoint.to_string(),
            is_active: true,
            priority: 10,
            config_json: None,
            batch_policy: None,
            metadata: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_batch_policy_round_trips_through_serde() {
        let policy = ProviderBatchPolicy {
            provider_family_id: "doubleword".to_string(),
            provider_profile_id: Some("batch.small".to_string()),
            cadence_kind: ProviderBatchCadenceKind::Interval,
            scope_kind: ProviderBatchScopeKind::Space,
            flush_policy: ProviderBatchFlushPolicy::OnInterval,
            ordering_key: Some("space_id".to_string()),
            dedupe_key: Some("request_hash".to_string()),
            batch_window: Some(ProviderBatchWindow {
                interval_seconds: Some(60),
                max_items: Some(100),
                max_age_seconds: Some(600),
                timezone: Some("UTC".to_string()),
            }),
        };

        let encoded = serde_json::to_string(&policy).expect("serialize batch policy");
        let decoded: ProviderBatchPolicy =
            serde_json::from_str(&encoded).expect("deserialize batch policy");

        assert_eq!(decoded, policy);
    }
}
