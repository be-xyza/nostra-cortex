use nostra_workflow_core::alignment::{
    AlignmentSignal, Operator, OrchestrationAction, OrchestrationPolicy, PolicyTrigger, SignalType,
};

struct MockHistory {
    forks: u32,
    merges: u32,
}

impl MockHistory {
    fn extract_signal(&self, agent_id: String) -> AlignmentSignal {
        let total = self.forks + self.merges;
        let value = if total == 0 {
            0.0
        } else {
            self.forks as f64 / total as f64
        };

        AlignmentSignal {
            agent_id,
            signal_type: SignalType::ForkPressure,
            value,
            time_window_seconds: 3600,
            context_space_id: "space-1".to_string(),
            context_workflow_id: None,
        }
    }
}

fn main() {
    println!("=== Adaptive Alignment Loop: Game Day Simulation ===");

    // 1. Setup Policy: "The Brake"
    let brake_policy = OrchestrationPolicy {
        id: "policy-brake".to_string(),
        target_role: "*".to_string(),
        triggers: vec![PolicyTrigger {
            signal: SignalType::ForkPressure,
            operator: Operator::GreaterThan,
            threshold: 0.6,
            duration_seconds: 0, // Instant
        }],
        action: OrchestrationAction::PauseExecution,
        recovery_condition: None,
    };

    println!(
        "Loaded Policy: {} (Threshold: > 0.6 Fork Pressure)",
        brake_policy.id
    );

    // 2. Scenario A: Good Agent
    let good_agent_id = "agent-good-001".to_string();
    let good_history = MockHistory {
        forks: 1,
        merges: 10,
    }; // 10% Pressure
    let signal_a = good_history.extract_signal(good_agent_id);
    println!(
        "\nScenario A: Good Agent (Forks: {}, Merges: {}) -> Signal: {:.2}",
        good_history.forks, good_history.merges, signal_a.value
    );

    match brake_policy.evaluate(&signal_a) {
        Some(action) => println!(">>> ACTION TRIGGERED: {:?}", action),
        None => println!(">>> NO ACTION (Agent is Safe)"),
    }

    // 3. Scenario B: Rogue Agent
    let rogue_agent_id = "agent-rogue-anon".to_string();
    let rogue_history = MockHistory {
        forks: 8,
        merges: 2,
    }; // 80% Pressure
    let signal_b = rogue_history.extract_signal(rogue_agent_id);
    println!(
        "\nScenario B: Rogue Agent (Forks: {}, Merges: {}) -> Signal: {:.2}",
        rogue_history.forks, rogue_history.merges, signal_b.value
    );

    match brake_policy.evaluate(&signal_b) {
        Some(action) => {
            println!(">>> ACTION TRIGGERED: {:?}", action);
            if action == OrchestrationAction::PauseExecution {
                println!("✅ SUCCESS: System correctly paused the Rogue Agent.");
            } else {
                println!("❌ FAILURE: Wrong action triggered.");
            }
        }
        None => println!("❌ FAILURE: Rogue Agent was not caught!"),
    }
}
