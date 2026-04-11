# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

Only the latest release is actively supported with security updates.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not open a public issue.**
2. Send a report via [GitHub Security Advisories](https://github.com/hiboma/cloudapps-cli/security/advisories/new).
3. Include a description of the vulnerability, steps to reproduce, and potential impact.

You can expect an initial response within 72 hours. We will work with you to understand and address the issue before any public disclosure.

## Security Practices

- Dependencies are audited with `cargo audit` in CI.
- Static analysis is performed with CodeQL and Clippy.
- OpenSSF Scorecard runs weekly to monitor security posture.
- All test data uses anonymized values; no real credentials or tenant IDs are committed.
