# 01 - Project Overview

## Purpose

`cloudapps` is a CLI tool that interacts with the Microsoft Defender for Cloud Apps REST API.
It provides command-line access to activities, alerts, entities, files, and data enrichment (subnet) resources.

## Target API

- Microsoft Defender for Cloud Apps REST API
- Reference: https://learn.microsoft.com/en-us/defender-cloud-apps/api-introduction

## CLI Interface

```
cloudapps <resource> <action> [options]
```

### Supported Resources and Actions

| Resource         | Actions                                                        |
|------------------|----------------------------------------------------------------|
| activities       | list, fetch, feedback                                          |
| alerts           | list, fetch, close-benign, close-false-positive, close-true-positive, mark-read, mark-unread |
| entities         | list, fetch, fetch-tree                                        |
| files            | list, fetch                                                    |
| data-enrichment  | list, create, update, delete                                   |

### Global Options

| Option            | Description                          | Environment Variable      |
|-------------------|--------------------------------------|---------------------------|
| `--api-url`       | Defender for Cloud Apps API base URL | `CLOUDAPPS_API_URL`       |
| `--token`         | API token for authentication         | `CLOUDAPPS_API_TOKEN`     |
| `--output`        | Output format: `json`, `table`       | `CLOUDAPPS_OUTPUT_FORMAT` |
| `--verbose`       | Enable verbose output                |                           |
| `--help`          | Show help information                |                           |
| `--version`       | Show version information             |                           |

## Technology Stack

- Language: Rust
- CLI framework: `clap` (with derive macros)
- HTTP client: `reqwest`
- Serialization: `serde` / `serde_json`
- Async runtime: `tokio`
- Testing: built-in `#[cfg(test)]` + `mockito` for HTTP mocking
- Error handling: `thiserror` / `anyhow`

## Distribution

- Binary releases for Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)
- Homebrew tap for macOS installation
- GitHub Releases with semantic versioning

## Repository

- Public repository: `github.com/hiboma/cloudapps-cli`
- License: MIT
