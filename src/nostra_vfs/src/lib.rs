use candid::{CandidType, Deserialize};
use nostra_core::types::richtext::RichContent;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub enum FileNode {
    File {
        content: RichContent,
        metadata: FileMetadata,
    },
    Directory {
        children: HashMap<String, FileNode>,
        metadata: FileMetadata,
    },
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct FileMetadata {
    pub created_at: u64,
    pub modified_at: u64,
    pub version: u64,
    pub owner: String, // Principal
}
