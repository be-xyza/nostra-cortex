use crate::config::{ProjectorSettings, RepoConfig};
use serde_json::Value;

fn esc(raw: &str) -> String {
    raw.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn upsert_command(
    settings: &ProjectorSettings,
    ty: &str,
    name: &str,
    description: &str,
    attributes: &[(String, String)],
    tags: &[String],
    references: &[String],
) -> String {
    let mut cmd = format!(
        "UPSERT {{ type: \"{}\", name: \"{}\", description: \"{}\"",
        esc(ty),
        esc(name),
        esc(description)
    );

    if settings.emit_attributes && !attributes.is_empty() {
        cmd.push_str(", attributes: [");
        for (i, (k, v)) in attributes.iter().enumerate() {
            if i > 0 {
                cmd.push_str(", ");
            }
            cmd.push_str("(\"");
            cmd.push_str(&esc(k));
            cmd.push_str("\", \"");
            cmd.push_str(&esc(v));
            cmd.push_str("\")");
        }
        cmd.push(']');
    }

    if !tags.is_empty() {
        cmd.push_str(", tags: [");
        for (i, t) in tags.iter().enumerate() {
            if i > 0 {
                cmd.push_str(", ");
            }
            cmd.push('"');
            cmd.push_str(&esc(t));
            cmd.push('"');
        }
        cmd.push(']');
    }

    for target in references {
        cmd.push_str(&format!(" @ \"references\" \"{}\"", esc(target)));
    }

    cmd.push_str(" }");
    cmd
}

pub fn project_event_to_kip(
    repo_cfg: &RepoConfig,
    event: &str,
    payload: &Value,
    settings: &ProjectorSettings,
) -> Result<Vec<String>, String> {
    match event {
        "push" => project_push(repo_cfg, payload, settings),
        "pull_request" => project_pull_request(repo_cfg, payload, settings),
        "issues" => project_issue(repo_cfg, payload, settings),
        other => {
            tracing::debug!("ignoring github event {}", other);
            Ok(Vec::new())
        }
    }
}

fn stable_repo_id(full_name: &str) -> String {
    format!("github:repo:{}", full_name)
}

fn stable_commit_id(full_name: &str, sha: &str) -> String {
    format!("github:commit:{}@{}", full_name, sha)
}

fn stable_pr_id(full_name: &str, number: i64) -> String {
    format!("github:pr:{}#{}", full_name, number)
}

fn stable_issue_id(full_name: &str, number: i64) -> String {
    format!("github:issue:{}#{}", full_name, number)
}

fn project_repo(
    repo_cfg: &RepoConfig,
    payload: &Value,
    settings: &ProjectorSettings,
) -> Result<(String, String), String> {
    let repo = payload
        .get("repository")
        .ok_or_else(|| "missing repository".to_string())?;
    let full_name = repo
        .get("full_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing repository.full_name".to_string())?;

    let html_url = repo.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
    let default_branch = repo
        .get("default_branch")
        .and_then(|v| v.as_str())
        .unwrap_or("main");
    let visibility = repo.get("visibility").and_then(|v| v.as_str()).unwrap_or("");

    let repo_id = stable_repo_id(full_name);
    let mut attributes = vec![
        ("provider".to_string(), "github".to_string()),
        ("repo_full_name".to_string(), full_name.to_string()),
        ("html_url".to_string(), html_url.to_string()),
        ("default_branch".to_string(), default_branch.to_string()),
        ("visibility".to_string(), visibility.to_string()),
        ("space_id".to_string(), repo_cfg.space_id.to_string()),
    ];

    let description = if settings.emit_attributes {
        format!("GitHub repo {}", full_name)
    } else {
        serde_json::json!({
            "provider": "github",
            "repo_full_name": full_name,
            "html_url": html_url,
            "default_branch": default_branch,
            "visibility": visibility,
            "space_id": repo_cfg.space_id,
        })
        .to_string()
    };

    if !settings.emit_attributes {
        attributes.clear();
    }

    let repo_cmd = upsert_command(
        settings,
        "Library",
        repo_id.as_str(),
        description.as_str(),
        &attributes,
        &repo_cfg.tags,
        &[],
    );
    Ok((repo_id, repo_cmd))
}

fn project_push(
    repo_cfg: &RepoConfig,
    payload: &Value,
    settings: &ProjectorSettings,
) -> Result<Vec<String>, String> {
    if !repo_cfg.ingest_push {
        return Ok(Vec::new());
    }

    let (repo_id, repo_cmd) = project_repo(repo_cfg, payload, settings)?;

    let full_name = payload
        .get("repository")
        .and_then(|v| v.get("full_name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing repository.full_name".to_string())?;

    let mut out = vec![repo_cmd];
    let commits = payload.get("commits").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    for commit in commits {
        let sha = commit.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if sha.is_empty() {
            continue;
        }
        let message = commit.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let url = commit.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let timestamp = commit.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let author_name = commit
            .get("author")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let author_email = commit
            .get("author")
            .and_then(|v| v.get("email"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let id = stable_commit_id(full_name, sha);
        let mut attributes = vec![
            ("provider".to_string(), "github".to_string()),
            ("repo_full_name".to_string(), full_name.to_string()),
            ("commit_sha".to_string(), sha.to_string()),
            ("message_headline".to_string(), message.lines().next().unwrap_or(message).to_string()),
            ("commit_url".to_string(), url.to_string()),
            ("committed_at".to_string(), timestamp.to_string()),
            ("author_name".to_string(), author_name.to_string()),
            ("source_event".to_string(), "push".to_string()),
        ];
        if settings.store_author_email && !author_email.is_empty() {
            attributes.push(("author_email".to_string(), author_email.to_string()));
        }

        let description = if settings.emit_attributes {
            message.lines().next().unwrap_or(message).to_string()
        } else {
            serde_json::json!({
                "repo_full_name": full_name,
                "commit_sha": sha,
                "message_headline": message.lines().next().unwrap_or(message),
                "commit_url": url,
                "committed_at": timestamp,
                "author_name": author_name,
                "author_email": if settings.store_author_email { author_email } else { "" },
                "source_event": "push",
            })
            .to_string()
        };

        if !settings.emit_attributes {
            attributes.clear();
        }

        out.push(upsert_command(
            settings,
            "Artifact",
            id.as_str(),
            description.as_str(),
            &attributes,
            &[],
            &[repo_id.clone()],
        ));
    }
    Ok(out)
}

fn project_pull_request(
    repo_cfg: &RepoConfig,
    payload: &Value,
    settings: &ProjectorSettings,
) -> Result<Vec<String>, String> {
    if !repo_cfg.ingest_pull_request {
        return Ok(Vec::new());
    }
    let (repo_id, repo_cmd) = project_repo(repo_cfg, payload, settings)?;
    let repo_full_name = payload
        .get("repository")
        .and_then(|v| v.get("full_name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing repository.full_name".to_string())?;

    let pr = payload
        .get("pull_request")
        .ok_or_else(|| "missing pull_request".to_string())?;
    let number = pr.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
    if number <= 0 {
        return Ok(vec![repo_cmd]);
    }

    let title = pr.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let state = pr.get("state").and_then(|v| v.as_str()).unwrap_or("");
    let draft = pr.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
    let merged = pr.get("merged").and_then(|v| v.as_bool()).unwrap_or(false);
    let html_url = pr.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at = pr.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");
    let base_ref = pr
        .get("base")
        .and_then(|v| v.get("ref"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let head_ref = pr
        .get("head")
        .and_then(|v| v.get("ref"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let head_sha = pr
        .get("head")
        .and_then(|v| v.get("sha"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let pr_id = stable_pr_id(repo_full_name, number);
    let mut refs = vec![repo_id.clone()];
    if !head_sha.is_empty() {
        refs.push(stable_commit_id(repo_full_name, head_sha));
    }

    let mut attributes = vec![
        ("provider".to_string(), "github".to_string()),
        ("repo_full_name".to_string(), repo_full_name.to_string()),
        ("pr_number".to_string(), number.to_string()),
        ("title".to_string(), title.to_string()),
        ("state".to_string(), state.to_string()),
        ("draft".to_string(), draft.to_string()),
        ("merged".to_string(), merged.to_string()),
        ("html_url".to_string(), html_url.to_string()),
        ("updated_at".to_string(), updated_at.to_string()),
        ("base_branch".to_string(), base_ref.to_string()),
        ("head_branch".to_string(), head_ref.to_string()),
        ("head_sha".to_string(), head_sha.to_string()),
    ];

    let description = if settings.emit_attributes {
        title.to_string()
    } else {
        serde_json::json!({
            "repo_full_name": repo_full_name,
            "pr_number": number,
            "title": title,
            "state": state,
            "draft": draft,
            "merged": merged,
            "html_url": html_url,
            "updated_at": updated_at,
            "base_branch": base_ref,
            "head_branch": head_ref,
            "head_sha": head_sha,
        })
        .to_string()
    };

    if !settings.emit_attributes {
        attributes.clear();
    }

    let mut out = vec![repo_cmd];
    out.push(upsert_command(
        settings,
        "Proposal",
        pr_id.as_str(),
        description.as_str(),
        &attributes,
        &[],
        &refs,
    ));
    Ok(out)
}

fn project_issue(
    repo_cfg: &RepoConfig,
    payload: &Value,
    settings: &ProjectorSettings,
) -> Result<Vec<String>, String> {
    if !repo_cfg.ingest_issues {
        return Ok(Vec::new());
    }
    let (repo_id, repo_cmd) = project_repo(repo_cfg, payload, settings)?;
    let repo_full_name = payload
        .get("repository")
        .and_then(|v| v.get("full_name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing repository.full_name".to_string())?;

    let issue = payload.get("issue").ok_or_else(|| "missing issue".to_string())?;
    let number = issue.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
    if number <= 0 {
        return Ok(vec![repo_cmd]);
    }
    let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let state = issue.get("state").and_then(|v| v.as_str()).unwrap_or("");
    let html_url = issue.get("html_url").and_then(|v| v.as_str()).unwrap_or("");
    let updated_at = issue.get("updated_at").and_then(|v| v.as_str()).unwrap_or("");

    let id = stable_issue_id(repo_full_name, number);
    let mut attributes = vec![
        ("provider".to_string(), "github".to_string()),
        ("repo_full_name".to_string(), repo_full_name.to_string()),
        ("issue_number".to_string(), number.to_string()),
        ("title".to_string(), title.to_string()),
        ("state".to_string(), state.to_string()),
        ("html_url".to_string(), html_url.to_string()),
        ("updated_at".to_string(), updated_at.to_string()),
    ];

    let description = if settings.emit_attributes {
        title.to_string()
    } else {
        serde_json::json!({
            "repo_full_name": repo_full_name,
            "issue_number": number,
            "title": title,
            "state": state,
            "html_url": html_url,
            "updated_at": updated_at,
        })
        .to_string()
    };

    if !settings.emit_attributes {
        attributes.clear();
    }

    Ok(vec![
        repo_cmd,
        upsert_command(
            settings,
            "Issue",
            id.as_str(),
            description.as_str(),
            &attributes,
            &[],
            &[repo_id],
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projector_uses_stable_ids() {
        let repo_cfg = RepoConfig {
            enabled: true,
            repo_full_name: "o/r".to_string(),
            branch: Some("main".to_string()),
            space_id: "s".to_string(),
            ingest_push: true,
            ingest_pull_request: true,
            ingest_issues: true,
            interval_secs: 1,
            lookback_secs: 1,
            tags: vec![],
        };

        let payload = serde_json::json!({
            "repository": { "full_name": "o/r", "html_url": "https://github.com/o/r", "default_branch": "main", "visibility": "public" },
            "commits": [ { "id": "abc", "message": "m", "url": "u", "timestamp": "t", "author": { "name": "n", "email": "e" } } ]
        });

        let settings = ProjectorSettings {
            emit_attributes: false,
            store_author_email: false,
        };
        let cmds = project_event_to_kip(&repo_cfg, "push", &payload, &settings).unwrap();
        assert!(cmds.iter().any(|c| c.contains("name: \"github:repo:o/r\"")));
        assert!(cmds.iter().any(|c| c.contains("name: \"github:commit:o/r@abc\"")));
    }
}
