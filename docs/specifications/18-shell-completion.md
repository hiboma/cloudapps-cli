# Shell Completion

`cloudapps-cli completion <shell>` は shell 補完スクリプトを標準出力に書き出します。`clap_complete` を使って実装しており、CLI 定義（`clap` derive macro）から自動生成されます。

## サポート対象シェル

| Shell       | 値            |
|-------------|---------------|
| Bash        | `bash`        |
| Zsh         | `zsh`         |
| Fish        | `fish`        |
| PowerShell  | `powershell`  |
| Elvish      | `elvish`      |

## 実装概要

- `src/cli/mod.rs`
  - `CompletionShell` enum（`ValueEnum` 派生）
  - `Commands::Completion { shell }` バリアント（`hide = true`）
- `src/main.rs`
  - `run()` の早い段階で `Commands::Completion` を検知し、認証情報の解決や agent ルーティングを行わずに `print_completion()` を呼び出します。
  - `print_completion()` は `clap_complete::generate` を対応 shell 向けに呼び出します。
- `src/help_for_ai.rs`
  - `Commands::Completion` 用の簡易 help 文字列を追加しています。

## 使い方

### zsh

```zsh
cloudapps-cli completion zsh > "${fpath[1]}/_cloudapps-cli"
autoload -U compinit && compinit
```

生成物は `#compdef cloudapps-cli` ヘッダーを持つ標準的な zsh completion ファイルです。

### bash

```bash
cloudapps-cli completion bash > /usr/local/etc/bash_completion.d/cloudapps-cli
```

### fish

```fish
cloudapps-cli completion fish > ~/.config/fish/completions/cloudapps-cli.fish
```

### PowerShell

```powershell
cloudapps-cli completion powershell | Out-String | Invoke-Expression
```

### Elvish

```elvish
cloudapps-cli completion elvish > ~/.config/elvish/lib/cloudapps-cli.elv
```

## 設計上の注意

- `Commands::Completion` は `hide = true` で、通常の `--help` 出力には現れません。`after_help` テキストに案内を掲載しています。
- 補完スクリプト生成には API 認証情報を必要としないため、`resolve_credentials` より前に分岐しています。
- shell 名の追加、リネームは `CompletionShell` enum と `print_completion()` の 2 か所を更新します。
