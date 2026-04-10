use anyhow::{Result, anyhow};
use chrono::Utc;
use nostra_workflow_core::types::Context;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

pub const ACP_COLLECT_OP: &str = "ops.acp.collect_metrics";
pub const ACP_EVALUATE_OP: &str = "ops.acp.evaluate_slo";
pub const ACP_PUBLISH_OP: &str = "ops.acp.publish_evidence";

const ACP_COLLECT_SCRIPT: &str = "/Users/xaoj/ICP/scripts/acp_collect_metrics.sh";
const ACP_EVALUATE_SCRIPT: &str = "/Users/xaoj/ICP/scripts/acp_evaluate_slo.sh";
const DEFAULT_ACP_METRICS_LOG: &str = "/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/staging/metrics/acp_metrics_window.jsonl";
const DEFAULT_ACP_EVIDENCE_FILE: &str =
    "/Users/xaoj/ICP/research/103-agent-client-protocol-alignment/PILOT_STAGING_EVIDENCE.md";

#[derive(Debug, Clone, Default)]
pub struct AdapterExecutionResult {
    pub outputs: HashMap<String, String>,
    pub message: Option<String>,
}

pub trait OperationAdapter: Send + Sync {
    fn execute(&self, payload: &str, context: &mut Context) -> Result<AdapterExecutionResult>;
}

#[derive(Clone, Default)]
pub struct OperationRegistry {
    adapters: HashMap<String, Arc<dyn OperationAdapter>>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register_adapter(
        &mut self,
        op_type: impl Into<String>,
        adapter: Arc<dyn OperationAdapter>,
    ) {
        self.adapters.insert(op_type.into(), adapter);
    }

    pub fn has_adapter(&self, op_type: &str) -> bool {
        self.adapters.contains_key(op_type)
    }

    pub fn execute(
        &self,
        op_type: &str,
        payload: &str,
        context: &mut Context,
    ) -> Result<AdapterExecutionResult> {
        let adapter = self
            .adapters
            .get(op_type)
            .ok_or_else(|| anyhow!("no adapter registered for {}", op_type))?;
        adapter.execute(payload, context)
    }
}

pub fn create_default_registry() -> OperationRegistry {
    let mut registry = OperationRegistry::new();
    registry.register_adapter(ACP_COLLECT_OP, Arc::new(AcpCollectMetricsAdapter));
    registry.register_adapter(ACP_EVALUATE_OP, Arc::new(AcpEvaluateSloAdapter));
    registry.register_adapter(ACP_PUBLISH_OP, Arc::new(AcpPublishEvidenceAdapter));
    registry
}

#[derive(Default)]
pub struct AcpCollectMetricsAdapter;

impl OperationAdapter for AcpCollectMetricsAdapter {
    fn execute(&self, payload: &str, _context: &mut Context) -> Result<AdapterExecutionResult> {
        let payload = parse_payload(payload)?;
        let base_url = payload
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or("http://127.0.0.1:3000");
        let out_file = payload
            .get("out_file")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_ACP_METRICS_LOG);

        let status = Command::new(ACP_COLLECT_SCRIPT)
            .arg(base_url)
            .arg(out_file)
            .status()
            .map_err(|e| anyhow!("failed to run {}: {}", ACP_COLLECT_SCRIPT, e))?;

        if !status.success() {
            return Err(anyhow!(
                "ACP metrics collection failed with status {:?}",
                status.code()
            ));
        }

        let mut outputs = HashMap::new();
        outputs.insert(
            "acp.metrics.snapshot_path".to_string(),
            out_file.to_string(),
        );
        outputs.insert(
            "acp.event.last".to_string(),
            "AcpSloWindowSampleCaptured".to_string(),
        );
        outputs.insert("acp.event.timestamp".to_string(), Utc::now().to_rfc3339());

        Ok(AdapterExecutionResult {
            outputs,
            message: Some("ACP metrics snapshot captured".to_string()),
        })
    }
}

#[derive(Default)]
pub struct AcpEvaluateSloAdapter;

impl OperationAdapter for AcpEvaluateSloAdapter {
    fn execute(&self, payload: &str, context: &mut Context) -> Result<AdapterExecutionResult> {
        let payload = parse_payload(payload)?;
        let input_file = payload
            .get("input_file")
            .and_then(|v| v.as_str())
            .or_else(|| context.get("acp.metrics.snapshot_path").map(|s| s.as_str()))
            .unwrap_or(DEFAULT_ACP_METRICS_LOG);

        let status = Command::new(ACP_EVALUATE_SCRIPT)
            .arg(input_file)
            .status()
            .map_err(|e| anyhow!("failed to run {}: {}", ACP_EVALUATE_SCRIPT, e))?;

        let result = if status.success() { "PASS" } else { "FAIL" };
        let reason = if status.success() {
            "SLO thresholds satisfied"
        } else {
            "SLO thresholds not satisfied"
        };

        let mut outputs = HashMap::new();
        outputs.insert("acp.slo.result".to_string(), result.to_string());
        outputs.insert("acp.slo.reason".to_string(), reason.to_string());
        outputs.insert("acp.slo.input_file".to_string(), input_file.to_string());
        outputs.insert(
            "acp.event.last".to_string(),
            "AcpSloEvaluationComputed".to_string(),
        );
        outputs.insert("acp.event.timestamp".to_string(), Utc::now().to_rfc3339());

        Ok(AdapterExecutionResult {
            outputs,
            message: Some(format!("ACP SLO evaluation result: {}", result)),
        })
    }
}

#[derive(Default)]
pub struct AcpPublishEvidenceAdapter;

impl OperationAdapter for AcpPublishEvidenceAdapter {
    fn execute(&self, payload: &str, context: &mut Context) -> Result<AdapterExecutionResult> {
        let payload = parse_payload(payload)?;
        let evidence_file = payload
            .get("evidence_file")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_ACP_EVIDENCE_FILE);

        let confidence_score = payload
            .get("confidence_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.85);
        let source_reliability = payload
            .get("source_reliability")
            .and_then(|v| v.as_str())
            .unwrap_or("high");
        let validation_proof = payload
            .get("validation_proof")
            .and_then(|v| v.as_str())
            .unwrap_or(ACP_EVALUATE_SCRIPT);

        let timestamp = Utc::now().to_rfc3339();
        let fragment = format!(
            "\n\n### Automation Event: AcpPromotionGateDecisionRequested ({timestamp})\n\
             - `acp.slo.result`: `{}`\n\
             - `acp.slo.reason`: `{}`\n\
             - `confidence_score`: `{confidence_score:.2}`\n\
             - `source_reliability`: `{source_reliability}`\n\
             - `validation_proof`: `{validation_proof}`\n",
            context
                .get("acp.slo.result")
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            context
                .get("acp.slo.reason")
                .cloned()
                .unwrap_or_else(|| "No reason captured".to_string()),
        );

        let evidence_path = PathBuf::from(evidence_file);
        if let Some(parent) = evidence_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut existing = if evidence_path.exists() {
            std::fs::read_to_string(&evidence_path)?
        } else {
            String::new()
        };
        existing.push_str(&fragment);
        std::fs::write(&evidence_path, existing)?;

        let mut outputs = HashMap::new();
        outputs.insert(
            "acp.evidence.updated_files".to_string(),
            evidence_file.to_string(),
        );
        outputs.insert(
            "acp.event.last".to_string(),
            "AcpPromotionGateDecisionRequested".to_string(),
        );
        outputs.insert("acp.event.timestamp".to_string(), timestamp);

        Ok(AdapterExecutionResult {
            outputs,
            message: Some("ACP evidence updated".to_string()),
        })
    }
}

fn parse_payload(payload: &str) -> Result<Value> {
    if payload.trim().is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str::<Value>(payload).map_err(|e| anyhow!("invalid SystemOp payload: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockAdapter;

    impl OperationAdapter for MockAdapter {
        fn execute(
            &self,
            _payload: &str,
            _context: &mut Context,
        ) -> Result<AdapterExecutionResult> {
            let mut outputs = HashMap::new();
            outputs.insert("k".to_string(), "v".to_string());
            Ok(AdapterExecutionResult {
                outputs,
                message: Some("ok".to_string()),
            })
        }
    }

    #[test]
    fn registry_dispatches_to_registered_adapter() {
        let mut registry = OperationRegistry::new();
        registry.register_adapter("ops.test", Arc::new(MockAdapter));
        let mut ctx = Context::new();
        let result = registry.execute("ops.test", "{}", &mut ctx).unwrap();
        assert_eq!(result.outputs.get("k").map(String::as_str), Some("v"));
    }

    #[test]
    fn registry_reports_unknown_operation() {
        let registry = OperationRegistry::new();
        let mut ctx = Context::new();
        let err = registry.execute("ops.unknown", "{}", &mut ctx).unwrap_err();
        assert!(err.to_string().contains("no adapter registered"));
    }

    #[test]
    fn publish_adapter_writes_structured_fragment() {
        let adapter = AcpPublishEvidenceAdapter;
        let mut ctx = Context::new();
        ctx.set("acp.slo.result", "PASS");
        ctx.set("acp.slo.reason", "All thresholds met");

        let target =
            std::env::temp_dir().join(format!("acp_evidence_{}.md", uuid::Uuid::new_v4().simple()));
        let payload = serde_json::json!({
            "evidence_file": target.display().to_string(),
            "confidence_score": 0.9,
            "source_reliability": "high",
            "validation_proof": "test-proof"
        })
        .to_string();

        let result = adapter.execute(&payload, &mut ctx).unwrap();
        assert!(
            result
                .outputs
                .get("acp.evidence.updated_files")
                .map(|v| v == &target.display().to_string())
                .unwrap_or(false)
        );
        let raw = std::fs::read_to_string(target).unwrap();
        assert!(raw.contains("AcpPromotionGateDecisionRequested"));
        assert!(raw.contains("confidence_score"));
    }
}
