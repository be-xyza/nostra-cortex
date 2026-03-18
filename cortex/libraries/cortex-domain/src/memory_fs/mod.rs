use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;

/// A cryptographic hash representing a content-addressable object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Oid(String);

impl Oid {
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn compute(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        Self(hex::encode(result))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Blob represents a raw file's contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob {
    pub content: Vec<u8>,
}

impl Blob {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }

    pub fn oid(&self) -> Oid {
        // Prefix with "blob " and length for standard Git-like hashing, or just hash content.
        // For simplicity and purity, we just hash the content directly or with a simple header.

        let mut header = format!("blob {}\0", self.content.len()).into_bytes();
        header.extend_from_slice(&self.content);
        Oid::compute(&header)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeEntry {
    Blob(Oid),
    Tree(Oid),
}

/// A Tree represents a directory containing blobs or other trees.
/// BTreeMap ensures deterministic ordering of entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree {
    pub entries: BTreeMap<String, TreeEntry>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, entry: TreeEntry) {
        self.entries.insert(name, entry);
    }

    pub fn oid(&self) -> Oid {
        // Deterministic serialization of tree entries.
        // We use BTreeMap so iteration is sorted by key.
        let mut buffer = Vec::new();
        for (name, entry) in &self.entries {
            let mode = match entry {
                TreeEntry::Tree(_) => "040000 tree ",
                TreeEntry::Blob(_) => "100644 blob ",
            };
            buffer.extend_from_slice(mode.as_bytes());
            buffer.extend_from_slice(name.as_bytes());
            buffer.push(b'\0');
            let oid_str = match entry {
                TreeEntry::Tree(oid) => oid.as_str(),
                TreeEntry::Blob(oid) => oid.as_str(),
            };
            buffer.extend_from_slice(oid_str.as_bytes());
            buffer.push(b'\n');
        }

        let mut header = format!("tree {}\0", buffer.len()).into_bytes();
        header.extend_from_slice(&buffer);
        Oid::compute(&header)
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

/// A Commit represents a snapshot of a Tree, with an optional parent and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commit {
    pub tree: Oid,
    pub parents: Vec<Oid>,
    pub author: String,
    pub message: String,
    pub timestamp: u64, // Unix timestamp in seconds
}

impl Commit {
    pub fn oid(&self) -> Oid {
        let mut content = String::new();
        content.push_str(&format!("tree {}\n", self.tree.as_str()));
        for parent in &self.parents {
            content.push_str(&format!("parent {}\n", parent.as_str()));
        }
        content.push_str(&format!("author {} {}\n\n", self.author, self.timestamp));
        content.push_str(&self.message);

        let content_bytes = content.as_bytes();
        let mut header = format!("commit {}\0", content_bytes.len()).into_bytes();
        header.extend_from_slice(content_bytes);
        Oid::compute(&header)
    }
}

/// Represents a pointer to a specific commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Oid,
}

impl Branch {
    pub fn new(name: impl Into<String>, commit: Oid) -> Self {
        Self {
            name: name.into(),
            commit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blob_oid() {
        let blob = Blob::new(b"hello world".to_vec());
        let oid = blob.oid();
        assert_eq!(
            oid.as_str(),
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03" // sha256 of "blob 11\0hello world"
        );
    }

    #[test]
    fn test_tree_oid_deterministic() {
        let mut tree1 = Tree::new();
        tree1.add_entry(
            "b.txt".to_string(),
            TreeEntry::Blob(Oid::new("hash2".to_string())),
        );
        tree1.add_entry(
            "a.txt".to_string(),
            TreeEntry::Blob(Oid::new("hash1".to_string())),
        );

        let mut tree2 = Tree::new();
        // Insert in different order
        tree2.add_entry(
            "a.txt".to_string(),
            TreeEntry::Blob(Oid::new("hash1".to_string())),
        );
        tree2.add_entry(
            "b.txt".to_string(),
            TreeEntry::Blob(Oid::new("hash2".to_string())),
        );

        assert_eq!(tree1.oid(), tree2.oid());
    }
}
