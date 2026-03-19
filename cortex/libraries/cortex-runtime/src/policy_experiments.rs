use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::collections::VecDeque;
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedFunctionContract {
    pub function_name: String,
    pub input_schema_ref: String,
    pub output_schema_ref: String,
}

pub trait TypedFunction<I, O> {
    fn contract(&self) -> &TypedFunctionContract;
    fn invoke(&self, input: &I) -> Result<O, ProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryStrategy {
    ConstantDelay {
        delay_ms: u64,
    },
    ExponentialBackoff {
        delay_ms: u64,
        multiplier: u32,
        max_delay_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub strategy: RetryStrategy,
}

impl RetryPolicy {
    pub fn backoff_ms(&self, retry_index: u32) -> u64 {
        match self.strategy {
            RetryStrategy::ConstantDelay { delay_ms } => delay_ms,
            RetryStrategy::ExponentialBackoff {
                delay_ms,
                multiplier,
                max_delay_ms,
            } => {
                let pow = multiplier.saturating_pow(retry_index);
                let next = delay_ms.saturating_mul(u64::from(pow));
                next.min(max_delay_ms)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    Transient,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderError {
    pub provider: String,
    pub kind: ErrorKind,
    pub message: String,
}

pub trait Provider<I, O>: Send + Sync {
    fn name(&self) -> &str;
    fn call(&self, input: &I) -> Result<O, ProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAttempt {
    pub provider: String,
    pub attempt: u32,
    pub outcome: String,
    pub next_backoff_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationTrace<O> {
    pub attempts: Vec<ProviderAttempt>,
    pub result: Result<O, ProviderError>,
}

#[derive(Clone)]
pub struct FallbackExecutor<I, O> {
    providers: Vec<Arc<dyn Provider<I, O>>>,
    retry_policy: RetryPolicy,
}

impl<I, O> FallbackExecutor<I, O>
where
    I: Send + Sync,
    O: Clone + Send + Sync,
{
    pub fn new(providers: Vec<Arc<dyn Provider<I, O>>>, retry_policy: RetryPolicy) -> Self {
        Self {
            providers,
            retry_policy,
        }
    }

    pub fn run(&self, input: &I) -> InvocationTrace<O> {
        let mut attempts = Vec::new();
        let mut last_error = ProviderError {
            provider: "none".to_string(),
            kind: ErrorKind::Fatal,
            message: "no providers configured".to_string(),
        };

        for provider in &self.providers {
            for retry_index in 0..=self.retry_policy.max_retries {
                match provider.call(input) {
                    Ok(output) => {
                        attempts.push(ProviderAttempt {
                            provider: provider.name().to_string(),
                            attempt: retry_index + 1,
                            outcome: "success".to_string(),
                            next_backoff_ms: None,
                        });
                        return InvocationTrace {
                            attempts,
                            result: Ok(output),
                        };
                    }
                    Err(err) => {
                        let should_retry = matches!(err.kind, ErrorKind::Transient)
                            && retry_index < self.retry_policy.max_retries;

                        attempts.push(ProviderAttempt {
                            provider: provider.name().to_string(),
                            attempt: retry_index + 1,
                            outcome: format!("error:{}", err.message),
                            next_backoff_ms: should_retry
                                .then(|| self.retry_policy.backoff_ms(retry_index)),
                        });

                        last_error = err;
                        if !should_retry {
                            break;
                        }
                    }
                }
            }
        }

        InvocationTrace {
            attempts,
            result: Err(last_error),
        }
    }
}

pub struct RoundRobinCursor {
    cursor: AtomicUsize,
    pool_size: usize,
}

impl RoundRobinCursor {
    pub fn new(start: usize, pool_size: usize) -> Self {
        Self {
            cursor: AtomicUsize::new(start % pool_size.max(1)),
            pool_size,
        }
    }

    pub fn next(&self) -> usize {
        let next = self.cursor.fetch_add(1, Ordering::Relaxed);
        if self.pool_size == 0 {
            0
        } else {
            next % self.pool_size
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummarizeInput {
    pub content: String,
    pub tone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummarizeOutput {
    pub summary: String,
    pub risk_level: String,
}

pub struct SummarizeFunction {
    contract: TypedFunctionContract,
    executor: FallbackExecutor<SummarizeInput, SummarizeOutput>,
}

impl SummarizeFunction {
    pub fn new(executor: FallbackExecutor<SummarizeInput, SummarizeOutput>) -> Self {
        Self {
            contract: TypedFunctionContract {
                function_name: "SummarizeMutationImpact".to_string(),
                input_schema_ref: "nostra.workflow.mutation_input.v1".to_string(),
                output_schema_ref: "nostra.workflow.mutation_summary.v1".to_string(),
            },
            executor,
        }
    }

    pub fn invoke_with_trace(&self, input: &SummarizeInput) -> InvocationTrace<SummarizeOutput> {
        self.executor.run(input)
    }
}

impl TypedFunction<SummarizeInput, SummarizeOutput> for SummarizeFunction {
    fn contract(&self) -> &TypedFunctionContract {
        &self.contract
    }

    fn invoke(&self, input: &SummarizeInput) -> Result<SummarizeOutput, ProviderError> {
        self.executor.run(input).result
    }
}

#[cfg(test)]
#[derive(Clone)]
struct ScriptedProvider<O> {
    name: &'static str,
    queue: Arc<Mutex<VecDeque<Result<O, ProviderError>>>>,
}

#[cfg(test)]
impl<O> ScriptedProvider<O>
where
    O: Clone,
{
    fn from_steps(name: &'static str, steps: Vec<Result<O, ProviderError>>) -> Self {
        Self {
            name,
            queue: Arc::new(Mutex::new(VecDeque::from(steps))),
        }
    }
}

#[cfg(test)]
impl<I, O> Provider<I, O> for ScriptedProvider<O>
where
    O: Clone + Send + Sync + 'static,
    I: Send + Sync,
{
    fn name(&self) -> &str {
        self.name
    }

    fn call(&self, _input: &I) -> Result<O, ProviderError> {
        let mut guard = self.queue.lock().expect("queue lock poisoned");
        guard.pop_front().unwrap_or_else(|| {
            Err(ProviderError {
                provider: self.name.to_string(),
                kind: ErrorKind::Fatal,
                message: "script exhausted".to_string(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transient(provider: &str, message: &str) -> ProviderError {
        ProviderError {
            provider: provider.to_string(),
            kind: ErrorKind::Transient,
            message: message.to_string(),
        }
    }

    fn fatal(provider: &str, message: &str) -> ProviderError {
        ProviderError {
            provider: provider.to_string(),
            kind: ErrorKind::Fatal,
            message: message.to_string(),
        }
    }

    #[test]
    fn exponential_backoff_clamps_to_max() {
        let policy = RetryPolicy {
            max_retries: 4,
            strategy: RetryStrategy::ExponentialBackoff {
                delay_ms: 100,
                multiplier: 3,
                max_delay_ms: 500,
            },
        };

        assert_eq!(policy.backoff_ms(0), 100);
        assert_eq!(policy.backoff_ms(1), 300);
        assert_eq!(policy.backoff_ms(2), 500);
        assert_eq!(policy.backoff_ms(3), 500);
    }

    #[test]
    fn retries_and_then_fallback_succeeds() {
        let primary = ScriptedProvider::from_steps(
            "primary",
            vec![
                Err(transient("primary", "timeout")),
                Err(transient("primary", "rate-limit")),
                Err(fatal("primary", "quota-exhausted")),
            ],
        );
        let secondary = ScriptedProvider::from_steps(
            "secondary",
            vec![Ok(SummarizeOutput {
                summary: "stable result".to_string(),
                risk_level: "medium".to_string(),
            })],
        );

        let executor = FallbackExecutor::new(
            vec![Arc::new(primary), Arc::new(secondary)],
            RetryPolicy {
                max_retries: 2,
                strategy: RetryStrategy::ConstantDelay { delay_ms: 200 },
            },
        );
        let function = SummarizeFunction::new(executor);

        let trace = function.invoke_with_trace(&SummarizeInput {
            content: "proposed canister mutation".to_string(),
            tone: "formal".to_string(),
        });

        assert_eq!(trace.attempts.len(), 4);
        assert_eq!(trace.attempts[0].provider, "primary");
        assert_eq!(trace.attempts[1].provider, "primary");
        assert_eq!(trace.attempts[2].provider, "primary");
        assert_eq!(trace.attempts[3].provider, "secondary");

        let result = trace.result.expect("fallback should succeed");
        assert_eq!(result.summary, "stable result");
        assert_eq!(
            function.contract().output_schema_ref,
            "nostra.workflow.mutation_summary.v1"
        );
    }

    #[test]
    fn round_robin_cursor_wraps_deterministically() {
        let rr = RoundRobinCursor::new(1, 3);
        assert_eq!(rr.next(), 1);
        assert_eq!(rr.next(), 2);
        assert_eq!(rr.next(), 0);
        assert_eq!(rr.next(), 1);
    }
}
