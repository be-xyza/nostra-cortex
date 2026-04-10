use anyhow::Result;
use std::collections::HashMap;
use std::sync::RwLock;

/// Sandbox trait for isolated benchmark execution
pub trait Sandbox: Send + Sync {
    /// Read a file from the virtual filesystem
    fn read_file(&self, path: &str) -> Result<String>;

    /// Write a file to the virtual filesystem
    fn write_file(&self, path: &str, content: &str) -> Result<()>;

    /// Delete a file from the virtual filesystem
    fn delete_file(&self, path: &str) -> Result<()>;

    /// Execute a shell command in the sandbox
    fn exec(&self, command: &str, args: &[String]) -> Result<String>;

    /// Set the virtual time (ISO 8601 format)
    fn set_time(&self, iso_timestamp: &str) -> Result<()>;

    /// Get the current virtual time (Unix timestamp)
    fn get_current_time(&self) -> u64;
}

/// Virtual sandbox with in-memory filesystem and mock time
pub struct VirtualSandbox {
    files: RwLock<HashMap<String, String>>,
    mock_time: RwLock<Option<u64>>,
}

impl VirtualSandbox {
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
            mock_time: RwLock::new(None),
        }
    }

    /// Create sandbox with pre-set mock time
    pub fn with_mock_time(iso_timestamp: &str) -> Result<Self> {
        let sandbox = Self::new();
        sandbox.set_time(iso_timestamp)?;
        Ok(sandbox)
    }
}

impl Default for VirtualSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox for VirtualSandbox {
    fn read_file(&self, path: &str) -> Result<String> {
        self.files
            .read()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path))
    }

    fn write_file(&self, path: &str, content: &str) -> Result<()> {
        self.files
            .write()
            .unwrap()
            .insert(path.to_string(), content.to_string());
        Ok(())
    }

    fn delete_file(&self, path: &str) -> Result<()> {
        let mut files = self.files.write().unwrap();
        if files.remove(path).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("File not found: {}", path))
        }
    }

    fn exec(&self, command: &str, args: &[String]) -> Result<String> {
        // Mock execution - returns command echo
        Ok(format!("Executed: {} {:?}", command, args))
    }

    fn set_time(&self, iso_timestamp: &str) -> Result<()> {
        let datetime = chrono::DateTime::parse_from_rfc3339(iso_timestamp)
            .map_err(|e| anyhow::anyhow!("Invalid ISO 8601 timestamp: {}", e))?;
        let unix_ts = datetime.timestamp() as u64;
        *self.mock_time.write().unwrap() = Some(unix_ts);
        Ok(())
    }

    fn get_current_time(&self) -> u64 {
        self.mock_time
            .read()
            .unwrap()
            .unwrap_or_else(|| chrono::Utc::now().timestamp() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_time_injection() {
        let sandbox = VirtualSandbox::new();
        sandbox.set_time("2023-11-01T10:00:00Z").unwrap();
        let time = sandbox.get_current_time();
        let expected = chrono::DateTime::parse_from_rfc3339("2023-11-01T10:00:00Z")
            .unwrap()
            .timestamp() as u64;
        assert_eq!(time, expected);
    }

    #[test]
    fn test_file_operations() {
        let sandbox = VirtualSandbox::new();
        sandbox.write_file("/test.txt", "Hello").unwrap();
        assert_eq!(sandbox.read_file("/test.txt").unwrap(), "Hello");
        sandbox.delete_file("/test.txt").unwrap();
        assert!(sandbox.read_file("/test.txt").is_err());
    }
}
