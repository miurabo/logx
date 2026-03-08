use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use super::LogParser;
use crate::types::{LogEntry, LogLevel};

// ISO 8601: 2026-03-08T12:00:00+09:00 or 2026-03-08 12:00:00
static ISO_TIMESTAMP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:[+-]\d{2}:?\d{2}|Z)?)")
        .unwrap()
});

// ログレベルのパターン
static LEVEL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(DEBUG|INFO|WARN(?:ING)?|ERROR|FATAL|CRIT(?:ICAL)?)\b").unwrap()
});

fn parse_timestamp(s: &str) -> Option<DateTime<FixedOffset>> {
    // RFC 3339 / ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f%z") {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z") {
        return Some(dt);
    }
    None
}

fn parse_level(s: &str) -> Option<LogLevel> {
    match s.to_uppercase().as_str() {
        "DEBUG" => Some(LogLevel::Debug),
        "INFO" => Some(LogLevel::Info),
        "WARN" | "WARNING" => Some(LogLevel::Warn),
        "ERROR" => Some(LogLevel::Error),
        "FATAL" => Some(LogLevel::Fatal),
        "CRIT" | "CRITICAL" => Some(LogLevel::Critical),
        _ => None,
    }
}

pub struct PlainTextParser;

impl LogParser for PlainTextParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        if line.trim().is_empty() {
            return None;
        }

        let timestamp = ISO_TIMESTAMP_RE
            .find(line)
            .and_then(|m| parse_timestamp(m.as_str()));

        let level = LEVEL_RE.find(line).and_then(|m| parse_level(m.as_str()));

        Some(LogEntry {
            raw: line.to_string(),
            timestamp,
            level,
            status_code: None,
            method: None,
            path: None,
            ip: None,
            message: Some(line.to_string()),
            line_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_text_with_iso_timestamp_and_level() {
        let parser = PlainTextParser;
        let line = "2026-03-08T12:00:00+09:00 ERROR something went wrong";
        let entry = parser.parse_line(line, 1).unwrap();
        assert!(entry.timestamp.is_some());
        assert_eq!(entry.level, Some(LogLevel::Error));
    }

    #[test]
    fn parse_plain_text_without_timestamp() {
        let parser = PlainTextParser;
        let line = "WARNING: disk usage above 90%";
        let entry = parser.parse_line(line, 1).unwrap();
        assert!(entry.timestamp.is_none());
        assert_eq!(entry.level, Some(LogLevel::Warn));
    }

    #[test]
    fn parse_plain_text_no_metadata() {
        let parser = PlainTextParser;
        let line = "just some random log text";
        let entry = parser.parse_line(line, 1).unwrap();
        assert!(entry.timestamp.is_none());
        assert!(entry.level.is_none());
        assert_eq!(entry.message.as_deref(), Some("just some random log text"));
    }

    #[test]
    fn parse_plain_text_empty_line_returns_none() {
        let parser = PlainTextParser;
        assert!(parser.parse_line("", 1).is_none());
        assert!(parser.parse_line("   ", 1).is_none());
    }
}
