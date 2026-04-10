use anyhow::{Context, Result};
use ic_agent::Agent;
use ic_agent::identity::{AnonymousIdentity, BasicIdentity, Secp256k1Identity};
use std::path::PathBuf;

fn candidate_identity_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(path) = std::env::var("NOSTRA_AGENT_IDENTITY_PEM") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            paths.push(PathBuf::from(trimmed));
        }
    }

    for var in ["ICP_IDENTITY_PEM"] {
        if let Ok(path) = std::env::var(var) {
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                paths.push(PathBuf::from(trimmed));
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let identity_name = std::env::var("ICP_IDENTITY").unwrap_or_else(|_| "default".to_string());

        paths.push(
            PathBuf::from(&home)
                .join(".config")
                .join("icp")
                .join("identity")
                .join(&identity_name)
                .join("identity.pem"),
        );
        if identity_name != "default" {
            paths.push(
                PathBuf::from(&home)
                    .join(".config")
                    .join("icp")
                    .join("identity")
                    .join("default")
                    .join("identity.pem"),
            );
        }
    }

    paths
}

pub fn build_agent_with_resolved_identity(url: &str) -> Result<(Agent, String)> {
    for path in candidate_identity_paths() {
        if !path.exists() {
            continue;
        }

        if let Ok(identity) = Secp256k1Identity::from_pem_file(&path) {
            let agent = Agent::builder()
                .with_url(url)
                .with_identity(identity)
                .build()
                .with_context(|| {
                    format!(
                        "failed to build agent with secp256k1 identity {}",
                        path.display()
                    )
                })?;
            return Ok((agent, format!("secp256k1:{}", path.display())));
        }

        if let Ok(identity) = BasicIdentity::from_pem_file(&path) {
            let agent = Agent::builder()
                .with_url(url)
                .with_identity(identity)
                .build()
                .with_context(|| {
                    format!(
                        "failed to build agent with basic identity {}",
                        path.display()
                    )
                })?;
            return Ok((agent, format!("basic:{}", path.display())));
        }
    }

    let agent = Agent::builder()
        .with_url(url)
        .with_identity(AnonymousIdentity)
        .build()
        .context("failed to build agent with anonymous identity")?;

    Ok((agent, "anonymous".to_string()))
}
