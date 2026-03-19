use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Metrics {
    inner: Arc<Inner>,
}

struct Inner {
    started_at: Instant,
    deliveries_accepted_total: AtomicU64,
    deliveries_rejected_total: AtomicU64,
    deliveries_idempotent_total: AtomicU64,
    deliveries_failed_total: AtomicU64,
    execute_kip_failures_total: AtomicU64,
    reconcile_runs_total: AtomicU64,
    reconcile_failures_total: AtomicU64,
    reconcile_repo_runs_total: AtomicU64,
    reconcile_repo_success_total: AtomicU64,
    reconcile_repo_failures_total: AtomicU64,
    reconcile_repo_last_duration_ms: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                started_at: Instant::now(),
                deliveries_accepted_total: AtomicU64::new(0),
                deliveries_rejected_total: AtomicU64::new(0),
                deliveries_idempotent_total: AtomicU64::new(0),
                deliveries_failed_total: AtomicU64::new(0),
                execute_kip_failures_total: AtomicU64::new(0),
                reconcile_runs_total: AtomicU64::new(0),
                reconcile_failures_total: AtomicU64::new(0),
                reconcile_repo_runs_total: AtomicU64::new(0),
                reconcile_repo_success_total: AtomicU64::new(0),
                reconcile_repo_failures_total: AtomicU64::new(0),
                reconcile_repo_last_duration_ms: AtomicU64::new(0),
            }),
        }
    }

    pub fn uptime(&self) -> Duration {
        self.inner.started_at.elapsed()
    }

    pub fn inc_delivery_accepted(&self) {
        self.inner
            .deliveries_accepted_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_delivery_rejected(&self) {
        self.inner
            .deliveries_rejected_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_delivery_idempotent(&self) {
        self.inner
            .deliveries_idempotent_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_delivery_failed(&self) {
        self.inner
            .deliveries_failed_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_execute_kip_failure(&self) {
        self.inner
            .execute_kip_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reconcile_run(&self) {
        self.inner
            .reconcile_runs_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reconcile_failure(&self) {
        self.inner
            .reconcile_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reconcile_repo_run(&self) {
        self.inner
            .reconcile_repo_runs_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reconcile_repo_success(&self) {
        self.inner
            .reconcile_repo_success_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reconcile_repo_failure(&self) {
        self.inner
            .reconcile_repo_failures_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_reconcile_repo_last_duration_ms(&self, ms: u64) {
        self.inner
            .reconcile_repo_last_duration_ms
            .store(ms, Ordering::Relaxed);
    }

    pub fn render_prometheus(&self) -> String {
        let uptime_secs = self.uptime().as_secs_f64();
        let mut out = String::new();

        macro_rules! counter {
            ($name:literal, $help:literal, $value:expr) => {{
                out.push_str("# HELP ");
                out.push_str($name);
                out.push(' ');
                out.push_str($help);
                out.push('\n');
                out.push_str("# TYPE ");
                out.push_str($name);
                out.push_str(" counter\n");
                out.push_str($name);
                out.push(' ');
                out.push_str(&$value.to_string());
                out.push('\n');
            }};
        }

        out.push_str("# HELP cortex_git_adapter_uptime_seconds Process uptime in seconds.\n");
        out.push_str("# TYPE cortex_git_adapter_uptime_seconds gauge\n");
        out.push_str("cortex_git_adapter_uptime_seconds ");
        out.push_str(&format!("{uptime_secs:.3}\n"));

        counter!(
            "cortex_git_adapter_deliveries_accepted_total",
            "Webhook deliveries accepted and processed.",
            self.inner
                .deliveries_accepted_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_deliveries_rejected_total",
            "Webhook deliveries rejected (bad request/unauthorized/unregistered).",
            self.inner
                .deliveries_rejected_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_deliveries_idempotent_total",
            "Webhook deliveries skipped because delivery id was already seen.",
            self.inner
                .deliveries_idempotent_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_deliveries_failed_total",
            "Webhook deliveries that failed after verification (projection/sink errors).",
            self.inner.deliveries_failed_total.load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_execute_kip_failures_total",
            "Failures calling executeKip on the Nostra backend.",
            self.inner
                .execute_kip_failures_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_reconcile_runs_total",
            "Total reconcile scheduler iterations.",
            self.inner.reconcile_runs_total.load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_reconcile_failures_total",
            "Total reconcile failures (any repo).",
            self.inner
                .reconcile_failures_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_reconcile_repo_runs_total",
            "Total reconcile repo runs attempted.",
            self.inner
                .reconcile_repo_runs_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_reconcile_repo_success_total",
            "Total reconcile repo runs that completed successfully.",
            self.inner
                .reconcile_repo_success_total
                .load(Ordering::Relaxed)
        );
        counter!(
            "cortex_git_adapter_reconcile_repo_failures_total",
            "Total reconcile repo failures.",
            self.inner
                .reconcile_repo_failures_total
                .load(Ordering::Relaxed)
        );

        out.push_str(
            "# HELP cortex_git_adapter_reconcile_repo_last_duration_ms Duration of the last reconcile_repo run.\n",
        );
        out.push_str("# TYPE cortex_git_adapter_reconcile_repo_last_duration_ms gauge\n");
        out.push_str("cortex_git_adapter_reconcile_repo_last_duration_ms ");
        out.push_str(
            &self
                .inner
                .reconcile_repo_last_duration_ms
                .load(Ordering::Relaxed)
                .to_string(),
        );
        out.push('\n');

        out
    }
}
