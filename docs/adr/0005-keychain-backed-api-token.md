# 0005 — macOS Keychain–backed `api_token`

## Status

Accepted

## Context

`cloudapps-cli` authenticates against the Microsoft Defender for Cloud
Apps REST API via a Legacy API Token. Prior to this ADR, the token was
resolved from one of three places:

1. `CLOUDAPPS_API_TOKEN` environment variable.
2. `credentials.toml` under the project directory or
   `$XDG_CONFIG_HOME/cloudapps-cli/`.
3. A CLI flag — **deliberately not offered** for the token, to keep it
   out of `ps` / `/proc/<pid>/cmdline`.

The environment-variable path is already scrubbed after resolution (the
process overwrites the value in the C `environ` array and then calls
`remove_var`), so `ps -E` / `/proc/<pid>/environ` cannot observe the
token once it has been consumed. That leaves `credentials.toml` as the
sole long-lived plaintext home of the secret.

Plaintext in `$HOME` has several failure modes that environment
scrubbing does not cover:

- Time Machine backups and iCloud Drive sync pick up `$HOME` contents
  and carry the plaintext to off-host storage.
- A user accidentally running `git add .` (or pushing a rescue dotfile
  tarball) publishes the file.
- Malware running under the same uid trivially reads the file — on
  macOS, Keychain-held secrets require an ACL decision from the user
  (or a codesigned allow-list) to release, which raises the bar.

## Decision

Introduce an OS credential store abstraction and use the **macOS
Keychain** as the first-class storage tier for the API token on macOS.

Only `api_token` is moved. `api_url` remains in the toml because it is
not sensitive (Microsoft's regional portal URL).

## Consequences

### Scope

- A new `CredentialStore` trait (`get` / `set` / `delete`) with two
  implementations:
  - `KeychainStore` on macOS, via the `keyring` crate's `apple-native`
    feature.
  - `MemoryStore` for tests.
- A new `cloudapps-cli credentials` subcommand with four actions:
  - `set api-token [--stdin]` — prompts via `rpassword`, or reads from
    stdin.
  - `delete api-token` — remove the entry.
  - `status` — report presence only, never the value.
  - `migrate [--dry-run]` — move the token from `credentials.toml` into
    the Keychain and remove (or optionally back up) the plaintext line.
- The resolver changes to consult the credential store between the env
  var and the toml:
  ```
  CLI args > env var > Keychain > credentials.toml > defaults
  ```

### Unavailable vs. Backend

The `StoreError` enum distinguishes two classes of failure:

- `Unavailable`: the backend is not present at all — CI sandbox without
  a default keychain, non-macOS build. Resolution falls through quietly
  to `credentials.toml`.
- `Backend`: a real access failure — denied Keychain prompt, daemon
  down, ACL mismatch. Resolution **refuses** to fall through. Silently
  picking up a stale plaintext value would defeat the migration.

We classify errors by OSStatus (via downcasting the boxed
`security_framework::base::Error` inside
`keyring::Error::PlatformFailure`) because the Display string from
Security.framework is localized — matching on strings would slip
Japanese macOS's `errSecNoDefaultKeychain` past the allowlist and force
clean-install users into the no-fallback branch.

### `migrate` safety properties

- Writes to the Keychain first. No toml change yet, so a failure at
  this step requires no rollback.
- After confirming disposal, either:
  - Rewrites the toml atomically via `tempfile::NamedTempFile::new_in` +
    `persist()` (mode 0o600). If the rewrite fails, the Keychain entry
    is deleted so the user is not left in a half-migrated state.
  - Or, opt-in, writes a 0o600 backup alongside the rewritten toml.
- Refuses to migrate `api_token` values in unsupported quote forms
  (literal strings, multi-line basic, escaped quotes) rather than
  silently wiping the line.

### `get` is intentionally absent

There is no legitimate workflow that requires reading the plaintext
token back out. A `get` subcommand would invite leakage into:

- shell history (`history` / `HISTFILE`),
- terminal scrollback and tmux buffers,
- AI-agent transcripts (this very transcript, for instance),
- PR descriptions that paste in command output.

Operators who need to confirm a token should re-issue it from the
Microsoft Defender for Cloud Apps portal.

### Default backup is `off`

The initial design wrote a 0o600 backup of `credentials.toml` before
rewriting. Reviewers pointed out that this inverts the safety default:
a backup file containing the plaintext token defeats the very migration
we just performed, and a user who skims the closing message may never
get around to running `rm`.

The final design asks the user explicitly, with the default (Enter-key)
choice being the safe one (remove the line). The opt-in backup path
prints a multi-line WARNING block.

### Secrets are zeroized on drop

Secret-bearing `String`s in `set_value` / `migrate` are wrapped in
`zeroize::Zeroizing` so the heap allocation is wiped on drop. This
narrows the window where a swap-out, core dump, or panic-time
backtrace could expose the token. The `Debug` impl of
`CloudAppsCredentials` is hand-written to mask `api_token` with `***`.

## Alternatives considered

- **Shell out to `security`.** Rejected: extra process, brittle output
  parsing, no structured errors.
- **Use `security-framework` directly.** Rejected: reinvents what
  `keyring` already wraps, and we already pick up
  `security-framework` transitively to downcast OSStatus values.
- **Put `api_url` in the Keychain too.** Rejected: it is not sensitive
  and would cost a Keychain prompt per invocation without a safety
  benefit.
- **Default-keep backup.** Rejected: see "Default backup is `off`".
- **`secrecy::Secret<String>` for typed secret handling.** Rejected for
  now: `zeroize::Zeroizing` provides the drop-time wipe we want without
  requiring a type-wide refactor. Revisit if the secret threads through
  more modules.

## References

- PR #52 (mde-cli) — the original Keychain implementation we translated.
- PR #55 (mde-cli) — the follow-up hardening (tempfile, zeroize,
  OSStatus classification, DX fixes) translated here.
- `docs/specifications/19-credentials-storage.md` — user-facing docs.
