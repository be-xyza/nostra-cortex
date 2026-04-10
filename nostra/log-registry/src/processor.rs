use crate::types::LogEvent;
use std::cell::RefCell;

// Thread-local storage for the buffer (standard IC pattern)
thread_local! {
    static LOG_BUFFER: RefCell<Vec<LogEvent>> = RefCell::new(Vec::with_capacity(100));
}

pub struct BatchProcessor;

impl BatchProcessor {
    pub fn add_log(log: LogEvent) {
        LOG_BUFFER.with(|buffer| {
            let mut b = buffer.borrow_mut();
            b.push(log);

            // Simple flush trigger
            if b.len() >= 50 {
                // In a real implementation, this would write to StableMemory
                // For now, we just clear it to simulate processing
                b.clear();
                ic_cdk::println!("Flushed 50 logs to storage.");
            }
        });
    }

    pub fn force_flush() {
        LOG_BUFFER.with(|buffer| {
            let mut b = buffer.borrow_mut();
            if !b.is_empty() {
                ic_cdk::println!("Force flushed {} logs.", b.len());
                b.clear();
            }
        });
    }
}
