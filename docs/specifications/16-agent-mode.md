# 16 - Agent Mode

## Overview

Agent mode isolates API credentials from LLM agent processes using the ssh-agent model. The agent process holds the actual API token and executes API calls on behalf of the client via a Unix domain socket (UDS).

## Architecture

```
LLM Agent --> cloudapps (client) --UDS--> cloudapps agent (under op run)
              holds session token only    holds API token, executes API
```

## CLI Commands

### `cloudapps agent start`

Starts the agent process in the background (default) or foreground.

```
cloudapps agent start [--socket PATH] [--config PATH] [--foreground]
```

Outputs shell variables for `eval`:
```bash
CLOUDAPPS_AGENT_SOCKET=/path/to/socket; export CLOUDAPPS_AGENT_SOCKET;
CLOUDAPPS_AGENT_TOKEN=<session-token>; export CLOUDAPPS_AGENT_TOKEN;
CLOUDAPPS_AGENT_PID=<pid>; export CLOUDAPPS_AGENT_PID;
echo Agent pid <pid>;
```

### `cloudapps agent stop`

Stops the agent process by reading its PID file and sending SIGTERM.

```
cloudapps agent stop [--socket PATH] [--all]
```

### `cloudapps agent status`

Shows the agent's running status.

```
cloudapps agent status [--socket PATH]
```

### Hidden Global Options

The following global options are set by `agent start` output and used for auto-routing:

- `--socket` / `CLOUDAPPS_AGENT_SOCKET` — agent socket path
- `--token` / `CLOUDAPPS_AGENT_TOKEN` — session token

These are hidden from `--help` output.

## Auto-routing

When `CLOUDAPPS_AGENT_TOKEN` is set (via environment variable or `--token` flag), all commands are automatically routed through the agent. No explicit flag is required. Global flags like `--output` and `--raw` are passed through to the agent.

## Shutdown

The agent shuts down on any of the following:

1. Watchdog detects session leader exit (`getsid(0)` monitoring, 30s interval)
2. Idle timeout (8 hours by default)
3. SIGINT (Ctrl-C) or SIGTERM
4. `cloudapps agent stop` command

## IPC Protocol

JSON Lines over UDS.

### AgentRequest

```json
{
  "token": "<session-token>",
  "request_id": "<uuid>",
  "command": "alerts",
  "action": "list",
  "args": ["--severity", "HIGH"]
}
```

### AgentResponse

```json
{
  "request_id": "<uuid>",
  "status": "success | error | denied",
  "output": "...",
  "error": null
}
```

## Security

See ADR-0001 for the 10-layer security model.

### Connection-level checks (in handle_connection)

1. Peer UID verification — rejects connections from different UIDs
2. Peer binary verification — macOS: code signing check, Linux: /proc/PID/exe path match

### Request-level checks (in handle_request)

3. Session token verification (constant-time via `subtle` crate)
4. Command name validation (alphanumeric, hyphen, underscore only)
5. Command whitelist check
6. Rate limit check (token bucket)

### Server-level protections

7. Socket permissions set via `umask(0o077)` before bind (prevents TOCTOU)
8. Socket directory permissions `0700`
9. Request size limit: 1 MiB (enforced via `take()` before `read_line`)
10. Concurrent connection limit: 64 (Semaphore)
11. Stdout capture serialized via `tokio::sync::Mutex` to prevent data races

## PID Files

Each agent instance creates a `.pid` file alongside its socket (e.g., `cloudapps-1234.pid`). The PID file is used by `agent stop` to send SIGTERM. Both files are cleaned up on shutdown.

## Configuration

`agent.toml` in the config directory:

```toml
[whitelist]
allowed_commands = ["activities", "alerts", "entities", "files", "data-enrichment"]

[rate_limit]
requests_per_minute = 60

[watchdog]
idle_timeout_hours = 8
check_interval_secs = 30
```

## Socket Path Resolution

1. `CLOUDAPPS_AGENT_SOCKET` environment variable
2. Auto-discover: scan socket directory, use if exactly one socket exists
3. Fallback to default path

## Module Structure

```
src/agent/
  mod.rs         - Socket path resolution, token generation, PID management
  protocol.rs    - AgentRequest/AgentResponse definitions
  server.rs      - UDS server, fork/daemonize, watchdog, peer verification
  handler.rs     - Token verification, rate limit, whitelist, dispatch
  client.rs      - send_command, status, stop, stop_all
  security.rs    - CommandWhitelist, RateLimiter, AuditLog, UID verification
  peer_verify.rs - macOS code signing / Linux path verification
src/dispatch.rs  - Command dispatch shared between main.rs and agent handler
```
