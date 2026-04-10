use anyhow::Result;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_agent::{Agent, export::Principal};
use std::sync::Arc;

#[derive(CandidType, Deserialize, Debug)]
pub enum Error {
    MemoryError,
    UniqueViolation,
    DimensionMismatch,
    NotFound,
    Unauthorized,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum VectorResult {
    Ok,
    Err(Error),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum SearchResult {
    Ok(Vec<String>),
    Err(Error),
}

#[derive(Clone)]
pub struct VectorClient {
    agent: Arc<Agent>,
    canister_id: Principal,
}

impl VectorClient {
    pub fn new(agent: Arc<Agent>, canister_id: Principal) -> Self {
        Self { agent, canister_id }
    }

    pub async fn create_collection(&self, name: &str, dim: u64) -> Result<()> {
        let args = Encode!(&name, &dim)?;
        let response = self
            .agent
            .update(&self.canister_id, "create_collection")
            .with_arg(args)
            .call_and_wait()
            .await?;

        let result: VectorResult = Decode!(response.as_slice(), VectorResult)?;
        match result {
            VectorResult::Ok => Ok(()),
            VectorResult::Err(e) => Err(anyhow::anyhow!("Vector DB Error: {:?}", e)),
        }
    }

    pub async fn insert(
        &self,
        collection: &str,
        vectors: Vec<Vec<f32>>,
        ids: Vec<String>,
        label: &str,
    ) -> Result<()> {
        let args = Encode!(&collection, &vectors, &ids, &label)?;
        let response = self
            .agent
            .update(&self.canister_id, "insert")
            .with_arg(args)
            .call_and_wait()
            .await?;

        let result: VectorResult = Decode!(response.as_slice(), VectorResult)?;
        match result {
            VectorResult::Ok => Ok(()),
            VectorResult::Err(e) => Err(anyhow::anyhow!("Vector DB Error: {:?}", e)),
        }
    }

    pub async fn build_index(&self, collection: &str) -> Result<()> {
        let args = Encode!(&collection)?;
        let response = self
            .agent
            .update(&self.canister_id, "build_index")
            .with_arg(args)
            .call_and_wait()
            .await?;

        let result: VectorResult = Decode!(response.as_slice(), VectorResult)?;
        match result {
            VectorResult::Ok => Ok(()),
            VectorResult::Err(e) => Err(anyhow::anyhow!("Vector DB Error: {:?}", e)),
        }
    }

    pub async fn search(&self, collection: &str, vector: Vec<f32>, k: i32) -> Result<Vec<String>> {
        let args = Encode!(&collection, &vector, &k)?;
        let response = self
            .agent
            .query(&self.canister_id, "query")
            .with_arg(args)
            .call()
            .await?;

        let result: SearchResult = Decode!(response.as_slice(), SearchResult)?;
        match result {
            SearchResult::Ok(ids) => Ok(ids),
            SearchResult::Err(e) => Err(anyhow::anyhow!("Vector DB Error: {:?}", e)),
        }
    }
}
