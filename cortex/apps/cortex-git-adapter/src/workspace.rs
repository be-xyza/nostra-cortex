use std::path::PathBuf;

pub fn discover_workspace_root(mut start: PathBuf) -> Option<PathBuf> {
    loop {
        if start.join("AGENTS.md").exists() && start.join("research").is_dir() && start.join("cortex").is_dir() {
            return Some(start);
        }
        if !start.pop() {
            return None;
        }
    }
}

