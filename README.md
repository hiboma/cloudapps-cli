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

**Environment variable:**

```bash
export CLOUDAPPS_API_TOKEN="your-api-token"
export CLOUDAPPS_API_URL="https://your-tenant.us3.portal.cloudappsecurity.com"
```

**CLI options:**

```bash
cloudapps-cli --api-url "https://..." alerts list
```

> **Note:** `CLOUDAPPS_API_TOKEN` has no CLI flag to prevent exposure in process lists. Use environment variables, `.env` files, or `credentials.toml`. Ensure these files have restrictive permissions (`chmod 600`).

### Credentials File (TOML)

You can configure credentials using a `credentials.toml` file. Files are loaded in the following order:

1. `./.cloudapps-credentials.toml` (project-local)
2. `$XDG_CONFIG_HOME/cloudapps-cli/credentials.toml` (default: `~/.config/cloudapps-cli/credentials.toml`)

Priority: CLI arguments > environment variables > credentials.toml

Template:

```toml
# cloudapps-cli credentials configuration
#
# Security notes:
#   - This file contains sensitive information
#   - Set file permissions to 0600: chmod 600 credentials.toml
#   - Add to .gitignore to prevent committing to the repository
#   - Consider using environment variables or a secrets manager instead

[credentials]
# Microsoft Defender for Cloud Apps API URL (required)
# Example: https://your-tenant.us3.portal.cloudappsecurity.com
api_url = ""

# API token (required)
api_token = ""
```

Setup:

```bash
# Global configuration (XDG Base Directory)
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/cloudapps-cli"
cp credentials.toml "${XDG_CONFIG_HOME:-$HOME/.config}/cloudapps-cli/credentials.toml"
chmod 600 "${XDG_CONFIG_HOME:-$HOME/.config}/cloudapps-cli/credentials.toml"

# Project-local configuration
cp credentials.toml .cloudapps-credentials.toml
chmod 600 .cloudapps-credentials.toml
echo ".cloudapps-credentials.toml" >> .gitignore
```

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
