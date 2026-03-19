use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResilienceProbeInput {
    pub replica_running: bool,
    pub gateway_online: bool,
    pub queue_export_ok: bool,
    pub queue_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResilienceScoreBreakdown {
    pub las: f32,
    pub ds: f32,
    pub local_data_availability: f32,
    pub offline_write_capability: f32,
    pub execution_autonomy: f32,
    pub control_plane_decentralization: f32,
    pub execution_dependency: f32,
    pub data_custody: f32,
    pub notes: Vec<String>,
}

pub fn calculate(input: &ResilienceProbeInput) -> ResilienceScoreBreakdown {
    let mut notes = Vec::new();

    let lda = if input.replica_running {
        100.0
    } else {
        notes.push("DFX replica probe failed; local data availability degraded.".to_string());
        45.0
    };

    let owc = if input.queue_export_ok {
        if input.queue_size > 0 {
            notes.push(format!(
                "Local gateway queue has {} pending mutation(s); offline buffering active.",
                input.queue_size
            ));
            92.0
        } else {
            100.0
        }
    } else {
        notes.push("Local gateway queue export probe failed.".to_string());
        35.0
    };

    let ea = match (input.replica_running, input.gateway_online) {
        (true, true) => 100.0,
        (false, true) => 72.0,
        (true, false) => 78.0,
        (false, false) => 58.0,
    };

    if ea < 100.0 {
        notes.push(format!(
            "Execution autonomy reduced (replica_running={}, local_gateway_online={}).",
            input.replica_running, input.gateway_online
        ));
    }

    let las_score = (0.4 * lda) + (0.35 * owc) + (0.25 * ea);
    let cpd = if input.queue_export_ok { 82.0 } else { 61.0 };
    let ed = if input.replica_running { 90.0 } else { 62.0 };
    let dc = if input.replica_running && input.queue_export_ok {
        96.0
    } else if input.queue_export_ok {
        78.0
    } else {
        54.0
    };

    if dc < 90.0 {
        notes.push("Data custody confidence degraded by probe outcomes.".to_string());
    }

    let ds_score = (0.35 * cpd) + (0.35 * ed) + (0.30 * dc);

    ResilienceScoreBreakdown {
        las: las_score,
        ds: ds_score,
        local_data_availability: lda,
        offline_write_capability: owc,
        execution_autonomy: ea,
        control_plane_decentralization: cpd,
        execution_dependency: ed,
        data_custody: dc,
        notes,
    }
}
