use crate::primitives::{Action, AsyncProviderStrategy, AsyncRetryPolicy, Step, Transition};

/// Factory to build the Eudaemon Alpha "Agentic Loop" FSM
/// This directly maps the 8-phase cycle from Python `main_loop.py` to `nostra-workflow-core`.
pub fn build_agentic_loop_fsm() -> Vec<Step> {
    let mut steps = Vec::new();

    // Step 1: Observe (Fetch context bundle to defeat amnesia)
    let observe = Step::new("observe", "Observe space context and fetch bundle")
        .with_action(Action::AsyncExternalOp {
            target: "FetchContextBundle".to_string(),
            input: "{}".to_string(),
            timeout_secs: 15,
            retry_policy: AsyncRetryPolicy::default(),
            provider_strategy: AsyncProviderStrategy::Single,
        })
        .with_transition(Transition::to("resource_check"));
    steps.push(observe);

    // Step 1.5: Resource Governance Check
    let resource_check = Step::new("resource_check", "Check execution strategy and resource budgets")
        .with_action(Action::SystemOp {
            op_type: "CheckResourceBudget".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("analyze")); // Ideally needs a conditional transition if failed
    steps.push(resource_check);

    // Step 2: Analyze & Formulate
    let analyze = Step::new("analyze", "Run pattern detection execution via LLM/Local")
        .with_action(Action::AsyncExternalOp {
            target: "RunPatternDetection".to_string(),
            input: "{}".to_string(),
            timeout_secs: 60,
            retry_policy: AsyncRetryPolicy::default(),
            provider_strategy: AsyncProviderStrategy::Single,
        })
        .with_transition(Transition::to("grade"));
    steps.push(analyze);

    // Step 2.5: Grade
    let grade = Step::new("grade", "Grade the analysis results (Benchmark)")
        .with_action(Action::SystemOp {
            op_type: "GradeAnalysis".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("validate"));
    steps.push(grade);

    // Step 3: Validate (Dry-run sandbox)
    let validate = Step::new("validate", "Validate Graph Changes deterministically")
        .with_action(Action::SystemOp {
            op_type: "GraphValidator".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("publish")); // Needs conditional branch
    steps.push(validate);

    // Step 4: Chronicle publishing & Queue submission
    let publish = Step::new("publish", "Emit approved ProposalBlocks to Heap")
        .with_action(Action::AsyncExternalOp {
            target: "EmitProposal".to_string(),
            input: "{}".to_string(),
            timeout_secs: 15,
            retry_policy: AsyncRetryPolicy::default(),
            provider_strategy: AsyncProviderStrategy::Single,
        })
        .with_transition(Transition::to("emit_lineage"));
    steps.push(publish);

    // Step 5: Emit Lineage Record & Persist Git
    let emit_lineage = Step::new("emit_lineage", "Emit ExecutionRecordBlock and trajectory")
        .with_action(Action::AsyncExternalOp {
            target: "EmitExecutionRecord".to_string(),
            input: "{}".to_string(),
            timeout_secs: 15,
            retry_policy: AsyncRetryPolicy::default(),
            provider_strategy: AsyncProviderStrategy::Single,
        })
        .with_transition(Transition::to("self_optimize"));
    steps.push(emit_lineage);

    // Step 6: Self-Optimization Loop
    let self_optimize = Step::new("self_optimize", "Evaluate self-optimization proposals")
        .with_action(Action::SystemOp {
            op_type: "EvaluateOptimization".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("sleep"));
    steps.push(self_optimize);

    // Step 8: Sleep
    let sleep = Step::new("sleep", "Durable sleep interval")
        .with_action(Action::SystemOp {
            op_type: "DurableSleep".to_string(),
            payload: "{}".to_string(),
        })
        .with_transition(Transition::to("observe")); // Loop back
    steps.push(sleep);

    steps
}
