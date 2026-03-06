# CLAUDE.md

## Project

`cloudapps` is a CLI tool for the Microsoft Defender for Cloud Apps REST API, written in Rust.

```
cloudapps <resource> <action> [options]
```

Resources: `activities`, `alerts`, `entities`, `files`, `data-enrichment`

## Specifications

Detailed specifications are in `docs/specifications/`. See `docs/specifications/00-index.md` for the full index.

| Doc | Topic |
|-----|-------|
| 01  | Project overview, scope, technology stack |
| 02  | Authentication (Legacy API Token, OAuth 2.0 future) |
| 03  | API client (rate limiting, pagination, filters, retry) |
| 04-08 | Resource definitions (activities, alerts, entities, files, data-enrichment) |
| 09  | Output formats (JSON, table), exit codes |
| 10  | Project structure, modules, dependencies |
| 11  | Testing strategy, coverage, testdata anonymization, race condition checks |
| 12  | CI/CD (GitHub Actions), security requirements |
| 13  | Homebrew distribution |
| 14  | Pre-release checklist |
| 15  | CLI UX improvements (named enums, shorthand filters, unified close) |

## Build and Test

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy -- -D warnings
cargo audit
```

## Repository

- Public: github.com/hiboma/cloudapps-cli
- License: MIT

## Rules

- All test data in `testdata/` must use anonymized values (see `docs/specifications/11-testing.md`).
- No secrets, tenant IDs, or real user data in source code or test data.
- GitHub Actions must follow `docs/specifications/12-ci-cd.md` security requirements.
- Files must end with a newline (POSIX).
