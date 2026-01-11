use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use better_term::Color;

#[derive(Clone)]
pub struct LogEntry {
    pub message: String,
    pub color: Color,
}

#[derive(Clone)]
pub struct LogBuffer {
    pub logs: Arc<Mutex<VecDeque<LogEntry>>>,
    pub max_lines: usize,
}

impl LogBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
            max_lines,
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut logs = self.logs.lock().unwrap();
        if logs.len() >= self.max_lines {
            logs.pop_front();
        }
        logs.push_back(entry);
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap();
        logs.iter().cloned().collect()
    }
}

// Global instance for convenience/standard logging interception
use std::sync::OnceLock;

pub static GLOBAL_LOG_BUFFER: OnceLock<LogBuffer> = OnceLock::new();

pub fn push_log(message: String, color: Color) {
    if let Some(buffer) = GLOBAL_LOG_BUFFER.get() {
        buffer.push(LogEntry { message, color });
    }
}
