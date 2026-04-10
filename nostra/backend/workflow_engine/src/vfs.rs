use candid::{CandidType, Deserialize};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::Serialize;
use std::borrow::Cow;
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;
const VFS_MAGIC: &[u8; 4] = b"NVF1";

// --- Types ---

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct FileMetadata {
    pub mime_type: String,
    pub size: u64,
    pub last_modified: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct VfsFile {
    pub content: Vec<u8>,
    pub metadata: FileMetadata,
}

// Wrapper for Storable String Key
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PathKey(String);

impl Storable for PathKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.as_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        PathKey(String::from_utf8(bytes.into_owned()).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

impl Storable for VfsFile {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(self).unwrap();
        let mut bytes = Vec::with_capacity(VFS_MAGIC.len() + payload.len());
        bytes.extend_from_slice(VFS_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        if !bytes.starts_with(VFS_MAGIC) {
            panic!("Legacy VFS storage format detected; reinstall required.");
        }
        postcard::from_bytes(&bytes[VFS_MAGIC.len()..]).unwrap()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}


// --- Storage ---

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static VFS_MAP: RefCell<StableBTreeMap<PathKey, VfsFile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );
}

// --- API ---

pub fn vfs_write(path: String, content: Vec<u8>, mime_type: String) -> Result<(), String> {
    let metadata = FileMetadata {
        mime_type,
        size: content.len() as u64,
        last_modified: ic_cdk::api::time(),
    };
    let file = VfsFile { content, metadata };

    VFS_MAP.with(|p| {
        p.borrow_mut().insert(PathKey(path), file);
    });
    Ok(())
}

pub fn vfs_read(path: String) -> Result<Vec<u8>, String> {
    VFS_MAP.with(|p| {
        if let Some(file) = p.borrow().get(&PathKey(path)) {
            Ok(file.content)
        } else {
            Err("File not found".to_string())
        }
    })
}

pub fn vfs_list(prefix: String) -> Vec<(String, FileMetadata)> {
    VFS_MAP.with(|p| {
        p.borrow()
            .iter()
            .map(|item| (item.key().0.clone(), item.value().metadata.clone()))
            .filter(|(path, _)| path.starts_with(&prefix))
            .collect()
    })
}
