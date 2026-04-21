//! Abstraction over an OS credential store backend.
//!
//! Today this is used to back `api_token` with the macOS Keychain instead of
//! plaintext `credentials.toml`. The trait keeps the call sites
//! backend-agnostic so a Linux (secret-service) or Windows (credential
//! manager) backend can be added later without touching the resolver.

use std::collections::HashMap;
use std::sync::Mutex;

/// Keychain service name used by `KeychainStore`. The Keychain's "account"
/// attribute is the key supplied to `get` / `set` / `delete` (e.g.
/// `"api_token"`).
pub const KEYCHAIN_SERVICE: &str = "dev.cloudapps-cli";

/// Logical key for the Microsoft Defender for Cloud Apps API token.
pub const KEY_API_TOKEN: &str = "api_token";

/// Error classes produced by a credential store.
///
/// `Unavailable` means the backend is not present at all (e.g. CI sandbox
/// without a default keychain) — the resolver is free to fall through to
/// the next source (such as `credentials.toml`). `Backend` means a real
/// access failure that the user should investigate — the resolver must
/// NOT silently fall back to plaintext, because that would defeat the
/// point of moving the secret out of the file in the first place.
#[derive(Debug)]
pub enum StoreError {
    Unavailable(String),
    Backend(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Unavailable(msg) => write!(f, "credential store unavailable: {}", msg),
            StoreError::Backend(msg) => write!(f, "credential store error: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

/// Trait implemented by credential store backends.
pub trait CredentialStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, StoreError>;
    fn set(&self, key: &str, value: &str) -> Result<(), StoreError>;
    fn delete(&self, key: &str) -> Result<(), StoreError>;
}

/// Return the platform-default credential store.
///
/// On macOS this is the Keychain. On other platforms we currently return a
/// store that always reports `Unavailable`, which makes the resolver fall
/// through to `credentials.toml` on those platforms.
pub fn default_store() -> Box<dyn CredentialStore> {
    #[cfg(target_os = "macos")]
    {
        Box::new(keychain::KeychainStore::new(KEYCHAIN_SERVICE))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(UnsupportedStore)
    }
}

/// Placeholder store for non-macOS builds. Every call reports
/// `Unavailable` so the resolver falls through to the toml.
#[cfg(not(target_os = "macos"))]
pub struct UnsupportedStore;

#[cfg(not(target_os = "macos"))]
impl CredentialStore for UnsupportedStore {
    fn get(&self, _key: &str) -> Result<Option<String>, StoreError> {
        Err(StoreError::Unavailable(
            "keychain backend is only compiled on macOS".to_string(),
        ))
    }
    fn set(&self, _key: &str, _value: &str) -> Result<(), StoreError> {
        Err(StoreError::Unavailable(
            "keychain backend is only compiled on macOS".to_string(),
        ))
    }
    fn delete(&self, _key: &str) -> Result<(), StoreError> {
        Err(StoreError::Unavailable(
            "keychain backend is only compiled on macOS".to_string(),
        ))
    }
}

#[cfg(target_os = "macos")]
pub mod keychain {
    use super::{CredentialStore, StoreError};

    pub struct KeychainStore {
        service: String,
    }

    impl KeychainStore {
        pub fn new(service: &str) -> Self {
            Self {
                service: service.to_string(),
            }
        }

        /// `keyring::Entry::new(service, account)` — the second argument is
        /// the Keychain "account" attribute, which we use as our logical
        /// key (e.g. "api_token").
        fn entry(&self, key: &str) -> Result<keyring::Entry, StoreError> {
            keyring::Entry::new(&self.service, key).map_err(classify_keyring_err)
        }
    }

    impl CredentialStore for KeychainStore {
        fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
            let entry = self.entry(key)?;
            match entry.get_password() {
                Ok(v) => Ok(Some(v)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(classify_keyring_err(e)),
            }
        }

        fn set(&self, key: &str, value: &str) -> Result<(), StoreError> {
            let entry = self.entry(key)?;
            entry.set_password(value).map_err(classify_keyring_err)
        }

        fn delete(&self, key: &str) -> Result<(), StoreError> {
            let entry = self.entry(key)?;
            match entry.delete_credential() {
                Ok(()) => Ok(()),
                Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(classify_keyring_err(e)),
            }
        }
    }

    /// `errSecNoDefaultKeychain` from `Security.framework`.
    /// See <https://developer.apple.com/documentation/security/errsecnodefaultkeychain>.
    /// Locale-independent: the OSStatus is the same on every macOS install.
    pub(super) const ERR_SEC_NO_DEFAULT_KEYCHAIN: i32 = -25307;
    pub(super) const ERR_SEC_INVALID_KEYCHAIN: i32 = -25295;

    /// Classify a `keyring::Error` into `Unavailable` (the store as a whole
    /// is not present, e.g. CI sandbox without a default keychain) vs
    /// `Backend` (an actual access failure that the user should investigate
    /// — denied prompt, daemon down, ACL mismatch).
    ///
    /// We prefer to inspect the underlying `security_framework::base::Error`
    /// OSStatus when available: the codes are locale-independent, whereas
    /// the human-readable message text is translated by macOS (e.g. Japanese
    /// macOS reports the same condition with different wording, which would
    /// slip past a string-match allowlist and force the user into the
    /// `Backend` branch on a clean machine).
    ///
    /// We keep the previous string-match heuristic as a fallback for the
    /// `NoStorageAccess` variant and for unexpected error shapes.
    pub(super) fn classify_keyring_err(e: keyring::Error) -> StoreError {
        // First try to extract a `security_framework::base::Error` from the
        // boxed source. `keyring`'s apple-native backend always wraps a
        // security_framework error inside `PlatformFailure`, so the
        // downcast succeeds in practice.
        if let keyring::Error::PlatformFailure(ref boxed) = e
            && let Some(sf_err) = boxed.downcast_ref::<security_framework::base::Error>()
        {
            let code = sf_err.code();
            let msg = e.to_string();
            if code == ERR_SEC_NO_DEFAULT_KEYCHAIN || code == ERR_SEC_INVALID_KEYCHAIN {
                return StoreError::Unavailable(msg);
            }
            return StoreError::Backend(msg);
        }

        // Fallback for non-PlatformFailure errors (Invalid, NoStorageAccess)
        // or for unrecognized boxed source types: keep the locale-fragile
        // string match as a last line of defense, but err on the side of
        // Backend (the cautious choice — refuses to fall through to the
        // toml).
        let msg = e.to_string();
        let lower = msg.to_lowercase();
        let unavailable = lower.contains("no default keychain")
            || lower.contains("no platform credential store")
            || lower.contains("keychain not found");
        if unavailable {
            StoreError::Unavailable(msg)
        } else {
            StoreError::Backend(msg)
        }
    }
}

/// In-memory store for tests so `resolve_with_store` can be exercised
/// without touching the real Keychain.
pub struct MemoryStore {
    inner: Mutex<HashMap<String, String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for MemoryStore {
    fn get(&self, key: &str) -> Result<Option<String>, StoreError> {
        Ok(self.inner.lock().unwrap().get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.inner
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.inner.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_roundtrip() {
        let s = MemoryStore::new();
        assert_eq!(s.get("key").unwrap(), None);
        s.set("key", "value").unwrap();
        assert_eq!(s.get("key").unwrap().as_deref(), Some("value"));
        s.delete("key").unwrap();
        assert_eq!(s.get("key").unwrap(), None);
    }

    #[test]
    fn memory_store_delete_missing_is_ok() {
        let s = MemoryStore::new();
        s.delete("missing").unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn classify_keyring_err_recognizes_no_default_keychain_by_osstatus() {
        // Build a security_framework error directly from the OSStatus, box
        // it through keyring::Error::PlatformFailure, and confirm the
        // classifier maps it to Unavailable regardless of the localized
        // message text.
        let sf = security_framework::base::Error::from_code(
            super::keychain::ERR_SEC_NO_DEFAULT_KEYCHAIN,
        );
        let kr = keyring::Error::PlatformFailure(Box::new(sf));
        match super::keychain::classify_keyring_err(kr) {
            StoreError::Unavailable(_) => {}
            other => panic!("expected Unavailable, got {:?}", other),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn classify_keyring_err_treats_other_osstatus_as_backend() {
        // errSecAuthFailed = -25293 — a real access denial, NOT an
        // "unavailable backend". Must surface as Backend so resolve()
        // refuses the toml fallback.
        let sf = security_framework::base::Error::from_code(-25293);
        let kr = keyring::Error::PlatformFailure(Box::new(sf));
        match super::keychain::classify_keyring_err(kr) {
            StoreError::Backend(_) => {}
            other => panic!("expected Backend, got {:?}", other),
        }
    }
}
