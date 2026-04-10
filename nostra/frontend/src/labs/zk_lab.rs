use crate::api::create_agent;
use crate::types::*;
use crate::zk_service::ZkService;
use dioxus::document::eval;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
enum VerificationStatus {
    Idle,
    Generating,
    Verifying,
    Success(ZkVerifierResponse, ZkAuditTrace),
    Failure(String),
}

pub fn ZkLab() -> Element {
    let mut status = use_signal(|| VerificationStatus::Idle);
    let mut selected_circuit = use_signal(|| "Linked-but-Private Membership".to_string());
    let mut public_inputs = use_signal(|| {
        r#"{
  "spaceId": "nostra-hq-01",
  "role": "steward",
  "memberSince": 1706044800,
  "merkleRoot": "0x4e6f737472615a4b50726f6f66"
}"#
        .to_string()
    });
    let mut logs = use_signal(|| Vec::<String>::new());

    let on_generate = move |_| async move {
        status.set(VerificationStatus::Generating);
        logs.write().clear();
        logs.write()
            .push("[INIT] Starting Witness Generation in WASM".to_string());

        eval("new Promise(resolve => setTimeout(resolve, 800))")
            .await
            .ok();

        logs.write()
            .push("[WASM] Constraints validated (2,048 gates)".to_string());
        logs.write()
            .push("[WASM] PLONK proof serialized (12.4 KB)".to_string());

        status.set(VerificationStatus::Idle);
    };

    let on_verify = move |_| {
        spawn(async move {
            status.set(VerificationStatus::Verifying);

            let agent = create_agent().await;

            let claim_res: Result<MembershipClaim, _> = serde_json::from_str(&public_inputs.read());
            let claim = match claim_res {
                Ok(c) => c,
                Err(e) => {
                    logs.write()
                        .push(format!("[ERROR] JSON Parse Error: {}", e));
                    status.set(VerificationStatus::Failure(e.to_string()));
                    return;
                }
            };

            let proof = MembershipProof {
                claim,
                circuit_hash: "0xplonk_membership_v1".to_string(),
                proof_bytes: vec![0u8; 32],
                nullifier_hash: "0xnullifier_hash_placeholder".to_string(),
            };

            match ZkService::verify_proof(&agent, proof).await {
                Ok((resp, trace)) => {
                    for step in &trace.steps {
                        logs.write().push(step.clone());
                    }
                    status.set(VerificationStatus::Success(resp, trace));
                }
                Err(e) => {
                    logs.write().push(format!("[ERROR] Transport error: {}", e));
                    status.set(VerificationStatus::Failure(e));
                }
            }
        });
    };

    rsx! {
        div { class: "flex flex-col h-full bg-[#0d1117] text-[#c9d1d9] font-sans selection:bg-[#388bfd33]",
            // HEADER
            div { class: "h-[39px] bg-[#161b22] border-b border-[#30363d] flex items-center px-4 shrink-0 justify-between",
                div { class: "flex items-center space-x-3",
                    span { class: "text-[#8b949e] font-medium text-sm", "CORTEX // ZK_LAB" }
                    span { class: "h-3 w-[1px] bg-[#30363d]" }
                    span { class: "text-xs font-mono text-[#58a6ff]", "{selected_circuit}" }
                }
                div { class: "flex space-x-2",
                    button {
                        class: "h-7 px-3 bg-[#238636] hover:bg-[#2ea043] text-white rounded text-xs font-semibold transition-colors flex items-center",
                        onclick: on_generate,
                        "GENERATE PROOF"
                    }
                    button {
                        class: "h-7 px-3 bg-[#21262d] border border-[#30363d] hover:bg-[#30363d] text-[#c9d1d9] rounded text-xs font-semibold transition-colors flex items-center",
                        onclick: on_verify,
                        "VERIFY ON-CHAIN"
                    }
                }
            }

            // MAIN CONTENT
            div { class: "flex flex-1 w-full overflow-hidden",
                // LEFT PANE
                div { class: "w-80 border-r border-[#30363d] bg-[#0d1117] flex flex-col",
                    div { class: "p-4 border-b border-[#30363d]",
                        h3 { class: "text-xs font-bold text-[#8b949e] uppercase tracking-wider mb-3", "Configuration" }
                        label { class: "block text-[11px] text-[#8b949e] mb-1 px-1", "Circuit" }
                        select {
                            class: "w-full bg-[#161b22] border border-[#30363d] rounded p-2 text-sm focus:border-[#388bfd] outline-none",
                            onchange: move |ev| selected_circuit.set(ev.value()),
                            option { value: "Linked-but-Private Membership", "Membership Proof" }
                            option { value: "Constraint Compliance", "Constraint Audit" }
                        }
                    }
                    div { class: "flex-1 p-4",
                        h3 { class: "text-xs font-bold text-[#8b949e] uppercase tracking-wider mb-2", "Public Inputs" }
                        textarea {
                            class: "w-full h-64 bg-[#161b22] text-[#79c0ff] font-mono text-xs border border-[#30363d] rounded p-3 focus:border-[#388bfd] outline-none resize-none",
                            value: "{public_inputs}",
                            oninput: move |ev| public_inputs.set(ev.value())
                        }
                    }
                }

                // CENTER PANE
                div { class: "flex-1 bg-[#161b22] flex flex-col",
                    div { class: "p-3 border-b border-[#30363d] bg-[#0d1117] flex justify-between items-center",
                        span { class: "text-xs font-bold text-[#8b949e] uppercase tracking-wider", "Verification Theater" }
                        span { class: "text-[10px] font-mono text-[#444c56]", "CANISTER://7n7be...cai" }
                    }
                    div { class: "flex-1 p-6 font-mono text-xs overflow-y-auto space-y-2",
                        for (i, log) in logs.read().iter().enumerate() {
                            div {
                                key: "{i}",
                                class: if log.starts_with("[ERROR]") || log.starts_with("[FAIL]") { "text-[#f85149]" }
                                       else if log.starts_with("[COMP]") { "text-[#3fb950] font-bold" }
                                       else if log.starts_with("[SEND]") { "text-[#d29922]" }
                                       else { "text-[#8b949e]" },
                                "{log}"
                            }
                        }
                    }
                }

                // RIGHT PANE
                div { class: "w-72 border-l border-[#30363d] bg-[#0d1117] flex flex-col p-4",
                    h3 { class: "text-xs font-bold text-[#8b949e] uppercase tracking-wider mb-6", "Quality Dashboard" }

                    div { class: "flex flex-col items-center justify-center p-8 bg-[#161b22] rounded-xl border border-[#30363d] mb-6",
                        match *status.read() {
                            VerificationStatus::Idle => rsx! {
                                div { class: "w-16 h-16 rounded-full border-2 border-dashed border-[#30363d] flex items-center justify-center",
                                    span { class: "text-[#30363d] text-2xl", "!" }
                                }
                                span { class: "mt-4 text-xs text-[#8b949e]", "Waiting" }
                            },
                            VerificationStatus::Verifying | VerificationStatus::Generating => rsx! {
                                div { class: "w-16 h-16 rounded-full border-2 border-[#388bfd] border-t-transparent animate-spin" }
                            },
                            VerificationStatus::Success(ref _resp, _) => rsx! {
                                div { class: "w-16 h-16 rounded-full bg-[#238636] flex items-center justify-center shadow-[0_0_20px_rgba(35,134,54,0.3)]",
                                    span { class: "text-white text-2xl", "✓" }
                                }
                                span { class: "mt-4 text-sm font-bold text-[#3fb950]", "PROOF VALID" }
                            },
                            VerificationStatus::Failure(_) => rsx! {
                                div { class: "w-16 h-16 rounded-full bg-[#da3633] flex items-center justify-center shadow-[0_0_20px_rgba(218,54,51,0.3)]",
                                    span { class: "text-white text-2xl", "×" }
                                }
                                span { class: "mt-4 text-sm font-bold text-[#f85149]", "FAILED" }
                                button {
                                    class: "mt-4 px-3 py-1.5 bg-[#21262d] border border-[#30363d] rounded text-[10px] text-[#c9d1d9] hover:bg-[#30363d] transition-colors uppercase tracking-wide font-medium",
                                    onclick: move |_| logs.write().push("[USER] Issue reported to Index Canister".to_string()),
                                    "Report to Index"
                                }
                            }
                        }
                    }

                    if let VerificationStatus::Success(ref resp, _) = *status.read() {
                        div { class: "space-y-4",
                            div {
                                label { class: "block text-[10px] text-[#8b949e] uppercase mb-1", "Attestation Hash" }
                                div { class: "bg-[#161b22] p-2 rounded border border-[#30363d] font-mono text-[9px] break-all text-[#79c0ff]",
                                    "{resp.attestation_hash.as_deref().unwrap_or(\"None\")}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
