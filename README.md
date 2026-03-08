# logx

設定不要のログ調査CLIツール。

ログファイルを渡すだけで、フォーマットを自動検出し、エラー抽出・概要表示・フィルタリングができます。

## インストール

```bash
cargo install logx
```

## 使い方

```bash
# ログファイルの概要を表示
logx scan /var/log/apache2/access.log

# エラーだけ抽出
logx errors /var/log/app/error.log
logx errors /var/log/app/*.log --since 1h

# 条件で絞り込み
logx filter /var/log/apache2/access.log --status 500
logx filter /var/log/app.log --level error --since 30m
```

## 対応フォーマット

- Apache Combined / Common
- Nginx
- JSON Lines
- Syslog
- プレーンテキスト（フォールバック）

## ライセンス

Apache-2.0
