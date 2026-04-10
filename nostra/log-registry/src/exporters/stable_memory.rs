use crate::pipeline::Exporter;
use crate::types::LogEvent;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static LOG_STORAGE: RefCell<StableBTreeMap<u64, LogEvent, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

pub struct StableMemoryExporter;

impl Exporter for StableMemoryExporter {
    fn export(&self, batch: Vec<LogEvent>) {
        LOG_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            for event in batch {
                storage.insert(event.time_unix_nano, event);
            }
        });
    }
}
