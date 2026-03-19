use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// A generic Task envelope passed through the middleware chain
pub trait TaskContext: Send + Sync + std::fmt::Debug {
    fn trace_id(&self) -> String;
    fn name(&self) -> String;
}

/// A generic Outcome from a task execution
pub type Outcome = Result<String>;

/// The Next step in the chain
/// Changed to 'static to simplify async ownership
pub type Next = Box<dyn FnOnce(Box<dyn TaskContext>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Outcome> + Send>> + Send>;

/// The Core Interceptor Trait (Glass Box)
#[async_trait]
pub trait Interceptor: Send + Sync {
    async fn intercept(&self, task: Box<dyn TaskContext>, next: Next) -> Outcome;
    fn name(&self) -> &str;
}

/// The Stack Manager
#[derive(Clone)]
pub struct MiddlewareStack {
    chain: Vec<Arc<dyn Interceptor>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self { chain: Vec::new() }
    }

    pub fn add<I: Interceptor + 'static>(mut self, interceptor: I) -> Self {
        self.chain.push(Arc::new(interceptor));
        self
    }

    pub async fn execute<F, Fut>(&self, task: Box<dyn TaskContext>, core_logic: F) -> Outcome
    where
        F: FnOnce(Box<dyn TaskContext>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Outcome> + Send + 'static,
    {
        let chain = self.chain.clone();
        let next: Next = Box::new(move |t| Box::pin(core_logic(t)));
        
        Self::run_chain(chain, 0, task, next).await
    }

    fn run_chain(
        chain: Vec<Arc<dyn Interceptor>>,
        idx: usize,
        task: Box<dyn TaskContext>,
        final_logic: Next,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Outcome> + Send>> {
        Box::pin(async move {
            if idx >= chain.len() {
                return final_logic(task).await;
            }

            let interceptor = chain[idx].clone();
            // Recursively call run_chain with incremented index
            // We clone chain again for the closure
            let next_chain = chain.clone(); 
            let next_step: Next = Box::new(move |t| Self::run_chain(next_chain, idx + 1, t, final_logic));
            
            interceptor.intercept(task, next_step).await
        })
    }
}
