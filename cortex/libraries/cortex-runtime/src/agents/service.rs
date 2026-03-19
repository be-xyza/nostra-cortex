use crate::agents::types::Modality;

pub fn worker_modality(modality: &Modality) -> &'static str {
    match modality {
        Modality::Text => "text",
        Modality::Image => "image",
        Modality::Audio => "audio",
        Modality::Video => "video",
    }
}

pub fn modality_from_worker(raw: Option<&str>) -> Modality {
    match raw.unwrap_or("text").trim().to_ascii_lowercase().as_str() {
        "image" => Modality::Image,
        "audio" => Modality::Audio,
        "video" => Modality::Video,
        _ => Modality::Text,
    }
}

pub fn build_index_idempotency_key(doc_id: &str, timestamp_ms: i64) -> String {
    format!("{}:{}", doc_id, timestamp_ms)
}
