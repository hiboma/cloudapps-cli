# 12 - CI/CD (GitHub Actions)

## Overview

CI/CD pipelines are implemented with GitHub Actions. All workflows follow security best practices.

## Workflows

### ci.yml - Continuous Integration

Triggers: `push` to main, `pull_request` to main.

Jobs:

1. **check** - Format check, clippy lint, build
2. **test** - Run tests with coverage
3. **audit** - `cargo audit` for dependency vulnerabilities
4. **race-check** - Thread sanitizer (nightly, Linux only)

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

permissions:
  contents: read

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<commit-hash>
      - uses: dtolnay/rust-toolchain@<commit-hash>
        with:
          toolchain: stable
          components: rustfmt, clippy
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo build --release

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<commit-hash>
      - uses: dtolnay/rust-toolchain@<commit-hash>
        with:
          toolchain: stable
      - run: cargo install cargo-tarpaulin
      - run: cargo tarpaulin --out xml --output-dir coverage/
      - name: Upload coverage
        uses: actions/upload-artifact@<commit-hash>
        with:
          name: coverage
          path: coverage/

  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<commit-hash>
      - run: cargo install cargo-audit
      - run: cargo audit

  race-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<commit-hash>
      - uses: dtolnay/rust-toolchain@<commit-hash>
        with:
          toolchain: nightly
      - run: |
          RUSTFLAGS="-Z sanitizer=thread" \
          cargo +nightly test \
          --target x86_64-unknown-linux-gnu
```

### release.yml - Release

Triggers: Push of semver tag (`v*.*.*`).

Jobs:

1. **build** - Cross-compile for all target platforms
2. **release** - Create GitHub Release and upload binaries
3. **homebrew** - Update Homebrew tap formula

Target platforms:

| Target                        | OS      | Arch    |
|-------------------------------|---------|---------|
| `x86_64-unknown-linux-gnu`    | Linux   | x86_64  |
| `aarch64-unknown-linux-gnu`   | Linux   | aarch64 |
| `x86_64-apple-darwin`         | macOS   | x86_64  |
| `aarch64-apple-darwin`        | macOS   | aarch64 |
| `x86_64-pc-windows-msvc`      | Windows | x86_64  |

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

permissions:
  contents: write

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@<commit-hash>
      - uses: dtolnay/rust-toolchain@<commit-hash>
        with:
          toolchain: stable
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@<commit-hash>
        with:
          name: cloudapps-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/cloudapps*

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@<commit-hash>
      - name: Create Release
        uses: softprops/action-gh-release@<commit-hash>
        with:
          files: cloudapps-*/cloudapps*
          generate_release_notes: true
```

## Security Requirements for Workflows

1. Use `pull_request` trigger (never `pull_request_target`).
2. Minimize `permissions` to the least required scope.
3. Pin all third-party actions to commit hashes (not tags).
4. Do not use user-controlled input in shell commands without quoting.
5. Do not expose secrets in logs.
6. Use `cargo audit` to check dependencies.

## Semantic Versioning

- Version format: `MAJOR.MINOR.PATCH` (e.g., `1.0.0`).
- Version is defined in `Cargo.toml`.
- Release is triggered by pushing a tag matching `v*.*.*`.
- The tag version must match the `Cargo.toml` version.
