use crate::api;
use chrono::Utc;
use gloo_storage::{LocalStorage, Storage};
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

// --- Constants ---
const VFS_STORAGE_KEY: &str = "nostra_vfs_data";
const VFS_SIZE_LIMIT: usize = 50 * 1024 * 1024; // 50MB Limit
static VFS_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_vfs_id() -> String {
    let counter = VFS_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * 1_000_000_000.0) as u64;
    format!("vfs-{timestamp}-{counter}-{random}")
}

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn default_root() -> FileNode {
    let mut root = FileNode::new_dir("root");
    root.children
        .insert("lib".to_string(), FileNode::new_dir("lib"));

    if let Some(lib) = root.children.get_mut("lib") {
        lib.children
            .insert("artifacts".to_string(), FileNode::new_dir("artifacts"));
        lib.children
            .insert("books".to_string(), FileNode::new_dir("books"));
        lib.children
            .insert("dpubs".to_string(), FileNode::new_dir("dpubs"));
        lib.children
            .insert("chronicle".to_string(), FileNode::new_dir("chronicle"));
        lib.children.insert(
            "audit_traces".to_string(),
            FileNode::new_dir("audit_traces"),
        );
    }

    root
}

fn global_root() -> &'static Mutex<FileNode> {
    static ROOT: OnceLock<Mutex<FileNode>> = OnceLock::new();

    ROOT.get_or_init(|| {
        if let Ok(data) = LocalStorage::get::<FileNode>(VFS_STORAGE_KEY) {
            console_log!("VFS: Hydrated from LocalStorage");
            Mutex::new(data)
        } else {
            console_log!("VFS: Initializing fresh Root");
            Mutex::new(default_root())
        }
    })
}

fn persist_root(root: &FileNode) {
    let _ = LocalStorage::set(VFS_STORAGE_KEY, root);
}

// --- Types ---

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Directory,
    File {
        mime_type: String,
        size: usize,
        last_modified: i64, // timestamp
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub content: Option<Vec<u8>>,            // None for Directory
    pub children: HashMap<String, FileNode>, // Name -> Node
}

impl FileNode {
    pub fn new_dir(name: &str) -> Self {
        Self {
            id: next_vfs_id(),
            name: name.to_string(),
            node_type: NodeType::Directory,
            content: None,
            children: HashMap::new(),
        }
    }

    pub fn new_file(name: &str, content: Vec<u8>, mime_type: &str) -> Self {
        Self {
            id: next_vfs_id(),
            name: name.to_string(),
            node_type: NodeType::File {
                mime_type: mime_type.to_string(),
                size: content.len(),
                last_modified: Utc::now().timestamp_millis(),
            },
            content: Some(content),
            children: HashMap::new(),
        }
    }
}

// --- Service ---

#[derive(Clone, Copy)]
pub struct VfsService;

impl VfsService {
    pub fn new() -> Self {
        let _ = global_root();
        Self
    }

    // --- Operations ---

    pub fn write_file(&self, path: &str, content: Vec<u8>, mime_type: &str) -> Result<(), String> {
        // 1. Check size limit
        let current_size = self.calculate_total_size();
        if current_size + content.len() > VFS_SIZE_LIMIT {
            return Err("VFS Storage Limit Exceeded (50MB)".to_string());
        }

        // 2. Traverse and write
        let mut root = lock_unpoisoned(global_root());
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return Err("Invalid path".to_string());
        }

        let file_name = parts.last().copied().unwrap_or_default();
        let dir_parts = &parts[0..parts.len() - 1];

        // Navigate/Create Dirs
        let mut current_node = &mut *root;
        for part in dir_parts {
            if !current_node.children.contains_key(*part) {
                current_node
                    .children
                    .insert(part.to_string(), FileNode::new_dir(part));
            }
            if let Some(next_node) = current_node.children.get_mut(*part) {
                current_node = next_node;
            } else {
                return Err(format!("Unable to create/find directory segment: {part}"));
            }
        }

        // Write File
        current_node.children.insert(
            file_name.to_string(),
            FileNode::new_file(file_name, content, mime_type),
        );

        // 3. Persist
        persist_root(&root);

        // 4. Indexing Hook (Stub)
        console_log!("VFS: Indexed artifact at {}", path);

        Ok(())
    }

    pub fn list_dir(&self, path: &str) -> Vec<FileNode> {
        let root = lock_unpoisoned(global_root());
        let mut current = &*root;

        for part in path.split('/').filter(|s| !s.is_empty()) {
            if let Some(node) = current.children.get(part) {
                current = node;
            } else {
                return vec![];
            }
        }

        current.children.values().cloned().collect()
    }

    pub fn read_file_string(&self, path: &str) -> Option<String> {
        let root = lock_unpoisoned(global_root());
        let mut current = &*root;

        for part in path.split('/').filter(|s| !s.is_empty()) {
            if let Some(node) = current.children.get(part) {
                current = node;
            } else {
                return None;
            }
        }

        current
            .content
            .as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
    }

    pub fn read_file_bytes(&self, path: &str) -> Option<Vec<u8>> {
        let root = lock_unpoisoned(global_root());
        let mut current = &*root;

        for part in path.split('/').filter(|s| !s.is_empty()) {
            if let Some(node) = current.children.get(part) {
                current = node;
            } else {
                return None;
            }
        }

        current.content.clone()
    }

    pub async fn sync_to_backend(&self, agent: &Agent, path: &str) -> Result<(), String> {
        if let Some(content) = self.read_file_bytes(path) {
            let mime = if path.ends_with(".md") {
                "text/markdown"
            } else if path.ends_with(".json") {
                "application/json"
            } else if path.ends_with(".png") {
                "image/png"
            } else {
                "application/octet-stream"
            };

            api::write_file(agent, path.to_string(), content, mime.to_string()).await?;
            console_log!("VFS: Synced {} to Canister", path);
            Ok(())
        } else {
            Err(format!("File not found locally: {}", path))
        }
    }

    pub async fn sync_from_backend(&self, agent: &Agent, prefix: &str) -> Result<usize, String> {
        let remote_files = api::list_files(agent, prefix.to_string()).await?;
        let mut synced_count = 0;

        for (path, metadata) in remote_files {
            let exists_locally = self.read_file_bytes(&path).is_some();

            if !exists_locally {
                console_log!("VFS: Pulling {} from Canister...", path);
                let content = api::read_file(agent, path.clone()).await?;
                self.write_file(&path, content, &metadata.mime_type)?;
                synced_count += 1;
            }
        }
        Ok(synced_count)
    }

    pub async fn sync_dpub_from_backend_guarded(
        &self,
        agent: &Agent,
        prefix: &str,
        viewer_space_did: Option<String>,
        treaty_token: Option<String>,
    ) -> Result<usize, String> {
        let viewer = viewer_space_did
            .as_deref()
            .map(|v| v.trim())
            .unwrap_or("");
        if viewer.is_empty() {
            return Err("Viewer space required for guarded sync".to_string());
        }
        let remote_files = api::list_dpub_files_guarded(
            agent,
            prefix.to_string(),
            viewer_space_did.clone(),
            treaty_token.clone(),
        )
        .await?;
        let mut synced_count = 0;

        for (path, metadata) in remote_files {
            let exists_locally = self.read_file_bytes(&path).is_some();
            if !exists_locally {
                console_log!("VFS: Pulling {} from Canister (guarded)...", path);
                let content = api::read_dpub_file_guarded(
                    agent,
                    path.clone(),
                    viewer_space_did.clone(),
                    treaty_token.clone(),
                )
                .await?;
                self.write_file(&path, content, &metadata.mime_type)?;
                synced_count += 1;
            }
        }
        Ok(synced_count)
    }

    pub async fn sync_from_backend_guarded(
        &self,
        agent: &Agent,
        prefix: &str,
        viewer_space_did: Option<String>,
        treaty_token: Option<String>,
    ) -> Result<usize, String> {
        let viewer = viewer_space_did
            .as_deref()
            .map(|v| v.trim())
            .unwrap_or("");
        if viewer.is_empty() {
            return Err("Viewer space required for guarded sync".to_string());
        }
        let token = treaty_token
            .as_deref()
            .map(|t| t.trim())
            .unwrap_or("");
        if token.is_empty() {
            return Err("Treaty token required for guarded sync".to_string());
        }
        let remote_files = api::list_vfs_guarded(
            agent,
            prefix.to_string(),
            viewer_space_did.clone(),
            treaty_token.clone(),
        )
        .await?;
        let mut synced_count = 0;

        for (path, metadata) in remote_files {
            let exists_locally = self.read_file_bytes(&path).is_some();
            if !exists_locally {
                console_log!("VFS: Pulling {} from Canister (guarded)...", path);
                let content = api::read_vfs_guarded(
                    agent,
                    path.clone(),
                    viewer_space_did.clone(),
                    treaty_token.clone(),
                )
                .await?;
                self.write_file(&path, content, &metadata.mime_type)?;
                synced_count += 1;
            }
        }
        Ok(synced_count)
    }

    // --- Helpers ---

    fn calculate_total_size(&self) -> usize {
        fn calc_node(node: &FileNode) -> usize {
            let mut size = 0;
            if let Some(c) = &node.content {
                size += c.len();
            }
            for child in node.children.values() {
                size += calc_node(child);
            }
            size
        }

        let root = lock_unpoisoned(global_root());
        calc_node(&root)
    }

    pub fn get_tree_json(&self) -> String {
        let root = lock_unpoisoned(global_root());
        serde_json::to_string_pretty(&*root).unwrap_or_default()
    }
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()))
}
use console_log;
