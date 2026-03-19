use crate::{Document, ExtractionStatus, Stage};
use anyhow::Result;

pub struct VerifyStage;

impl Stage for VerifyStage {
    fn name(&self) -> &'static str {
        "Verify (Schema)"
    }

    fn process(&self, input: Document) -> Result<Document> {
        let mut doc = input;
        let min_confidence = doc.request.fallback_policy.min_confidence.clamp(0.0, 1.0);

        if doc.candidate_entities.is_empty() {
            doc.status = ExtractionStatus::NeedsReview;
            doc.flags
                .push("verification_failed_no_entities".to_string());
        } else if doc.confidence < min_confidence {
            doc.status = ExtractionStatus::NeedsReview;
            doc.flags.push(format!(
                "verification_confidence_below_threshold:{:.2}<{:.2}",
                doc.confidence, min_confidence
            ));
        } else {
            doc.status = ExtractionStatus::Completed;
        }

        doc.provenance.confidence = doc.confidence;
        doc.metadata["verification_status"] = serde_json::json!(match doc.status {
            ExtractionStatus::Completed => "completed",
            ExtractionStatus::NeedsReview => "needs_review",
            ExtractionStatus::Failed => "failed",
            ExtractionStatus::Running => "running",
            ExtractionStatus::Submitted => "submitted",
        });

        Ok(doc)
    }
}
