use std::fmt;
use std::net::IpAddr;

use chrono::{DateTime, FixedOffset};

/// ログファイルのフォーマット種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    ApacheCombined,
    ApacheCommon,
    Nginx,
    JsonLines,
    Syslog,
    PlainText,
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogFormat::ApacheCombined => write!(f, "Apache Combined"),
            LogFormat::ApacheCommon => write!(f, "Apache Common"),
            LogFormat::Nginx => write!(f, "Nginx"),
            LogFormat::JsonLines => write!(f, "JSON Lines"),
            LogFormat::Syslog => write!(f, "Syslog"),
            LogFormat::PlainText => write!(f, "Plain Text"),
        }
    }
}

/// ログの重要度レベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// パース済みの1行分のログデータ
#[derive(Debug, Clone, Default)]
pub struct LogEntry {
    pub raw: String,
    pub timestamp: Option<DateTime<FixedOffset>>,
    pub level: Option<LogLevel>,
    pub status_code: Option<u16>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub ip: Option<IpAddr>,
    pub message: Option<String>,
    pub line_number: usize,
}

/// フォーマット自動検出の結果
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub format: LogFormat,
    pub confidence: f64,
    pub sample_lines: usize,
}
