# cloudapps

A CLI tool for the [Microsoft Defender for Cloud Apps REST API](https://learn.microsoft.com/en-us/defender-cloud-apps/api-introduction), written in Rust.

## Installation

### Homebrew (macOS)

```bash
brew tap hiboma/tap
brew install cloudapps
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

### API Token

Set your API token via environment variable or configuration file.

**Environment variable:**

```bash
export CLOUDAPPS_TOKEN="your-api-token"
export CLOUDAPPS_API_URL="https://your-tenant.us3.portal.cloudappsecurity.com"
```

**Configuration file** (`~/.config/cloudapps/config.toml`):

```toml
[auth]
token = "your-api-token"

[api]
base_url = "https://your-tenant.us3.portal.cloudappsecurity.com"
```

**CLI options** (highest priority):

```bash
cloudapps --token "your-api-token" --api-url "https://..." alerts list
```

Priority order: CLI options > environment variables > configuration file.

## Usage

```
cloudapps <resource> <action> [options]
```

### Resources

#### Activities

```bash
# List activities
cloudapps activities list --limit 50

# List with filters
cloudapps activities list --user user@example.com --ip 192.0.2.1
cloudapps activities list --country US --query "login"

# Fetch single activity
cloudapps activities fetch <id>
```

#### Alerts

```bash
# List alerts
cloudapps alerts list --limit 50
cloudapps alerts list --severity high
cloudapps alerts list --resolution open
cloudapps alerts list --open
cloudapps alerts list --closed

# Fetch single alert
cloudapps alerts fetch <id>

# Close alerts
cloudapps alerts close <id> --as benign
cloudapps alerts close <id> --as false-positive
cloudapps alerts close <id> --as true-positive --comment "confirmed threat"

# Bulk close
cloudapps alerts close <id1> <id2> --as benign

# Mark read/unread
cloudapps alerts mark-read <id1> <id2>
cloudapps alerts mark-unread <id>
```

#### Entities

```bash
# List entities
cloudapps entities list --limit 50
cloudapps entities list --type user --domain example.com
cloudapps entities list --is-admin --status active

# Fetch single entity
cloudapps entities fetch <id>

# Fetch entity tree
cloudapps entities fetch-tree <id>
```

#### Files

```bash
# List files
cloudapps files list --limit 50
cloudapps files list --filetype document --sharing private
cloudapps files list --extension xlsx

# Fetch single file
cloudapps files fetch <id>
```

#### Data Enrichment (IP Ranges)

```bash
# List IP ranges
cloudapps data-enrichment list
cloudapps data-enrichment list --category corporate
cloudapps data-enrichment list --builtin
cloudapps data-enrichment list --custom

# Create IP range
cloudapps data-enrichment create \
  --name "Office Network" \
  --subnets "192.0.2.0/24,198.51.100.0/24" \
  --category corporate \
  --organization "Example Corp"

# Update IP range
cloudapps data-enrichment update <id> --name "Updated Name"

# Delete IP range
cloudapps data-enrichment delete <id>
```

### Global Options

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--api-url` | `CLOUDAPPS_API_URL` | API base URL |
| `--token` | `CLOUDAPPS_TOKEN` | API token |
| `--output` | - | Output format: `json` (default), `table` |
| `--verbose` | - | Enable verbose output |

### Raw Filters

All list commands support `--filter` for raw JSON filter expressions:

```bash
cloudapps activities list --filter '{"actionType":{"eq":["LOGIN"]}}'
```

### Pagination

Use `--limit` and `--skip` for manual pagination, or `--all` to fetch all records automatically:

```bash
cloudapps alerts list --all
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
