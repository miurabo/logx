# 要求内容

## 概要

ログファイルのフォーマットを自動検出する機能（F1）と、それに必要なプロジェクト基盤（共通型定義、CLIスケルトン、パーサー基盤）を実装する。

## 背景

logxのコア機能（scan, errors, filter）はすべてログフォーマットの自動検出に依存する。F1はlogxの土台であり、最初に実装すべき機能。また、初回実装のため、Cargo.tomlの依存追加やモジュール構成の構築も含む。

## 実装対象の機能

### 1. プロジェクト基盤

- Cargo.tomlに必要な依存クレートを追加
- 共通型定義（LogEntry, LogFormat, LogLevel等）
- CLIスケルトン（clap によるサブコマンド定義）
- エラー型の定義

### 2. ログフォーマット自動検出（FormatDetector）

- ファイルの先頭20行を読み取り、フォーマットを推定する
- 各フォーマットの正規表現パターンでスコアリング
- 検出結果（フォーマット、信頼度）を返す

### 3. ログパーサー基盤（LogParser trait + 各フォーマット実装）

- LogParser traitの定義
- Apache Combined / Common パーサー
- Nginx パーサー
- JSON Lines パーサー
- Syslog パーサー
- PlainText パーサー（フォールバック）

## 受け入れ条件

### フォーマット自動検出

- [ ] Apache Combined形式を自動検出できる
- [ ] Apache Common形式を自動検出できる
- [ ] Nginx形式を自動検出できる
- [ ] JSON Lines形式を自動検出できる
- [ ] Syslog形式を自動検出できる
- [ ] 未知のフォーマットはPlainTextにフォールバックする
- [ ] 適合率50%未満の場合はPlainTextになる

### ログパーサー

- [ ] 各フォーマットの正常行をパースしてLogEntryを返せる
- [ ] パースできない行はNoneを返し、クラッシュしない
- [ ] タイムスタンプ、ステータスコード、パス、IP等を抽出できる

### CLI

- [ ] `logx --help` でヘルプが表示される
- [ ] `logx --version` でバージョンが表示される
- [ ] scan, errors, filter サブコマンドの定義（中身は未実装でOK）

## 成功指標

- 上記4フォーマットの検出精度: テストスイートで95%以上
- `cargo test` が全パス
- `cargo clippy -- -D warnings` で警告なし

## スコープ外

以下はこのフェーズでは実装しません:

- scan, errors, filter サブコマンドの本体ロジック（F2〜F4）
- Analyzer（分析・フィルタリング）
- Renderer（出力整形・カラー表示）
- パフォーマンス最適化（正規表現のプリコンパイル等は行うが、mmap等は対象外）

## 参照ドキュメント

- `docs/product-requirements.md` - プロダクト要求定義書（F1の受け入れ条件）
- `docs/functional-design.md` - 機能設計書（データモデル、アルゴリズム設計）
- `docs/architecture.md` - 技術仕様書（レイヤー構成、依存関係）
- `docs/repository-structure.md` - リポジトリ構造定義書（ファイル配置）
- `docs/development-guidelines.md` - 開発ガイドライン（コーディング規約）
