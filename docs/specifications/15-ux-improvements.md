# 15 - CLI UX Improvements

## Overview

The raw API uses integer codes, complex JSON filter syntax, and epoch-millisecond timestamps.
This document defines how the CLI translates these into a human-friendly interface.

---

## 1. Named Enum Values Instead of Magic Numbers

The CLI accepts and displays human-readable names. Integer codes are handled internally.

### Alerts

#### `--severity`

| CLI value       | API value |
|-----------------|-----------|
| `low`           | 0         |
| `medium`        | 1         |
| `high`          | 2         |
| `informational` | 3         |

#### `--resolution`

| CLI value        | API value |
|------------------|-----------|
| `open`           | 0         |
| `dismissed`      | 1         |
| `resolved`       | 2         |
| `false-positive` | 3         |
| `benign`         | 4         |
| `true-positive`  | 5         |

#### Table output mapping

| Field                  | Raw value | Display       |
|------------------------|-----------|---------------|
| `severityValue`        | 2         | `HIGH`        |
| `resolutionStatusValue`| 0         | `OPEN`        |
| `statusValue`          | 0         | `UNREAD`      |
| `intent` values        | 2         | `INITIAL_ACCESS` (MITRE ATT&CK) |

### Files

#### `--filetype`

| CLI value      | API value |
|----------------|-----------|
| `other`        | 0         |
| `document`     | 1         |
| `spreadsheet`  | 2         |
| `presentation` | 3         |
| `text`         | 4         |
| `image`        | 5         |
| `folder`       | 6         |

#### `--sharing`

| CLI value  | API value |
|------------|-----------|
| `private`  | 0         |
| `internal` | 1         |
| `external` | 2         |
| `public`   | 3         |
| `internet` | 4         |

### Activities

#### `--ip-category`

| CLI value        | API value |
|------------------|-----------|
| `corporate`      | 1         |
| `administrative` | 2         |
| `risky`          | 3         |
| `vpn`            | 4         |
| `cloud-provider` | 5         |
| `other`          | 6         |

#### `--device-type`

| CLI value  | API value   |
|------------|-------------|
| `desktop`  | `DESKTOP`   |
| `mobile`   | `MOBILE`    |
| `tablet`   | `TABLET`    |
| `other`    | `OTHER`     |

### Entities

#### `--status`

| CLI value   | API value |
|-------------|-----------|
| `na`        | 0         |
| `staged`    | 1         |
| `active`    | 2         |
| `suspended` | 3         |
| `deleted`   | 4         |

### Data Enrichment

#### `--category`

| CLI value        | API value |
|------------------|-----------|
| `corporate`      | 1         |
| `administrative` | 2         |
| `risky`          | 3         |
| `vpn`            | 4         |
| `cloud-provider` | 5         |
| `other`          | 6         |

---

## 2. Shorthand Filter Options

Instead of requiring raw JSON filters, the CLI provides typed options that are common in daily operations.
The `--filter` option is still available for advanced queries.

### Activities

```bash
# Instead of: --filter '{"date":{"gte":1700000000000},"user.username":{"eq":["user@example.com"]}}'
cloudapps activities list --date-gte 2024-01-01 --user user@example.com

# Instead of: --filter '{"ip.address":{"eq":["192.0.2.1"]},"location.country":{"eq":["JP"]}}'
cloudapps activities list --ip 192.0.2.1 --country JP

# Instead of: --filter '{"date":{"gte_ndays":7}}'
cloudapps activities list --last 7d

# Instead of: --filter '{"device.type":{"eq":["MOBILE"]}}'
cloudapps activities list --device-type mobile
```

### Alerts

```bash
# Instead of: --filter '{"severity":{"eq":[2]},"alertOpen":{"eq":true}}'
cloudapps alerts list --severity high --open

# Instead of: --filter '{"date":{"gte_ndays":1}}'
cloudapps alerts list --last 24h

# Instead of: --filter '{"resolutionStatus":{"eq":[0]},"severity":{"eq":[2]}}'
cloudapps alerts list --resolution open --severity high
```

### Files

```bash
# Instead of: --filter '{"sharing":{"eq":[4]},"fileType":{"eq":[1]}}'
cloudapps files list --sharing internet --filetype document

# Instead of: --filter '{"extension":{"eq":["pdf"]}}'
cloudapps files list --extension pdf
```

### Shorthand `--last` duration syntax

The `--last` option accepts a human-readable duration format as a shorthand for `gte_ndays` / `gte` timestamp filters.

| Input   | Meaning              |
|---------|----------------------|
| `1h`    | Last 1 hour          |
| `24h`   | Last 24 hours        |
| `7d`    | Last 7 days          |
| `30d`   | Last 30 days         |

Implementation: Convert the duration to an epoch-millisecond timestamp and apply a `gte` filter on the `date` field.

---

## 3. Timestamp Handling

### Input

The CLI accepts multiple timestamp formats and converts them to epoch milliseconds internally.

| Format                    | Example                  |
|---------------------------|--------------------------|
| ISO 8601                  | `2024-01-15T10:30:00Z`   |
| Date only (UTC 00:00:00)  | `2024-01-15`             |
| Relative (`--last`)       | `7d`, `24h`              |
| Epoch milliseconds (raw)  | `1705312200000`          |

### Output

| Output mode | Display format                          |
|-------------|-----------------------------------------|
| `json`      | Epoch milliseconds (raw API value)      |
| `table`     | ISO 8601 UTC (e.g., `2024-01-15T10:30:00Z`) |

---

## 4. Unified `alerts close` Subcommand

Instead of three separate commands, provide a single `close` command with a `--as` option.

### Before (API-mirroring)

```bash
cloudapps alerts close-benign <id>
cloudapps alerts close-false-positive <id>
cloudapps alerts close-true-positive <id>
```

### After (improved)

```bash
cloudapps alerts close <id> --as benign [--comment "..."]
cloudapps alerts close <id> --as false-positive [--comment "..."]
cloudapps alerts close <id> --as true-positive [--comment "..."]
```

The `--as` option is required. Accepted values: `benign`, `false-positive`, `true-positive`.

The original three subcommands (`close-benign`, `close-false-positive`, `close-true-positive`) are retained as hidden aliases for scripting compatibility.

---

## 5. Bulk Operations

For alerts and activities, allow operating on multiple IDs at once.

```bash
# Close multiple alerts
cloudapps alerts close <id1> <id2> <id3> --as benign

# Mark multiple alerts as read
cloudapps alerts mark-read <id1> <id2> <id3>

# Pipe IDs from another command
cloudapps alerts list --severity high --open --output json \
  | jq -r '.data[]._id' \
  | xargs cloudapps alerts close --as true-positive --comment "batch triage"
```

Implementation: Accept multiple positional arguments for the ID field. Execute API calls sequentially, respecting the rate limit (30 req/min). Display progress on stderr.

---

## 6. `--query` for Full-Text Search

Activities and alerts support full-text search via the `text` filter operator.
The CLI exposes this as a `--query` option.

```bash
cloudapps activities list --query "failed login"
cloudapps alerts list --query "suspicious"
```

Translates to:

```json
{
  "filters": {
    "text": {
      "text": "failed login"
    }
  }
}
```

---

## 7. Table Output Column Selection

The default table output shows a fixed set of key columns per resource.
Users can customize visible columns with `--columns`.

```bash
cloudapps alerts list --output table --columns id,title,severity,timestamp
```

### Default Table Columns

| Resource        | Default columns                                     |
|-----------------|-----------------------------------------------------|
| activities      | timestamp, user, action, ip, country, service       |
| alerts          | id, title, severity, resolution, timestamp          |
| entities        | id, type, name, domain, status                      |
| files           | id, filename, filetype, sharing, owner, modified    |
| data-enrichment | id, name, subnets, category, organization           |

---

## Summary of Changes to Existing Specs

| Spec file | Changes required                                          |
|-----------|-----------------------------------------------------------|
| 04        | Add `--last`, `--query`, `--device-type`, `--ip-category`, bulk support |
| 05        | Add `--last`, `--query`, unified `close --as`, bulk support |
| 07        | Named values for `--filetype`, `--sharing`                |
| 08        | Named values for `--category`                             |
| 09        | Add `--columns` option, default column definitions        |
