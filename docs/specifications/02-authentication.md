# 02 - Authentication

## Overview

Microsoft Defender for Cloud Apps API requires an API token in the `Authorization` header for all requests.

## Authentication Methods

### Legacy API Token

The API token is passed in the `Authorization` header:

```
Authorization: Token <your_token_key>
```

### OAuth 2.0 (Recommended by Microsoft)

Microsoft recommends using OAuth 2.0 Authorization Code Flow via Microsoft Entra (Azure AD) applications.

Two contexts are supported:

1. **Application Context** - For daemon/service applications without a signed-in user
2. **User Context** - For applications acting on behalf of a user

## CLI Configuration

### Token Resolution

The API token is resolved from the following sources, in priority order:

1. `CLOUDAPPS_API_TOKEN` environment variable
2. macOS Keychain (login keychain, `service=dev.cloudapps-cli`,
   `account=api_token`) — see `docs/specifications/19-credentials-storage.md`
3. `credentials.toml` under the project directory or
   `$XDG_CONFIG_HOME/cloudapps-cli/`

If the Keychain backend reports a real access failure (not "no default
keychain" / "no entry"), resolution refuses to fall back to
`credentials.toml` so a stale plaintext value cannot silently mask the
intended Keychain secret.

### API URL Resolution Order

1. `--api-url` command-line option (highest priority)
2. `CLOUDAPPS_API_URL` environment variable
3. `credentials.toml`

## Security Considerations

- The preferred storage for the token on macOS is the Keychain (via
  `cloudapps-cli credentials`).
- The `Debug` impl of `CloudAppsCredentials` masks `api_token` with
  `***`, so an accidental `dbg!()` / `{:?}` on the struct does not leak
  the value into logs.
- `CLOUDAPPS_API_TOKEN` is scrubbed from the process environment (value
  overwritten in the C `environ` array then removed) immediately after
  resolution, so `ps -E` / `/proc/<pid>/environ` cannot observe the
  secret for the lifetime of the process.
- The token must not be stored in source code or committed to version
  control.

## Implementation Notes

- The first version supports Legacy API Token authentication only.
- OAuth 2.0 support is out of scope for the initial release but should be considered in the architecture.
- The `AuthProvider` trait abstracts authentication to allow future extension.

```rust
pub trait AuthProvider: Send + Sync {
    fn token(&self) -> Result<String>;
}
```
