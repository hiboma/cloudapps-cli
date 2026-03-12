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

Stops the agent process.

```
cloudapps agent stop [--socket PATH] [--all]
```

### `cloudapps agent status`

Shows the agent's running status.

```
cloudapps agent status [--socket PATH]
```

## Auto-routing

When `CLOUDAPPS_AGENT_TOKEN` is set, all commands are automatically routed through the agent. No explicit flag is required.

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
  "status": "success",
  "output": "...",
  "error": null
}
```

## Security

See ADR-0001 for the 10-layer security model.

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
  server.rs      - UDS server, fork/daemonize, watchdog
  handler.rs     - Token verification, rate limit, whitelist, dispatch
  client.rs      - send_command, status, stop, stop_all
  security.rs    - CommandWhitelist, RateLimiter, AuditLog, UID verification
  peer_verify.rs - macOS code signing / Linux path verification
```
