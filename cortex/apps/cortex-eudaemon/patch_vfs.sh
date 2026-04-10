#!/bin/bash
sed -i '' 's/pub fn from_env()/pub async fn from_env()/' src/services/workflow_engine_client.rs
sed -i '' 's/WorkflowCanisterClient::from_env().map_err/WorkflowCanisterClient::from_env().await.map_err/' src/services/workflow_engine_client.rs

sed -i '' 's/WorkflowEngineClient::from_env() {/WorkflowEngineClient::from_env().await {/g' src/gateway/server.rs
sed -i '' 's/WorkflowEngineClient::from_env() else {/WorkflowEngineClient::from_env().await else {/g' src/gateway/server.rs
sed -i '' 's/WorkflowEngineClient::from_env().ok()/WorkflowEngineClient::from_env().await.ok()/g' src/gateway/server.rs

sed -i '' 's/WorkflowEngineClient::from_env() {/WorkflowEngineClient::from_env().await {/g' src/services/agent_evaluation_loop.rs

