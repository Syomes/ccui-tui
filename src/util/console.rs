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

    /// Internal log method.
    pub fn log_level(&self, level: LogLevel, args: std::fmt::Arguments) {
        let mut logs = self.logs.lock();
        logs.push(LogEntry {
            level,
            message: args.to_string(),
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

/// Log an info message.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::util::console::console().log_level(
            $crate::util::console::LogLevel::Info,
            ::std::format_args!($($arg)*),
        )
    };
}

/// Log a debug message.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::util::console::console().log_level(
            $crate::util::console::LogLevel::Debug,
            ::std::format_args!($($arg)*),
        )
    };
}

/// Log an info message.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::util::console::console().log_level(
            $crate::util::console::LogLevel::Info,
            ::std::format_args!($($arg)*),
        )
    };
}

/// Log a warning message.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::util::console::console().log_level(
            $crate::util::console::LogLevel::Warn,
            ::std::format_args!($($arg)*),
        )
    };
}

/// Log an error message.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::util::console::console().log_level(
            $crate::util::console::LogLevel::Error,
            ::std::format_args!($($arg)*),
        )
    };
}

// Re-export to make them accessible from the module path (console::log!("..."))
#[allow(unused_imports)]
pub(crate) use crate::debug;
#[allow(unused_imports)]
pub(crate) use crate::error;
#[allow(unused_imports)]
pub(crate) use crate::info;
#[allow(unused_imports)]
pub(crate) use crate::log;
#[allow(unused_imports)]
pub(crate) use crate::warn;
