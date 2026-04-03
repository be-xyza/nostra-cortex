use crate::ic::resolve_canister_id_any;
use async_trait::async_trait;
use candid::{CandidType, Decode, Encode, Principal};
use cortex_domain::streaming::types::{
    ArtifactRealtimeConnectAck, ArtifactRealtimeDisconnectAck, ArtifactRealtimeEnvelope,
    ArtifactRealtimePollResult,
};
use cortex_runtime::RuntimeError;
use cortex_runtime::ports::StreamingTransportAdapter;
use ic_agent::Agent;
use ic_agent::identity::AnonymousIdentity;

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsClientKey {
    client_principal: Principal,
    client_nonce: u64,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsOpenArgs {
    client_nonce: u64,
    gateway_principal: Principal,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsCloseArgs {
    client_key: WsClientKey,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsMessageArgs {
    client_key: WsClientKey,
    sequence_num: u64,
    timestamp: u64,
    is_service_message: bool,
    content: Vec<u8>,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsGetMessagesArgs {
    nonce: u64,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsCertifiedMessage {
    client_key: WsClientKey,
    sequence_num: u64,
    timestamp: u64,
    is_service_message: bool,
    content: Vec<u8>,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct WsGetMessagesResult {
    messages: Vec<WsCertifiedMessage>,
    cert: Vec<u8>,
    tree: Vec<u8>,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct ChatMessage {
    msg_type: String,
    content: String,
    conversation_id: Option<String>,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
enum WsResult {
    Ok(()),
    Err(String),
}

#[derive(Clone, Debug)]
pub struct IcStreamingTransportAdapter {
    host: String,
    canister_id: Principal,
    gateway_principal: Principal,
}

impl IcStreamingTransportAdapter {
    pub async fn from_env() -> Result<Self, RuntimeError> {
        let host = std::env::var("NOSTRA_IC_HOST")
            .or_else(|_| std::env::var("IC_HOST"))
            .unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());
        let canister_id_text = resolve_canister_id_any(
            &["CANISTER_ID_NOSTRA_STREAMING", "CANISTER_ID_STREAMING"],
            "nostra_streaming",
        )
        .await
        .map_err(RuntimeError::Network)?;
        let canister_id = Principal::from_text(canister_id_text)
            .map_err(|err| RuntimeError::Network(format!("invalid streaming principal: {err}")))?;
        let gateway_principal = std::env::var("NOSTRA_STREAMING_GATEWAY_PRINCIPAL")
            .ok()
            .and_then(|value| Principal::from_text(value.trim()).ok())
            .unwrap_or_else(Principal::anonymous);
        Ok(Self {
            host,
            canister_id,
            gateway_principal,
        })
    }

    async fn agent(&self) -> Result<Agent, RuntimeError> {
        let agent = Agent::builder()
            .with_url(self.host.clone())
            .with_identity(AnonymousIdentity)
            .build()
            .map_err(|err| RuntimeError::Network(format!("failed to build ic-agent: {err}")))?;

        if self.host.contains("127.0.0.1") || self.host.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|err| RuntimeError::Network(format!("failed to fetch root key: {err}")))?;
        }
        Ok(agent)
    }
}

#[async_trait]
impl StreamingTransportAdapter for IcStreamingTransportAdapter {
    async fn connect(
        &self,
        actor_id: &str,
        artifact_id: &str,
        client_nonce: u64,
    ) -> Result<ArtifactRealtimeConnectAck, RuntimeError> {
        let agent = self.agent().await?;
        let args = WsOpenArgs {
            client_nonce,
            gateway_principal: self.gateway_principal,
        };
        let payload = Encode!(&args).map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .update(&self.canister_id, "ws_open")
            .with_arg(payload)
            .call_and_wait()
            .await
            .map_err(|err| RuntimeError::Network(format!("ws_open failed: {err}")))?;
        match Decode!(&bytes, WsResult)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?
        {
            WsResult::Ok(()) => Ok(ArtifactRealtimeConnectAck {
                connected: true,
                actor_id: actor_id.to_string(),
                artifact_id: artifact_id.to_string(),
                channel: format!("cortex:artifact:{artifact_id}"),
                mode: "canister_primary".to_string(),
                connected_at: String::new(),
            }),
            WsResult::Err(err) => Err(RuntimeError::Network(err)),
        }
    }

    async fn disconnect(
        &self,
        actor_id: &str,
        artifact_id: &str,
        client_nonce: Option<u64>,
    ) -> Result<ArtifactRealtimeDisconnectAck, RuntimeError> {
        let nonce = client_nonce.unwrap_or(0);
        let agent = self.agent().await?;
        let args = WsCloseArgs {
            client_key: WsClientKey {
                client_principal: Principal::anonymous(),
                client_nonce: nonce,
            },
        };
        let payload = Encode!(&args).map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .update(&self.canister_id, "ws_close")
            .with_arg(payload)
            .call_and_wait()
            .await
            .map_err(|err| RuntimeError::Network(format!("ws_close failed: {err}")))?;
        match Decode!(&bytes, WsResult)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?
        {
            WsResult::Ok(()) => Ok(ArtifactRealtimeDisconnectAck {
                disconnected: true,
                actor_id: actor_id.to_string(),
                artifact_id: artifact_id.to_string(),
                channel: format!("cortex:artifact:{artifact_id}"),
                disconnected_at: String::new(),
            }),
            WsResult::Err(err) => Err(RuntimeError::Network(err)),
        }
    }

    async fn publish(
        &self,
        envelope: &ArtifactRealtimeEnvelope,
        client_nonce: u64,
        timestamp_ms: u64,
    ) -> Result<(), RuntimeError> {
        let agent = self.agent().await?;
        let content = serde_json::to_vec(envelope)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let args = WsMessageArgs {
            client_key: WsClientKey {
                client_principal: Principal::anonymous(),
                client_nonce,
            },
            sequence_num: envelope.sequence,
            timestamp: timestamp_ms,
            is_service_message: false,
            content,
        };
        let chat = ChatMessage {
            msg_type: envelope.event_type.clone(),
            content: envelope.channel.clone(),
            conversation_id: Some(envelope.artifact_id.clone()),
        };
        let payload = Encode!(&args, &Some(chat))
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .update(&self.canister_id, "ws_message")
            .with_arg(payload)
            .call_and_wait()
            .await
            .map_err(|err| RuntimeError::Network(format!("ws_message failed: {err}")))?;
        match Decode!(&bytes, WsResult)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?
        {
            WsResult::Ok(()) => Ok(()),
            WsResult::Err(err) => Err(RuntimeError::Network(err)),
        }
    }

    async fn poll(&self, nonce: u64) -> Result<ArtifactRealtimePollResult, RuntimeError> {
        let agent = self.agent().await?;
        let args = WsGetMessagesArgs { nonce };
        let payload = Encode!(&args).map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "ws_get_messages")
            .with_arg(payload)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("ws_get_messages failed: {err}")))?;
        let response = Decode!(&bytes, WsGetMessagesResult)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;

        let mut events = Vec::new();
        let mut next_nonce = nonce;
        for message in response.messages {
            next_nonce = next_nonce.max(message.sequence_num);
            if let Ok(envelope) =
                serde_json::from_slice::<ArtifactRealtimeEnvelope>(&message.content)
            {
                events.push(envelope);
            }
        }
        Ok(ArtifactRealtimePollResult { next_nonce, events })
    }
}
