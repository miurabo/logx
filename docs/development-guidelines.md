# 開発ガイドライン (Development Guidelines)

## コーディング規約

### 命名規則

**Rust命名規則**:

```rust
// ✅ 良い例
struct FormatDetector;                    // 構造体: PascalCase
enum LogLevel { Error, Warn, Info }       // 列挙型: PascalCase
trait LogParser { }                       // トレイト: PascalCase
fn detect_format(line: &str) -> LogFormat // 関数: snake_case
let line_number: usize = 0;              // 変数: snake_case
const MAX_SAMPLE_LINES: usize = 20;      // 定数: UPPER_SNAKE_CASE
mod json_lines;                           // モジュール: snake_case

// ❌ 悪い例
struct format_detector;                   // 構造体にsnake_case
fn DetectFormat(line: &str) {}            // 関数にPascalCase
let LineNumber = 0;                       // 変数にPascalCase
```

**Boolean変数**: `is_`, `has_`, `should_` で始める

```rust
let is_error = status_code >= 400;
let has_timestamp = entry.timestamp.is_some();
let should_skip = line.is_empty();
```

### コードフォーマット

- **フォーマッター**: `rustfmt`（デフォルト設定）
- **行の長さ**: 最大100文字
- **インデント**: 4スペース（Rust標準）

### コメント規約

**ドキュメンテーションコメント**:

```rust
/// ログファイルのフォーマットを自動検出する
///
/// ファイルの先頭20行を読み取り、各フォーマットの正規表現で
/// マッチングを行い、最も適合率の高いフォーマットを返す。
///
/// # Arguments
///
/// * `reader` - ログファイルのBufReader
///
/// # Returns
///
/// 検出結果（フォーマット、信頼度、検査行数）
///
/// # Examples
///
/// ```
/// let result = FormatDetector::detect(&mut reader);
/// println!("Format: {:?}, Confidence: {}", result.format, result.confidence);
/// ```
pub fn detect(reader: &mut impl BufRead) -> DetectionResult {
    // ...
}
```

**インラインコメント**:

```rust
// ✅ 良い例: なぜそうするかを説明
// Apache CombinedはCommonのスーパーセットなので、先にCombinedを試す
if apache_combined_score > 0.0 { /* ... */ }

// ❌ 悪い例: コードを見れば分かること
// スコアが0より大きいかチェック
if apache_combined_score > 0.0 { /* ... */ }
```

### エラーハンドリング

**原則**:

- `unwrap()` / `expect()` はテストコードのみで使用
- アプリケーションコードでは `Result<T, E>` で伝播
- ライブラリ層は `thiserror`、アプリケーション層は `anyhow` を使用

```rust
// ライブラリ層: 具体的なエラー型を定義
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Malformed log line at line {line}: {reason}")]
    MalformedLine { line: usize, reason: String },
}

// アプリケーション層: anyhow で簡潔にハンドリング
fn run(args: Args) -> anyhow::Result<()> {
    let file = File::open(&args.path)
        .with_context(|| format!("Failed to open: {}", args.path))?;
    // ...
    Ok(())
}
```

## Git運用ルール

### ブランチ戦略

個人開発フェーズのため、シンプルなブランチ戦略を採用:

- `main`: リリース可能な状態
- `feature/[機能名]`: 新機能開発
- `fix/[修正内容]`: バグ修正

```text
main
  ├── feature/format-detection
  ├── feature/error-extraction
  └── fix/apache-timestamp-parse
```

### コミットメッセージ規約

Conventional Commits フォーマット:

```text
<type>: <description>

[optional body]
```

**Type**: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert

**例**:

```text
feat: Apache Combinedフォーマットの自動検出を実装

先頭20行の正規表現マッチングでフォーマットを推定する。
適合率50%未満の場合はPlainTextにフォールバックする。
```

### プルリクエストプロセス

**作成前のチェック**:

- [ ] `cargo test` が全てパス
- [ ] `cargo clippy` の警告がない
- [ ] `cargo fmt --check` が通る

## テスト戦略

### テストの種類

#### ユニットテスト

**対象**: 各モジュールの関数・メソッド
**カバレッジ目標**: 80%以上
**配置**: 各ソースファイル内の `#[cfg(test)] mod tests`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_error_returns_true_for_500() {
        let entry = LogEntry {
            status_code: Some(500),
            ..Default::default()
        };
        assert!(Analyzer::is_error(&entry));
    }

    #[test]
    fn is_error_returns_false_for_200() {
        let entry = LogEntry {
            status_code: Some(200),
            ..Default::default()
        };
        assert!(!Analyzer::is_error(&entry));
    }
}
```

#### 統合テスト

**対象**: サブコマンド単位の入出力
**配置**: `tests/` ディレクトリ

```rust
// tests/scan_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn scan_apache_combined_log() {
    Command::cargo_bin("logx")
        .unwrap()
        .args(["scan", "tests/fixtures/apache_combined.log"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Apache Combined"))
        .stdout(predicate::str::contains("Lines:"));
}
```

### テスト命名規則

**パターン**: `[対象]_[条件]_[期待結果]`（snake_case）

```rust
// ✅ 良い例
#[test]
fn detect_returns_apache_combined_for_valid_log() {}
#[test]
fn parse_line_returns_none_for_malformed_input() {}
#[test]
fn is_error_returns_true_for_5xx_status() {}

// ❌ 悪い例
#[test]
fn test1() {}
#[test]
fn it_works() {}
```

## コードレビュー基準

### レビューポイント

**機能性**:

- [ ] PRDの受け入れ条件を満たしているか
- [ ] エッジケースが考慮されているか（空ファイル、壊れた行、巨大ファイル）
- [ ] エラーハンドリングが適切か（`unwrap()` が本番コードにないか）

**Rust固有**:

- [ ] 所有権・ライフタイムが適切か
- [ ] 不要な `clone()` がないか
- [ ] `clippy` の警告が解消されているか

**パフォーマンス**:

- [ ] ストリーミング処理が維持されているか（メモリにファイル全体を読み込んでいないか）
- [ ] 正規表現がループ内でコンパイルされていないか

## 開発環境セットアップ

### 必要なツール

| ツール | バージョン | インストール方法 |
|--------|-----------|-----------------|
| Rust | 1.83+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| cargo-nextest | latest | `cargo install cargo-nextest` |
| cargo-llvm-cov | latest | `cargo install cargo-llvm-cov` |

### セットアップ手順

```bash
# 1. リポジトリのクローン
git clone https://github.com/miurabo/logx.git
cd logx

# 2. ビルド
cargo build

# 3. テスト実行
cargo nextest run

# 4. リンター実行
cargo clippy -- -D warnings

# 5. フォーマットチェック
cargo fmt --check
```

### よく使うコマンド

```bash
cargo run -- scan /path/to/log        # 開発中の実行
cargo test                             # テスト実行
cargo clippy -- -D warnings            # lint
cargo fmt                              # フォーマット
cargo llvm-cov                         # カバレッジ計測
```
