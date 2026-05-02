const REDACTED: &str = "[REDACTED]";

pub fn redact_runtime_text(input: &str) -> String {
    let mut out = redact_private_key_block(input);
    out = redact_secret_tokens(&out);
    redact_ssn_like(&out)
}

fn redact_private_key_block(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(start) = remaining.find("-----BEGIN ") {
        output.push_str(&remaining[..start]);
        let after_start = &remaining[start..];
        if let Some(end) = after_start.find("-----END ") {
            let after_end = &after_start[end..];
            if let Some(close) = after_end.find("-----") {
                output.push_str(REDACTED);
                remaining = &after_end[close + 5..];
                continue;
            }
        }
        output.push_str(&remaining[start..]);
        return output;
    }
    output.push_str(remaining);
    output
}

fn redact_secret_tokens(input: &str) -> String {
    sensitive_tokens(input)
        .into_iter()
        .fold(input.to_string(), |acc, token| {
            acc.replace(&token, REDACTED)
        })
}

fn sensitive_tokens(input: &str) -> Vec<String> {
    let specs = [
        ("sk-or-v1-", 32usize),
        ("sk-ant-", 32usize),
        ("sk-proj-", 40usize),
        ("github_pat_", 40usize),
        ("ghp_", 34usize),
        ("gho_", 34usize),
        ("ghu_", 34usize),
        ("ghs_", 34usize),
        ("ghr_", 34usize),
        ("sk-", 36usize),
    ];
    let mut tokens = Vec::new();
    for (prefix, min_len) in specs {
        let mut offset = 0;
        while let Some(relative_start) = input[offset..].find(prefix) {
            let start = offset + relative_start;
            let end = input[start..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')))
                .map(|relative_end| start + relative_end)
                .unwrap_or(input.len());
            let token = &input[start..end];
            if token.len() >= min_len {
                tokens.push(token.to_string());
            }
            offset = end;
        }
    }
    tokens
}

fn redact_ssn_like(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    while index < bytes.len() {
        if index + 11 <= bytes.len()
            && bytes[index + 3] == b'-'
            && bytes[index + 6] == b'-'
            && bytes[index..index + 11]
                .iter()
                .enumerate()
                .all(|(idx, byte)| idx == 3 || idx == 6 || byte.is_ascii_digit())
        {
            output.push_str(REDACTED);
            index += 11;
        } else {
            output.push(bytes[index] as char);
            index += 1;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_provider_keys_bearer_values_private_keys_and_pii() {
        let raw = concat!(
            "fake Authorization: Bearer sk-or-v1-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n",
            "fake error key=sk-proj-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB\n",
            "fake pii=123-45-6789\n",
            "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----"
        );

        let redacted = redact_runtime_text(raw);

        assert!(!redacted.contains("sk-or-v1-"));
        assert!(!redacted.contains("sk-proj-"));
        assert!(!redacted.contains("123-45-6789"));
        assert!(!redacted.contains("BEGIN PRIVATE KEY"));
        assert!(redacted.contains(REDACTED));
    }

    #[test]
    fn leaves_safe_operational_metadata_intact() {
        let raw = "model=~moonshotai/kimi-latest max_tokens=8192 cost_per_1k_tokens=0.002";
        assert_eq!(redact_runtime_text(raw), raw);
    }
}
