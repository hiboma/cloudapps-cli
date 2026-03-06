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

### Token Resolution Order

1. `--token` command-line option (highest priority)
2. `CLOUDAPPS_API_TOKEN` environment variable
3. Configuration file (`~/.config/cloudapps/config.toml`)

### API URL Resolution Order

1. `--api-url` command-line option (highest priority)
2. `CLOUDAPPS_API_URL` environment variable
3. Configuration file (`~/.config/cloudapps/config.toml`)

### Configuration File Format

```toml
[auth]
token = "your_api_token"

[api]
url = "https://<tenant_id>.<tenant_region>.portal.cloudappsecurity.com/api"
```

## Security Considerations

- The token must not be stored in source code or committed to version control.
- The configuration file permissions should be `0600` (owner read/write only).
- The CLI validates file permissions on the configuration file and warns if they are too permissive.
- The `--token` option value is masked in verbose output.

## Implementation Notes

- The first version supports Legacy API Token authentication only.
- OAuth 2.0 support is out of scope for the initial release but should be considered in the architecture.
- The `AuthProvider` trait abstracts authentication to allow future extension.

```rust
pub trait AuthProvider: Send + Sync {
    fn token(&self) -> Result<String>;
}
```
