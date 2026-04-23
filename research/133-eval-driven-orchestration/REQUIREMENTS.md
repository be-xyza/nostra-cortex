# Requirements

1. **Deterministic Benchmarking**: The Eval Grader must rely on deterministic, programmatic assertions or strict LLM-as-a-judge rubrics.
2. **First-class A2UI**: Feedback projection must not rely on secondary web servers; it must use the A2UI protocol.
3. **Lineage Preservation**: The outputs of all subagents in a `ParallelAgentTask` must be preserved in the `AgentExecutionRecord` history, even the discarded ones.
