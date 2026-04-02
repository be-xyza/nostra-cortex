use crate::parser_contract::{
    ParserAdapterOutputV1, ParserAdapterPayloadV1, build_local_parser_output,
    sanitize_flag_component,
};
use anyhow::{Context, Result, anyhow, bail};
use std::io::{Read, Write};
use std::process::{Command, Stdio};

pub fn main_for_backend(backend: &'static str) -> Result<()> {
    let mut input = Vec::new();
    std::io::stdin()
        .read_to_end(&mut input)
        .context("failed to read adapter stdin payload")?;
    let payload: ParserAdapterPayloadV1 =
        serde_json::from_slice(&input).context("failed to parse adapter stdin payload")?;
    let output = match run_upstream_command(backend, &payload) {
        Ok(mut output) => {
            output
                .parser_backend
                .get_or_insert_with(|| backend.to_string());
            if output
                .parser_profile
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                output.parser_profile = payload.parser_profile.clone();
            }
            if output
                .model_id
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                output.model_id = Some(format!("{backend}:bridge-upstream-v1"));
            }
            output.flags.push(format!(
                "parser_bridge_upstream:{}",
                sanitize_flag_component(backend)
            ));
            output
        }
        Err(err) => build_local_output(
            backend,
            &payload,
            vec![
                bridge_mode_flag(backend, &err),
                format!(
                    "parser_bridge_error:{}:{}",
                    sanitize_flag_component(backend),
                    sanitize_flag_component(&err.to_string())
                ),
            ],
        ),
    };

    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &output).context("failed to write adapter stdout")?;
    stdout
        .write_all(b"\n")
        .context("failed to terminate adapter output")?;
    Ok(())
}

fn run_upstream_command(
    backend: &'static str,
    payload: &ParserAdapterPayloadV1,
) -> Result<ParserAdapterOutputV1> {
    let command_spec = std::env::var(upstream_command_env_key(backend))
        .with_context(|| format!("missing {}", upstream_command_env_key(backend)))?;
    let command = parse_command_spec(&command_spec)?;
    let executable = command
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("upstream command for {backend} is empty"))?;
    let payload_json =
        serde_json::to_vec(payload).context("failed to serialize adapter payload for upstream")?;
    let mut child = Command::new(&executable)
        .args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn upstream parser command `{executable}`"))?;
    child
        .stdin
        .as_mut()
        .context("upstream parser stdin unavailable")?
        .write_all(&payload_json)
        .context("failed to write payload to upstream parser")?;
    let _ = child.stdin.take();
    let output = child
        .wait_with_output()
        .context("failed waiting for upstream parser")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let suffix = if stderr.is_empty() {
            String::new()
        } else {
            format!(": {stderr}")
        };
        bail!(
            "upstream parser command exited with status {}{}",
            output.status,
            suffix
        );
    }
    let parsed: ParserAdapterOutputV1 =
        serde_json::from_slice(&output.stdout).context("failed to parse upstream parser output")?;
    if parsed.pages.is_empty() {
        bail!("upstream parser output contained no pages");
    }
    Ok(parsed)
}

fn parse_command_spec(spec: &str) -> Result<Vec<String>> {
    let parsed: Vec<String> =
        serde_json::from_str(spec).context("upstream command must be a JSON array of strings")?;
    if parsed.is_empty() {
        bail!("upstream command JSON array must not be empty");
    }
    Ok(parsed)
}

fn upstream_command_env_key(backend: &str) -> String {
    format!(
        "NOSTRA_{}_UPSTREAM_COMMAND_JSON",
        backend.to_ascii_uppercase()
    )
}

fn bridge_mode_flag(backend: &str, err: &anyhow::Error) -> String {
    if err.to_string().contains(&upstream_command_env_key(backend)) {
        format!(
            "parser_bridge_local_only:{}",
            sanitize_flag_component(backend)
        )
    } else {
        format!(
            "parser_bridge_fallback:{}:local_normalizer",
            sanitize_flag_component(backend)
        )
    }
}

fn build_local_output(
    backend: &str,
    payload: &ParserAdapterPayloadV1,
    flags: Vec<String>,
) -> ParserAdapterOutputV1 {
    build_local_parser_output(
        backend,
        payload.parser_profile.as_deref(),
        &payload.resolved_content,
        payload.content_ref.as_deref(),
        flags,
        0.76,
        format!("{backend}:bridge-local-v1"),
    )
}
