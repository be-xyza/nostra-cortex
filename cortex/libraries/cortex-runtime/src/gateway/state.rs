use crate::gateway::types::GatewayResponseEnvelope;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct GatewayRuntimeState {
    idempotency_cache: Mutex<BTreeMap<String, GatewayResponseEnvelope>>,
}

impl GatewayRuntimeState {
    pub fn replay(&self, key: &str) -> Option<GatewayResponseEnvelope> {
        self.idempotency_cache.lock().unwrap().get(key).cloned()
    }

    pub fn store(&self, key: String, response: GatewayResponseEnvelope) {
        self.idempotency_cache.lock().unwrap().insert(key, response);
    }
}
