use crate::types::{LogBatch, LogEvent};

pub trait Receiver {
    fn receive(&self, batch: LogBatch);
}

pub trait Processor {
    fn process(&self, event: LogEvent) -> Option<LogEvent>;
}

pub trait Exporter {
    fn export(&self, batch: Vec<LogEvent>);
}

pub struct Pipeline {
    pub processors: Vec<Box<dyn Processor>>,
    pub exporter: Box<dyn Exporter>,
}

impl Pipeline {
    pub fn new(exporter: Box<dyn Exporter>) -> Self {
        Self {
            processors: Vec::new(),
            exporter,
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn Processor>) {
        self.processors.push(processor);
    }

    pub fn run(&self, batch: LogBatch) {
        let mut processed_logs = Vec::new();
        for log in batch.logs {
            let mut current_log = Some(log);
            for processor in &self.processors {
                if let Some(log) = current_log {
                    current_log = processor.process(log);
                } else {
                    break;
                }
            }
            if let Some(log) = current_log {
                processed_logs.push(log);
            }
        }
        if !processed_logs.is_empty() {
            self.exporter.export(processed_logs);
        }
    }
}
