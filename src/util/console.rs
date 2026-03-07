use parking_lot::Mutex;
use std::sync::Arc;
use std::time::SystemTime;

/// Log level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// A single log entry.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: SystemTime,
}

/// Global console instance.
pub struct Console {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    max_logs: usize,
}

impl Console {
    /// Create a new console with max log entries.
    pub fn new(max_logs: usize) -> Self {
        Console {
            logs: Arc::new(Mutex::new(Vec::new())),
            max_logs,
        }
    }

    /// Log a message.
    pub fn log(&self, message: impl Into<String>) {
        self.log_level(LogLevel::Info, message);
    }

    /// Log a debug message.
    pub fn debug(&self, message: impl Into<String>) {
        self.log_level(LogLevel::Debug, message);
    }

    /// Log an info message.
    pub fn info(&self, message: impl Into<String>) {
        self.log_level(LogLevel::Info, message);
    }

    /// Log a warning message.
    pub fn warn(&self, message: impl Into<String>) {
        self.log_level(LogLevel::Warn, message);
    }

    /// Log an error message.
    pub fn error(&self, message: impl Into<String>) {
        self.log_level(LogLevel::Error, message);
    }

    /// Internal log method.
    fn log_level(&self, level: LogLevel, message: impl Into<String>) {
        let mut logs = self.logs.lock();
        logs.push(LogEntry {
            level,
            message: message.into(),
            timestamp: SystemTime::now(),
        });

        while logs.len() > self.max_logs {
            logs.remove(0);
        }
    }

    /// Get all logs.
    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.lock().clone()
    }

    /// Get the latest log entry (top of stack).
    pub fn last(&self) -> Option<LogEntry> {
        let logs = self.logs.lock();
        logs.last().cloned()
    }

    /// Get the latest N log entries.
    pub fn latest(&self, n: usize) -> Vec<LogEntry> {
        let logs = self.logs.lock();
        logs.iter().rev().take(n).cloned().collect()
    }

    /// Filter logs by level.
    pub fn filter(&self, level: LogLevel) -> Vec<LogEntry> {
        let logs = self.logs.lock();
        logs.iter()
            .filter(|log| log.level == level)
            .cloned()
            .collect()
    }

    /// Clear all logs.
    pub fn clear(&self) {
        self.logs.lock().clear();
    }

    /// Get the number of logs.
    pub fn len(&self) -> usize {
        self.logs.lock().len()
    }

    /// Check if there are no logs.
    pub fn is_empty(&self) -> bool {
        self.logs.lock().is_empty()
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new(1000)
    }
}

// Global singleton
use once_cell::sync::Lazy;

static GLOBAL_CONSOLE: Lazy<Console> = Lazy::new(Console::default);

/// Get the global console instance.
pub fn console() -> &'static Console {
    &GLOBAL_CONSOLE
}

// Convenience functions (same as browser)
pub fn log(message: impl Into<String>) {
    console().log(message);
}

pub fn debug(message: impl Into<String>) {
    console().debug(message);
}

pub fn info(message: impl Into<String>) {
    console().info(message);
}

pub fn warn(message: impl Into<String>) {
    console().warn(message);
}

pub fn error(message: impl Into<String>) {
    console().error(message);
}
