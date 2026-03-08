use std::collections::HashMap;

use anyhow::{Result, bail};
use chrono::TimeDelta;

use crate::types::{ErrorSummary, LogEntry, LogLevel};

/// LogEntryがエラーかどうか判定する
pub fn is_error(entry: &LogEntry) -> bool {
    // 1. HTTPステータスコードが4xx or 5xx
    if let Some(code) = entry.status_code
        && code >= 400
    {
        return true;
    }

    // 2. ログレベルがError以上
    if let Some(ref level) = entry.level
        && matches!(
            level,
            LogLevel::Error | LogLevel::Fatal | LogLevel::Critical
        )
    {
        return true;
    }

    // 3. メッセージにエラーパターンが含まれる
    if let Some(ref msg) = entry.message {
        let msg_lower = msg.to_lowercase();
        if msg_lower.contains("fatal error")
            || msg_lower.contains("exception")
            || msg_lower.contains("stack trace")
            || msg_lower.contains("panic")
        {
            return true;
        }
    }

    false
}

/// エラーのカテゴリを決定する
fn error_category(entry: &LogEntry) -> String {
    if let Some(code) = entry.status_code
        && code >= 400
    {
        return format!("{code}");
    }

    if let Some(ref level) = entry.level
        && matches!(
            level,
            LogLevel::Error | LogLevel::Fatal | LogLevel::Critical
        )
    {
        return level.to_string();
    }

    if let Some(ref msg) = entry.message {
        let msg_lower = msg.to_lowercase();
        if msg_lower.contains("fatal error") {
            return "Fatal Error".to_string();
        }
        if msg_lower.contains("exception") {
            return "Exception".to_string();
        }
        if msg_lower.contains("stack trace") {
            return "Stack Trace".to_string();
        }
        if msg_lower.contains("panic") {
            return "Panic".to_string();
        }
    }

    "Unknown Error".to_string()
}

/// エラーをカテゴリ別にグループ化してサマリーを生成する
pub fn summarize_errors(entries: &[LogEntry]) -> Vec<ErrorSummary> {
    let mut groups: HashMap<String, ErrorSummary> = HashMap::new();

    for entry in entries {
        let category = error_category(entry);
        let summary = groups.entry(category.clone()).or_insert(ErrorSummary {
            category,
            count: 0,
            first_seen: entry.timestamp,
            last_seen: entry.timestamp,
        });
        summary.count += 1;
        if let Some(ts) = entry.timestamp {
            if summary.first_seen.is_none() || summary.first_seen.is_some_and(|f| ts < f) {
                summary.first_seen = Some(ts);
            }
            if summary.last_seen.is_none() || summary.last_seen.is_some_and(|l| ts > l) {
                summary.last_seen = Some(ts);
            }
        }
    }

    let mut result: Vec<ErrorSummary> = groups.into_values().collect();
    result.sort_by(|a, b| b.count.cmp(&a.count));
    result
}

/// duration文字列をパースする（"1h", "30m", "2d", "90s"）
pub fn parse_duration(s: &str) -> Result<TimeDelta> {
    let s = s.trim();
    if s.len() < 2 {
        bail!("Invalid duration: '{s}' (expected format: 1h, 30m, 2d, 90s)");
    }

    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str.parse().map_err(|_| {
        anyhow::anyhow!("Invalid duration: '{s}' (expected format: 1h, 30m, 2d, 90s)")
    })?;

    match unit {
        "s" => TimeDelta::try_seconds(num),
        "m" => TimeDelta::try_minutes(num),
        "h" => TimeDelta::try_hours(num),
        "d" => TimeDelta::try_days(num),
        _ => bail!("Invalid duration unit: '{unit}' (expected: s, m, h, d)"),
    }
    .ok_or_else(|| anyhow::anyhow!("Duration overflow: '{s}'"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LogEntry;

    #[test]
    fn is_error_returns_true_for_5xx() {
        let entry = LogEntry {
            status_code: Some(500),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_true_for_4xx() {
        let entry = LogEntry {
            status_code: Some(404),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_false_for_2xx() {
        let entry = LogEntry {
            status_code: Some(200),
            ..Default::default()
        };
        assert!(!is_error(&entry));
    }

    #[test]
    fn is_error_returns_false_for_3xx() {
        let entry = LogEntry {
            status_code: Some(301),
            ..Default::default()
        };
        assert!(!is_error(&entry));
    }

    #[test]
    fn is_error_returns_true_for_error_level() {
        let entry = LogEntry {
            level: Some(LogLevel::Error),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_true_for_fatal_level() {
        let entry = LogEntry {
            level: Some(LogLevel::Fatal),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_false_for_warn_level() {
        let entry = LogEntry {
            level: Some(LogLevel::Warn),
            ..Default::default()
        };
        assert!(!is_error(&entry));
    }

    #[test]
    fn is_error_returns_true_for_exception_message() {
        let entry = LogEntry {
            message: Some("java.lang.NullPointerException at ...".to_string()),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_true_for_fatal_error_message() {
        let entry = LogEntry {
            message: Some("PHP Fatal error: Allowed memory size".to_string()),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_true_for_panic_message() {
        let entry = LogEntry {
            message: Some("thread 'main' panicked at ...".to_string()),
            ..Default::default()
        };
        assert!(is_error(&entry));
    }

    #[test]
    fn is_error_returns_false_for_normal_message() {
        let entry = LogEntry {
            message: Some("Request completed successfully".to_string()),
            ..Default::default()
        };
        assert!(!is_error(&entry));
    }

    #[test]
    fn summarize_errors_groups_by_category() {
        let entries = vec![
            LogEntry {
                status_code: Some(500),
                ..Default::default()
            },
            LogEntry {
                status_code: Some(500),
                ..Default::default()
            },
            LogEntry {
                status_code: Some(404),
                ..Default::default()
            },
            LogEntry {
                level: Some(LogLevel::Error),
                ..Default::default()
            },
        ];
        let summary = summarize_errors(&entries);
        assert_eq!(summary.len(), 3);
        // 最多が先頭
        assert_eq!(summary[0].category, "500");
        assert_eq!(summary[0].count, 2);
    }

    #[test]
    fn summarize_errors_empty_returns_empty() {
        let summary = summarize_errors(&[]);
        assert!(summary.is_empty());
    }

    #[test]
    fn parse_duration_seconds() {
        let d = parse_duration("90s").unwrap();
        assert_eq!(d.num_seconds(), 90);
    }

    #[test]
    fn parse_duration_minutes() {
        let d = parse_duration("30m").unwrap();
        assert_eq!(d.num_minutes(), 30);
    }

    #[test]
    fn parse_duration_hours() {
        let d = parse_duration("1h").unwrap();
        assert_eq!(d.num_hours(), 1);
    }

    #[test]
    fn parse_duration_days() {
        let d = parse_duration("2d").unwrap();
        assert_eq!(d.num_days(), 2);
    }

    #[test]
    fn parse_duration_invalid_unit() {
        assert!(parse_duration("1x").is_err());
    }

    #[test]
    fn parse_duration_invalid_number() {
        assert!(parse_duration("abch").is_err());
    }

    #[test]
    fn parse_duration_too_short() {
        assert!(parse_duration("h").is_err());
    }
}
