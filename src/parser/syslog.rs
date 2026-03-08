use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset, Local, NaiveDateTime};
use regex::Regex;

use super::LogParser;
use crate::types::{LogEntry, LogLevel};

// RFC 3164: Mar  8 12:00:00 hostname app[1234]: message
static SYSLOG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^([A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(\S+)\s+(\S+?)(?:\[(\d+)\])?:\s*(.*)",
    )
    .unwrap()
});

fn parse_syslog_timestamp(s: &str) -> Option<DateTime<FixedOffset>> {
    let current_year = Local::now().format("%Y").to_string();
    let with_year = format!("{current_year} {s}");
    NaiveDateTime::parse_from_str(&with_year, "%Y %b %e %H:%M:%S")
        .ok()
        .map(|naive| {
            let local_offset = *Local::now().offset();
            naive.and_local_timezone(local_offset).unwrap()
        })
        .map(|dt| dt.fixed_offset())
}

fn detect_level(message: &str) -> Option<LogLevel> {
    let msg_upper = message.to_uppercase();
    if msg_upper.contains("CRITICAL") || msg_upper.contains("CRIT") {
        Some(LogLevel::Critical)
    } else if msg_upper.contains("FATAL") {
        Some(LogLevel::Fatal)
    } else if msg_upper.contains("ERROR") || msg_upper.contains("ERR") {
        Some(LogLevel::Error)
    } else if msg_upper.contains("WARNING") || msg_upper.contains("WARN") {
        Some(LogLevel::Warn)
    } else if msg_upper.contains("INFO") {
        Some(LogLevel::Info)
    } else if msg_upper.contains("DEBUG") {
        Some(LogLevel::Debug)
    } else {
        None
    }
}

pub struct SyslogParser;

impl LogParser for SyslogParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = SYSLOG_RE.captures(line)?;
        let timestamp = caps.get(1).and_then(|m| parse_syslog_timestamp(m.as_str()));
        let message = caps.get(5).map(|m| m.as_str().to_string());
        let level = message.as_deref().and_then(detect_level);

        Some(LogEntry {
            raw: line.to_string(),
            timestamp,
            level,
            status_code: None,
            method: None,
            path: None,
            ip: None,
            message,
            line_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_syslog_valid_line() {
        let parser = SyslogParser;
        let line = "Mar  8 12:00:00 myhost sshd[1234]: Accepted publickey for user from 10.0.0.1";
        let entry = parser.parse_line(line, 1).unwrap();
        assert!(entry.timestamp.is_some());
        assert_eq!(
            entry.message.as_deref(),
            Some("Accepted publickey for user from 10.0.0.1")
        );
    }

    #[test]
    fn parse_syslog_with_error_level() {
        let parser = SyslogParser;
        let line = "Mar  8 12:00:00 myhost kernel: ERROR: segfault at 0x0";
        let entry = parser.parse_line(line, 1).unwrap();
        assert_eq!(entry.level, Some(LogLevel::Error));
    }

    #[test]
    fn parse_syslog_without_pid() {
        let parser = SyslogParser;
        let line = "Mar  8 12:00:00 myhost cron: job completed";
        let entry = parser.parse_line(line, 1).unwrap();
        assert_eq!(entry.message.as_deref(), Some("job completed"));
    }

    #[test]
    fn parse_syslog_returns_none_for_garbage() {
        let parser = SyslogParser;
        assert!(parser.parse_line("not syslog", 1).is_none());
    }
}
