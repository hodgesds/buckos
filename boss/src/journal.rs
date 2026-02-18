//! Journal module for structured service logging.
//!
//! This module provides a simple journal implementation for capturing
//! and storing service output (stdout/stderr) with timestamps.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Maximum number of log entries to keep in memory per service.
const MAX_MEMORY_ENTRIES: usize = 1000;

/// Priority level for journal entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Emergency - system is unusable
    Emergency = 0,
    /// Alert - action must be taken immediately
    Alert = 1,
    /// Critical - critical conditions
    Critical = 2,
    /// Error - error conditions
    Error = 3,
    /// Warning - warning conditions
    Warning = 4,
    /// Notice - normal but significant condition
    Notice = 5,
    /// Info - informational messages
    #[default]
    Info = 6,
    /// Debug - debug-level messages
    Debug = 7,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Emergency => write!(f, "EMERG"),
            Priority::Alert => write!(f, "ALERT"),
            Priority::Critical => write!(f, "CRIT"),
            Priority::Error => write!(f, "ERR"),
            Priority::Warning => write!(f, "WARN"),
            Priority::Notice => write!(f, "NOTICE"),
            Priority::Info => write!(f, "INFO"),
            Priority::Debug => write!(f, "DEBUG"),
        }
    }
}

/// A single journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Timestamp of the entry
    pub timestamp: DateTime<Utc>,
    /// Service name
    pub service: String,
    /// Process ID that generated the entry
    pub pid: Option<u32>,
    /// Priority level
    pub priority: Priority,
    /// Log message
    pub message: String,
    /// Stream: stdout or stderr
    pub stream: String,
}

impl JournalEntry {
    /// Create a new journal entry.
    pub fn new(service: &str, message: &str, stream: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            service: service.to_string(),
            pid: None,
            priority: if stream == "stderr" {
                Priority::Error
            } else {
                Priority::Info
            },
            message: message.to_string(),
            stream: stream.to_string(),
        }
    }

    /// Create a journal entry with PID.
    pub fn with_pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Create a journal entry with priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Format the entry for display.
    pub fn format(&self) -> String {
        let pid_str = self.pid.map(|p| format!("[{}]", p)).unwrap_or_default();
        format!(
            "{} {} {}{}: {}",
            self.timestamp.format("%b %d %H:%M:%S"),
            self.service,
            pid_str,
            self.priority,
            self.message
        )
    }
}

/// Service logs stored in memory.
#[derive(Debug, Default)]
struct ServiceLogs {
    entries: VecDeque<JournalEntry>,
}

impl ServiceLogs {
    fn add(&mut self, entry: JournalEntry) {
        self.entries.push_back(entry);
        while self.entries.len() > MAX_MEMORY_ENTRIES {
            self.entries.pop_front();
        }
    }

    fn get_entries(&self, limit: Option<usize>) -> Vec<JournalEntry> {
        match limit {
            Some(n) => self.entries.iter().rev().take(n).rev().cloned().collect(),
            None => self.entries.iter().cloned().collect(),
        }
    }
}

/// Journal for storing and retrieving service logs.
pub struct Journal {
    /// In-memory logs per service
    logs: Arc<RwLock<std::collections::HashMap<String, ServiceLogs>>>,
    /// Directory for persistent log files
    log_dir: PathBuf,
}

impl Journal {
    /// Create a new journal.
    pub fn new(log_dir: PathBuf) -> Self {
        Self {
            logs: Arc::new(RwLock::new(std::collections::HashMap::new())),
            log_dir,
        }
    }

    /// Ensure the log directory exists.
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        if !self.log_dir.exists() {
            std::fs::create_dir_all(&self.log_dir)?;
        }
        Ok(())
    }

    /// Add a log entry.
    pub async fn log(&self, entry: JournalEntry) {
        let service = entry.service.clone();

        // Add to memory
        {
            let mut logs = self.logs.write().await;
            logs.entry(service.clone()).or_default().add(entry.clone());
        }

        // Write to file
        if let Err(e) = self.write_to_file(&entry) {
            tracing::warn!(error = %e, "Failed to write log entry to file");
        }
    }

    /// Write an entry to the service's log file.
    fn write_to_file(&self, entry: &JournalEntry) -> std::io::Result<()> {
        let _ = self.ensure_dir();

        let log_path = self.log_dir.join(format!("{}.log", entry.service));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        writeln!(file, "{}", entry.format())?;
        Ok(())
    }

    /// Get log entries for a service.
    pub async fn get_logs(
        &self,
        service: &str,
        limit: Option<usize>,
        follow: bool,
    ) -> Vec<JournalEntry> {
        if follow {
            // For follow mode, we just return current entries
            // The caller should poll for updates
            let logs = self.logs.read().await;
            logs.get(service)
                .map(|l| l.get_entries(limit))
                .unwrap_or_default()
        } else {
            // Try to read from file first for historical entries
            let file_entries = self.read_from_file(service, limit);
            if !file_entries.is_empty() {
                return file_entries;
            }

            // Fall back to memory
            let logs = self.logs.read().await;
            logs.get(service)
                .map(|l| l.get_entries(limit))
                .unwrap_or_default()
        }
    }

    /// Read entries from a service's log file.
    fn read_from_file(&self, service: &str, limit: Option<usize>) -> Vec<JournalEntry> {
        let log_path = self.log_dir.join(format!("{}.log", service));
        if !log_path.exists() {
            return Vec::new();
        }

        let file = match File::open(&log_path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

        let entries: Vec<JournalEntry> = match limit {
            Some(n) => lines.iter().rev().take(n).rev(),
            None => lines.iter().rev().take(lines.len()).rev(),
        }
        .map(|line| {
            // Parse the line back into a JournalEntry
            // Format: "timestamp service [pid]PRIORITY: message"
            JournalEntry {
                timestamp: Utc::now(), // We lose timestamp precision here
                service: service.to_string(),
                pid: None,
                priority: Priority::Info,
                message: line.clone(),
                stream: "stdout".to_string(),
            }
        })
        .collect();

        entries
    }

    /// Get all log entries across all services.
    pub async fn get_all_logs(&self, limit: Option<usize>) -> Vec<JournalEntry> {
        let logs = self.logs.read().await;
        let mut all_entries: Vec<JournalEntry> = logs
            .values()
            .flat_map(|l| l.entries.iter().cloned())
            .collect();

        // Sort by timestamp
        all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        match limit {
            Some(n) => all_entries.into_iter().rev().take(n).rev().collect(),
            None => all_entries,
        }
    }

    /// Clear logs for a service.
    pub async fn clear(&self, service: &str) {
        let mut logs = self.logs.write().await;
        logs.remove(service);

        // Also remove the log file
        let log_path = self.log_dir.join(format!("{}.log", service));
        let _ = std::fs::remove_file(log_path);
    }

    /// Get the log file path for a service.
    pub fn log_path(&self, service: &str) -> PathBuf {
        self.log_dir.join(format!("{}.log", service))
    }
}

impl Default for Journal {
    fn default() -> Self {
        Self::new(PathBuf::from("/var/log/buckos"))
    }
}

/// Create a pipe pair for capturing process output.
pub fn create_output_pipe() -> std::io::Result<(std::fs::File, std::fs::File)> {
    use std::os::unix::io::FromRawFd;

    let mut fds = [0i32; 2];
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };

    if result == -1 {
        return Err(std::io::Error::last_os_error());
    }

    let read_end = unsafe { std::fs::File::from_raw_fd(fds[0]) };
    let write_end = unsafe { std::fs::File::from_raw_fd(fds[1]) };

    Ok((read_end, write_end))
}

/// Output reader that reads from a pipe and logs to journal.
pub struct OutputReader {
    journal: Arc<Journal>,
    service: String,
    pid: u32,
    stream: String,
}

impl OutputReader {
    /// Create a new output reader.
    pub fn new(journal: Arc<Journal>, service: String, pid: u32, stream: String) -> Self {
        Self {
            journal,
            service,
            pid,
            stream,
        }
    }

    /// Read from the given file and log to journal.
    pub async fn read_and_log(&self, mut reader: BufReader<std::fs::File>) {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let entry = JournalEntry::new(&self.service, line.trim(), &self.stream)
                        .with_pid(self.pid);
                    self.journal.log(entry).await;
                }
                Err(_) => break,
            }
        }
    }
}
