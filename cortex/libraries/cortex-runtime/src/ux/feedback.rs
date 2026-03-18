use cortex_domain::ux::{
    UX_STATUS_BLOCKED_MISSING_BASELINE, UX_STATUS_OVERDUE_REMEASUREMENT, UxFeedbackEvent,
    UxFeedbackQueueItem,
};

pub fn feedback_dedupe_key(event: &UxFeedbackEvent) -> String {
    format!(
        "{}::{}::{}::{}",
        event.route_id.trim(),
        event.view_id.trim(),
        event.friction_tag.trim().to_ascii_lowercase(),
        event.severity.trim().to_ascii_lowercase(),
    )
}

pub fn feedback_default_priority(severity: &str) -> String {
    match severity.trim().to_ascii_lowercase().as_str() {
        "critical" => "p0".to_string(),
        "high" => "p1".to_string(),
        "medium" => "p2".to_string(),
        _ => "p3".to_string(),
    }
}

pub fn mark_overdue_if_shipped(item: &mut UxFeedbackQueueItem) {
    if item.status == "shipped" {
        item.status = UX_STATUS_OVERDUE_REMEASUREMENT.to_string();
    }
}

pub fn requires_baseline_before_remeasure(item: &UxFeedbackQueueItem) -> bool {
    item.status == UX_STATUS_BLOCKED_MISSING_BASELINE || item.baseline_metric_date.is_none()
}
