use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// CloudEvents v1.0 Specification
/// https://github.com/cloudevents/spec/blob/v1.0.2/cloudevents/spec.md
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Identifies the event.
    #[serde(rename = "id")]
    pub id: String,

    /// Identifies the context in which an event happened.
    #[serde(rename = "source")]
    pub source: String,

    /// The version of the CloudEvents specification which the event uses.
    #[serde(rename = "specversion")]
    pub spec_version: String,

    /// This attribute contains a value describing the type of event related to the originating occurrence.
    #[serde(rename = "type")]
    pub type_: String,

    /// Content type of data value. This attribute enables "data" to carry any type of content,
    /// whereby format and encoding might differ from that of the chosen event format.
    #[serde(rename = "datacontenttype", skip_serializing_if = "Option::is_none")]
    pub data_content_type: Option<String>,

    /// Identifies the schema that data adheres to.
    #[serde(rename = "dataschema", skip_serializing_if = "Option::is_none")]
    pub data_schema: Option<Url>,

    /// This describes the subject of the event in the context of the event producer (identified by source).
    #[serde(rename = "subject", skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// Timestamp of when the occurrence happened.
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<DateTime<Utc>>,

    /// The event payload.
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source: "nostra://unknown".to_string(),
            spec_version: "1.0".to_string(),
            type_: "nostra.event".to_string(),
            data_content_type: Some("application/json".to_string()),
            data_schema: None,
            subject: None,
            time: Some(Utc::now()),
            data: None,
        }
    }
}

impl Event {
    pub fn new(source: impl Into<String>, type_: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            type_: type_.into(),
            ..Default::default()
        }
    }

    pub fn with_data(mut self, data: impl Serialize) -> Result<Self, serde_json::Error> {
        self.data = Some(serde_json::to_value(data)?);
        Ok(self)
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let event = Event::new("nostra://source", "nostra.test.event")
            .with_data(serde_json::json!({"foo": "bar"}))
            .unwrap();

        let json = serde_json::to_string(&event).unwrap();
        println!("JSON: {}", json);

        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.source, deserialized.source);
        assert_eq!(event.type_, deserialized.type_);
        assert_eq!(event.data.unwrap()["foo"], "bar");
    }
}
