pub mod exporters;
pub mod pipeline;
mod processor;
mod tests;
mod types;

use ic_cdk_macros::{query, update};
use processor::BatchProcessor;
use types::{LogBatch, LogEvent};

#[update]
fn ingest_logs(batch: LogBatch) {
    // Pipeline: Receiver (here) -> Processor (BatchProcessor) -> Exporter (Mocked flush)
    for log in batch.logs {
        BatchProcessor::add_log(log);
    }
}

#[query]
fn get_recent_logs(_count: usize) -> Vec<LogEvent> {
    // Placeholder implementation since we aren't using real Stable Memory in this prototype
    Vec::new()
}

// Ensure .did generation works
ic_cdk::export_candid!();
