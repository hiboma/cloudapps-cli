# 14 - Pre-Release Checklist

## Security

- [ ] No `pull_request_target` trigger in any workflow file.
- [ ] No shell injection vectors in GitHub Actions (all user inputs quoted, no `${{ github.event.* }}` in `run:` blocks).
- [ ] All third-party GitHub Actions pinned to commit hashes.
- [ ] Workflow permissions set to minimum required scope.
- [ ] `cargo audit` passes with no known vulnerabilities.
- [ ] No API tokens, secrets, or credentials in source code.
- [ ] No hardcoded tenant IDs or real API URLs in source code or test data.
- [ ] Configuration file permission check implemented (warn on world-readable config).

## Data Privacy

- [ ] All test data uses anonymized/masked values (see 11-testing.md for rules).
- [ ] No real usernames, email addresses, or IP addresses in test data.
- [ ] No company names or organization-specific identifiers in source or docs.
- [ ] No internal URLs or references to private infrastructure.

## Code Quality

- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] All tests pass (`cargo test`).
- [ ] Test coverage meets minimum threshold (80%).
- [ ] Race condition check passes (thread sanitizer).
- [ ] No `unsafe` code blocks (or justified and documented if present).

## Documentation

- [ ] README.md written in English.
- [ ] README includes: project description, installation instructions, usage examples, configuration guide, license.
- [ ] CLAUDE.md prepared with public-safe instructions for AI assistants.
- [ ] Specification documents in `docs/specifications/` are up to date.

## Licensing

- [ ] LICENSE file contains MIT license text.
- [ ] `cargo deny check licenses` or equivalent confirms all dependency licenses are compatible with MIT.
- [ ] No copyleft (GPL, AGPL) dependencies unless explicitly accepted.

## Release

- [ ] Version in `Cargo.toml` matches the release tag.
- [ ] Binaries build successfully for all target platforms.
- [ ] Homebrew formula updated with correct version and SHA256.
- [ ] GitHub Release created with release notes.

## Dependencies

Acceptable dependency licenses:

- MIT
- Apache-2.0
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Zlib
- Unicode-3.0
- BSL-1.0

Licenses requiring review:

- MPL-2.0 (case-by-case, generally acceptable for linking)

Licenses to reject:

- GPL-2.0, GPL-3.0 (copyleft)
- AGPL-3.0 (strong copyleft)
- SSPL (server-side restriction)
