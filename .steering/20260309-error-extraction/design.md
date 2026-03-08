# 設計書

## アーキテクチャ概要

F1で構築した検出・パースレイヤーの上に、分析レイヤー（Analyzer）と表示レイヤー（Renderer）を追加する。

```text
CLI (errors サブコマンド)
  │
  ├── FormatDetector  ← F1で実装済み
  ├── LogParser       ← F1で実装済み
  ├── Analyzer        ← 今回実装
  │     ├── is_error()
  │     ├── summarize_errors()
  │     └── parse_duration()
  └── Renderer        ← 今回実装
        └── render_errors()
```

## コンポーネント設計

### 1. Analyzer（分析）

**責務**:

- LogEntryがエラーかどうか判定する
- エラーをカテゴリ別に集計する
- durationパース（"1h" → Duration）

**実装の要点**:

```rust
pub fn is_error(entry: &LogEntry) -> bool {
    // 1. status_code >= 400
    // 2. level が Error/Fatal/Critical
    // 3. message に "fatal error", "exception", "stack trace", "panic" を含む
}

pub fn summarize_errors(entries: &[LogEntry]) -> Vec<ErrorSummary> {
    // エラー種別でグループ化し、件数・初回/最終時刻を集計
}

pub fn parse_duration(s: &str) -> Result<chrono::Duration> {
    // "1h" → 1時間, "30m" → 30分, "2d" → 2日, "90s" → 90秒
}
```

### 2. Renderer（表示）

**責務**:

- エラー一覧をカラーで表示
- サマリー（エラー種別ごとの件数）を表示

**実装の要点**:

- termcolor を使用してクロスプラットフォーム対応
- NO_COLOR 環境変数に対応
- 5xx=赤、4xx=黄、ログレベルError系=赤

### 3. errors サブコマンド

**実装の要点**:

- ファイルオープン → FormatDetector::detect() → create_parser()
- BufReaderで1行ずつ読み取り、parse_line() → is_error() でフィルタ
- --since があればタイムスタンプでフィルタ
- Rendererでエラー一覧 + サマリーを表示

## データフロー

### logx errors app.log --since 1h

```text
1. ファイルオープン（存在確認、権限確認）
2. FormatDetector::detect() でフォーマット推定
3. create_parser(format) でパーサー生成
4. BufReaderで1行ずつ:
   a. parser.parse_line(line, line_number) → Option<LogEntry>
   b. Analyzer::is_error(&entry) → bool
   c. --since フィルタ（entry.timestamp >= now - duration）
   d. エラーならリストに追加
5. Analyzer::summarize_errors(&errors) → Vec<ErrorSummary>
6. Renderer::render_errors(&errors, &summary)
```

## エラーハンドリング戦略

- ファイル不在: `LogxError::FileNotFound` → エラーメッセージ表示して終了
- 権限不足: `LogxError::PermissionDenied` → "try: sudo logx ..." を提案
- 空ファイル: 警告メッセージを表示して正常終了
- パース不能行: スキップ（エラーにしない）
- 不正な --since: エラーメッセージ表示して終了

## テスト戦略

### ユニットテスト

- `is_error`: 各条件の境界値テスト
- `summarize_errors`: 複数エラー種別のグループ化
- `parse_duration`: 各単位（s, m, h, d）のパース、不正入力

### 統合テスト

- フィクスチャファイルでの `logx errors` 実行
- `--since` フィルタの動作
- エラー0件の場合のメッセージ
- 空ファイルの処理

## 依存ライブラリ

```toml
# 既存に追加
termcolor = "1"
```

## ディレクトリ構造

```text
src/
├── analyzer.rs     ← 新規作成
├── renderer.rs     ← 新規作成
├── cli.rs          ← errors サブコマンド実装
└── ...             ← 既存ファイル
tests/
├── errors_test.rs  ← 新規作成
└── fixtures/
    └── mixed_errors.log  ← 新規作成（複数エラー種別混在）
```

## 実装の順序

1. Cargo.toml に termcolor 追加
2. src/analyzer.rs（is_error, summarize_errors, parse_duration）
3. types.rs に ErrorSummary 追加
4. src/renderer.rs（render_errors）
5. src/cli.rs の errors サブコマンド実装
6. main.rs の dead_code allow 削除（使用されるようになるため）
7. テストフィクスチャ追加
8. ユニットテスト・統合テスト

## セキュリティ考慮事項

- ログファイルは読み取り専用（O_RDONLY）
- エラーメッセージにログ内容を含めない

## パフォーマンス考慮事項

- ストリーミング処理を維持（1行ずつ処理、エラー行のみメモリに保持）
- --since で時間範囲外の行をスキップ（早期終了はログが時系列順の場合のみ）
