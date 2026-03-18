pub use cortex_runtime::policy::adapter::{
    AcpPolicyConfig, AcpPolicyError, AcpSessionUpdateKind, EnvVariable, FsReadTextFileRequest,
    FsReadTextFileResponse, FsWriteTextFileRequest, NostraProjectionKind, OperationAdapter,
    TerminalCreateRequest, ValidatedTerminalCreate, map_session_update_to_projection,
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AcpAdapter {
    cfg: AcpPolicyConfig,
    canonical_roots: Vec<PathBuf>,
}

impl AcpAdapter {
    pub fn new(cfg: AcpPolicyConfig) -> Result<Self, AcpPolicyError> {
        if cfg.allowed_roots.is_empty() {
            return Err(AcpPolicyError::NoAllowedRootsConfigured);
        }

        let canonical_roots = cfg
            .allowed_roots
            .iter()
            .map(|root| canonicalize_or_absolute(root))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            cfg,
            canonical_roots,
        })
    }

    pub fn read_text_file(
        &self,
        req: FsReadTextFileRequest,
    ) -> Result<FsReadTextFileResponse, AcpPolicyError> {
        let canonical = self.validate_existing_path(&req.path)?;
        let content =
            fs::read_to_string(&canonical).map_err(|e| AcpPolicyError::Io(e.to_string()))?;

        let line = req.line.unwrap_or(1);
        if line == 0 {
            return Err(AcpPolicyError::InvalidLineNumber(line));
        }

        let max_limit = self.cfg.max_read_lines.max(1);
        let limit = req.limit.unwrap_or(max_limit);
        if limit == 0 {
            return Err(AcpPolicyError::InvalidLimit(limit));
        }
        let limit = limit.min(max_limit);

        let start = line.saturating_sub(1);
        let lines: Vec<&str> = content.lines().collect();
        if start >= lines.len() {
            return Ok(FsReadTextFileResponse {
                content: String::new(),
            });
        }

        let slice = lines
            .iter()
            .skip(start)
            .take(limit)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");

        Ok(FsReadTextFileResponse { content: slice })
    }

    pub fn write_text_file(&self, req: FsWriteTextFileRequest) -> Result<(), AcpPolicyError> {
        let target = self.validate_write_path(&req.path)?;
        fs::write(target, req.content).map_err(|e| AcpPolicyError::Io(e.to_string()))
    }

    pub fn validate_terminal_create(
        &self,
        req: TerminalCreateRequest,
    ) -> Result<ValidatedTerminalCreate, AcpPolicyError> {
        let command = req.command.trim();
        if command.is_empty() {
            return Err(AcpPolicyError::EmptyCommand);
        }

        let command_key = command_basename(command).to_ascii_lowercase();
        if !self.cfg.allowed_terminal_commands.is_empty()
            && !self.cfg.allowed_terminal_commands.contains(&command_key)
        {
            return Err(AcpPolicyError::CommandNotAllowed(command_key));
        }

        let cwd = match req.cwd {
            Some(cwd) => self.validate_existing_path(&cwd)?,
            None => self.canonical_roots[0].clone(),
        };

        for entry in &req.env {
            if !self.cfg.allowed_env_vars.contains(&entry.name) {
                return Err(AcpPolicyError::EnvVarNotAllowed(entry.name.clone()));
            }
        }

        let output_byte_limit = req
            .output_byte_limit
            .unwrap_or(self.cfg.max_output_byte_limit)
            .min(self.cfg.max_output_byte_limit);

        if let Some(requested) = req.output_byte_limit {
            if requested > self.cfg.max_output_byte_limit {
                return Err(AcpPolicyError::OutputLimitExceeded {
                    requested,
                    max: self.cfg.max_output_byte_limit,
                });
            }
        }

        Ok(ValidatedTerminalCreate {
            command: command.to_string(),
            args: req.args,
            env: req.env,
            cwd,
            output_byte_limit,
            max_wait_ms: self.cfg.max_terminal_wait_ms,
            max_runtime_ms: self.cfg.max_terminal_runtime_ms,
        })
    }

    pub fn validate_workspace_path(&self, path: &str) -> Result<PathBuf, AcpPolicyError> {
        self.validate_existing_path(path)
    }

    fn validate_existing_path(&self, path: &str) -> Result<PathBuf, AcpPolicyError> {
        let raw = PathBuf::from(path);
        if !raw.is_absolute() {
            return Err(AcpPolicyError::PathNotAbsolute(path.to_string()));
        }
        let canonical = canonicalize_or_absolute(&raw)?;
        self.ensure_within_roots(&canonical)?;
        Ok(canonical)
    }

    fn validate_write_path(&self, path: &str) -> Result<PathBuf, AcpPolicyError> {
        let raw = PathBuf::from(path);
        if !raw.is_absolute() {
            return Err(AcpPolicyError::PathNotAbsolute(path.to_string()));
        }

        let resolved = if raw.exists() {
            canonicalize_or_absolute(&raw)?
        } else {
            let parent = raw
                .parent()
                .ok_or_else(|| AcpPolicyError::PathOutsideAllowedRoots(path.to_string()))?;
            let canonical_parent = canonicalize_or_absolute(parent)?;
            let file_name = raw
                .file_name()
                .ok_or_else(|| AcpPolicyError::PathOutsideAllowedRoots(path.to_string()))?;
            canonical_parent.join(file_name)
        };

        self.ensure_within_roots(&resolved)?;
        Ok(resolved)
    }

    fn ensure_within_roots(&self, candidate: &Path) -> Result<(), AcpPolicyError> {
        if self
            .canonical_roots
            .iter()
            .any(|root| candidate.starts_with(root))
        {
            Ok(())
        } else {
            Err(AcpPolicyError::PathOutsideAllowedRoots(
                candidate.display().to_string(),
            ))
        }
    }
}

impl OperationAdapter for AcpAdapter {
    fn validate_workspace_path(&self, path: &str) -> Result<PathBuf, AcpPolicyError> {
        self.validate_workspace_path(path)
    }

    fn validate_terminal_create(
        &self,
        req: TerminalCreateRequest,
    ) -> Result<ValidatedTerminalCreate, AcpPolicyError> {
        self.validate_terminal_create(req)
    }
}

fn canonicalize_or_absolute(path: &Path) -> Result<PathBuf, AcpPolicyError> {
    if path.exists() {
        fs::canonicalize(path).map_err(|e| AcpPolicyError::Io(e.to_string()))
    } else if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Err(AcpPolicyError::PathNotAbsolute(path.display().to_string()))
    }
}

fn command_basename(command: &str) -> String {
    Path::new(command)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(command)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("acp_adapter_{}_{}", name, nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn baseline_adapter(root: PathBuf) -> AcpAdapter {
        let mut cfg = AcpPolicyConfig::baseline(vec![root]);
        cfg.allowed_terminal_commands =
            HashSet::from_iter(["cargo".to_string(), "git".to_string()]);
        cfg.allowed_env_vars = HashSet::from_iter(["RUST_LOG".to_string()]);
        AcpAdapter::new(cfg).unwrap()
    }

    #[test]
    fn fs_read_allows_absolute_path_within_root() {
        let root = unique_temp_dir("read_inside");
        let file = root.join("hello.txt");
        fs::write(&file, "a\nb\nc\n").unwrap();
        let adapter = baseline_adapter(root);

        let res = adapter
            .read_text_file(FsReadTextFileRequest {
                session_id: "sess_1".to_string(),
                path: file.display().to_string(),
                line: Some(2),
                limit: Some(2),
            })
            .unwrap();

        assert_eq!(res.content, "b\nc");
    }

    #[test]
    fn fs_read_rejects_path_outside_root() {
        let root = unique_temp_dir("read_outside_root");
        let outside_dir = unique_temp_dir("outside");
        let outside_file = outside_dir.join("outside.txt");
        fs::write(&outside_file, "nope").unwrap();
        let adapter = baseline_adapter(root);

        let err = adapter
            .read_text_file(FsReadTextFileRequest {
                session_id: "sess_1".to_string(),
                path: outside_file.display().to_string(),
                line: None,
                limit: None,
            })
            .unwrap_err();

        assert!(matches!(err, AcpPolicyError::PathOutsideAllowedRoots(_)));
    }

    #[test]
    fn fs_write_rejects_non_absolute_path() {
        let root = unique_temp_dir("write_non_abs");
        let adapter = baseline_adapter(root);

        let err = adapter
            .write_text_file(FsWriteTextFileRequest {
                session_id: "sess_1".to_string(),
                path: "relative.txt".to_string(),
                content: "x".to_string(),
            })
            .unwrap_err();

        assert!(matches!(err, AcpPolicyError::PathNotAbsolute(_)));
    }

    #[test]
    fn terminal_create_rejects_disallowed_command() {
        let root = unique_temp_dir("term_cmd_deny");
        let adapter = baseline_adapter(root);

        let err = adapter
            .validate_terminal_create(TerminalCreateRequest {
                session_id: "sess_1".to_string(),
                command: "rm".to_string(),
                args: vec!["-rf".to_string(), "/".to_string()],
                env: vec![],
                cwd: None,
                output_byte_limit: Some(2048),
            })
            .unwrap_err();

        assert!(matches!(err, AcpPolicyError::CommandNotAllowed(_)));
    }

    #[test]
    fn terminal_create_accepts_allowed_command_with_valid_env() {
        let root = unique_temp_dir("term_ok");
        let adapter = baseline_adapter(root.clone());

        let res = adapter
            .validate_terminal_create(TerminalCreateRequest {
                session_id: "sess_1".to_string(),
                command: "cargo".to_string(),
                args: vec!["check".to_string()],
                env: vec![EnvVariable {
                    name: "RUST_LOG".to_string(),
                    value: "info".to_string(),
                }],
                cwd: Some(root.display().to_string()),
                output_byte_limit: Some(8_192),
            })
            .unwrap();

        assert_eq!(res.command, "cargo");
        assert_eq!(res.args, vec!["check".to_string()]);
        assert_eq!(res.output_byte_limit, 8_192);
    }

    #[test]
    fn terminal_create_rejects_env_var_outside_allowlist() {
        let root = unique_temp_dir("term_env_deny");
        let adapter = baseline_adapter(root.clone());

        let err = adapter
            .validate_terminal_create(TerminalCreateRequest {
                session_id: "sess_1".to_string(),
                command: "git".to_string(),
                args: vec!["status".to_string()],
                env: vec![EnvVariable {
                    name: "AWS_SECRET_ACCESS_KEY".to_string(),
                    value: "x".to_string(),
                }],
                cwd: Some(root.display().to_string()),
                output_byte_limit: None,
            })
            .unwrap_err();

        assert!(matches!(err, AcpPolicyError::EnvVarNotAllowed(_)));
    }

    #[test]
    fn session_update_projection_mapping_is_stable() {
        assert_eq!(
            map_session_update_to_projection(AcpSessionUpdateKind::ToolCallUpdate),
            NostraProjectionKind::ToolCallProgress
        );
    }
}
