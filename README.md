# ☁️ cloudapps-cli

A CLI tool for the [Microsoft Defender for Cloud Apps REST API](https://learn.microsoft.com/en-us/defender-cloud-apps/api-introduction), written in Rust.

## Installation

### Homebrew (macOS)

```bash
brew tap hiboma/tap
brew install cloudapps-cli
```

### From GitHub Releases

Download pre-built binaries from the [Releases](https://github.com/hiboma/cloudapps-cli/releases) page.

Available platforms:
- Linux (x86_64, aarch64)
- macOS (x86_64, aarch64)
- Windows (x86_64)

### From Source

```bash
cargo install --git https://github.com/hiboma/cloudapps-cli.git
```

## Configuration

### Environment Variables

Set your API token via environment variable or CLI options.

```bash
export CLOUDAPPS_API_TOKEN="your-api-token"
export CLOUDAPPS_API_URL="https://your-tenant.us3.portal.cloudappsecurity.com"
```

CLI options (`--api-url`) override environment variables.

> **Note:** `CLOUDAPPS_API_TOKEN` has no CLI flag to prevent exposure in process lists. Use environment variables, `.env` files, or `credentials.toml`. Ensure these files have restrictive permissions (`chmod 600`).

### Credentials File (TOML)

You can configure credentials using a `credentials.toml` file. The first file found is used (files are not merged):

1. `./.cloudapps-credentials.toml` (project-local)
2. `$XDG_CONFIG_HOME/cloudapps-cli/credentials.toml` (default: `~/.config/cloudapps-cli/credentials.toml`)

Priority: CLI arguments > environment variables > credential store (macOS Keychain) > credentials.toml

Template:

```toml
# cloudapps-cli credentials configuration
#
# Security notes:
#   - This file contains sensitive information
#   - Set file permissions to 0600: chmod 600 credentials.toml
#   - Add to .gitignore to prevent committing to the repository

[credentials]
# Microsoft Defender for Cloud Apps API token (required)
api_token = ""

# API base URL (required)
# e.g. https://your-tenant.us3.portal.cloudappsecurity.com
api_url = ""
```

Setup:

```bash
# Global configuration (XDG Base Directory)
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/cloudapps-cli"
# Edit and save credentials.toml, then:
chmod 600 "${XDG_CONFIG_HOME:-$HOME/.config}/cloudapps-cli/credentials.toml"

# Project-local configuration
# Edit and save .cloudapps-credentials.toml, then:
chmod 600 .cloudapps-credentials.toml
```

### Credential storage (macOS Keychain)

On macOS, `cloudapps-cli` can back `CLOUDAPPS_API_TOKEN` with the login
Keychain so it does not have to live in plaintext `credentials.toml`.

Resolution order (highest priority first):

1. `CLOUDAPPS_API_TOKEN` environment variable
2. **macOS Keychain** (login keychain, `service=dev.cloudapps-cli`,
   `account=api_token`)
3. `credentials.toml`, searched in this order:
   1. `./.cloudapps-credentials.toml` (current working directory)
   2. `$XDG_CONFIG_HOME/cloudapps-cli/credentials.toml` (falls back to
      `~/.config/cloudapps-cli/credentials.toml`)

Storing the token in the Keychain keeps it out of plaintext config
files (and out of dotfile backups, Time Machine snapshots, accidental
`git add` of the home directory, malware reading the home directory
under the same uid).

#### Storing a token

```bash
cloudapps-cli credentials set api-token
# Enter api_token (input hidden):

# or, from a password manager:
pbpaste | cloudapps-cli credentials set api-token --stdin
```

#### Migrating from credentials.toml

If you already have a token in `credentials.toml`, move it directly
into the Keychain in one step:

```bash
cloudapps-cli credentials migrate
```

`migrate` writes the token to the Keychain and then offers to dispose
of the plaintext copy:

- **Recommended (default)**: the `api_token` line is removed from the
  toml via an atomic temp-file rename. No plaintext copy remains on
  disk.
- **Opt-in**: a 0o600 backup of the original toml is kept alongside
  the rewritten file. Choose this only if you need to roll back to the
  old setup.

> ⚠️ **The opt-in backup still contains the plaintext token.** A backup
> under `$HOME` is typically included in Time Machine / iCloud / rsync
> snapshots and defeats the point of moving the token into the
> Keychain. Delete it as soon as you have confirmed the new setup works
> with `cloudapps-cli credentials status`.

If the rewrite fails partway through, migrate rolls back the Keychain
entry it just wrote so you are not left in a half-migrated state.

##### Recovering from an interrupted migrate

If you hit Ctrl-C (or your machine loses power) **between** the
Keychain write and the toml rewrite, both copies of the token exist:
the new Keychain entry *and* the untouched `credentials.toml`. The
process is idempotent — re-running `cloudapps-cli credentials migrate`
on the same file will detect that the token is still present in the
toml and re-run the disposal step. Alternatively, if you want to bail
out entirely, `cloudapps-cli credentials delete api-token` removes the
Keychain entry and the toml stays as it was.

#### Inspecting the entry

```bash
cloudapps-cli credentials status
# Credential store: macOS Keychain (service=dev.cloudapps-cli)
#   api_token : stored
```

`status` only reports presence — it never prints the stored value. The
`get` subcommand is intentionally absent: there is no legitimate
workflow that requires reading the plaintext token back out, and
exposing one would invite accidental leakage into shell history,
terminal scrollback, AI-agent transcripts, and PR descriptions. If you
need to confirm a token, re-issue it from the Microsoft Defender for
Cloud Apps portal.

#### Deleting the entry

```bash
cloudapps-cli credentials delete api-token
```

#### Notes on Keychain prompts

macOS Keychain prompts the user on first access. Users who pin the
entry with "Always Allow" see no further prompts unless the binary's
signature changes (for example, after a `cargo install` rebuild).

If `credentials status` shows `error (UNIX[Operation not permitted])`
or similar, the stored ACL entry is stale. Recover by opening
**Keychain Access.app**, finding
`dev.cloudapps-cli` / `api_token`, deleting the Access Control entry
for `cloudapps-cli`, and re-running `status` so macOS re-creates the
ACL entry against the new binary signature.

## Usage

```
cloudapps-cli <resource> <action> [options]
```

### Resources

#### Activities

```bash
# List activities
cloudapps-cli activities list --limit 50

# List with filters
cloudapps-cli activities list --user user@example.com --ip 192.0.2.1
cloudapps-cli activities list --country US --query "login"

# Fetch single activity
cloudapps-cli activities fetch <id>
```

#### Alerts

```bash
# List alerts
cloudapps-cli alerts list --limit 50
cloudapps-cli alerts list --severity high
cloudapps-cli alerts list --resolution open
cloudapps-cli alerts list --open
cloudapps-cli alerts list --closed

# Fetch single alert
cloudapps-cli alerts fetch <id>

# Close alerts
cloudapps-cli alerts close <id> --as benign
cloudapps-cli alerts close <id> --as false-positive
cloudapps-cli alerts close <id> --as true-positive --comment "confirmed threat"

# Bulk close
cloudapps-cli alerts close <id1> <id2> --as benign

# Mark read/unread
cloudapps-cli alerts mark-read <id1> <id2>
cloudapps-cli alerts mark-unread <id>
```

#### Entities

```bash
# List entities
cloudapps-cli entities list --limit 50
cloudapps-cli entities list --type user --domain example.com
cloudapps-cli entities list --is-admin --status active

# Fetch single entity
cloudapps-cli entities fetch <id>

# Fetch entity tree
cloudapps-cli entities fetch-tree <id>
```

#### Files

```bash
# List files
cloudapps-cli files list --limit 50
cloudapps-cli files list --filetype document --sharing private
cloudapps-cli files list --extension xlsx

# Fetch single file
cloudapps-cli files fetch <id>
```

#### Data Enrichment (IP Ranges)

```bash
# List IP ranges
cloudapps-cli data-enrichment list
cloudapps-cli data-enrichment list --category corporate
cloudapps-cli data-enrichment list --builtin
cloudapps-cli data-enrichment list --custom

# Create IP range
cloudapps-cli data-enrichment create \
  --name "Office Network" \
  --subnets "192.0.2.0/24,198.51.100.0/24" \
  --category corporate \
  --organization "Example Corp"

# Update IP range
cloudapps-cli data-enrichment update <id> --name "Updated Name"

# Delete IP range
cloudapps-cli data-enrichment delete <id>
```

### Global Options

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--api-url` | `CLOUDAPPS_API_URL` | API base URL |
| - | `CLOUDAPPS_API_TOKEN` | API token |
| `--output` | - | Output format: `json` (default), `table` |
| `--verbose` | - | Enable verbose output |

### Raw Filters

All list commands support `--filter` for raw JSON filter expressions:

```bash
cloudapps-cli activities list --filter '{"actionType":{"eq":["LOGIN"]}}'
```

### Pagination

Use `--limit` and `--skip` for manual pagination, or `--all` to fetch all records automatically:

```bash
cloudapps-cli alerts list --all
```

## Shell Completion

`cloudapps-cli completion <shell>` は補完スクリプトを標準出力に書き出します。対応シェルは `bash`、`zsh`、`fish`、`powershell`、`elvish` です。

### zsh

```zsh
# ユーザー専用の fpath に保存する例
cloudapps-cli completion zsh > "${fpath[1]}/_cloudapps-cli"
# 反映
autoload -U compinit && compinit
```

### bash

```bash
cloudapps-cli completion bash > /usr/local/etc/bash_completion.d/cloudapps-cli
```

### fish

```fish
cloudapps-cli completion fish > ~/.config/fish/completions/cloudapps-cli.fish
```

## Development

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy -- -D warnings
cargo audit
```

## License

[MIT](LICENSE)
