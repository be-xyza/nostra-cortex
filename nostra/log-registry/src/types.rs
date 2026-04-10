use candid::{CandidType, Deserialize};
use std::collections::HashMap;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum SeverityNumber {
    Unspecified = 0,
    Trace = 1,
    Trace2 = 2,
    Trace3 = 3,
    Trace4 = 4,
    Debug = 5,
    Debug2 = 6,
    Debug3 = 7,
    Debug4 = 8,
    Info = 9,
    Info2 = 10,
    Info3 = 11,
    Info4 = 12,
    Warn = 13,
    Warn2 = 14,
    Warn3 = 15,
    Warn4 = 16,
    Error = 17,
    Error2 = 18,
    Error3 = 19,
    Error4 = 20,
    Fatal = 21,
    Fatal2 = 22,
    Fatal3 = 23,
    Fatal4 = 24,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LogEvent {
    pub time_unix_nano: u64,
    pub severity_number: SeverityNumber,
    pub severity_text: Option<String>,
    pub body: String, // Simplified from pdata AnyValue
    pub attributes: HashMap<String, String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct LogBatch {
    pub resource_attributes: HashMap<String, String>, // Resource level attributes (e.g., service.name)
    pub logs: Vec<LogEvent>,
}

use ic_stable_structures::Storable;
use ic_stable_structures::storable::Bound;
use std::borrow::Cow;

impl Storable for LogEvent {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: HashMap<String, String>,
    pub status: SpanStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}
