use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Clone)]
pub struct WorktreeService {
    hooks_dir: PathBuf,
}

impl WorktreeService {
    pub fn new(base_storage_path: PathBuf) -> Self {
        let hooks_dir = base_storage_path.join("hooks");
        Self { hooks_dir }
    }

    /// Creates a new git worktree for a specific task.
    ///
    /// # Arguments
    /// * `repo_path`: The path to the main git repository (or bare repo).
    /// * `task_id`: The unique ID of the task (used for directory name).
    /// * `branch_name`: The name of the new branch to check out.
    /// * `base_ref`: The commit/branch to branch off from (e.g., "main").
    pub async fn create_worktree(
        &self,
        repo_path: &Path,
        task_id: &str,
        branch_name: &str,
        base_ref: &str,
    ) -> Result<PathBuf> {
        // Ensure hooks directory exists
        if !self.hooks_dir.exists() {
            tokio::fs::create_dir_all(&self.hooks_dir)
                .await
                .context("Failed to create hooks directory")?;
        }

        let worktree_path = self.hooks_dir.join(task_id);

        if worktree_path.exists() {
            // Already exists, assume it's valid or needs cleanup
            // For now, return existing path
            return Ok(worktree_path);
        }

        // git -C {repo_path} worktree add -b {branch_name} {worktree_path} {base_ref}
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(branch_name)
            .arg(&worktree_path)
            .arg(base_ref)
            .output()
            .await
            .context("Failed to execute git worktree add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("git worktree add failed: {}", stderr);
        }

        Ok(worktree_path)
    }

    /// Removes a worktree and its associated directory.
    pub async fn remove_worktree(&self, _repo_path: &Path, task_id: &str) -> Result<()> {
        let worktree_path = self.hooks_dir.join(task_id);

        if !worktree_path.exists() {
            return Ok(());
        }

        // git worktree remove {path} --force
        // We run this from the worktree itself or the main repo.
        // Safer to just remove the directory if it's not locked,
        // but 'git worktree remove' is cleaner as it updates .git/worktrees.

        // However, since we might be "in" the repo passed as argument, let's try running from there.
        // Actually, `git worktree remove <path>` is the standard way.
        let output = Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(&worktree_path)
            .output()
            .await
            .context("Failed to execute git worktree remove")?;

        // If git failed (maybe because it's not a git repo anymore?), force delete directory
        if !output.status.success() {
            tokio::fs::remove_dir_all(&worktree_path)
                .await
                .context("Failed to force remove worktree directory")?;
        }

        Ok(())
    }

    pub async fn list_worktrees(&self) -> Result<Vec<String>> {
        let mut entries = tokio::fs::read_dir(&self.hooks_dir).await?;
        let mut worktrees = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if let Ok(file_name) = entry.file_name().into_string() {
                worktrees.push(file_name);
            }
        }
        Ok(worktrees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_worktree_lifecycle() -> Result<()> {
        // Setup: Create a temp dir for the "Repo" and "Methods"
        let temp_dir = std::env::temp_dir().join(format!("nostra_test_{}", uuid::Uuid::new_v4()));
        let repo_dir = temp_dir.join("repo");
        let hooks_dir = temp_dir.join("nostra_hooks");

        fs::create_dir_all(&repo_dir).await?;
        fs::create_dir_all(&hooks_dir).await?;

        // Initialize a dummy git repo
        Command::new("git")
            .arg("init")
            .current_dir(&repo_dir)
            .output()
            .await?;
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(&repo_dir)
            .output()
            .await?;
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(&repo_dir)
            .output()
            .await?;

        // Create an initial commit (needed for branching)
        fs::write(repo_dir.join("README.md"), "# Initial").await?;
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&repo_dir)
            .output()
            .await?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(&repo_dir)
            .output()
            .await?;

        // Initialize Service
        let service = WorktreeService {
            hooks_dir: hooks_dir.clone(),
        };

        // 1. Create Worktree
        let task_id = "task-123";
        let branch_name = "hooks/task-123";
        let outcome = service
            .create_worktree(&repo_dir, task_id, branch_name, "master")
            .await;

        // Note: 'master' might be 'main' depending on git config. Let's try to detect or force it.
        // Actually, git init usually creates master or main. Let's just create 'main' explicitly to be safe.
        Command::new("git")
            .arg("branch")
            .arg("-M")
            .arg("main")
            .current_dir(&repo_dir)
            .output()
            .await?;

        let outcome = service
            .create_worktree(&repo_dir, task_id, branch_name, "main")
            .await;
        assert!(
            outcome.is_ok(),
            "Failed to create worktree: {:?}",
            outcome.err()
        );

        let path = outcome.unwrap();
        assert!(path.exists(), "Worktree path does not exist");
        assert!(
            path.join("README.md").exists(),
            "Worktree did not checkout files"
        );

        // 2. List Worktrees
        let list = service.list_worktrees().await?;
        assert!(
            list.contains(&task_id.to_string()),
            "List should contain task_id"
        );

        // 3. Remove Worktree
        service.remove_worktree(&repo_dir, task_id).await?;
        assert!(!path.exists(), "Worktree directory should be removed");

        // Cleanup
        fs::remove_dir_all(&temp_dir).await?;

        Ok(())
    }
}
