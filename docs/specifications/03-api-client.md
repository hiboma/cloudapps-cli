# 03 - API Client

## Overview

The API client module encapsulates HTTP communication with the Microsoft Defender for Cloud Apps REST API.

## Base URL

The API URL format is:

```
https://<portal_url>/api/<endpoint>
```

Example:

```
https://mytenant.us2.portal.cloudappsecurity.com/api/v1/activities/
```

## Request/Response Format

### Request Headers

All requests include:

```
Authorization: Token <your_token_key>
Content-Type: application/json
```

### Response Format

Responses are JSON. List endpoints return:

```json
{
  "total": 100,
  "hasNext": true,
  "data": [...]
}
```

## Rate Limiting

- Throttle limit: 30 requests per minute per tenant.
- The client implements retry with exponential backoff when receiving `429 Too Many Requests`.
- Default retry count: 3
- Backoff intervals: 2s, 4s, 8s

## Pagination

- Default limit: 100 items per request.
- The `hasNext` field indicates whether more records exist.
- The client supports automatic pagination with `--all` flag.
- Manual pagination is supported via `--limit` and `--skip` options.

## Filtering

Filters are sent in the request body as JSON:

```json
{
  "filters": {
    "field_name": {
      "operator": ["value1", "value2"]
    }
  },
  "skip": 0,
  "limit": 100
}
```

### Supported Filter Operators

| Operator          | Value Type   | Description                                    |
|-------------------|-------------|------------------------------------------------|
| `eq`              | list        | Equals any of the specified values             |
| `neq`             | list        | Not equal to any of the specified values       |
| `contains`        | string list | Contains any of the specified strings          |
| `ncontains`       | string list | Does not contain any of the specified strings  |
| `startswith`      | string list | Starts with any of the specified strings       |
| `doesnotstartwith`| string list | Does not start with any of the specified strings|
| `endswith`        | string list | Ends with any of the specified strings         |
| `gt`              | single      | Greater than the specified value               |
| `gte`             | single      | Greater than or equal to the specified value   |
| `lt`              | single      | Less than the specified value                  |
| `lte`             | single      | Less than or equal to the specified value      |
| `isset`           | boolean     | Field has a value                              |
| `isnotset`        | boolean     | Field does not have a value                    |
| `range`           | object list | Within the specified range(s)                  |
| `gte_ndays`       | number      | Date is after N days ago                       |
| `lte_ndays`       | number      | Date is before N days ago                      |
| `text`            | string      | Full text search                               |

## Error Handling

- HTTP 4xx errors: Display the error message from the API response body.
- HTTP 5xx errors: Retry with exponential backoff, then display error.
- Network errors: Display connection error with the target URL.
- Authentication errors (401): Display a message suggesting token verification.
- Rate limit errors (429): Automatically retry with backoff.

## Timestamps

- The API uses Unix timestamps in milliseconds (epoch milliseconds).
- The CLI accepts human-readable date formats (ISO 8601) and converts them to epoch milliseconds.
- Output displays timestamps in ISO 8601 format by default.

## Module Structure

```
src/
  client/
    mod.rs          // CloudAppsClient struct and builder
    request.rs      // Request building and execution
    response.rs     // Response parsing
    pagination.rs   // Pagination handling
    retry.rs        // Retry logic with backoff
```
