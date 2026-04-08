# 17 - Resource: Policies

> **WARNING: Undocumented API**
>
> The policies API is **not listed in the official Microsoft Defender for Cloud Apps REST API documentation**.
> These endpoints are discovered through community research and may change or be removed without notice.
> Use at your own risk.
>
> Official API reference (policies are NOT listed here):
> https://learn.microsoft.com/en-us/defender-cloud-apps/api-introduction

## API Endpoints

| Action | Method | Path                              |
|--------|--------|-----------------------------------|
| list   | GET    | `/api/v1/policies/`               |
| fetch  | GET    | `/api/v1/policy/{type}/{id}/`     |

### Response format

The list endpoint returns a **JSON array** directly, unlike other resources which return `{"data": [...], "total": N, "hasNext": bool}`.

## CLI Commands

### List Policies

```
cloudapps policies list
```

Returns all policies configured in the tenant.

### Fetch Policy

```
cloudapps policies fetch --type <TYPE> --id <ID>
```

Options:

| Option         | Description                  | Required |
|----------------|------------------------------|----------|
| `--type <TYPE>`| Policy type (see below)      | Yes      |
| `--id <ID>`    | Policy ID                    | Yes      |

## Policy Types

| CLI value           | API path segment     | Description                                |
|---------------------|----------------------|--------------------------------------------|
| `activity`          | `activity`           | Activity policies (audit log monitoring)   |
| `anomaly`           | `anomaly`            | Anomaly detection policies                 |
| `discovery`         | `discovery`          | App discovery policies                     |
| `discovery-anomaly` | `discovery_anomaly`  | Cloud discovery anomaly policies           |
| `file`              | `file`               | File / malware detection policies          |
| `app-permissions`   | `app_permissions`    | OAuth app policies                         |
| `session`           | `session`            | Session policies                           |

## Key Response Fields

| Field             | Type    | Description                               |
|-------------------|---------|-------------------------------------------|
| `_id`             | string  | Policy ID                                 |
| `name`            | string  | Policy display name                       |
| `policyType`      | string  | Policy type (`AUDIT`, `ANOMALY_DETECTION`, `FILE`, etc.) |
| `enabled`         | boolean | Whether the policy is active              |
| `alertSeverity`   | array   | Severity level `[int, string]`            |
| `description`     | string  | Policy description                        |
| `enableAlerts`    | boolean | Whether alerts are enabled                |
| `threshold`       | integer | Alert threshold                           |
| `windowSizeInMillis` | integer | Time window in milliseconds            |
| `consoleFilters`  | string  | JSON-encoded filter configuration         |
| `lastModified`    | number  | Last modification timestamp               |
