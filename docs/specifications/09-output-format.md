# 09 - Output Format

## Overview

The CLI supports multiple output formats to accommodate different use cases.

## Supported Formats

### JSON (default)

Full JSON output of the API response. This is the default format.

```
cloudapps alerts list --output json
```

Output:

```json
{
  "total": 2,
  "hasNext": false,
  "data": [
    {
      "_id": "abc123",
      "title": "Suspicious login",
      "severityValue": 2,
      "timestamp": 1700000000000
    }
  ]
}
```

### Table

Human-readable table format for terminal output.

```
cloudapps alerts list --output table
```

Output:

```
ID       TITLE             SEVERITY  STATUS  TIMESTAMP
abc123   Suspicious login  HIGH      OPEN    2023-11-14T22:13:20Z
def456   Mass download     MEDIUM    OPEN    2023-11-14T21:00:00Z
```

## Timestamp Display

- JSON output: Displays timestamps as epoch milliseconds (raw API value).
- Table output: Converts epoch milliseconds to ISO 8601 format (UTC).

## Exit Codes

| Code | Description                     |
|------|---------------------------------|
| 0    | Success                         |
| 1    | General error                   |
| 2    | Authentication error            |
| 3    | API error (4xx response)        |
| 4    | Network error                   |
| 5    | Invalid input / argument error  |

## Stderr vs Stdout

- Successful data output goes to **stdout**.
- Error messages, verbose logs, and progress indicators go to **stderr**.
- This allows piping output to other commands (e.g., `jq`).

## Verbosity

- Default: Only data output and errors.
- `--verbose`: Include HTTP request/response details (method, URL, status code, timing). Token values are masked.
