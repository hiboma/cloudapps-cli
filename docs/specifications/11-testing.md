# 11 - Testing Strategy

## Overview

The project uses a multi-layered testing approach to ensure correctness, safety, and reliability.

## Test Layers

### Unit Tests

Location: `#[cfg(test)]` modules within each source file.

Coverage targets:

- **Models**: Serialization/deserialization of all API response types.
- **Filters**: Filter construction and JSON serialization.
- **Config**: Configuration file parsing, environment variable resolution, priority order.
- **Auth**: Token resolution from various sources.
- **Output**: JSON and table formatting for each resource type.
- **Error**: Error type conversions and display formatting.

### Integration Tests

Location: `tests/` directory.

Coverage targets:

- **API client**: HTTP request construction, response parsing with mock server (`mockito`).
- **Pagination**: Automatic pagination across multiple pages.
- **Retry**: Retry behavior on 429 and 5xx responses.
- **CLI end-to-end**: Command execution with `assert_cmd`.

### Test Data

Location: `testdata/` directory.

- All test data files contain anonymized, masked data.
- No real tenant IDs, usernames, IP addresses, or organization names.
- Use placeholder values: `tenant-id-xxxx`, `user@example.com`, `192.0.2.x` (RFC 5737 documentation range).

#### Anonymization Rules

| Field Type       | Mask Pattern                          |
|------------------|---------------------------------------|
| Tenant ID        | `tenant-xxxx-xxxx-xxxx`               |
| Username / Email | `user{N}@example.com`                 |
| IP address       | `192.0.2.{N}` (IPv4), `2001:db8::{N}` (IPv6) |
| Organization     | `Example Corp`, `Test Organization`   |
| File names       | `document-{N}.pdf`, `report-{N}.xlsx` |
| Alert IDs        | `alert-xxxx-{N}`                      |
| Entity IDs       | `entity-xxxx-{N}`                     |
| Domain names     | `example.com`, `test.example.org`     |
| Country codes    | Use actual ISO codes (non-sensitive)   |

## Coverage

- Use `cargo-tarpaulin` or `cargo-llvm-cov` for coverage measurement.
- Target: 80% line coverage minimum.
- CI enforces coverage does not decrease on PRs.

## Race Condition Checks

- Use `--cfg test` with `tokio::test` for async tests.
- Run `cargo test` under thread sanitizer (TSan) in CI:

```bash
RUSTFLAGS="-Z sanitizer=thread" cargo +nightly test --target x86_64-unknown-linux-gnu
```

- Ensure all shared state uses appropriate synchronization primitives.
- The API client is designed to be `Send + Sync` safe.

## Security Audit

- Run `cargo audit` in CI to check for known vulnerabilities in dependencies.
- Fail the build if any advisory is found.
- Schedule periodic runs (weekly) in addition to PR checks.

## Mocking Strategy

- Use `mockito` for HTTP-level mocking in integration tests.
- Define a `trait HttpClient` to allow unit testing without network access.
- Test data is loaded from `testdata/` files to keep tests maintainable.

```rust
#[cfg(test)]
mod tests {
    use mockito::Server;

    #[tokio::test]
    async fn test_list_activities() {
        let mut server = Server::new_async().await;
        let mock = server.mock("POST", "/api/v1/activities/")
            .with_status(200)
            .with_body(include_str!("../../testdata/activities/list_response.json"))
            .create_async()
            .await;

        // ... test logic ...

        mock.assert_async().await;
    }
}
```
