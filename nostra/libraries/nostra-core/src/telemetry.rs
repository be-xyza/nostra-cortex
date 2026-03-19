use crate::interceptor::{Interceptor, Next, Outcome, TaskContext};
use async_trait::async_trait;
use log::error;

/// A Glass Box Interceptor that logs every execution as a "Span"
pub struct OtelInterceptor {
    pub service_name: String,
}

impl OtelInterceptor {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
}

#[async_trait]
impl Interceptor for OtelInterceptor {
    fn name(&self) -> &str {
        "OtelInterceptor"
    }

    async fn intercept(&self, task: Box<dyn TaskContext>, next: Next) -> Outcome {
        let task_name = task.name();
        let trace_id = task.trace_id();
        let start = std::time::SystemTime::now(); // On IC this might be constant or 0, we'll patch later

        // 1. Emit Start Span (Console Log for MVP)
        // Format: [OTEL] Span:Start | TraceID:xyz | Service:Worker | Op:ChatSkill
        println!(
            "[OTEL] Span:Start | TraceID:{} | Service:{} | Op:{}",
            trace_id, self.service_name, task_name
        );

        // 2. Execute Next (The Logic)
        let result = next(task).await;

        // 3. Emit End Span
        // Calculate duration if possible, else 0
        let elapsed = start
            .elapsed()
            .unwrap_or(std::time::Duration::from_millis(0));

        match &result {
            Ok(_) => {
                println!(
                    "[OTEL] Span:End   | TraceID:{} | Status:OK | Duration:{}ms",
                    trace_id,
                    elapsed.as_millis()
                );
            }
            Err(e) => {
                println!(
                    "[OTEL] Span:End   | TraceID:{} | Status:ERROR | Duration:{}ms | Error:{}",
                    trace_id,
                    elapsed.as_millis(),
                    e
                );
                error!("TraceID:{} Failed: {}", trace_id, e);
            }
        }

        result
    }
}
