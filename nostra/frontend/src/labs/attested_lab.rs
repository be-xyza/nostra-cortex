use crate::types::UserProfile;
use dioxus::prelude::*;
// use nostra_core::ProofEnvelope; -- Types are in crate::types for now

#[derive(PartialEq, Clone)]
enum AccessState {
    Locked,
    Verifying,
    Unlocked(String), // Payload summary
}

#[component]
pub fn AttestedLab(user_profile: UserProfile) -> Element {
    let _ = user_profile;
    let mut access_state = use_signal(|| AccessState::Locked);
    let mut proof_display = use_signal(|| String::new());

    // Mock Signing Flow (Phase 1)
    let sign_attestation = move |_| {
        access_state.set(AccessState::Verifying);

        spawn(async move {
            // Simulate async signing & verification latency
            gloo_timers::future::TimeoutFuture::new(1500).await;

            // In Phase 1, we just mock the success of a Level 0 Signature
            // Logic would go here:
            // 1. Agent.sign_message(claim)
            // 2. Construct ProofEnvelope
            // 3. Verify(ProofEnvelope) -> Ok

            let mock_payload = "Claim: Member of DAO-99 (Level 0 Signature)";
            proof_display.set(format!(
                r#"{{
  "payload": "{}",
  "proof_level": 0,
  "proof": {{
    "mechanism": "ed25519",
    "value": "sig_mock_12345..."
  }}
}}"#,
                mock_payload
            ));

            access_state.set(AccessState::Unlocked(mock_payload.to_string()));
        });
    };

    rsx! {
        div {
            class: "flex flex-col h-full p-6 space-y-6 overflow-y-auto",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight", "Attested Lab (Phase 1)" }
                    p { class: "text-muted-foreground", "Tests the 'Async Gating' flow with cryptographic signatures." }
                }
                div {
                    class: format!(
                        "px-3 py-1 rounded-full text-xs font-medium border {}",
                        match *access_state.read() {
                            AccessState::Locked => "bg-red-500/10 text-red-600 border-red-200",
                            AccessState::Verifying => "bg-yellow-500/10 text-yellow-600 border-yellow-200 animate-pulse",
                            AccessState::Unlocked(_) => "bg-green-500/10 text-green-600 border-green-200",
                        }
                    ),
                    match *access_state.read() {
                        AccessState::Locked => "LOCKED (No Proof)",
                        AccessState::Verifying => "VERIFYING...",
                        AccessState::Unlocked(_) => "ACCESS GRANTED",
                    }
                }
            }

            div { class: "grid gap-6 md:grid-cols-2",
                 // Left Panel: Gate
                 div { class: "space-y-4",
                    div { class: "card border rounded-lg shadow-sm bg-card text-card-foreground p-6 h-full flex flex-col justify-center items-center text-center space-y-4",
                        match *access_state.read() {
                            AccessState::Locked => rsx! {
                                div { class: "w-16 h-16 bg-muted rounded-full flex items-center justify-center mb-2",
                                    svg { class: "w-8 h-8 text-muted-foreground", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" }
                                    }
                                }
                                h3 { class: "font-semibold text-lg", "Restricted Area" }
                                p { class: "text-muted-foreground max-w-xs",
                                    "This lab requires a Level 0 Proof (Signature) to access. Please sign the attestation."
                                }
                                button {
                                    class: "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2 mt-4",
                                    onclick: sign_attestation,
                                    "Sign Attestation"
                                }
                            },
                            AccessState::Verifying => rsx! {
                                div { class: "w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mb-2 animate-spin",
                                    svg { class: "w-8 h-8 text-primary", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" }
                                    }
                                }
                                h3 { class: "font-semibold text-lg", "Verifying Proof..." }
                                p { class: "text-muted-foreground", "Checking cryptographic signature via Policy Engine..." }
                            },
                            AccessState::Unlocked(ref payload) => rsx! {
                                div { class: "w-16 h-16 bg-green-500/10 rounded-full flex items-center justify-center mb-2",
                                    svg { class: "w-8 h-8 text-green-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M5 13l4 4L19 7" }
                                    }
                                }
                                h3 { class: "font-semibold text-lg", "Welcome to the Lab" }
                                p { class: "text-muted-foreground", "{payload}" }
                                button {
                                    class: "text-sm text-primary hover:underline mt-4",
                                    onclick: move |_| access_state.set(AccessState::Locked),
                                    "Lock Lab"
                                }
                            }
                        }
                    }
                 }

                 // Right Panel: Debug
                 div { class: "space-y-4",
                    div { class: "card border rounded-lg shadow-sm bg-card text-card-foreground p-6",
                        h3 { class: "font-semibold mb-4", "Proof Envelope Inspection" }
                        if proof_display.read().is_empty() {
                            div { class: "p-4 bg-muted/50 rounded text-muted-foreground italic text-center",
                                "Waiting for proof generation..."
                            }
                        } else {
                            pre { class: "bg-muted p-4 rounded text-xs font-mono overflow-x-auto whitespace-pre-wrap",
                                "{proof_display}"
                            }
                        }
                    }
                 }
            }
        }
    }
}
