use std::io::BufRead;
use std::sync::LazyLock;

use regex::Regex;

use crate::types::{DetectionResult, LogFormat};

const MAX_SAMPLE_LINES: usize = 20;
const MIN_CONFIDENCE: f64 = 0.5;

// Apache Combined: IP - user [timestamp] "METHOD path HTTP/x.x" status size "referer" "ua"
static APACHE_COMBINED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\S+ \S+ \S+ \[[^\]]+\] "\S+ \S+ \S+" \d{3} (\d+|-) "[^"]*" "[^"]*""#).unwrap()
});

// Apache Common: IP - user [timestamp] "METHOD path HTTP/x.x" status size
static APACHE_COMMON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\S+ \S+ \S+ \[[^\]]+\] "\S+ \S+ \S+" \d{3} (\d+|-)\s*$"#).unwrap()
});

// Syslog RFC 3164: Mon DD HH:MM:SS hostname process[pid]: message
static SYSLOG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}\s+\S+\s+").unwrap());

/// フォーマット検出で評価するフォーマットとその優先順位
struct FormatCandidate {
    format: LogFormat,
    score: f64,
}

pub struct FormatDetector;

impl FormatDetector {
    /// ログファイルのフォーマットを自動検出する
    ///
    /// 先頭20行を読み取り、各フォーマットの正規表現でマッチングを行い、
    /// 最も適合率の高いフォーマットを返す。
    pub fn detect(reader: &mut dyn BufRead) -> DetectionResult {
        let mut lines = Vec::new();
        let mut buf = String::new();

        while lines.len() < MAX_SAMPLE_LINES {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = buf.trim();
                    if !trimmed.is_empty() {
                        lines.push(trimmed.to_string());
                    }
                }
                Err(_) => break,
            }
        }

        if lines.is_empty() {
            return DetectionResult {
                format: LogFormat::PlainText,
                confidence: 0.0,
                sample_lines: 0,
            };
        }

        let sample_lines = lines.len();
        let candidates = Self::score_formats(&lines);

        // 最高スコアのフォーマットを選択
        // 同スコアの場合はvec内の先頭（優先順位が高い）を採用するため
        // a >= b のとき Greater を返す
        let best = candidates
            .into_iter()
            .reduce(|a, b| if a.score >= b.score { a } else { b })
            .unwrap();

        if best.score < MIN_CONFIDENCE {
            return DetectionResult {
                format: LogFormat::PlainText,
                confidence: best.score,
                sample_lines,
            };
        }

        DetectionResult {
            format: best.format,
            confidence: best.score,
            sample_lines,
        }
    }

    fn score_formats(lines: &[String]) -> Vec<FormatCandidate> {
        let total = lines.len() as f64;

        // JSON Lines: 行がJSONオブジェクトとしてパースできるか
        let json_count = lines
            .iter()
            .filter(|l| {
                serde_json::from_str::<serde_json::Value>(l)
                    .ok()
                    .is_some_and(|v| v.is_object())
            })
            .count();

        // Apache Combined（Commonのスーパーセットなので先に判定）
        let combined_count = lines
            .iter()
            .filter(|l| APACHE_COMBINED_RE.is_match(l))
            .count();

        // Apache Common（Combinedにマッチしない行のみ）
        let common_count = lines
            .iter()
            .filter(|l| APACHE_COMMON_RE.is_match(l))
            .count();

        // Syslog
        let syslog_count = lines.iter().filter(|l| SYSLOG_RE.is_match(l)).count();

        // 優先順位順に並べる（同スコアの場合、先頭が選ばれる）
        vec![
            FormatCandidate {
                format: LogFormat::ApacheCombined,
                score: combined_count as f64 / total,
            },
            FormatCandidate {
                format: LogFormat::Nginx,
                // NginxのデフォルトフォーマットはApache Combinedと同一なので同じスコア
                // ただしCombinedが優先される（vecの順序）
                score: combined_count as f64 / total,
            },
            FormatCandidate {
                format: LogFormat::JsonLines,
                score: json_count as f64 / total,
            },
            FormatCandidate {
                format: LogFormat::Syslog,
                score: syslog_count as f64 / total,
            },
            FormatCandidate {
                format: LogFormat::ApacheCommon,
                score: common_count as f64 / total,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn detect_from_str(s: &str) -> DetectionResult {
        let mut reader = Cursor::new(s.as_bytes());
        FormatDetector::detect(&mut reader)
    }

    #[test]
    fn detect_apache_combined() {
        let log = r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08"
192.168.1.1 - - [10/Oct/2000:13:56:00 -0700] "POST /api/users HTTP/1.1" 500 1234 "-" "curl/7.68"
"#;
        let result = detect_from_str(log);
        assert_eq!(result.format, LogFormat::ApacheCombined);
        assert!(result.confidence >= 0.5);
    }

    #[test]
    fn detect_apache_common() {
        let log = r#"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326
192.168.1.1 - - [10/Oct/2000:13:56:00 -0700] "POST /api HTTP/1.1" 404 0
"#;
        let result = detect_from_str(log);
        assert_eq!(result.format, LogFormat::ApacheCommon);
        assert!(result.confidence >= 0.5);
    }

    #[test]
    fn detect_json_lines() {
        let log = r#"{"timestamp":"2026-03-08T12:00:00Z","level":"info","message":"started"}
{"timestamp":"2026-03-08T12:00:01Z","level":"error","message":"failed"}
"#;
        let result = detect_from_str(log);
        assert_eq!(result.format, LogFormat::JsonLines);
        assert!(result.confidence >= 0.5);
    }

    #[test]
    fn detect_syslog() {
        let log = "Mar  8 12:00:00 myhost sshd[1234]: Accepted publickey for user\nMar  8 12:00:01 myhost kernel: segfault at 0x0\n";
        let result = detect_from_str(log);
        assert_eq!(result.format, LogFormat::Syslog);
        assert!(result.confidence >= 0.5);
    }

    #[test]
    fn detect_plain_text_fallback_for_unknown() {
        let log = "just some random text\nanother line of text\n";
        let result = detect_from_str(log);
        assert_eq!(result.format, LogFormat::PlainText);
    }

    #[test]
    fn detect_empty_input_returns_plain_text() {
        let result = detect_from_str("");
        assert_eq!(result.format, LogFormat::PlainText);
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.sample_lines, 0);
    }

    #[test]
    fn detect_mixed_content_below_threshold_returns_plain_text() {
        // 5行中1行だけApache = 20% < 50%
        let log = r#"random text
127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET / HTTP/1.0" 200 100 "-" "Mozilla"
another random line
yet another line
final line
"#;
        let result = detect_from_str(log);
        assert_eq!(result.format, LogFormat::PlainText);
    }
}
