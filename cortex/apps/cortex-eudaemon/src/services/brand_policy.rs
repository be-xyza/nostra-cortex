use crate::services::gateway_config::gateway_base;
use cortex_domain::brand::policy::BrandPolicyDocument;
use cortex_ic_adapter::brand_policy::BrandPolicyCanisterClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct BrandPolicyRegistryService {
    inner: BrandPolicyCanisterClient,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrandPolicyBundle {
    pub policy: BrandPolicyDocument,
    pub policy_version: u64,
    pub policy_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GatewayBrandPolicyResponse {
    pub policy: BrandPolicyDocument,
    pub policy_version: u64,
    pub policy_digest: String,
    pub active_temporal_state: String,
    pub server_time_utc: String,
    pub source_of_truth: String,
    #[serde(default)]
    pub degraded_reason: Option<String>,
    #[serde(default)]
    pub policy_normalization: Option<String>,
}

impl BrandPolicyRegistryService {
    pub async fn from_env() -> Result<Self, String> {
        let inner = BrandPolicyCanisterClient::from_env().await.map_err(|err| err.to_string())?;
        Ok(Self { inner })
    }

    pub async fn get_brand_policy_bundle(&self) -> Result<BrandPolicyBundle, String> {
        let policy = self
            .inner
            .get_brand_policy()
            .await
            .map_err(|err| err.to_string())?;
        let policy_version = self
            .inner
            .get_brand_policy_version()
            .await
            .map_err(|err| err.to_string())?;
        let policy_digest = self
            .inner
            .get_brand_policy_digest()
            .await
            .map_err(|err| err.to_string())?;

        Ok(BrandPolicyBundle {
            policy,
            policy_version,
            policy_digest,
        })
    }
}

pub async fn fetch_gateway_brand_policy() -> Option<GatewayBrandPolicyResponse> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/system/brand-policy", gateway_base()))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    response.json::<GatewayBrandPolicyResponse>().await.ok()
}
