use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zeroize::Zeroizing;

use crate::cli::credentials::{CredentialField, CredentialsCommand};
use crate::config::credential_store::{CredentialStore, KEY_API_TOKEN, default_store};
use crate::error::AppError;

/// 0o600 — owner read/write only. Used for both the backup we sometimes
/// write and for the tempfile that replaces the toml atomically.
const SECRET_FILE_MODE: u32 = 0o600;

/// Dispatch entrypoint for `cloudapps-cli credentials <action>`.
pub fn handle(cmd: &CredentialsCommand) -> Result<(), AppError> {
    let store = default_store();
    match cmd {
        CredentialsCommand::Set { field, stdin } => set_value(&*store, *field, *stdin),
        CredentialsCommand::Delete { field } => delete_value(&*store, *field),
        CredentialsCommand::Status => print_status(&*store),
        CredentialsCommand::Migrate { dry_run } => migrate(&*store, *dry_run),
    }
}

fn set_value(
    store: &dyn CredentialStore,
    field: CredentialField,
    from_stdin: bool,
) -> Result<(), AppError> {
    // Wrap the secret in Zeroizing so the heap allocation is wiped on drop.
    // This narrows the window where a swap-out, core dump, or panic-time
    // backtrace could expose the value. The buffer used to read stdin is
    // also zeroized for the same reason.
    let value: Zeroizing<String> = if from_stdin {
        let mut buf: Zeroizing<String> = Zeroizing::new(String::new());
        io::stdin()
            .read_line(&mut buf)
            .map_err(|e| AppError::Config(format!("failed to read stdin: {}", e)))?;
        // Trim full whitespace (not just CRLF) so a stray trailing space
        // pasted from a password manager does not silently corrupt the secret.
        let trimmed = Zeroizing::new(buf.trim().to_string());
        if trimmed.is_empty() {
            return Err(AppError::InvalidInput("empty value from stdin".to_string()));
        }
        trimmed
    } else {
        let prompt = format!("Enter {} (input hidden): ", field.key());
        Zeroizing::new(
            rpassword::prompt_password(prompt)
                .map_err(|e| AppError::Config(format!("failed to read password: {}", e)))?,
        )
    };

    if value.is_empty() {
        return Err(AppError::InvalidInput("empty value".to_string()));
    }

    store
        .set(field.key(), &value)
        .map_err(|e| AppError::Config(e.to_string()))?;
    println!("Stored {} in credential store", field.key());
    println!("Verify with: cloudapps-cli credentials status");
    Ok(())
}

fn delete_value(store: &dyn CredentialStore, field: CredentialField) -> Result<(), AppError> {
    store
        .delete(field.key())
        .map_err(|e| AppError::Config(e.to_string()))?;
    println!("Deleted {} from credential store", field.key());
    Ok(())
}

fn print_status(store: &dyn CredentialStore) -> Result<(), AppError> {
    // Status is intentionally coarse — it reports a field name
    // ("api_token") and a presence flag — never the credential value.
    let keys = [KEY_API_TOKEN];
    println!("Credential store: macOS Keychain (service=dev.cloudapps-cli)");
    let mut saw_error = false;
    for key in keys {
        match store.get(key) {
            Ok(Some(_)) => println!("  {} : stored", key),
            Ok(None) => println!("  {} : not stored", key),
            Err(e) => {
                println!("  {} : error ({})", key, e);
                saw_error = true;
            }
        }
    }
    if saw_error {
        println!();
        println!(
            "One or more entries could not be accessed. This usually means a \
             Keychain ACL change (often triggered by a `cargo install` rebuild \
             that changed the binary's code signature). See the README section \
             \"Credential storage\" -> \"Notes on Keychain prompts\" for how to \
             re-grant access via Keychain Access.app."
        );
    }
    Ok(())
}

/// Migrate `api_token` from credentials.toml into the OS credential store.
///
/// Flow:
/// 1. Find the first credentials.toml in the search path, read it, locate
///    the `api_token` line.
/// 2. Refuse unsupported quote forms (literal strings, multi-line basic,
///    strings containing escaped quotes) rather than silently
///    mishandling them.
/// 3. Confirm with the user, then write to the Keychain.
/// 4. Ask how to dispose of the plaintext copy: default is an atomic
///    rewrite that removes the line; opt-in keeps a 0o600 backup
///    alongside (with a loud warning that it defeats the migration).
/// 5. If the rewrite fails partway through, roll back the Keychain
///    entry so the user is not left in a half-migrated state.
fn migrate(store: &dyn CredentialStore, dry_run: bool) -> Result<(), AppError> {
    let Some(path) = find_credentials_toml() else {
        return Err(AppError::Config(
            "no credentials.toml found in search path. Use `cloudapps-cli credentials set api-token` \
             to store the token directly."
                .to_string(),
        ));
    };

    // canonicalize so the confirmation prompt echoes an absolute path —
    // a hostile cwd that contains a planted `.cloudapps-credentials.toml`
    // becomes visible at that point.
    let canonical = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    println!("Found credentials.toml: {}", canonical.display());

    // Read the toml into a Zeroizing buffer: it contains the plaintext
    // secret we are about to migrate, and we want every copy of those
    // bytes wiped before this scope returns.
    let original: Zeroizing<String> =
        Zeroizing::new(fs::read_to_string(&path).map_err(|e| {
            AppError::Config(format!("failed to read {}: {}", canonical.display(), e))
        })?);

    let secret: Zeroizing<String> = match extract_api_token(&original) {
        SecretScan::Present(s) => Zeroizing::new(s),
        SecretScan::Absent => {
            println!("  api_token: not present (nothing to migrate)");
            println!();
            println!("If you meant to store a token, use: cloudapps-cli credentials set api-token");
            return Ok(());
        }
        SecretScan::Unsupported(reason) => {
            return Err(AppError::Config(format!(
                "refusing to migrate: api_token uses an unsupported quote form ({}). \
                 Rewrite it as a normal double-quoted string first.",
                reason
            )));
        }
    };

    println!("  api_token: present (will migrate)");
    println!();

    if dry_run {
        println!("(dry-run) Would write api_token to Keychain and remove the line from the toml.");
        return Ok(());
    }

    // Confirmation: the user is about to mutate their credentials file.
    // Default is "no" so a stray Enter-key does not alter state.
    if !confirm("Proceed with migration? [y/N] ", false)? {
        println!("Aborted.");
        return Ok(());
    }

    // Write to Keychain first. No toml change yet, so if this fails there
    // is nothing to roll back.
    store
        .set(KEY_API_TOKEN, &secret)
        .map_err(|e| AppError::Config(format!("keychain write failed: {}", e)))?;
    println!("Wrote api_token to Keychain.");

    // Now ask about disposal. Default is remove (Y) — no plaintext copy
    // remains on disk. The opt-in backup still contains the plaintext
    // copy on disk that the user has to remember to delete; we surface
    // that risk loudly when they choose to keep it.
    let mode = prompt_disposal()?;
    // `updated` is derived by removing the secret line from `original`,
    // so it does not contain the secret. Plain String is fine.
    let updated = remove_api_token_line(&original);

    match mode {
        DisposalMode::RemoveLine => {
            if let Err(e) = atomic_replace(&path, updated.as_bytes()) {
                // Rollback the Keychain entry before surfacing the error
                // so the user is not left in a half-migrated state.
                let _ = store.delete(KEY_API_TOKEN);
                return Err(AppError::Config(format!(
                    "failed to rewrite {} (rolled back Keychain entry): {}",
                    canonical.display(),
                    e
                )));
            }
            println!(
                "Removed api_token line from {} (no plaintext copy remains on disk).",
                canonical.display()
            );
        }
        DisposalMode::KeepBackup => {
            let backup = make_backup_path(&path);
            if let Err(e) = write_secret_file(&backup, original.as_bytes(), true) {
                let _ = store.delete(KEY_API_TOKEN);
                return Err(AppError::Config(format!(
                    "failed to write backup {} (rolled back Keychain entry): {}",
                    backup.display(),
                    e
                )));
            }
            if let Err(e) = atomic_replace(&path, updated.as_bytes()) {
                // Try to remove the backup we just created and roll back
                // the Keychain entry.
                let _ = fs::remove_file(&backup);
                let _ = store.delete(KEY_API_TOKEN);
                return Err(AppError::Config(format!(
                    "failed to rewrite {} (rolled back Keychain entry and removed backup): {}",
                    canonical.display(),
                    e
                )));
            }
            println!();
            println!("WARNING: a backup copy of the original toml has been written to:");
            println!("  {}", backup.display());
            println!(
                "This backup STILL CONTAINS THE PLAINTEXT API TOKEN. A backup under \
                 $HOME is typically included in Time Machine / iCloud / rsync snapshots \
                 and defeats the entire point of moving the secret into the Keychain. \
                 DELETE IT as soon as you have confirmed the new setup works with \
                 `cloudapps-cli credentials status`."
            );
        }
    }

    Ok(())
}

enum DisposalMode {
    RemoveLine,
    KeepBackup,
}

fn prompt_disposal() -> Result<DisposalMode, AppError> {
    // Default yes: remove the plaintext from the toml. The "safe" choice
    // — no plaintext copy remains on disk — is the Enter-key default.
    if confirm(
        "Remove the api_token line from credentials.toml? \
         (recommended — no plaintext copy remains) [Y/n] ",
        true,
    )? {
        Ok(DisposalMode::RemoveLine)
    } else {
        Ok(DisposalMode::KeepBackup)
    }
}

/// Print `prompt`, read a line from stdin, return true on y/Y and false
/// on n/N. Empty line defaults to `default_yes`.
fn confirm(prompt: &str, default_yes: bool) -> Result<bool, AppError> {
    print!("{}", prompt);
    io::stdout()
        .flush()
        .map_err(|e| AppError::Config(format!("failed to flush stdout: {}", e)))?;
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .map_err(|e| AppError::Config(format!("failed to read stdin: {}", e)))?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        return Ok(default_yes);
    }
    let first = trimmed.chars().next().unwrap_or('n').to_ascii_lowercase();
    Ok(first == 'y')
}

#[derive(Debug)]
enum SecretScan {
    Present(String),
    Absent,
    Unsupported(&'static str),
}

/// Extract the api_token value from credentials.toml.
///
/// We intentionally refuse to operate on quote forms we do not fully
/// understand, rather than silently returning `Absent` (which would
/// cause the blank/remove step to wipe the line later without having
/// actually surfaced the value — a data-loss footgun).
fn extract_api_token(content: &str) -> SecretScan {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("api_token") {
            continue;
        }
        // Match `api_token` as a whole token, not a prefix — so
        // `api_token_extra = ...` is left alone.
        let rest = &trimmed["api_token".len()..];
        let mut chars = rest.chars();
        match chars.next() {
            Some(c) if c.is_whitespace() || c == '=' => {}
            _ => continue,
        }
        let after_eq = match rest.split_once('=') {
            Some((_, after)) => after.trim(),
            None => continue,
        };

        // Handle the empty-string case ("already migrated") as Absent so a
        // second migrate run is a no-op instead of an error.
        if after_eq == "\"\"" {
            return SecretScan::Absent;
        }

        // Reject unsupported quote forms before we consider the line absent.
        if after_eq.starts_with("'") {
            return SecretScan::Unsupported("literal string");
        }
        if after_eq.starts_with("\"\"\"") {
            return SecretScan::Unsupported("multi-line basic string");
        }
        if let Some(rest2) = after_eq.strip_prefix('"') {
            // Refuse escaped quotes for simplicity — users with those can
            // rewrite their token out of that form.
            if rest2.contains('\\') {
                return SecretScan::Unsupported("escape sequences in basic string");
            }
            if let Some(end) = rest2.find('"') {
                return SecretScan::Present(rest2[..end].to_string());
            }
            return SecretScan::Unsupported("unterminated basic string");
        }
        return SecretScan::Unsupported("unquoted value");
    }
    SecretScan::Absent
}

/// Remove the line that declares `api_token` from the toml. We match
/// the key as a whole token so `api_token_extra = ...` is not touched.
fn remove_api_token_line(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let is_api_token = trimmed.starts_with("api_token")
            && match trimmed["api_token".len()..].chars().next() {
                Some(c) => c.is_whitespace() || c == '=',
                None => false,
            };
        if is_api_token {
            continue;
        }
        out.push_str(line);
    }
    out
}

fn make_backup_path(path: &Path) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".bak.{}", ts));
    path.with_file_name(name)
}

fn write_secret_file(path: &Path, bytes: &[u8], exclusive: bool) -> Result<(), AppError> {
    let mut opts = OpenOptions::new();
    opts.write(true).mode(SECRET_FILE_MODE);
    if exclusive {
        opts.create_new(true);
    } else {
        opts.create(true).truncate(true);
    }
    let mut f = opts
        .open(path)
        .map_err(|e| AppError::Config(format!("failed to open {}: {}", path.display(), e)))?;
    f.write_all(bytes)
        .map_err(|e| AppError::Config(format!("failed to write {}: {}", path.display(), e)))?;
    f.sync_all().ok();
    fs::set_permissions(path, fs::Permissions::from_mode(SECRET_FILE_MODE)).map_err(|e| {
        AppError::Config(format!(
            "failed to set permissions on {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

/// Atomically replace `path` with `bytes`. We write to a sibling tempfile
/// with mode 0o600 then `rename` over the original. The mode of the resulting
/// file is the mode of the tempfile (0o600), which is more restrictive than
/// the previous mode and therefore safe.
///
/// Uses `tempfile::NamedTempFile` so the tempfile name is random rather than
/// predictable (a unix-nanos suffix could be raced in shared dirs), and so
/// the tempfile is dropped automatically if the rename step fails — no
/// manual cleanup path to forget about.
fn atomic_replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::Builder::new()
        .prefix(".cloudapps-cred-")
        .suffix(".tmp")
        .permissions(fs::Permissions::from_mode(SECRET_FILE_MODE))
        .tempfile_in(dir)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all().ok();
    // persist() consumes the NamedTempFile; on success the file lives at
    // `path`. On failure the NamedTempFile is returned inside the error
    // and dropped, which unlinks it.
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

fn find_credentials_toml() -> Option<PathBuf> {
    // Mirror the search order in config::credentials_search_paths(). We
    // duplicate the logic rather than exposing the function because the
    // migrate flow needs to know *which* file was picked (to canonicalize
    // and display it), not just the resolved value.
    let cwd = PathBuf::from(".cloudapps-credentials.toml");
    if cwd.exists() {
        return Some(cwd);
    }
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(config_home)
            .join("cloudapps-cli")
            .join("credentials.toml");
        if p.exists() {
            return Some(p);
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home)
            .join(".config")
            .join("cloudapps-cli")
            .join("credentials.toml");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_present() {
        let s = r#"
[credentials]
api_token = "abc123"
api_url = "https://example.com"
"#;
        match extract_api_token(s) {
            SecretScan::Present(v) => assert_eq!(v, "abc123"),
            other => panic!("expected Present, got {:?}", other),
        }
    }

    #[test]
    fn extract_absent_when_empty_value() {
        let s = r#"
[credentials]
api_token = ""
"#;
        match extract_api_token(s) {
            SecretScan::Absent => {}
            other => panic!("expected Absent (empty value), got {:?}", other),
        }
    }

    #[test]
    fn extract_absent_when_missing() {
        let s = r#"
[credentials]
api_url = "https://example.com"
"#;
        match extract_api_token(s) {
            SecretScan::Absent => {}
            other => panic!("expected Absent, got {:?}", other),
        }
    }

    #[test]
    fn extract_rejects_literal_string() {
        let s = r#"api_token = 'raw'"#;
        match extract_api_token(s) {
            SecretScan::Unsupported(_) => {}
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    #[test]
    fn extract_rejects_multi_line_basic() {
        let s = r#"api_token = """multi""""#;
        match extract_api_token(s) {
            SecretScan::Unsupported(_) => {}
            other => panic!("expected Unsupported, got {:?}", other),
        }
    }

    #[test]
    fn extract_ignores_similar_key() {
        let s = r#"api_token_extra = "hidden""#;
        match extract_api_token(s) {
            SecretScan::Absent => {}
            other => panic!("expected Absent (similar key), got {:?}", other),
        }
    }

    #[test]
    fn remove_drops_only_api_token_line() {
        let s = "[credentials]\napi_token = \"abc\"\napi_url = \"https://example.com\"\n";
        let out = remove_api_token_line(s);
        // Use a boolean assertion so the content (which contained the
        // secret in the input) is not interpolated into an assert
        // message in the output. Static fixtures are safe but keep the
        // pattern consistent with mde-cli's approach.
        let ok = !out.contains("api_token = \"abc\"") && out.contains("api_url");
        assert!(ok);
    }

    #[test]
    fn remove_ignores_similar_key() {
        let s = "api_token_extra = \"hidden\"\napi_token = \"real\"\n";
        let out = remove_api_token_line(s);
        assert!(out.contains("api_token_extra"));
        assert!(!out.contains("api_token = \"real\""));
    }

    #[test]
    fn atomic_replace_writes_with_mode_0600() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.toml");
        fs::write(&target, b"old").unwrap();
        atomic_replace(&target, b"new").unwrap();
        let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, SECRET_FILE_MODE);
        assert_eq!(fs::read(&target).unwrap(), b"new");
    }
}
