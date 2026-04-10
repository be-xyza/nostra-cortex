#!/bin/bash
sed -i '' 's/pub fn from_env()/pub async fn from_env()/' src/services/governance_client.rs
sed -i '' 's/GovernanceCanisterClient::from_env().map_err/GovernanceCanisterClient::from_env().await.map_err/' src/services/governance_client.rs

sed -i '' 's/pub fn from_env()/pub async fn from_env()/' src/services/brand_policy.rs
sed -i '' 's/BrandPolicyCanisterClient::from_env().map_err/BrandPolicyCanisterClient::from_env().await.map_err/' src/services/brand_policy.rs

sed -i '' 's/SnapshotCanisterClient::from_env()?/SnapshotCanisterClient::from_env().await?/' src/services/snapshot_service.rs

sed -i '' 's/GovernanceClient::from_env()/GovernanceClient::from_env().await/g' src/gateway/server.rs
sed -i '' 's/BrandPolicyClient::from_env()/BrandPolicyClient::from_env().await/g' src/gateway/server.rs

