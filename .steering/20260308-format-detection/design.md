# 設計書

## アーキテクチャ概要

パイプライン・アーキテクチャの最下層3レイヤー（types, detector, parser）を構築する。CLIレイヤーはスケルトンのみ。

```text
CLI (clap) ── スケルトンのみ、サブコマンド定義
  │
  ├── FormatDetector ── 先頭20行でフォーマット推定
  │
  ├── LogParser trait ── フォーマット別パーサー
  │     ├── ApacheCombinedParser
  │     ├── ApacheCommonParser
  │     ├── NginxParser
  │     ├── JsonLinesParser
  │     ├── SyslogParser
  │     └── PlainTextParser
  │
  └── types ── LogEntry, LogFormat, LogLevel 等
```

## コンポーネント設計

### 1. types（共通型定義）

**責務**:

- 全コンポーネントで共有するデータ型を定義
- 依存なし（最下位レイヤー）

**実装の要点**:

- `LogFormat`: 6種類のログフォーマット列挙型
- `LogLevel`: Debug〜Critical の6段階
- `LogEntry`: パース済み1行データ（timestamp, status_code, path, ip, level, message, raw, line_number）
- `DetectionResult`: フォーマット検出結果（format, confidence, sample_lines）

### 2. FormatDetector（フォーマット自動検出）

**責務**:

- ファイルの先頭20行を読み取り、フォーマットを推定する
- 各フォーマットの正規表現でスコアリング

**実装の要点**:

- `std::sync::LazyLock` で正規表現をプリコンパイル（Edition 2024なのでonce_cell不要）
- 各フォーマットのマッチ率（適合率）を計算し、最高スコアのフォーマットを返す
- 適合率50%未満はPlainTextにフォールバック
- 同スコア時の優先順位: ApacheCombined > Nginx > JsonLines > Syslog > PlainText

### 3. LogParser trait + 各パーサー

**責務**:

- `parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry>`
- 1行をLogEntryに変換。パース失敗時はNoneを返す

**実装の要点**:

- 各パーサーは正規表現で行をキャプチャし、フィールドを抽出
- `create_parser(format: &LogFormat) -> Box<dyn LogParser>` ファクトリ関数で生成
- Apache Combined: IP, timestamp, method, path, status, referer, user-agent
- Apache Common: IP, timestamp, method, path, status
- Nginx: Apache Combinedと同一フォーマット（デフォルト設定の場合）
- JSON Lines: serde_jsonでパース、一般的なフィールド名を探索
- Syslog: timestamp, hostname, process, message
- PlainText: タイムスタンプのヒューリスティック検出、ログレベルのパターンマッチ

### 4. CLI（スケルトン）

**責務**:

- clapでサブコマンド定義（scan, errors, filter）
- 引数パース、--help, --version

**実装の要点**:

- `#[derive(Parser)]` でArgs構造体を定義
- サブコマンド本体は `todo!()` ではなく、未実装メッセージを表示して正常終了

## データフロー

### フォーマット検出フロー

```text
1. CLIがファイルパスを受け取る
2. FormatDetector::detect() に BufReader を渡す
3. 先頭20行を読み取り、空行をスキップ
4. 各フォーマットの正規表現でマッチング
5. スコア最大のフォーマットを DetectionResult として返す
```

### パースフロー

```text
1. DetectionResult.format に基づき create_parser() でパーサー生成
2. BufReader で1行ずつ読み取り
3. parser.parse_line(line, line_number) → Option<LogEntry>
4. None の行はスキップ
```

## エラーハンドリング戦略

### カスタムエラー型

```rust
#[derive(Debug, thiserror::Error)]
pub enum LogxError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path} (try: sudo logx ...)")]
    PermissionDenied { path: String },

    #[error("File is empty: {path}")]
    EmptyFile { path: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

### エラーハンドリングパターン

- パースエラー: Noneを返してスキップ（クラッシュしない）
- ファイルI/Oエラー: LogxError で伝播し、CLIレイヤーでユーザーフレンドリーに表示
- 検出失敗: PlainTextにフォールバック（エラーにしない）

## テスト戦略

### ユニットテスト

- FormatDetector: 各フォーマットのサンプル行で正しく検出できること
- 各LogParser: 正常行のパース、異常行でNone返却
- LogLevel/LogFormat: Display, Debug等のtrait

### 統合テスト

- フィクスチャファイル（tests/fixtures/）を使ったフォーマット検出
- CLIの --help, --version が正常に動作

## 依存ライブラリ

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
regex = "1"
chrono = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

## ディレクトリ構造

```text
src/
├── main.rs              # エントリーポイント（cli::run()を呼ぶだけ）
├── cli.rs               # CLIレイヤー（clap定義、サブコマンド）
├── detector.rs          # FormatDetector
├── error.rs             # LogxError
├── parser/
│   ├── mod.rs           # LogParser trait, create_parser()
│   ├── apache.rs        # ApacheCombinedParser, ApacheCommonParser
│   ├── nginx.rs         # NginxParser
│   ├── json_lines.rs    # JsonLinesParser
│   ├── syslog.rs        # SyslogParser
│   └── plain_text.rs    # PlainTextParser
└── types.rs             # LogEntry, LogFormat, LogLevel, DetectionResult
tests/
├── cli_test.rs          # --help, --version のE2Eテスト
└── fixtures/
    ├── apache_combined.log
    ├── apache_common.log
    ├── nginx.log
    ├── json_lines.log
    ├── syslog.log
    └── empty.log
```

## 実装の順序

1. Cargo.toml に依存追加
2. types.rs（共通型定義）
3. error.rs（エラー型）
4. parser/mod.rs（LogParser trait）
5. parser/apache.rs, nginx.rs, json_lines.rs, syslog.rs, plain_text.rs
6. detector.rs（FormatDetector）
7. cli.rs（CLIスケルトン）
8. main.rs（エントリーポイント更新）
9. テストフィクスチャ作成
10. ユニットテスト・統合テスト

## セキュリティ考慮事項

- ログファイルは `O_RDONLY` で開く（BufReader::new(File::open()) は読み取り専用）
- 1行あたりのバッファ上限はBufReaderのデフォルト（8KB）で十分。異常行はパースエラーとしてスキップ

## パフォーマンス考慮事項

- 正規表現は `std::sync::LazyLock` でコンパイル済みキャッシュ（ループ内コンパイル禁止）
- FormatDetectorは先頭20行のみ読み取り（ファイル全体を読まない）
- BufReaderで1行ずつ処理（メモリにファイル全体を読み込まない）

## 将来の拡張性

- LogParser traitを実装するだけで新フォーマット追加可能
- FormatDetectorに新パターンを登録するだけで検出対応
- CLIサブコマンドの追加は独立して行える
