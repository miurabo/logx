use std::net::IpAddr;

use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use super::LogParser;
use crate::types::{LogEntry, LogLevel};

pub struct JsonLinesParser;

impl JsonLinesParser {
    fn extract_timestamp(obj: &serde_json::Map<String, Value>) -> Option<DateTime<FixedOffset>> {
        let candidates = ["timestamp", "time", "@timestamp", "datetime", "date", "ts"];
        for key in &candidates {
            if let Some(Value::String(s)) = obj.get(*key) {
                if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                    return Some(dt);
                }
                if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f%z") {
                    return Some(dt);
                }
                if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z") {
                    return Some(dt);
                }
            }
        }
        None
    }

    fn extract_level(obj: &serde_json::Map<String, Value>) -> Option<LogLevel> {
        let candidates = ["level", "severity", "loglevel", "log_level"];
        for key in &candidates {
            if let Some(Value::String(s)) = obj.get(*key) {
                return parse_level(s);
            }
        }
        None
    }

    fn extract_string(obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
        for key in keys {
            if let Some(Value::String(s)) = obj.get(*key) {
                return Some(s.clone());
            }
        }
        None
    }

    fn extract_status(obj: &serde_json::Map<String, Value>) -> Option<u16> {
        let candidates = ["status", "status_code", "http_status", "statusCode"];
        for key in &candidates {
            match obj.get(*key) {
                Some(Value::Number(n)) => {
                    if let Some(code) = n.as_u64()
                        && code <= u16::MAX as u64
                    {
                        return Some(code as u16);
                    }
                }
                Some(Value::String(s)) => {
                    if let Ok(code) = s.parse::<u16>() {
                        return Some(code);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn extract_ip(obj: &serde_json::Map<String, Value>) -> Option<IpAddr> {
        let candidates = ["ip", "remote_addr", "client_ip", "source_ip", "clientIp"];
        for key in &candidates {
            if let Some(Value::String(s)) = obj.get(*key)
                && let Ok(ip) = s.parse::<IpAddr>()
            {
                return Some(ip);
            }
        }
        None
    }
}

fn parse_level(s: &str) -> Option<LogLevel> {
    match s.to_uppercase().as_str() {
        "DEBUG" | "TRACE" => Some(LogLevel::Debug),
        "INFO" | "INFORMATION" => Some(LogLevel::Info),
        "WARN" | "WARNING" => Some(LogLevel::Warn),
        "ERROR" | "ERR" => Some(LogLevel::Error),
        "FATAL" => Some(LogLevel::Fatal),
        "CRITICAL" | "CRIT" => Some(LogLevel::Critical),
        _ => None,
    }
}

impl LogParser for JsonLinesParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let value: Value = serde_json::from_str(line).ok()?;
        let obj = value.as_object()?;

        let timestamp = Self::extract_timestamp(obj);
        let level = Self::extract_level(obj);
        let message = Self::extract_string(obj, &["message", "msg", "log", "text"]);
        let method = Self::extract_string(obj, &["method", "http_method", "request_method"]);
        let path = Self::extract_string(obj, &["path", "url", "uri", "request_path"]);
        let status_code = Self::extract_status(obj);
        let ip = Self::extract_ip(obj);

        Some(LogEntry {
            raw: line.to_string(),
            timestamp,
            level,
            status_code,
            method,
            path,
            ip,
            message,
            line_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_lines_full_entry() {
        let parser = JsonLinesParser;
        let line = r#"{"timestamp":"2026-03-08T12:00:00+09:00","level":"error","message":"connection refused","status":500,"method":"POST","path":"/api/users","ip":"192.168.1.10"}"#;
        let entry = parser.parse_line(line, 1).unwrap();
        assert!(entry.timestamp.is_some());
        assert_eq!(entry.level, Some(LogLevel::Error));
        assert_eq!(entry.message.as_deref(), Some("connection refused"));
        assert_eq!(entry.status_code, Some(500));
        assert_eq!(entry.method.as_deref(), Some("POST"));
        assert_eq!(entry.path.as_deref(), Some("/api/users"));
        assert_eq!(entry.ip.unwrap().to_string(), "192.168.1.10");
    }

    #[test]
    fn parse_json_lines_minimal() {
        let parser = JsonLinesParser;
        let line = r#"{"msg":"hello"}"#;
        let entry = parser.parse_line(line, 1).unwrap();
        assert_eq!(entry.message.as_deref(), Some("hello"));
        assert!(entry.timestamp.is_none());
    }

    #[test]
    fn parse_json_lines_returns_none_for_invalid_json() {
        let parser = JsonLinesParser;
        assert!(parser.parse_line("not json", 1).is_none());
    }

    #[test]
    fn parse_json_lines_returns_none_for_json_array() {
        let parser = JsonLinesParser;
        assert!(parser.parse_line("[1, 2, 3]", 1).is_none());
    }

    #[test]
    fn parse_level_variants() {
        assert_eq!(parse_level("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(parse_level("info"), Some(LogLevel::Info));
        assert_eq!(parse_level("WARNING"), Some(LogLevel::Warn));
        assert_eq!(parse_level("ERR"), Some(LogLevel::Error));
        assert_eq!(parse_level("FATAL"), Some(LogLevel::Fatal));
        assert_eq!(parse_level("CRIT"), Some(LogLevel::Critical));
        assert_eq!(parse_level("unknown"), None);
    }
}
