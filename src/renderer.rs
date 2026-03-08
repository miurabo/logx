use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::types::{ErrorSummary, LogEntry};

pub struct Renderer {
    color_choice: ColorChoice,
}

impl Renderer {
    pub fn new(use_color: bool) -> Self {
        let color_choice = if use_color && std::env::var("NO_COLOR").is_err() {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };
        Self { color_choice }
    }

    /// エラー一覧とサマリーを表示する
    pub fn render_errors(
        &self,
        errors: &[LogEntry],
        summary: &[ErrorSummary],
        since_label: Option<&str>,
    ) {
        let mut stdout = StandardStream::stdout(self.color_choice);

        // ヘッダー
        let header = if let Some(label) = since_label {
            format!("── Errors (last {label}) ")
        } else {
            "── Errors ".to_string()
        };
        self.write_bold(&mut stdout, &format!("{header:─<50}"));
        writeln!(stdout).ok();

        if errors.is_empty() {
            writeln!(stdout, "No errors found.").ok();
            return;
        }

        writeln!(stdout, "Found {} errors", errors.len()).ok();
        writeln!(stdout).ok();

        // エラー一覧
        for entry in errors {
            self.render_error_line(&mut stdout, entry);
        }

        // サマリー
        if !summary.is_empty() {
            writeln!(stdout).ok();
            self.write_bold(&mut stdout, "Summary:");
            writeln!(stdout).ok();
            for s in summary {
                writeln!(stdout, "  {:<30} {}", s.category, s.count).ok();
            }
        }
    }

    /// エラー0件のメッセージを表示する
    #[allow(dead_code)]
    pub fn render_no_errors(&self) {
        let mut stdout = StandardStream::stdout(self.color_choice);
        self.write_bold(
            &mut stdout,
            "── Errors ────────────────────────────────────────",
        );
        writeln!(stdout).ok();
        writeln!(stdout, "No errors found.").ok();
    }

    /// 警告メッセージを表示する
    #[allow(dead_code)]
    pub fn render_warning(&self, message: &str) {
        let mut stderr = StandardStream::stderr(self.color_choice);
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))
            .ok();
        write!(stderr, "Warning").ok();
        stderr.reset().ok();
        writeln!(stderr, ": {message}").ok();
    }

    fn render_error_line(&self, stdout: &mut StandardStream, entry: &LogEntry) {
        // ステータスコード部分
        if let Some(code) = entry.status_code {
            let color = if code >= 500 {
                Color::Red
            } else if code >= 400 {
                Color::Yellow
            } else {
                Color::White
            };
            stdout.set_color(ColorSpec::new().set_fg(Some(color))).ok();
            write!(stdout, "[{code}]").ok();
            stdout.reset().ok();
            write!(stdout, " ").ok();
        } else if let Some(ref level) = entry.level {
            let color = match level {
                crate::types::LogLevel::Error
                | crate::types::LogLevel::Fatal
                | crate::types::LogLevel::Critical => Color::Red,
                _ => Color::White,
            };
            stdout.set_color(ColorSpec::new().set_fg(Some(color))).ok();
            write!(stdout, "[{level}]").ok();
            stdout.reset().ok();
            write!(stdout, " ").ok();
        }

        // タイムスタンプ
        if let Some(ts) = entry.timestamp {
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(245))))
                .ok();
            write!(stdout, "{}", ts.format("%Y-%m-%d %H:%M:%S")).ok();
            stdout.reset().ok();
            write!(stdout, "  ").ok();
        }

        // メソッド + パス
        if let Some(ref method) = entry.method {
            write!(stdout, "{method:<5}").ok();
        }
        if let Some(ref path) = entry.path {
            write!(stdout, " {path}").ok();
        }

        // メッセージ（メソッド/パスがない場合のみ）
        if entry.method.is_none()
            && entry.path.is_none()
            && let Some(ref msg) = entry.message
        {
            let truncated = if msg.len() > 100 {
                format!("{}...", &msg[..97])
            } else {
                msg.clone()
            };
            write!(stdout, "{truncated}").ok();
        }

        writeln!(stdout).ok();
    }

    fn write_bold(&self, stream: &mut StandardStream, text: &str) {
        stream.set_color(ColorSpec::new().set_bold(true)).ok();
        write!(stream, "{text}").ok();
        stream.reset().ok();
    }
}
