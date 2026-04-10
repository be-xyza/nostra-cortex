use cortex_runtime::resilience::{ResilienceProbeInput, calculate};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResilienceReport {
    pub las: f32,
    pub ds: f32,
    pub last_probed_at: String,
    pub local_data_availability: f32,
    pub offline_write_capability: f32,
    pub execution_autonomy: f32,
    pub control_plane_decentralization: f32,
    pub execution_dependency: f32,
    pub data_custody: f32,
    pub probe_notes: Vec<String>,
}

pub struct ResilienceService;

impl ResilienceService {
    pub fn new() -> Self {
        Self
    }

    pub async fn calculate_scores(&self) -> ResilienceReport {
        let ic_client = crate::services::ic_client::IcClient::new(None);
        let replica_running = ic_client.is_replica_running().await;
        let gateway_probe = crate::gateway::runtime_host::local_gateway_probe();
        let breakdown = calculate(&ResilienceProbeInput {
            replica_running,
            gateway_online: gateway_probe.gateway_online,
            queue_export_ok: gateway_probe.queue_export_ok,
            queue_size: gateway_probe.queue_size,
        });

        ResilienceReport {
            las: breakdown.las,
            ds: breakdown.ds,
            last_probed_at: chrono::Utc::now().to_rfc3339(),
            local_data_availability: breakdown.local_data_availability,
            offline_write_capability: breakdown.offline_write_capability,
            execution_autonomy: breakdown.execution_autonomy,
            control_plane_decentralization: breakdown.control_plane_decentralization,
            execution_dependency: breakdown.execution_dependency,
            data_custody: breakdown.data_custody,
            probe_notes: breakdown.notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[tokio::test]
    async fn resilience_report_contains_runtime_probe_fields() {
        let report = ResilienceService::new().calculate_scores().await;
        assert!(report.las >= 0.0 && report.las <= 100.0);
        assert!(report.ds >= 0.0 && report.ds <= 100.0);
        assert!(report.offline_write_capability >= 0.0 && report.offline_write_capability <= 100.0);
        assert!(DateTime::parse_from_rfc3339(&report.last_probed_at).is_ok());
    }
}
