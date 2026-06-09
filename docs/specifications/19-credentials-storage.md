# 19 - Credentials storage

## Overview

`cloudapps-cli credentials` manages the `CLOUDAPPS_API_TOKEN` value in an OS
credential store (macOS Keychain). Storing the token in the Keychain keeps
it out of plaintext config files, dotfile backups, Time Machine snapshots,
and accidental `git add` of the home directory.

## Resolution order

When `cloudapps-cli` needs the API token, the following sources are
consulted in priority order. The first source that yields a non-empty
value wins:

1. `CLOUDAPPS_API_TOKEN` environment variable
2. **macOS Keychain** (login keychain, `service=dev.cloudapps-cli`,
   `account=api_token`)
3. `credentials.toml`, searched in this order:
   1. `./.cloudapps-credentials.toml` (current working directory)
   2. `$XDG_CONFIG_HOME/cloudapps-cli/credentials.toml` (falls back to
      `~/.config/cloudapps-cli/credentials.toml`)

If the Keychain reports a **real access failure** (not "no default
keychain" / "no entry"), resolution refuses to fall back to
`credentials.toml` and reports `api_token not set`. Silently picking up a
stale plaintext value would defeat the migration.

## Subcommands

- `credentials set api-token` — prompt for a token (rpassword, hidden
  input) and store it. Use `--stdin` to read from a pipe.
- `credentials status` — report whether the token is stored. Never prints
  the token itself.
- `credentials delete api-token` — remove the entry.
- `credentials migrate [--dry-run]` — move the token from
  `credentials.toml` into the Keychain. Asks how to dispose of the
  plaintext copy: default is an atomic removal of the line; opt-in keeps
  a 0o600 backup file alongside (with a loud warning).

`credentials get` is intentionally absent — there is no legitimate
workflow that requires reading the plaintext token back out, and
exposing one would invite accidental leakage into shell history,
terminal scrollback, AI-agent transcripts, and PR descriptions.

## Keychain prompts and `cargo install`

macOS Keychain prompts the user on first access. Users who pin the entry
with "Always Allow" see no further prompts unless the binary's signature
changes (e.g. after a `cargo install` rebuild). If `credentials status`
shows `error (UNIX[Operation not permitted])`, the prior ACL entry is
stale — open Keychain Access.app, find
`dev.cloudapps-cli` / `api_token`, delete the Access Control entry for
`cloudapps-cli`, and re-run `status` to recreate it with the new
signature.

## Recovery from interrupted migrate

`migrate` writes to the Keychain first and then rewrites the toml. If
you hit Ctrl-C (or lose power) between those two steps, both copies of
the token exist. The process is idempotent — re-running `migrate`
detects the still-present toml entry and re-runs the disposal step.
Alternatively, `credentials delete api-token` removes the Keychain entry
and the toml stays untouched.
