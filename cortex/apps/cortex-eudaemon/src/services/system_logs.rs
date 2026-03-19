use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamFormat {
    Json,
    Jsonl,
    Text,
}

impl StreamFormat {
    fn as_str(self) -> &'static str {
        match self {
            StreamFormat::Json => "json",
            StreamFormat::Jsonl => "jsonl",
            StreamFormat::Text => "text",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LogStreamDescriptor {
    pub stream_id: String,
    pub label: String,
    pub format: String,
    pub required_role: String,
    pub source: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogStreamsResponse {
    pub schema_version: String,
    pub generated_at: String,
    pub streams: Vec<LogStreamDescriptor>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogTailEvent {
    pub ts: Option<String>,
    pub level: String,
    pub subsystem: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_text_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogTailResponse {
    pub schema_version: String,
    pub stream_id: String,
    pub format: String,
    pub cursor: u64,
    pub next_cursor: u64,
    pub reset: bool,
    pub events: Vec<LogTailEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamDefinition {
    pub stream_id: &'static str,
    pub label: &'static str,
    pub format: StreamFormat,
    pub required_role: &'static str,
    pub source: &'static str,
    pub description: &'static str,
}

fn workspace_root() -> PathBuf {
    std::env::var("NOSTRA_WORKSPACE_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn resolve_siq_log_dir() -> PathBuf {
    std::env::var("NOSTRA_SIQ_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("logs").join("siq"))
}

fn resolve_testing_log_dir() -> PathBuf {
    std::env::var("NOSTRA_TESTING_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("logs").join("testing"))
}

fn decision_surface_log_dir() -> PathBuf {
    std::env::var("NOSTRA_DECISION_SURFACE_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root()
                .join("logs")
                .join("system")
                .join("decision_surfaces")
        })
}

fn decision_actions_dir() -> PathBuf {
    decision_surface_log_dir().join("actions")
}

fn stream_definitions() -> Vec<StreamDefinition> {
    vec![
        StreamDefinition {
            stream_id: "siq_gate_summary_latest",
            label: "SIQ Gate Summary (latest)",
            format: StreamFormat::Json,
            required_role: "operator",
            source: "NOSTRA_SIQ_LOG_DIR",
            description: "Rolling latest SIQ gate verdict and failures.",
        },
        StreamDefinition {
            stream_id: "testing_gate_summary_latest",
            label: "Testing Gate Summary (latest)",
            format: StreamFormat::Json,
            required_role: "operator",
            source: "NOSTRA_TESTING_LOG_DIR",
            description: "Rolling latest testing gate verdict and failures.",
        },
        StreamDefinition {
            stream_id: "layout_spec_updates",
            label: "Layout Spec Updates",
            format: StreamFormat::Jsonl,
            required_role: "steward",
            source: "NOSTRA_DECISION_SURFACE_LOG_DIR",
            description: "Steward-gated UX contract updates (audit trail).",
        },
        StreamDefinition {
            stream_id: "gate_emit_audit",
            label: "Gate Emit Audit",
            format: StreamFormat::Jsonl,
            required_role: "operator",
            source: "NOSTRA_DECISION_SURFACE_LOG_DIR",
            description: "Audit trail for emitting gate summaries into Heap.",
        },
    ]
}

pub fn list_streams() -> Vec<LogStreamDescriptor> {
    let mut streams = stream_definitions()
        .into_iter()
        .map(|def| LogStreamDescriptor {
            stream_id: def.stream_id.to_string(),
            label: def.label.to_string(),
            format: def.format.as_str().to_string(),
            required_role: def.required_role.to_string(),
            source: def.source.to_string(),
            description: def.description.to_string(),
        })
        .collect::<Vec<_>>();
    streams.sort_by(|a, b| a.stream_id.cmp(&b.stream_id));
    streams
}

pub fn resolve_stream(stream_id: &str) -> Option<(StreamDefinition, PathBuf)> {
    let def = stream_definitions()
        .into_iter()
        .find(|candidate| candidate.stream_id == stream_id)?;

    let path = match def.stream_id {
        "siq_gate_summary_latest" => resolve_siq_log_dir().join("siq_gate_summary_latest.json"),
        "testing_gate_summary_latest" => {
            resolve_testing_log_dir().join("test_gate_summary_latest.json")
        }
        "layout_spec_updates" => decision_actions_dir().join("layout_spec_updates.jsonl"),
        "gate_emit_audit" => decision_actions_dir().join("gate_emit_audit.jsonl"),
        _ => return None,
    };
    Some((def, path))
}

pub fn role_allows(actor_role: &str, required_role: &str) -> bool {
    fn rank(role: &str) -> u8 {
        match role.trim().to_ascii_lowercase().as_str() {
            "steward" => 3,
            "operator" => 2,
            "viewer" => 1,
            _ => 0,
        }
    }
    rank(actor_role) >= rank(required_role)
}

pub fn tail_stream(
    stream_id: &str,
    cursor: u64,
    limit: u32,
) -> Result<(StreamDefinition, LogTailResponse), String> {
    let (def, path) = resolve_stream(stream_id).ok_or_else(|| "unknown stream".to_string())?;
    let stream_id_owned = def.stream_id.to_string();
    let format_owned = def.format.as_str().to_string();
    let limit = limit.clamp(1, 500) as usize;
    let max_bytes: usize = 256 * 1024;

    match def.format {
        StreamFormat::Json => {
            let raw = fs::read_to_string(&path)
                .map_err(|err| format!("read {}: {}", path.display(), err))?;
            let parsed: Value = serde_json::from_str(&raw)
                .map_err(|err| format!("parse {}: {}", path.display(), err))?;
            let event = LogTailEvent {
                ts: parsed
                    .get("generated_at")
                    .or_else(|| parsed.get("generatedAt"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                level: "info".to_string(),
                subsystem: stream_id_owned.clone(),
                message: "json_snapshot".to_string(),
                raw: Some(parsed),
                raw_text_line: None,
            };
            let response = LogTailResponse {
                schema_version: "1.0.0".to_string(),
                stream_id: stream_id_owned,
                format: format_owned,
                cursor: 0,
                next_cursor: 0,
                reset: false,
                events: vec![event],
            };
            return Ok((def, response));
        }
        StreamFormat::Jsonl | StreamFormat::Text => {
            let metadata =
                fs::metadata(&path).map_err(|err| format!("stat {}: {}", path.display(), err))?;
            let len = metadata.len();
            let mut effective_cursor = cursor;
            let mut reset = false;
            if effective_cursor > len {
                effective_cursor = 0;
                reset = true;
            }

            let mut file =
                fs::File::open(&path).map_err(|err| format!("open {}: {}", path.display(), err))?;
            file.seek(SeekFrom::Start(effective_cursor))
                .map_err(|err| format!("seek {}: {}", path.display(), err))?;

            let mut buf = vec![0u8; max_bytes];
            let bytes_read = file
                .read(&mut buf)
                .map_err(|err| format!("read {}: {}", path.display(), err))?;
            buf.truncate(bytes_read);

            let mut cutoff = buf.len();
            if let Some(pos) = buf.iter().rposition(|b| *b == b'\n') {
                cutoff = pos + 1;
            } else if bytes_read == max_bytes {
                cutoff = 0;
            }

            let chunk = &buf[..cutoff];
            let text = String::from_utf8_lossy(chunk);
            let mut events = Vec::new();
            for line in text.lines().take(limit) {
                if line.trim().is_empty() {
                    continue;
                }
                match def.format {
                    StreamFormat::Jsonl => match serde_json::from_str::<Value>(line) {
                        Ok(value) => events.push(LogTailEvent {
                            ts: value
                                .get("generatedAt")
                                .or_else(|| value.get("generated_at"))
                                .or_else(|| value.get("ts"))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            level: value
                                .get("level")
                                .and_then(|v| v.as_str())
                                .unwrap_or("info")
                                .to_string(),
                            subsystem: value
                                .get("subsystem")
                                .and_then(|v| v.as_str())
                                .unwrap_or(def.stream_id)
                                .to_string(),
                            message: value
                                .get("message")
                                .or_else(|| value.get("eventType"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("event")
                                .to_string(),
                            raw: Some(value),
                            raw_text_line: None,
                        }),
                        Err(_) => events.push(LogTailEvent {
                            ts: None,
                            level: "info".to_string(),
                            subsystem: stream_id_owned.clone(),
                            message: "raw_line".to_string(),
                            raw: None,
                            raw_text_line: Some(line.to_string()),
                        }),
                    },
                    StreamFormat::Text => events.push(LogTailEvent {
                        ts: None,
                        level: "info".to_string(),
                        subsystem: stream_id_owned.clone(),
                        message: "line".to_string(),
                        raw: None,
                        raw_text_line: Some(line.to_string()),
                    }),
                    StreamFormat::Json => {}
                }
            }

            let next_cursor = effective_cursor + cutoff as u64;
            let response = LogTailResponse {
                schema_version: "1.0.0".to_string(),
                stream_id: stream_id_owned,
                format: format_owned,
                cursor: effective_cursor,
                next_cursor,
                reset,
                events,
            };
            Ok((def, response))
        }
    }
}

pub fn inventory_response(generated_at: &str) -> LogStreamsResponse {
    LogStreamsResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: generated_at.to_string(),
        streams: list_streams(),
    }
}

pub fn stream_missing_error(stream_id: &str) -> Value {
    json!({ "streamId": stream_id })
}
