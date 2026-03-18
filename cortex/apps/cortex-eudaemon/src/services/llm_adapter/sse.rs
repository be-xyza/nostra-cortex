use futures_util::StreamExt;
use reqwest::Response;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseDataFrame {
    pub data: String,
}

fn extract_frames_from_lines(lines: &[String]) -> Vec<SseDataFrame> {
    let mut frames = Vec::new();
    let mut current_data: Vec<String> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            if !current_data.is_empty() {
                frames.push(SseDataFrame {
                    data: current_data.join("\n"),
                });
                current_data.clear();
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("data:") {
            current_data.push(rest.trim_start().to_string());
        }
    }

    if !current_data.is_empty() {
        frames.push(SseDataFrame {
            data: current_data.join("\n"),
        });
    }

    frames
}

pub async fn decode_sse_response(response: Response) -> Result<Vec<SseDataFrame>, String> {
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();

    if !content_type.contains("text/event-stream") {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "expected_sse_stream content_type='{}' status={} body={}",
            content_type, status, body
        ));
    }

    let mut buffer = String::new();
    let mut lines: Vec<String> = Vec::new();

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| format!("sse_read_failed:{err}"))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        while let Some(index) = buffer.find('\n') {
            let mut line = buffer[..index].to_string();
            if line.ends_with('\r') {
                line.pop();
            }
            lines.push(line);
            buffer = buffer[index + 1..].to_string();
        }
    }

    Ok(extract_frames_from_lines(&lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_frames_from_data_lines() {
        let lines = vec![
            "data: {\"type\":\"response.created\"}".to_string(),
            "".to_string(),
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"hi\"}".to_string(),
            "".to_string(),
            "data: [DONE]".to_string(),
            "".to_string(),
        ];

        let frames = extract_frames_from_lines(&lines);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].data, "{\"type\":\"response.created\"}");
        assert_eq!(
            frames[1].data,
            "{\"type\":\"response.output_text.delta\",\"delta\":\"hi\"}"
        );
        assert_eq!(frames[2].data, "[DONE]");
    }
}
