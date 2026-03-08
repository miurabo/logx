# リポジトリ構造定義書 (Repository Structure Document)

## プロジェクト構造

```text
logx/
├── src/                       # ソースコード
│   ├── main.rs                # エントリーポイント
│   ├── cli.rs                 # CLIレイヤー（clap定義）
│   ├── detector.rs            # フォーマット自動検出
│   ├── parser/                # ログパーサー
│   │   ├── mod.rs             # LogParser trait定義
│   │   ├── apache.rs          # Apache Combined/Common
│   │   ├── nginx.rs           # Nginx
│   │   ├── json_lines.rs      # JSON Lines
│   │   ├── syslog.rs          # Syslog
│   │   └── plain_text.rs      # プレーンテキスト（フォールバック）
│   ├── analyzer.rs            # 分析・フィルタリング
│   ├── renderer.rs            # 出力整形・カラー表示
│   └── types.rs               # 共通型定義（LogEntry, ScanResult等）
├── tests/                     # 統合テスト・E2Eテスト
│   ├── cli_test.rs            # CLIコマンドのE2Eテスト
│   ├── scan_test.rs           # scanサブコマンドのテスト
│   ├── errors_test.rs         # errorsサブコマンドのテスト
│   ├── filter_test.rs         # filterサブコマンドのテスト
│   └── fixtures/              # テスト用サンプルログファイル
│       ├── apache_combined.log
│       ├── apache_common.log
│       ├── nginx.log
│       ├── json_lines.log
│       ├── syslog.log
│       ├── mixed_errors.log
│       └── empty.log
├── docs/                      # プロジェクトドキュメント
│   ├── ideas/                 # アイデア・壁打ちメモ
│   ├── product-requirements.md
│   ├── functional-design.md
│   ├── architecture.md
│   ├── repository-structure.md
│   ├── development-guidelines.md
│   └── glossary.md
├── .steering/                 # 作業単位のドキュメント
├── .claude/                   # Claude Code設定
├── Cargo.toml                 # プロジェクト設定・依存関係
├── Cargo.lock                 # 依存関係のロックファイル
├── .gitignore                 # Git除外設定
├── LICENSE                    # ライセンス
└── README.md                  # プロジェクト説明
```

## ディレクトリ詳細

### src/ (ソースコードディレクトリ)

#### main.rs

**役割**: エントリーポイント。CLIの初期化と実行

**配置ファイル**:

- `main.rs`: `fn main()` のみ。cli::run() を呼び出す

#### cli.rs

**役割**: clapによるCLI定義、サブコマンドのディスパッチ

**配置ファイル**:

- `cli.rs`: Args構造体（derive(Parser)）、run関数

**依存関係**:

- 依存可能: detector, parser, analyzer, renderer, types
- 依存禁止: なし（最上位レイヤー）

#### detector.rs

**役割**: ログファイルのフォーマット自動検出

**配置ファイル**:

- `detector.rs`: FormatDetector構造体、detect関数

**依存関係**:

- 依存可能: types
- 依存禁止: cli, parser, analyzer, renderer

#### parser/

**役割**: フォーマット別のログ行パーサー

**配置ファイル**:

- `mod.rs`: LogParser trait定義、ファクトリ関数
- `apache.rs`: Apache Combined/Commonパーサー
- `nginx.rs`: Nginxパーサー
- `json_lines.rs`: JSON Linesパーサー
- `syslog.rs`: Syslogパーサー
- `plain_text.rs`: プレーンテキストパーサー（フォールバック）

**命名規則**:

- フォーマット名をsnake_caseで使用
- 各ファイルに1つのパーサー構造体

**依存関係**:

- 依存可能: types
- 依存禁止: cli, detector, analyzer, renderer

#### analyzer.rs

**役割**: フィルタリング・集計・エラー判定

**依存関係**:

- 依存可能: types
- 依存禁止: cli, detector, parser, renderer

#### renderer.rs

**役割**: ターミナル出力の整形・カラー表示

**依存関係**:

- 依存可能: types
- 依存禁止: cli, detector, parser, analyzer

#### types.rs

**役割**: 全コンポーネントで共有する型定義

**配置ファイル**:

- `types.rs`: LogEntry, LogFormat, LogLevel, ScanResult, FilterOptions等

**依存関係**:

- 依存可能: なし（最下位レイヤー）
- 依存禁止: 他の全モジュール

### tests/ (テストディレクトリ)

#### 統合テスト・E2Eテスト

**構造**:

```text
tests/
├── cli_test.rs           # --help、--version、不正な引数
├── scan_test.rs          # logx scan の入出力
├── errors_test.rs        # logx errors の入出力
├── filter_test.rs        # logx filter の入出力
└── fixtures/             # テスト用サンプルログ
```

**命名規則**:

- パターン: `[サブコマンド名]_test.rs`
- フィクスチャ: `[フォーマット名].log`

#### ユニットテスト

Rustの慣例に従い、各ソースファイル内に `#[cfg(test)] mod tests` で配置する。

```rust
// src/detector.rs 内
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_apache_combined() { /* ... */ }
}
```

### docs/ (ドキュメントディレクトリ)

**配置ドキュメント**:

- `product-requirements.md`: プロダクト要求定義書
- `functional-design.md`: 機能設計書
- `architecture.md`: 技術仕様書
- `repository-structure.md`: リポジトリ構造定義書（本ドキュメント）
- `development-guidelines.md`: 開発ガイドライン
- `glossary.md`: 用語集
- `ideas/`: アイデア・壁打ちメモ

## ファイル配置規則

### ソースファイル

| ファイル種別 | 配置先 | 命名規則 | 例 |
|------------|--------|---------|-----|
| エントリーポイント | `src/` | `main.rs` | `main.rs` |
| モジュール | `src/` | snake_case | `detector.rs`, `analyzer.rs` |
| サブモジュール | `src/[module]/` | snake_case | `parser/apache.rs` |
| 型定義 | `src/` | `types.rs` | `types.rs` |

### テストファイル

| テスト種別 | 配置先 | 命名規則 | 例 |
|-----------|--------|---------|-----|
| ユニットテスト | 各ソースファイル内 | `#[cfg(test)] mod tests` | `detector.rs` 内 |
| 統合テスト | `tests/` | `[対象]_test.rs` | `scan_test.rs` |
| テストフィクスチャ | `tests/fixtures/` | `[フォーマット名].log` | `apache_combined.log` |

## 命名規則

### ディレクトリ名

- snake_case（Rustの慣例に準拠）
- 例: `parser/`, `tests/fixtures/`

### ファイル名

- **Rustソースファイル**: snake_case
  - 例: `json_lines.rs`, `plain_text.rs`
- **設定ファイル**: ツール標準の命名
  - 例: `Cargo.toml`, `.gitignore`
- **ドキュメント**: kebab-case
  - 例: `product-requirements.md`, `functional-design.md`

### Rust命名規則

- **構造体・列挙型・トレイト**: PascalCase（`FormatDetector`, `LogEntry`）
- **関数・メソッド・変数**: snake_case（`detect_format`, `line_number`）
- **定数**: UPPER_SNAKE_CASE（`MAX_SAMPLE_LINES`, `DEFAULT_BUFFER_SIZE`）
- **モジュール**: snake_case（`json_lines`, `plain_text`）

## 依存関係のルール

### モジュール間の依存

```text
cli
 ├── detector
 ├── parser/* (LogParser trait経由)
 ├── analyzer
 ├── renderer
 └── types

detector → types
parser/* → types
analyzer → types
renderer → types
types → (依存なし)
```

**禁止される依存**:

- types → 他の全モジュール（循環依存防止）
- renderer → analyzer（表示ロジックに分析ロジックを混ぜない）
- parser → detector（パーサーは検出結果に依存しない）

## スケーリング戦略

### 新しいログフォーマットの追加

1. `src/parser/` に新しいファイルを追加（例: `cloudwatch.rs`）
2. `LogParser` traitを実装
3. `src/parser/mod.rs` にモジュール登録
4. `src/detector.rs` に検出パターンを追加
5. `tests/fixtures/` にサンプルログを追加

### ファイルサイズの管理

- 1ファイル: 300行以下を推奨
- 300行超: 機能分割を検討
- parser/のように、1ファイル1パーサーの原則を維持

## 除外設定

### .gitignore

```text
# ビルド成果物
/target/

# IDE設定
.idea/
.vscode/
*.swp
*.swo

# OS固有
.DS_Store

# 環境変数
.env

# ステアリングファイル
.steering/
```
