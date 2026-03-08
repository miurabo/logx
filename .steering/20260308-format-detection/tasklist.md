# タスクリスト

## 🚨 タスク完全完了の原則

**このファイルの全タスクが完了するまで作業を継続すること**

### 必須ルール

- **全てのタスクを`[x]`にすること**
- 「時間の都合により別タスクとして実施予定」は禁止
- 「実装が複雑すぎるため後回し」は禁止
- 未完了タスク（`[ ]`）を残したまま作業を終了しない

### タスクスキップが許可される唯一のケース

以下の技術的理由に該当する場合のみスキップ可能:

- 実装方針の変更により、機能自体が不要になった
- アーキテクチャ変更により、別の実装方法に置き換わった
- 依存関係の変更により、タスクが実行不可能になった

スキップ時は必ず理由を明記:

```markdown
- [x] ~~タスク名~~（実装方針変更により不要: 具体的な技術的理由）
```

---

## フェーズ1: プロジェクト基盤

- [x] Cargo.toml に依存クレートを追加
- [x] src/types.rs を作成（LogFormat, LogLevel, LogEntry, DetectionResult）
- [x] src/error.rs を作成（LogxError）
- [x] src/main.rs を更新（モジュール宣言、cli::run()呼び出し）

## フェーズ2: パーサー実装

- [x] src/parser/mod.rs を作成（LogParser trait, create_parser()）
- [x] src/parser/apache.rs を作成（ApacheCombinedParser, ApacheCommonParser）
  - [x] Apache Combined の正規表現パターンとパースロジック
  - [x] Apache Common の正規表現パターンとパースロジック
  - [x] ユニットテスト
- [x] src/parser/nginx.rs を作成（NginxParser）
  - [x] Nginx の正規表現パターンとパースロジック
  - [x] ユニットテスト
- [x] src/parser/json_lines.rs を作成（JsonLinesParser）
  - [x] serde_jsonによるパースとフィールド探索ロジック
  - [x] ユニットテスト
- [x] src/parser/syslog.rs を作成（SyslogParser）
  - [x] Syslog の正規表現パターンとパースロジック
  - [x] ユニットテスト
- [x] src/parser/plain_text.rs を作成（PlainTextParser）
  - [x] タイムスタンプのヒューリスティック検出
  - [x] ログレベルのパターンマッチ
  - [x] ユニットテスト

## フェーズ3: フォーマット検出

- [x] src/detector.rs を作成（FormatDetector）
  - [x] 先頭20行のサンプリングロジック
  - [x] 各フォーマットの正規表現パターン（LazyLock）
  - [x] スコアリングアルゴリズム（適合率計算、優先順位）
  - [x] ユニットテスト

## フェーズ4: CLIスケルトン

- [x] src/cli.rs を作成（clap derive）
  - [x] Args構造体（scan, errors, filter サブコマンド）
  - [x] run() 関数（サブコマンドのディスパッチ、未実装メッセージ）

## フェーズ5: テストフィクスチャと統合テスト

- [x] tests/fixtures/ にサンプルログファイルを作成
  - [x] apache_combined.log
  - [x] apache_common.log
  - [x] nginx.log
  - [x] json_lines.log
  - [x] syslog.log
  - [x] empty.log
- [x] tests/cli_test.rs を作成（E2Eテスト）
  - [x] --help の出力テスト
  - [x] --version の出力テスト
  - [x] 各サブコマンドの存在テスト

## フェーズ6: 品質チェック

- [x] `cargo test` が全パス
- [x] `cargo clippy -- -D warnings` で警告なし
- [x] `cargo fmt --check` が通る

---

## 実装後の振り返り

### 実装完了日

{YYYY-MM-DD}

### 計画と実績の差分

**計画と異なった点**:

- {計画時には想定していなかった技術的な変更点}
- {実装方針の変更とその理由}

**新たに必要になったタスク**:

- {実装中に追加したタスク}
- {なぜ追加が必要だったか}

**技術的理由でスキップしたタスク**（該当する場合のみ）:

- {タスク名}
  - スキップ理由: {具体的な技術的理由}
  - 代替実装: {何に置き換わったか}

**⚠️ 注意**: 「時間の都合」「難しい」などの理由でスキップしたタスクはここに記載しないこと。全タスク完了が原則。

### 学んだこと

**技術的な学び**:

- {実装を通じて学んだ技術的な知見}
- {新しく使った技術やパターン}

**プロセス上の改善点**:

- {タスク管理で良かった点}
- {ステアリングファイルの活用方法}

### 次回への改善提案

- {次回の機能追加で気をつけること}
- {より効率的な実装方法}
- {タスク計画の改善点}
