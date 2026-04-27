use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WorkflowFile {
    pub name: String,
    pub path: String,
    pub content: Option<String>,
}

pub struct FileSystemService;

impl FileSystemService {
    pub fn get_root_path() -> PathBuf {
        if let Ok(root) = std::env::var("NOSTRA_WORKSPACE_ROOT") {
            return PathBuf::from(root).join("_cortex").join("workflows");
        }

        std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("_cortex")
            .join("workflows")
    }

    pub fn list_workflows() -> Vec<WorkflowFile> {
        let root = Self::get_root_path();
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" {
                        files.push(WorkflowFile {
                            name: entry.file_name().to_string_lossy().to_string(),
                            path: entry.path().to_string_lossy().to_string(),
                            content: None, // Don't load content for list
                        });
                    }
                }
            }
        }
        files
    }

    pub fn read_file(path_str: &str) -> Option<String> {
        // Security check: ensure path is within the Cortex workflow scratch root.
        let root = Self::get_root_path();
        let path = PathBuf::from(path_str);

        if path.starts_with(&root) {
            fs::read_to_string(path).ok()
        } else {
            None
        }
    }

    pub fn save_file(path_str: &str, content: &str) -> Result<(), std::io::Error> {
        // Security check: ensure path is within the Cortex workflow scratch root.
        let root = Self::get_root_path();
        let path = PathBuf::from(path_str);

        if path.starts_with(&root) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, content)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Access Denied",
            ))
        }
    }
}
