use anyhow::{Context, Result, anyhow};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::{
    ExtractionRequestV1, ExtractionResultV1, NormalizedDocumentV1, PipelineAdapter,
    parser_contract::{
        ParserAdapterOutputV1, ParserAdapterPayloadV1, build_local_parser_output,
        sanitize_flag_component,
    },
    run_local_pipeline,
};

#[derive(Default)]
pub struct LocalPipelineAdapter;

impl PipelineAdapter for LocalPipelineAdapter {
    fn adapter_id(&self) -> &'static str {
        "local_pipeline"
    }

    fn run(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1> {
        run_local_pipeline(request)
    }
}

pub trait ParserAdapter: Send + Sync {
    fn adapter_id(&self) -> &'static str;
    fn run(
        &self,
        request: &ExtractionRequestV1,
        resolved_content: &str,
    ) -> Result<ParserAdapterExecution>;
}

#[derive(Clone, Debug)]
pub struct ParserAdapterExecution {
    pub normalized_document: NormalizedDocumentV1,
    pub parser_hint: String,
    pub model_id: String,
    pub flags: Vec<String>,
}

pub struct CommandParserAdapter {
    spec: &'static ParserBackendSpec,
}

impl CommandParserAdapter {
    fn new(spec: &'static ParserBackendSpec) -> Self {
        Self { spec }
    }

    fn executable(&self) -> Result<String> {
        let executable = std::env::var(self.spec.executable_env)
            .with_context(|| format!("missing {}", self.spec.executable_env))?;
        if executable.trim().is_empty() {
            return Err(anyhow!("{} is empty", self.spec.executable_env));
        }
        Ok(executable)
    }

    fn timeout_for(&self, request: &ExtractionRequestV1) -> Duration {
        Duration::from_secs(request.timeout_seconds.unwrap_or(90).clamp(5, 300))
    }
}

impl ParserAdapter for CommandParserAdapter {
    fn adapter_id(&self) -> &'static str {
        self.spec.id
    }

    fn run(
        &self,
        request: &ExtractionRequestV1,
        resolved_content: &str,
    ) -> Result<ParserAdapterExecution> {
        let executable = self.executable()?;
        let payload = ParserAdapterPayloadV1 {
            job_id: request.job_id.clone(),
            source_ref: request.source_ref.clone(),
            source_type: request.source_type.clone(),
            content_ref: request.content_ref.clone(),
            artifact_path: request.artifact_path.clone(),
            mime_type: request.mime_type.clone(),
            file_size: request.file_size,
            parser_profile: request.parser_profile.clone(),
            resolved_content: resolved_content.to_string(),
        };
        let payload_json = serde_json::to_vec(&payload)
            .with_context(|| format!("failed to serialize {} payload", self.adapter_id()))?;
        let mut child = Command::new(executable.trim())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to spawn {} adapter", self.adapter_id()))?;
        child
            .stdin
            .as_mut()
            .context("parser adapter stdin unavailable")?
            .write_all(&payload_json)
            .with_context(|| format!("failed to write {} payload", self.adapter_id()))?;
        let _ = child.stdin.take();
        let timeout = self.timeout_for(request);
        let (status, stdout, stderr) = wait_for_child_output_with_timeout(child, timeout)
            .with_context(|| {
                format!(
                    "failed to wait for {} adapter within {}s",
                    self.adapter_id(),
                    timeout.as_secs()
                )
            })?;
        if !status.success() {
            let stderr = String::from_utf8_lossy(&stderr).trim().to_string();
            let stderr_suffix = if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            };
            return Err(anyhow!(
                "{} adapter exited with status {}{}",
                self.adapter_id(),
                status,
                stderr_suffix
            ));
        }

        let parsed: ParserAdapterOutputV1 = serde_json::from_slice(&stdout)
            .with_context(|| format!("failed to decode {} adapter response", self.adapter_id()))?;
        if parsed.pages.is_empty() {
            return Err(anyhow!("{} adapter returned no pages", self.adapter_id()));
        }

        let parser_backend = parsed
            .parser_backend
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| self.adapter_id().to_string());
        Ok(ParserAdapterExecution {
            parser_hint: parser_hint_for_backend(&parser_backend).to_string(),
            model_id: parsed
                .model_id
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| format!("{}:command-adapter-v1", self.adapter_id())),
            flags: parsed.flags,
            normalized_document: NormalizedDocumentV1 {
                parser_backend,
                parser_profile: parsed
                    .parser_profile
                    .or_else(|| request.parser_profile.clone()),
                pages: parsed.pages,
            },
        })
    }
}

fn wait_for_child_output_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<(std::process::ExitStatus, Vec<u8>, Vec<u8>)> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().context("failed to poll child process")? {
            let stdout = read_pipe_to_end(child.stdout.take())?;
            let stderr = read_pipe_to_end(child.stderr.take())?;
            return Ok((status, stdout, stderr));
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow!(
                "parser adapter timed out after {}s",
                timeout.as_secs()
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn read_pipe_to_end<R: Read>(pipe: Option<R>) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    if let Some(mut pipe) = pipe {
        pipe.read_to_end(&mut buffer)
            .context("failed to read child output")?;
    }
    Ok(buffer)
}

struct LocalHeuristicParserAdapter {
    backend_id: String,
}

struct ParserBackendSpec {
    id: &'static str,
    executable_env: &'static str,
    parser_hint: &'static str,
}

const PARSER_BACKEND_SPECS: &[ParserBackendSpec] = &[
    ParserBackendSpec {
        id: "docling",
        executable_env: "NOSTRA_DOCLING_ADAPTER_EXECUTABLE",
        parser_hint: "docling+ocrmypdf",
    },
    ParserBackendSpec {
        id: "liteparse",
        executable_env: "NOSTRA_LITEPARSE_ADAPTER_EXECUTABLE",
        parser_hint: "liteparse+bbox",
    },
    ParserBackendSpec {
        id: "markitdown",
        executable_env: "NOSTRA_MARKITDOWN_ADAPTER_EXECUTABLE",
        parser_hint: "markitdown+text",
    },
];

impl LocalHeuristicParserAdapter {
    fn new(backend_id: impl Into<String>) -> Self {
        Self {
            backend_id: backend_id.into(),
        }
    }
}

impl ParserAdapter for LocalHeuristicParserAdapter {
    fn adapter_id(&self) -> &'static str {
        "local_heuristic"
    }

    fn run(
        &self,
        request: &ExtractionRequestV1,
        resolved_content: &str,
    ) -> Result<ParserAdapterExecution> {
        Ok(build_local_parser_execution(
            request,
            resolved_content,
            self.backend_id.as_str(),
            Vec::new(),
        ))
    }
}

pub fn run_parser_adapter(
    request: &ExtractionRequestV1,
    resolved_content: &str,
) -> Result<ParserAdapterExecution> {
    let preferred_backend = resolve_parser_backend(request);
    let local_adapter = LocalHeuristicParserAdapter::new(preferred_backend.clone());
    let mut local_flags = Vec::new();

    if let Some(command_adapter) = command_adapter_for_backend(&preferred_backend) {
        match command_adapter.executable() {
            Ok(_) => match command_adapter.run(request, resolved_content) {
                Ok(mut execution) => {
                    execution.flags.push(format!(
                        "parser_adapter_command:{}",
                        sanitize_flag_component(&preferred_backend)
                    ));
                    return Ok(execution);
                }
                Err(err) => {
                    local_flags.push(format!(
                        "parser_adapter_fallback:{}:local_heuristic",
                        sanitize_flag_component(&preferred_backend)
                    ));
                    local_flags.push(format!(
                        "parser_adapter_error:{}:{}",
                        sanitize_flag_component(&preferred_backend),
                        sanitize_flag_component(&err.to_string())
                    ));
                }
            },
            Err(_) => {
                local_flags.push(format!(
                    "parser_adapter_unconfigured:{}",
                    sanitize_flag_component(&preferred_backend)
                ));
            }
        }
    } else {
        local_flags.push(format!(
            "parser_adapter_unsupported:{}",
            sanitize_flag_component(&preferred_backend)
        ));
    }

    let mut execution = local_adapter.run(request, resolved_content)?;
    execution.flags.extend(local_flags);
    Ok(execution)
}

pub fn resolve_parser_backend(request: &ExtractionRequestV1) -> String {
    request
        .parser_profile
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| {
            let source_type = request.source_type.to_lowercase();
            let mime_type = request.mime_type.clone().unwrap_or_default().to_lowercase();
            default_parser_backend(&source_type, &mime_type)
        })
}

pub fn parser_hint_for_backend(parser_backend: &str) -> &'static str {
    parser_backend_spec(parser_backend)
        .map(|spec| spec.parser_hint)
        .unwrap_or("local-normalized")
}

fn build_local_parser_execution(
    request: &ExtractionRequestV1,
    resolved_content: &str,
    parser_backend: &str,
    flags: Vec<String>,
) -> ParserAdapterExecution {
    let output = build_local_parser_output(
        parser_backend,
        request.parser_profile.as_deref(),
        resolved_content,
        request.content_ref.as_deref(),
        flags,
        0.78,
        format!("{parser_backend}:local-simulated-v1"),
    );

    ParserAdapterExecution {
        parser_hint: parser_hint_for_backend(parser_backend).to_string(),
        model_id: output
            .model_id
            .clone()
            .unwrap_or_else(|| format!("{parser_backend}:local-simulated-v1")),
        flags: output.flags.clone(),
        normalized_document: NormalizedDocumentV1 {
            parser_backend: parser_backend.to_string(),
            parser_profile: output.parser_profile,
            pages: output.pages,
        },
    }
}

fn command_adapter_for_backend(parser_backend: &str) -> Option<CommandParserAdapter> {
    parser_backend_spec(parser_backend).map(CommandParserAdapter::new)
}

fn default_parser_backend(source_type: &str, mime_type: &str) -> String {
    if source_type.contains("pdf") || mime_type.contains("pdf") {
        "docling".to_string()
    } else if mime_type.starts_with("image/") {
        "liteparse".to_string()
    } else if source_type.contains("markdown") || source_type.contains("md") {
        "markitdown".to_string()
    } else {
        "docling".to_string()
    }
}

fn parser_backend_spec(parser_backend: &str) -> Option<&'static ParserBackendSpec> {
    PARSER_BACKEND_SPECS
        .iter()
        .find(|spec| spec.id == parser_backend)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn adapter_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn make_request(parser_profile: Option<&str>) -> ExtractionRequestV1 {
        ExtractionRequestV1 {
            job_id: Some("job-parser-test".to_string()),
            source_ref: "urn:test:parser".to_string(),
            source_type: "application/pdf".to_string(),
            schema_ref: None,
            space_id: Some("space:test".to_string()),
            content: String::new(),
            content_ref: Some("cortex://upload?id=parser-test".to_string()),
            artifact_path: Some("/tmp/parser-source.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            file_size: Some(128),
            parser_profile: parser_profile.map(str::to_string),
            extraction_mode: crate::ExtractionMode::Local,
            fallback_policy: Default::default(),
            timeout_seconds: Some(30),
            index_to_knowledge: false,
            idempotency_key: Some("idem-parser-test".to_string()),
            provenance_hint: None,
        }
    }

    #[test]
    fn unconfigured_parser_backend_falls_back_to_local_normalization() {
        let _guard = adapter_test_lock().lock().expect("adapter test lock");
        std::env::remove_var("NOSTRA_DOCLING_ADAPTER_EXECUTABLE");
        let request = make_request(Some("docling"));
        let execution =
            run_parser_adapter(&request, "Alpha page\x0cBeta page").expect("adapter result");
        assert_eq!(execution.normalized_document.parser_backend, "docling");
        assert_eq!(execution.normalized_document.pages.len(), 2);
        assert!(
            execution
                .flags
                .iter()
                .any(|flag| flag == "parser_adapter_unconfigured:docling")
        );
    }

    #[test]
    fn command_adapter_can_return_normalized_pages() {
        let _guard = adapter_test_lock().lock().expect("adapter test lock");
        let script_path = write_fake_parser_script(
            "docling-success",
            r#"#!/bin/sh
payload="$(cat)"
case "$payload" in
  *'"artifact_path":"/tmp/parser-source.pdf"'*) ;;
  *)
    printf '%s' 'missing artifact_path' >&2
    exit 9
    ;;
esac
printf '%s' '{"parser_backend":"docling","parser_profile":"docling","pages":[{"page_number":1,"page_image_ref":"cortex://upload?id=parser-test","blocks":[{"block_type":"text","text":"adapter-body","reading_order":1,"confidence":0.91}]}],"model_id":"docling:test-adapter","flags":["adapter_test_ok"]}'
"#,
        );
        std::env::set_var("NOSTRA_DOCLING_ADAPTER_EXECUTABLE", &script_path);
        let request = make_request(Some("docling"));
        let execution =
            run_parser_adapter(&request, "Nostra Cortex architecture").expect("adapter result");
        assert_eq!(execution.model_id, "docling:test-adapter");
        assert_eq!(execution.normalized_document.pages.len(), 1);
        assert_eq!(
            execution.normalized_document.pages[0].blocks[0].text,
            "adapter-body"
        );
        assert!(execution.flags.iter().any(|flag| flag == "adapter_test_ok"));
        assert!(
            execution
                .flags
                .iter()
                .any(|flag| flag == "parser_adapter_command:docling")
        );
        std::env::remove_var("NOSTRA_DOCLING_ADAPTER_EXECUTABLE");
        let _ = fs::remove_file(script_path);
    }

    #[test]
    fn failed_command_adapter_falls_back_to_local_normalization() {
        let _guard = adapter_test_lock().lock().expect("adapter test lock");
        let script_path = write_fake_parser_script(
            "docling-fail",
            r#"#!/bin/sh
printf '%s' 'boom' >&2
exit 7
"#,
        );
        std::env::set_var("NOSTRA_DOCLING_ADAPTER_EXECUTABLE", &script_path);
        let request = make_request(Some("docling"));
        let execution = run_parser_adapter(&request, "Fallback body text").expect("adapter result");
        assert_eq!(execution.normalized_document.parser_backend, "docling");
        assert!(
            execution
                .flags
                .iter()
                .any(|flag| flag == "parser_adapter_fallback:docling:local_heuristic")
        );
        assert!(
            execution
                .flags
                .iter()
                .any(|flag| flag.starts_with("parser_adapter_error:docling:"))
        );
        std::env::remove_var("NOSTRA_DOCLING_ADAPTER_EXECUTABLE");
        let _ = fs::remove_file(script_path);
    }

    fn write_fake_parser_script(name: &str, body: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("{name}-{unique}.sh"));
        fs::write(&path, body).expect("write fake parser script");
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&path).expect("script metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).expect("set executable bit");
        }
        path
    }
}
