# ADR-0003: Agent shared モードによる複数ターミナルからの利用

## ステータス

承認済み（実装中）

## 日付

2026-03-18

## コンテキスト

- cloudapps-cli の agent は ssh-agent モデルを採用しています
- `eval "$(cloudapps-cli agent start)"` で環境変数を設定します
- 別ターミナルからは手動でソケットパスとトークンをコピーする必要があります
- tmux、iTerm2 タブ等の複数ターミナルワークフローで不便です

## 決定

### `--shared` オプションの導入

- `cloudapps-cli agent start --shared` で session.json に書き出します
- eval 不要で、session.json から自動検出します
- watchdog のセッションリーダー監視を無効化します（アイドルタイムアウトは維持します）

### session.json の仕様

- パス: `$XDG_DATA_HOME/cloudapps-cli/session.json`（デフォルト: `~/.local/share/cloudapps-cli/session.json`）
- パーミッション: 0600（ディレクトリ: 0700）
- 内容: `socket_path`, `token`, `pid`, `started_at`

### コマンド実行時の優先順位

1. `--no-agent` フラグ → direct mode
2. `CLOUDAPPS_AGENT_TOKEN` 環境変数 → eval モードの agent 経由
3. session.json → shared モードの agent 経由
4. `CLOUDAPPS_API_TOKEN` 環境変数 → direct mode

### `--no-agent` フラグ

- session.json が存在しても agent 経由を抑制します

## 影響範囲

| ファイル | 変更内容 |
|----------|----------|
| `src/cli/agent.rs` | `AgentCommand::Start` に `--shared` フラグを追加します |
| `src/cli/mod.rs` | `Cli` に `--no-agent` フラグを追加します |
| `src/agent/session.rs` | session.json の読み書き・削除を実装します |
| `src/agent/mod.rs` | session モジュールを公開します |
| `src/agent/server.rs` | shared モード時の分岐、session.json の書き出し・削除を行います |
| `src/agent/client.rs` | stop 時の session.json クリーンアップを行います |
| `src/main.rs` | session.json フォールバック、`--no-agent` 処理を実装します |

## セキュリティ考慮事項

### リスク

- session.json にトークンがディスク保存されます

### 緩和策

- 0600 パーミッションを設定します
- 既存の UID/ピア検証は維持します
- agent 停止/タイムアウト時に session.json を削除します

### トレードオフ

- ユーザーが `--shared` で明示的に選択します。デフォルト動作は変更しません。
