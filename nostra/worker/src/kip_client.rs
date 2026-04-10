use anyhow::Result;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_agent::{Agent, export::Principal};
use std::sync::Arc;

#[allow(non_camel_case_types)]
#[derive(CandidType, Deserialize, Debug)]
pub enum KipResult {
    ok(String),
    err(String),
}

#[derive(Clone)]
pub struct KipClient {
    agent: Arc<Agent>,
    canister_id: Principal,
}

impl KipClient {
    pub fn new(agent: Arc<Agent>, canister_id: Principal) -> Self {
        Self { agent, canister_id }
    }

    /// Execute a read-only KIP query (FIND, DESCRIBE, SEARCH)
    pub async fn query(&self, command: &str) -> Result<String> {
        let args = Encode!(&command)?;
        let response = self
            .agent
            .query(&self.canister_id, "execute_kip")
            .with_arg(args)
            .call()
            .await?;

        let result: KipResult = Decode!(response.as_slice(), KipResult)?;

        match result {
            KipResult::ok(json) => Ok(json),
            KipResult::err(msg) => Err(anyhow::anyhow!("KIP Error: {}", msg)),
        }
    }

    /// Execute a mutating KIP command (UPSERT, DELETE)
    pub async fn mutate(&self, command: &str) -> Result<String> {
        let args = Encode!(&command)?;
        let response = self
            .agent
            .update(&self.canister_id, "execute_kip_mutation")
            .with_arg(args)
            .call_and_wait()
            .await?;

        let result: KipResult = Decode!(response.as_slice(), KipResult)?;

        match result {
            KipResult::ok(json) => Ok(json),
            KipResult::err(msg) => Err(anyhow::anyhow!("KIP Mutation Error: {}", msg)),
        }
    }
}
