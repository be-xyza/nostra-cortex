use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn workflow_digest_hex(value: &Value) -> String {
    let mut hasher = Sha256::new();
    if let Ok(bytes) = serde_json::to_vec(value) {
        hasher.update(bytes);
    }
    hex::encode(hasher.finalize())
}
