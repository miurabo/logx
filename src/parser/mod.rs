pub mod apache;
pub mod json_lines;
pub mod nginx;
pub mod plain_text;
pub mod syslog;

use crate::types::{LogEntry, LogFormat};

/// ログ行をパースするトレイト
pub trait LogParser {
    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry>;
}

/// LogFormatに対応するパーサーを生成する
pub fn create_parser(format: &LogFormat) -> Box<dyn LogParser> {
    match format {
        LogFormat::ApacheCombined => Box::new(apache::ApacheCombinedParser),
        LogFormat::ApacheCommon => Box::new(apache::ApacheCommonParser),
        LogFormat::Nginx => Box::new(nginx::NginxParser),
        LogFormat::JsonLines => Box::new(json_lines::JsonLinesParser),
        LogFormat::Syslog => Box::new(syslog::SyslogParser),
        LogFormat::PlainText => Box::new(plain_text::PlainTextParser),
    }
}
