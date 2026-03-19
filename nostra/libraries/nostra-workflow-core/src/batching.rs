//! Signal Batching Implementation
//!
//! Implements a signal accumulator for handling high-volume signals (e.g., voting).
//! Allows aggregating multiple signals into a single batch before processing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for signal batching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum number of signals to accumulate before processing.
    pub max_size: usize,
    /// Maximum time to wait (in seconds) before processing partial batch.
    pub timeout_seconds: u64,
}

/// Accumulator state for a specific signal type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAccumulator {
    /// The type of signal being accumulated (e.g., "vote").
    pub signal_type: String,
    /// Batch configuration.
    pub config: BatchConfig,
    /// Accumulated signals (payloads).
    pub signals: Vec<String>,
    /// Timestamp when the first signal in this batch was received.
    pub first_signal_at: Option<u64>,
}

impl SignalAccumulator {
    /// Create a new accumulator.
    pub fn new(signal_type: &str, config: BatchConfig) -> Self {
        Self {
            signal_type: signal_type.to_string(),
            config,
            signals: Vec::new(),
            first_signal_at: None,
        }
    }

    /// Add a signal to the batch.
    ///
    /// Returns `true` if the batch is ready to be processed (size limit reached).
    pub fn add_signal(&mut self, payload: String, current_time: u64) -> bool {
        if self.signals.is_empty() {
            self.first_signal_at = Some(current_time);
        }
        self.signals.push(payload);

        self.is_ready(current_time)
    }

    /// Check if the batch is ready to be processed.
    fn is_ready(&self, current_time: u64) -> bool {
        if self.signals.len() >= self.config.max_size {
            return true;
        }

        if let Some(start_time) = self.first_signal_at {
            if current_time >= start_time + self.config.timeout_seconds {
                return true;
            }
        }

        false
    }

    /// Drain the accumulator and return the batch.
    ///
    /// Returns `None` if the batch is empty.
    pub fn drain(&mut self) -> Option<Vec<String>> {
        if self.signals.is_empty() {
            return None;
        }

        let batch = std::mem::take(&mut self.signals);
        self.first_signal_at = None;
        Some(batch)
    }

    /// Check readiness based on time only (for periodic checks).
    pub fn check_timeout(&self, current_time: u64) -> bool {
        if let Some(start_time) = self.first_signal_at {
            current_time >= start_time + self.config.timeout_seconds
        } else {
            false
        }
    }
}

/// Registry of active accumulators for a workflow instance.
#[derive(Debug, Clone, Default)]
pub struct BatchRegistry {
    accumulators: HashMap<String, SignalAccumulator>,
}

impl BatchRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new accumulator configuration.
    pub fn register(&mut self, signal_type: &str, config: BatchConfig) {
        self.accumulators.insert(
            signal_type.to_string(),
            SignalAccumulator::new(signal_type, config),
        );
    }

    /// Handle an incoming signal.
    ///
    /// Returns `Some(batch)` if a batch is ready, `None` otherwise.
    pub fn handle_signal(
        &mut self,
        signal_type: &str,
        payload: String,
        current_time: u64,
    ) -> Option<Vec<String>> {
        if let Some(acc) = self.accumulators.get_mut(signal_type) {
            if acc.add_signal(payload, current_time) {
                return acc.drain();
            }
        }
        None
    }

    /// Check all accumulators for timeouts.
    ///
    /// Returns a map of signal_type -> batch for any timed-out batches.
    pub fn check_timeouts(&mut self, current_time: u64) -> HashMap<String, Vec<String>> {
        let mut ready_batches = HashMap::new();

        for (signal_type, acc) in self.accumulators.iter_mut() {
            if acc.check_timeout(current_time) {
                if let Some(batch) = acc.drain() {
                    ready_batches.insert(signal_type.clone(), batch);
                }
            }
        }

        ready_batches
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size_trigger() {
        let config = BatchConfig {
            max_size: 3,
            timeout_seconds: 100,
        };
        let mut acc = SignalAccumulator::new("vote", config);

        assert!(!acc.add_signal("vote1".to_string(), 0));
        assert!(!acc.add_signal("vote2".to_string(), 1));

        // Third vote should trigger ready (size limit)
        assert!(acc.add_signal("vote3".to_string(), 2));

        let batch = acc.drain().unwrap();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch, vec!["vote1", "vote2", "vote3"]);

        // Accumulator should be empty now
        assert!(acc.signals.is_empty());
        assert!(acc.first_signal_at.is_none());
    }

    #[test]
    fn test_batch_timeout_trigger() {
        let config = BatchConfig {
            max_size: 10,
            timeout_seconds: 5,
        };
        let mut acc = SignalAccumulator::new("vote", config);

        acc.add_signal("vote1".to_string(), 100);

        // Not ready at t=104 (elapsed 4)
        assert!(!acc.check_timeout(104));

        // Ready at t=105 (elapsed 5)
        assert!(acc.check_timeout(105));

        let batch = acc.drain().unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0], "vote1");
    }

    #[test]
    fn test_registry_handling() {
        let mut registry = BatchRegistry::new();
        registry.register(
            "vote",
            BatchConfig {
                max_size: 2,
                timeout_seconds: 10,
            },
        );

        // First vote - accumulated
        let result = registry.handle_signal("vote", "yes".to_string(), 0);
        assert!(result.is_none());

        // Second vote - triggers batch
        let result = registry.handle_signal("vote", "no".to_string(), 1);
        assert!(result.is_some());

        let batch = result.unwrap();
        assert_eq!(batch, vec!["yes", "no"]);
    }

    #[test]
    fn test_unknown_signal_ignored() {
        let mut registry = BatchRegistry::new();
        let result = registry.handle_signal("unknown", "data".to_string(), 0);
        assert!(result.is_none());
    }
}
