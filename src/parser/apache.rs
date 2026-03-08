use std::net::IpAddr;
use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use super::LogParser;
use crate::types::LogEntry;

// Apache Combined: 127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326 "http://www.example.com" "Mozilla/5.0"
static COMBINED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(\S+) \S+ \S+ \[([^\]]+)\] "(\S+) (\S+) \S+" (\d{3}) (\d+|-) "([^"]*)" "([^"]*)""#,
    )
    .unwrap()
});

// Apache Common: 127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326
static COMMON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(\S+) \S+ \S+ \[([^\]]+)\] "(\S+) (\S+) \S+" (\d{3}) (\d+|-)"#).unwrap()
});

fn parse_apache_timestamp(s: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(s, "%d/%b/%Y:%H:%M:%S %z").ok()
}

pub struct ApacheCombinedParser;

impl LogParser for ApacheCombinedParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = COMBINED_RE.captures(line)?;
        let ip = caps.get(1)?.as_str().parse::<IpAddr>().ok();
        let timestamp = caps.get(2).and_then(|m| parse_apache_timestamp(m.as_str()));
        let method = caps.get(3).map(|m| m.as_str().to_string());
        let path = caps.get(4).map(|m| m.as_str().to_string());
        let status_code = caps.get(5).and_then(|m| m.as_str().parse::<u16>().ok());

        Some(LogEntry {
            raw: line.to_string(),
            timestamp,
            level: None,
            status_code,
            method,
            path,
            ip,
            message: None,
            line_number,
        })
    }
}

pub struct ApacheCommonParser;

impl LogParser for ApacheCommonParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = COMMON_RE.captures(line)?;
        let ip = caps.get(1)?.as_str().parse::<IpAddr>().ok();
        let timestamp = caps.get(2).and_then(|m| parse_apache_timestamp(m.as_str()));
        let method = caps.get(3).map(|m| m.as_str().to_string());
        let path = caps.get(4).map(|m| m.as_str().to_string());
        let status_code = caps.get(5).and_then(|m| m.as_str().parse::<u16>().ok());

        Some(LogEntry {
            raw: line.to_string(),
            timestamp,
            level: None,
            status_code,
            method,
            path,
            ip,
            message: None,
            line_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_combined_valid_line() {
        let parser = ApacheCombinedParser;
        let line = r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08""#;
        let entry = parser.parse_line(line, 1).unwrap();
        assert_eq!(entry.ip.unwrap().to_string(), "127.0.0.1");
        assert_eq!(entry.status_code, Some(200));
        assert_eq!(entry.method.as_deref(), Some("GET"));
        assert_eq!(entry.path.as_deref(), Some("/apache_pb.gif"));
        assert!(entry.timestamp.is_some());
    }

    #[test]
    fn parse_combined_returns_none_for_garbage() {
        let parser = ApacheCombinedParser;
        assert!(parser.parse_line("not a log line", 1).is_none());
    }

    #[test]
    fn parse_common_valid_line() {
        let parser = ApacheCommonParser;
        let line =
            r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326"#;
        let entry = parser.parse_line(line, 1).unwrap();
        assert_eq!(entry.ip.unwrap().to_string(), "127.0.0.1");
        assert_eq!(entry.status_code, Some(200));
        assert_eq!(entry.method.as_deref(), Some("GET"));
        assert_eq!(entry.path.as_deref(), Some("/index.html"));
    }

    #[test]
    fn parse_common_returns_none_for_garbage() {
        let parser = ApacheCommonParser;
        assert!(parser.parse_line("random text", 1).is_none());
    }

    #[test]
    fn parse_combined_5xx_status() {
        let parser = ApacheCombinedParser;
        let line = r#"192.168.1.1 - - [08/Mar/2026:21:30:15 +0900] "POST /api/users HTTP/1.1" 500 1234 "-" "curl/7.68""#;
        let entry = parser.parse_line(line, 5).unwrap();
        assert_eq!(entry.status_code, Some(500));
        assert_eq!(entry.line_number, 5);
    }
}
