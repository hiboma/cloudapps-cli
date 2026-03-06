# 04 - Resource: Activities

## API Endpoints

| Action   | Method | Path                          |
|----------|--------|-------------------------------|
| list     | POST   | `/api/v1/activities/`         |
| fetch    | GET    | `/api/v1/activities/{id}/`    |
| feedback | POST   | `/api/v1/activities/{id}/feedback` |

## CLI Commands

### List Activities

```
cloudapps activities list [options]
```

Options:

| Option               | Description                              |
|----------------------|------------------------------------------|
| `--limit <N>`        | Maximum number of results (default: 100) |
| `--skip <N>`         | Number of results to skip                |
| `--all`              | Fetch all results with auto-pagination   |
| `--filter <JSON>`    | Raw JSON filter                          |
| `--service <ID>`     | Filter by service app ID                 |
| `--date-gte <DATE>`  | Filter activities after this date        |
| `--date-lte <DATE>`  | Filter activities before this date       |
| `--user <USERNAME>`  | Filter by username                       |
| `--ip <ADDRESS>`     | Filter by IP address                     |
| `--country <CODE>`   | Filter by country code                   |

### Fetch Activity

```
cloudapps activities fetch <id>
```

### Activity Feedback

```
cloudapps activities feedback <id> --feedback <VALUE>
```

## Available Filters

| Filter                  | Type      | Operators                        |
|-------------------------|-----------|----------------------------------|
| service                 | integer   | eq, neq                          |
| instance                | integer   | eq, neq                          |
| user.orgUnit            | string    | eq, neq, isset, isnotset         |
| actionType              | string    | contains, eq, neq, isset, isnotset|
| activity.eventActionType| string    | eq, neq                          |
| activity.id             | string    | eq                               |
| activity.impersonated   | boolean   | eq                               |
| activity.type           | boolean   | eq                               |
| activity.takenAction    | string    | eq, neq                          |
| device.type             | string    | eq, neq                          |
| device.tags             | string    | eq, neq                          |
| userAgent.userAgent     | string    | contains, ncontains              |
| userAgent.tags          | string    | eq, neq                          |
| location.country        | string    | eq, neq, isset, isnotset         |
| location.organizations  | string    | eq, neq, isset, isnotset         |
| ip.address              | string    | eq, startswith, doesnotstartwith, isset, isnotset, neq |
| ip.category             | integer   | eq, neq                          |
| ip.tags                 | string    | eq, neq                          |
| text                    | string    | eq, startswithsingle, text       |
| date                    | timestamp | lte, gte, range, lte_ndays, gte_ndays |
| source                  | string    | eq, neq                          |
| activity.alertId        | string    | eq                               |
| user.username           | string    | eq, neq, isset, isnotset, startswith |
| user.tags               | string    | eq, neq, isset, isnotset, startswith |
| user.domain             | string    | eq, neq, isset, isnotset         |
