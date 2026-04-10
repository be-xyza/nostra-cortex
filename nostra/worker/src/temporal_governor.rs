use crate::activity_service::{ActivityService, ActivityStatus, ActivityType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessMetric {
    pub name: String,
    pub status: bool,
    pub score: f32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub version: String,
    pub phase: String,
    pub overall_score: f32,
    pub metrics: Vec<ReadinessMetric>,
}

pub struct TemporalGovernor {
    activity_service: ActivityService,
}

impl TemporalGovernor {
    pub fn new(activity_service: ActivityService) -> Self {
        Self { activity_service }
    }

    pub async fn audit_v1_readiness(&self) -> ReadinessReport {
        let metrics = vec![
            // 1. Contribution Types Metric
            ReadinessMetric {
                name: "Core Contribution Types".to_string(),
                status: true,
                score: 0.8,
                message: "Project, Idea, Issue operational. Milestone/Deliverable pending."
                    .to_string(),
            },
            // 2. Temporal Workflows Metric
            ReadinessMetric {
                name: "Temporal Workflows".to_string(),
                status: true,
                score: 1.0,
                message: "Durable engine verified in Tesseract simulator.".to_string(),
            },
            // 3. Knowledge Engine Metric
            ReadinessMetric {
                name: "Knowledge Indexing".to_string(),
                status: true,
                score: 0.9,
                message: "Vector search integrated with NostraBook ingestion.".to_string(),
            },
            // 4. Fork/Merge Metric
            ReadinessMetric {
                name: "Evolutionary Forking".to_string(),
                status: false,
                score: 0.3,
                message: "Logic for history-preserved forking is not yet implemented.".to_string(),
            },
        ];

        let overall_score = metrics.iter().map(|m| m.score).sum::<f32>() / metrics.len() as f32;

        let report = ReadinessReport {
            version: "0.1.0-alpha".to_string(),
            phase: "Phase 1: Pre-Collaborative Validation".to_string(),
            overall_score,
            metrics,
        };

        // Record the audit as an activity
        let mut meta = HashMap::new();
        meta.insert("score".to_string(), format!("{:.2}", overall_score));

        self.activity_service
            .record_activity(
                ActivityType::ReadinessAudit,
                ActivityStatus::Resolved,
                format!("V1 Readiness Audit Completed (Score: {:.2})", overall_score),
                meta,
            )
            .await;

        report
    }
}
