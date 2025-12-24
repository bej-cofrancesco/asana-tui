use log::{Level, Log, Metadata, Record};
use std::sync::{Arc, Mutex};

/// Format a log record into a string for display
///
pub fn format_log(record: &Record) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let level_str = match record.level() {
        Level::Error => "ERROR",
        Level::Warn => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    };
    format!("{} {} {}", timestamp, level_str, record.args())
}

/// Custom logger that captures logs to state
///
pub struct CustomLogger {
    log_callback: Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>,
}

impl CustomLogger {
    pub fn new() -> Self {
        CustomLogger {
            log_callback: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_log_callback(&self, callback: Box<dyn Fn(String) + Send + Sync>) {
        *self.log_callback.lock().unwrap() = Some(callback);
    }
}

impl Log for CustomLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // Allow all logs
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Capture to state
            if let Ok(callback) = self.log_callback.lock() {
                if let Some(ref cb) = *callback {
                    let formatted = format_log(record);
                    cb(formatted);
                }
            }
        }
    }

    fn flush(&self) {
        // No-op
    }
}
