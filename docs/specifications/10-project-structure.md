# 10 - Project Structure

## Directory Layout

```
cloudapps-cli/
├── Cargo.toml
├── Cargo.lock
├── LICENSE                      # MIT License
├── README.md                    # English documentation
├── CLAUDE.md                    # AI assistant instructions
├── .github/
│   └── workflows/
│       ├── ci.yml               # Build, test, lint, audit
│       └── release.yml          # Release builds and publishing
├── docs/
│   └── specifications/
│       ├── 01-overview.md
│       ├── ...
│       └── 14-pre-release-checklist.md
├── src/
│   ├── main.rs                  # Entry point
│   ├── cli/
│   │   ├── mod.rs               # CLI argument definitions (clap)
│   │   ├── activities.rs        # Activities subcommand
│   │   ├── alerts.rs            # Alerts subcommand
│   │   ├── entities.rs          # Entities subcommand
│   │   ├── files.rs             # Files subcommand
│   │   └── data_enrichment.rs   # Data enrichment subcommand
│   ├── client/
│   │   ├── mod.rs               # CloudAppsClient
│   │   ├── request.rs           # Request building
│   │   ├── response.rs          # Response parsing
│   │   ├── pagination.rs        # Pagination logic
│   │   └── retry.rs             # Retry with exponential backoff
│   ├── auth/
│   │   ├── mod.rs               # AuthProvider trait
│   │   └── token.rs             # Token-based authentication
│   ├── config/
│   │   └── mod.rs               # Configuration loading
│   ├── models/
│   │   ├── mod.rs               # Shared types
│   │   ├── activity.rs          # Activity model
│   │   ├── alert.rs             # Alert model
│   │   ├── entity.rs            # Entity model
│   │   ├── file.rs              # File model
│   │   ├── data_enrichment.rs   # Data enrichment model
│   │   └── filter.rs            # Filter types
│   ├── output/
│   │   ├── mod.rs               # Output format dispatcher
│   │   ├── json.rs              # JSON formatter
│   │   └── table.rs             # Table formatter
│   ├── commands/
│   │   ├── mod.rs               # Command dispatcher
│   │   ├── activities.rs        # Activities command handler
│   │   ├── alerts.rs            # Alerts command handler
│   │   ├── entities.rs          # Entities command handler
│   │   ├── files.rs             # Files command handler
│   │   └── data_enrichment.rs   # Data enrichment command handler
│   └── error.rs                 # Error types
├── tests/
│   ├── common/
│   │   └── mod.rs               # Shared test utilities
│   ├── activities_test.rs       # Integration tests for activities
│   ├── alerts_test.rs           # Integration tests for alerts
│   ├── entities_test.rs         # Integration tests for entities
│   ├── files_test.rs            # Integration tests for files
│   └── data_enrichment_test.rs  # Integration tests for data enrichment
└── testdata/
    ├── activities/
    │   ├── list_response.json
    │   └── fetch_response.json
    ├── alerts/
    │   ├── list_response.json
    │   └── fetch_response.json
    ├── entities/
    │   ├── list_response.json
    │   └── fetch_response.json
    ├── files/
    │   ├── list_response.json
    │   └── fetch_response.json
    └── data_enrichment/
        ├── list_response.json
        └── create_response.json
```

## Module Responsibilities

| Module     | Responsibility                                           |
|------------|----------------------------------------------------------|
| `cli`      | Command-line argument parsing with clap derive macros    |
| `client`   | HTTP communication, request building, response parsing   |
| `auth`     | Authentication token management                          |
| `config`   | Configuration file loading and validation                |
| `models`   | Data structures for API request/response types           |
| `output`   | Output formatting (JSON, table)                          |
| `commands` | Business logic connecting CLI input to API calls         |
| `error`    | Error type definitions                                   |

## Crate Dependencies (planned)

| Crate       | Purpose                          |
|-------------|----------------------------------|
| `clap`      | CLI argument parsing             |
| `reqwest`   | HTTP client                      |
| `tokio`     | Async runtime                    |
| `serde`     | Serialization framework          |
| `serde_json`| JSON serialization               |
| `thiserror` | Error derive macros              |
| `anyhow`    | Error context                    |
| `toml`      | Configuration file parsing       |
| `chrono`    | Timestamp handling               |
| `tabled`    | Table output formatting          |

## Dev Dependencies

| Crate       | Purpose                          |
|-------------|----------------------------------|
| `mockito`   | HTTP mock server for testing     |
| `assert_cmd`| CLI integration testing          |
| `predicates`| Assertion helpers for testing    |
| `tempfile`  | Temporary files for config tests |
