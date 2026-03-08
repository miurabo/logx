# 要求内容

## 概要

`logx errors <file>` コマンドでログファイルからエラーを抽出・表示する機能（F2）を実装する。Analyzer（エラー判定・集計）とRenderer（カラー出力）も含む。

## 背景

F1でフォーマット検出とパーサーの基盤が完成した。F2はユーザーが最初に使う主要機能であり、「ログファイルを渡すだけでエラーが見える」というプロダクトコンセプトの中核。

## 実装対象の機能

### 1. Analyzer（エラー判定・集計）

- LogEntryがエラーかどうか判定する `is_error()` 関数
- エラーをカテゴリ別にグループ化してサマリーを生成する `summarize_errors()`
- `--since` オプション用の duration パース

### 2. Renderer（出力整形）

- エラー一覧のカラー表示
- エラーサマリー（カテゴリ別件数）の表示
- `--no-color` 対応

### 3. errors サブコマンド本体

- ファイルを開き、FormatDetector → LogParser → Analyzer → Renderer のパイプライン実行
- `--since` による時間フィルタリング
- 複数ファイル対応

## 受け入れ条件

### エラー判定

- [ ] HTTPステータスコード 4xx/5xx をエラーとして検出できる
- [ ] ログレベル ERROR/FATAL/CRITICAL をエラーとして検出できる
- [ ] メッセージ中の "fatal error", "exception", "stack trace", "panic" を検出できる

### 出力

- [ ] `logx errors <file>` でエラー行を一覧表示できる
- [ ] エラー件数のサマリーを表示できる（エラー種別ごとの件数）
- [ ] カラー表示される（5xx=赤、4xx=黄など）
- [ ] `--since 1h` で時間範囲を絞り込める

### 堅牢性

- [ ] 空ファイルでクラッシュしない
- [ ] エラーが0件の場合は適切なメッセージを表示する
- [ ] ファイルが見つからない場合はエラーメッセージを表示する

## 成功指標

- `cargo test` 全パス
- `cargo clippy -- -D warnings` で警告なし
- テストフィクスチャでの動作確認

## スコープ外

- scan / filter サブコマンドの本体（F3, F4）
- HTML/Markdown レポート出力（Post-MVP）
- リアルタイムモニタリング（Post-MVP）

## 参照ドキュメント

- `docs/product-requirements.md` - F2の受け入れ条件
- `docs/functional-design.md` - Analyzer, Renderer のインターフェース、UI設計
- `docs/architecture.md` - パイプラインアーキテクチャ
