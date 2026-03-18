use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const GITHUB_MCP_NAME: &str = "github";
const GITHUB_MCP_URL: &str = "https://api.githubcopilot.com/mcp/";
const GITHUB_PAT_ENV_VAR: &str = "GITHUB_PAT_TOKEN";
const KEYCHAIN_SERVICE: &str = "cortex-desktop/github-mcp";

const CODEX_CANDIDATES: &[&str] = &["codex", "/Applications/Codex.app/Contents/Resources/codex"];
const GH_CANDIDATES: &[&str] = &["gh", "/opt/homebrew/bin/gh", "/usr/local/bin/gh"];
const CORTEX_CANDIDATES: &[&str] = &[
    "cortex",
    "cortex-cli",
    "/opt/homebrew/bin/cortex",
    "/usr/local/bin/cortex",
];

const CORTEX_PAT_COMMANDS: &[&[&str]] = &[
    &["auth", "token", "github", "--raw"],
    &["auth", "github", "token", "--raw"],
    &["github", "token", "--raw"],
    &["auth", "token", "github"],
    &["github", "token"],
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GithubMcpStatus {
    pub codex_cli_path: Option<String>,
    pub gh_cli_path: Option<String>,
    pub cortex_cli_path: Option<String>,
    pub github_pat_in_env: bool,
    pub github_pat_in_keychain: bool,
    pub github_pat_in_shell_profile: bool,
    pub codex_config_exists: bool,
    pub codex_github_mcp_configured: bool,
    pub codex_github_mcp_url_matches: bool,
    pub codex_github_mcp_uses_env_var: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatSource {
    Environment,
    Keychain,
    CortexCli,
    GithubCli,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupOutcome {
    pub status: GithubMcpStatus,
    pub pat_source: PatSource,
    pub details: String,
}

pub struct GithubMcpService;

impl GithubMcpService {
    pub async fn status() -> Result<GithubMcpStatus, String> {
        tokio::task::spawn_blocking(Self::status_sync)
            .await
            .map_err(|e| format!("Failed to join status task: {}", e))?
    }

    pub async fn install_cortex_cli_if_missing() -> Result<String, String> {
        tokio::task::spawn_blocking(Self::install_cortex_cli_if_missing_sync)
            .await
            .map_err(|e| format!("Failed to join install task: {}", e))?
    }

    pub async fn setup_with_discovered_token() -> Result<SetupOutcome, String> {
        tokio::task::spawn_blocking(|| Self::setup_with_discovered_token_sync())
            .await
            .map_err(|e| format!("Failed to join setup task: {}", e))?
    }

    pub async fn setup_with_manual_pat(pat: String) -> Result<SetupOutcome, String> {
        tokio::task::spawn_blocking(move || Self::setup_with_manual_pat_sync(pat))
            .await
            .map_err(|e| format!("Failed to join manual setup task: {}", e))?
    }

    fn status_sync() -> Result<GithubMcpStatus, String> {
        let codex_cli_path = resolve_cli(CODEX_CANDIDATES);
        let gh_cli_path = resolve_cli(GH_CANDIDATES);
        let cortex_cli_path = resolve_cli(CORTEX_CANDIDATES);

        let profile_path = shell_profile_path();
        let github_pat_in_env = pat_in_env().is_some();
        let github_pat_in_keychain = pat_from_keychain().ok().is_some();
        let github_pat_in_shell_profile =
            profile_contains_export(&profile_path, GITHUB_PAT_ENV_VAR)?;

        let codex_config_path = codex_config_path();
        let codex_config_exists = codex_config_path.exists();

        let mut codex_github_mcp_configured = false;
        let mut codex_github_mcp_url_matches = false;
        let mut codex_github_mcp_uses_env_var = false;

        if codex_config_exists {
            let config = fs::read_to_string(&codex_config_path)
                .map_err(|e| format!("Failed to read {}: {}", codex_config_path.display(), e))?;
            codex_github_mcp_configured = config.contains("[mcp_servers.github]");
            codex_github_mcp_url_matches = config.contains(GITHUB_MCP_URL);
            codex_github_mcp_uses_env_var = config.contains(&format!(
                "bearer_token_env_var = \"{}\"",
                GITHUB_PAT_ENV_VAR
            ));
        }

        Ok(GithubMcpStatus {
            codex_cli_path,
            gh_cli_path,
            cortex_cli_path,
            github_pat_in_env,
            github_pat_in_keychain,
            github_pat_in_shell_profile,
            codex_config_exists,
            codex_github_mcp_configured,
            codex_github_mcp_url_matches,
            codex_github_mcp_uses_env_var,
        })
    }

    fn setup_with_discovered_token_sync() -> Result<SetupOutcome, String> {
        let (pat, source) = if let Some(token) = pat_in_env() {
            (token, PatSource::Environment)
        } else if let Ok(token) = pat_from_keychain() {
            (token, PatSource::Keychain)
        } else if let Some(cortex_cli) = resolve_cli(CORTEX_CANDIDATES) {
            match pat_from_cortex_cli(&cortex_cli) {
                Ok(token) => (token, PatSource::CortexCli),
                Err(_) => {
                    let gh_cli = resolve_cli(GH_CANDIDATES).ok_or_else(|| {
                        "Unable to obtain PAT from Cortex CLI and gh CLI is not installed."
                            .to_string()
                    })?;
                    (pat_from_gh_cli(&gh_cli)?, PatSource::GithubCli)
                }
            }
        } else {
            let gh_cli = resolve_cli(GH_CANDIDATES)
                .ok_or_else(|| "No PAT source available. Install Cortex CLI or GitHub CLI, or enter a manual token.".to_string())?;
            (pat_from_gh_cli(&gh_cli)?, PatSource::GithubCli)
        };

        setup_with_token(pat, source)
    }

    fn setup_with_manual_pat_sync(pat: String) -> Result<SetupOutcome, String> {
        setup_with_token(pat, PatSource::Manual)
    }

    fn install_cortex_cli_if_missing_sync() -> Result<String, String> {
        if resolve_cli(CORTEX_CANDIDATES).is_some() {
            return Ok("Cortex CLI is already installed.".to_string());
        }

        let install_cmd = env::var("CORTEX_CLI_INSTALL_CMD").map_err(|_| {
            "Cortex CLI not found. Set CORTEX_CLI_INSTALL_CMD to a valid installer command and try again."
                .to_string()
        })?;

        let output = Command::new("sh")
            .arg("-lc")
            .arg(&install_cmd)
            .output()
            .map_err(|e| format!("Failed to run install command: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(format!(
                "CORTEX_CLI_INSTALL_CMD failed with status {}: {}",
                output.status, stderr
            ));
        }

        if resolve_cli(CORTEX_CANDIDATES).is_none() {
            return Err(
                "Installer command completed, but Cortex CLI is still not discoverable in PATH."
                    .to_string(),
            );
        }

        Ok("Installed Cortex CLI using CORTEX_CLI_INSTALL_CMD.".to_string())
    }
}

fn setup_with_token(raw_pat: String, source: PatSource) -> Result<SetupOutcome, String> {
    let pat = sanitize_pat(&raw_pat)?;

    let codex_cli = resolve_cli(CODEX_CANDIDATES).ok_or_else(|| {
        "Codex CLI is not installed or not discoverable. Install Codex CLI before configuring MCP."
            .to_string()
    })?;

    // Ensure Codex MCP entry exists (idempotent add/update).
    run_command(
        &codex_cli,
        &[
            "mcp",
            "add",
            GITHUB_MCP_NAME,
            "--url",
            GITHUB_MCP_URL,
            "--bearer-token-env-var",
            GITHUB_PAT_ENV_VAR,
        ],
    )?;

    upsert_pat_in_keychain(&pat)?;
    persist_shell_profile_keychain_hook()?;
    env::set_var(GITHUB_PAT_ENV_VAR, &pat);

    let status = GithubMcpService::status_sync()?;
    Ok(SetupOutcome {
        status,
        pat_source: source,
        details: format!(
            "Configured Codex MCP server '{}', stored {} in keychain, and synced shell profile hook.",
            GITHUB_MCP_NAME, GITHUB_PAT_ENV_VAR
        ),
    })
}

fn sanitize_pat(raw: &str) -> Result<String, String> {
    let token = raw.trim();
    if token.is_empty() {
        return Err("GitHub PAT is empty.".to_string());
    }
    if token.contains(char::is_whitespace) {
        return Err("GitHub PAT contains whitespace.".to_string());
    }
    if token.len() < 20 {
        return Err("GitHub PAT looks too short; verify you pasted the full token.".to_string());
    }
    Ok(token.to_string())
}

fn pat_in_env() -> Option<String> {
    env::var(GITHUB_PAT_ENV_VAR)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn pat_from_keychain() -> Result<String, String> {
    let account = current_user();
    let output = run_command_capture(
        "security",
        &[
            "find-generic-password",
            "-a",
            &account,
            "-s",
            KEYCHAIN_SERVICE,
            "-w",
        ],
    )?;
    sanitize_pat(&output)
}

fn upsert_pat_in_keychain(pat: &str) -> Result<(), String> {
    let account = current_user();
    run_command(
        "security",
        &[
            "add-generic-password",
            "-a",
            &account,
            "-s",
            KEYCHAIN_SERVICE,
            "-w",
            pat,
            "-U",
        ],
    )
}

fn current_user() -> String {
    env::var("USER")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn pat_from_gh_cli(gh_cli: &str) -> Result<String, String> {
    let output = run_command_capture(gh_cli, &["auth", "token"])?;
    sanitize_pat(&output)
}

fn pat_from_cortex_cli(cortex_cli: &str) -> Result<String, String> {
    let mut last_error = "Cortex CLI did not return a token.".to_string();

    for args in CORTEX_PAT_COMMANDS {
        match run_command_capture(cortex_cli, args) {
            Ok(out) => match sanitize_pat(&out) {
                Ok(token) => return Ok(token),
                Err(e) => last_error = e,
            },
            Err(err) => last_error = err,
        }
    }

    Err(format!(
        "Unable to retrieve PAT from Cortex CLI. Last error: {}",
        last_error
    ))
}

fn resolve_cli(candidates: &[&str]) -> Option<String> {
    candidates.iter().find_map(|candidate| {
        if candidate.contains('/') {
            let path = Path::new(candidate);
            if path.exists() && path.is_file() {
                return Some(path.to_string_lossy().to_string());
            }
            return None;
        }

        if command_exists(candidate) {
            return Some((*candidate).to_string());
        }

        None
    })
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {} >/dev/null 2>&1", command))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_command(command: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", command, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "Command failed: {} {}\n{}",
            command,
            args.join(" "),
            stderr
        ));
    }

    Ok(())
}

fn run_command_capture(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", command, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "Command failed: {} {} -> {}",
            command,
            args.join(" "),
            stderr
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Err(format!(
            "Command produced no output: {} {}",
            command,
            args.join(" ")
        ));
    }

    Ok(stdout)
}

fn codex_config_path() -> PathBuf {
    home_dir().join(".codex").join("config.toml")
}

fn shell_profile_path() -> PathBuf {
    if let Ok(shell) = env::var("SHELL") {
        if shell.contains("bash") {
            return home_dir().join(".bashrc");
        }
        if shell.contains("zsh") {
            return home_dir().join(".zshrc");
        }
    }

    home_dir().join(".zshrc")
}

fn home_dir() -> PathBuf {
    home::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn persist_shell_profile_keychain_hook() -> Result<(), String> {
    let profile = shell_profile_path();
    ensure_parent_dir(&profile)?;

    let existing = fs::read_to_string(&profile).unwrap_or_default();
    let new_line = shell_export_line_from_keychain();

    let mut replaced = false;
    let mut output_lines = Vec::new();

    for line in existing.lines() {
        if is_export_line_for_var(line, GITHUB_PAT_ENV_VAR) {
            output_lines.push(new_line.clone());
            replaced = true;
        } else {
            output_lines.push(line.to_string());
        }
    }

    if !replaced {
        if !output_lines.is_empty() && !output_lines.last().unwrap_or(&String::new()).is_empty() {
            output_lines.push(String::new());
        }
        output_lines.push(new_line);
    }

    let mut final_content = output_lines.join("\n");
    if !final_content.ends_with('\n') {
        final_content.push('\n');
    }

    fs::write(&profile, final_content)
        .map_err(|e| format!("Failed to write {}: {}", profile.display(), e))
}

fn profile_contains_export(path: &Path, env_var: &str) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    Ok(content
        .lines()
        .any(|line| is_export_line_for_var(line, env_var)))
}

fn is_export_line_for_var(line: &str, env_var: &str) -> bool {
    let trimmed = line.trim_start();
    let prefix = format!("export {}=", env_var);
    trimmed.starts_with(&prefix)
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    Ok(())
}

fn shell_export_line_from_keychain() -> String {
    format!(
        "export {}=\"$(security find-generic-password -a \\\"$USER\\\" -s \\\"{}\\\" -w 2>/dev/null)\"",
        GITHUB_PAT_ENV_VAR, KEYCHAIN_SERVICE
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_pat_rejects_invalid_values() {
        assert!(sanitize_pat("   ").is_err());
        assert!(sanitize_pat("abc").is_err());
        assert!(sanitize_pat("ghp_a b c d e f g h i j k").is_err());
    }

    #[test]
    fn shell_export_line_uses_keychain_lookup() {
        let line = shell_export_line_from_keychain();
        assert!(line.contains("security find-generic-password"));
        assert!(line.contains(KEYCHAIN_SERVICE));
        assert!(line.contains("GITHUB_PAT_TOKEN"));
    }

    #[test]
    fn export_line_match_is_scoped() {
        assert!(is_export_line_for_var(
            "export GITHUB_PAT_TOKEN='x'",
            "GITHUB_PAT_TOKEN"
        ));
        assert!(!is_export_line_for_var(
            "export OTHER='x'",
            "GITHUB_PAT_TOKEN"
        ));
    }
}
