use crate::types::*;
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;

pub struct ZkService;

impl ZkService {
    /// Canister ID of the zCloak Cloaking Layer on IC Mainnet
    pub const ZCLOAK_BACKEND_ID: &'static str = "7n7be-naaaa-aaaag-qc4xa-cai";

    /// Verifies a ZK proof against the zCloak Cloaking Layer.
    /// This method performs the network call and returns a response plus a generated audit trace.
    pub async fn verify_proof(
        agent: &Agent,
        proof: MembershipProof,
    ) -> Result<(ZkVerifierResponse, ZkAuditTrace), String> {
        let canister_id = Principal::from_text(Self::ZCLOAK_BACKEND_ID).unwrap();

        let mut steps = Vec::new();
        steps.push("[INIT] Starting ZK Verification lifecycle".to_string());
        steps.push(format!("[PRE] Circuit Hash: {}", proof.circuit_hash));

        // In a real implementation, we would call the zCloak canister here.
        // For Phase 3, we implement the communication logic, but since we are sending mock proofs,
        // we handle the expected "Failure" gracefully while confirming the transport works.

        let verifier_signature = "nostra_prototype_v1".to_string();
        let program_hash = proof.circuit_hash.clone();
        // Public inputs are usually serialized text for zCloak zk_verify
        let public_inputs = serde_json::to_string(&proof.claim).unwrap_or_default();

        steps.push("[SEND] Calling zCloak 'zk_verify' endpoint".to_string());

        let response_blob = agent
            .query(&canister_id, "zk_verify")
            .with_arg(Encode!(&verifier_signature, &program_hash, &public_inputs).unwrap())
            .call()
            .await
            .map_err(|e| e.to_string())?;

        steps.push("[RECV] Response received from Cloaking Layer".to_string());

        // According to our research, zk_verify returns (text, text, vec text)
        // Which translates to (status, attestation, output_vec)
        let (status, attestation, _output_vec) =
            Decode!(&response_blob, String, String, Vec<String>)
                .map_err(|e| format!("Decode error: {}", e))?;

        let is_valid = status == "success";

        if is_valid {
            steps.push("[COMP] Proof verified successfully!".to_string());
        } else {
            steps.push(format!("[FAIL] Verification failed: {}", status));
        }

        let verifier_response = ZkVerifierResponse {
            is_valid,
            error: if is_valid { None } else { Some(status) },
            attestation_hash: Some(attestation.clone()),
        };

        let audit_trace = ZkAuditTrace {
            timestamp: js_sys::Date::now() as u64, // Mock timestamp for web
            steps,
            attestation_hash: Some(attestation),
            confidence_score: if is_valid { 1.0 } else { 0.0 },
        };

        Ok((verifier_response, audit_trace))
    }

}
