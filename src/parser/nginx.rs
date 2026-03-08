use std::net::IpAddr;
use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use regex::Regex;

use super::LogParser;
use crate::types::LogEntry;

// Nginx default format は Apache Combined と同一構造
// 違い: Nginxは $remote_addr がIPv6を含む場合がある
static NGINX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(\S+) - \S+ \[([^\]]+)\] "(\S+) (\S+) \S+" (\d{3}) (\d+|-) "([^"]*)" "([^"]*)""#)
        .unwrap()
});

fn parse_nginx_timestamp(s: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(s, "%d/%b/%Y:%H:%M:%S %z").ok()
}

pub struct NginxParser;

impl LogParser for NginxParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = NGINX_RE.captures(line)?;
        let ip = caps.get(1)?.as_str().parse::<IpAddr>().ok();
        let timestamp = caps.get(2).and_then(|m| parse_nginx_timestamp(m.as_str()));
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
    fn parse_nginx_valid_line() {
        let parser = NginxParser;
        let line = r#"93.180.71.3 - - [08/Mar/2026:12:00:00 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3""#;
        let entry = parser.parse_line(line, 1).unwrap();
        assert_eq!(entry.ip.unwrap().to_string(), "93.180.71.3");
        assert_eq!(entry.status_code, Some(304));
        assert_eq!(entry.method.as_deref(), Some("GET"));
        assert_eq!(entry.path.as_deref(), Some("/downloads/product_1"));
        assert!(entry.timestamp.is_some());
    }

    #[test]
    fn parse_nginx_returns_none_for_garbage() {
        let parser = NginxParser;
        assert!(parser.parse_line("not nginx", 1).is_none());
    }
}
