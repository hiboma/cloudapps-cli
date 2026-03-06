# 05 - Resource: Alerts

## API Endpoints

| Action              | Method | Path                                    |
|---------------------|--------|-----------------------------------------|
| list                | POST   | `/api/v1/alerts/`                       |
| fetch               | GET    | `/api/v1/alerts/{id}/`                  |
| close-benign        | POST   | `/api/v1/alerts/close_benign/`          |
| close-false-positive| POST   | `/api/v1/alerts/close_false_positive/`  |
| close-true-positive | POST   | `/api/v1/alerts/close_true_positive/`   |
| mark-read           | POST   | `/api/v1/alerts/{id}/read/`             |
| mark-unread         | POST   | `/api/v1/alerts/{id}/unread/`           |

## CLI Commands

### List Alerts

```
cloudapps alerts list [options]
```

Options:

| Option                    | Description                              |
|---------------------------|------------------------------------------|
| `--limit <N>`             | Maximum number of results (default: 100) |
| `--skip <N>`              | Number of results to skip                |
| `--all`                   | Fetch all results with auto-pagination   |
| `--filter <JSON>`         | Raw JSON filter                          |
| `--severity <LEVEL>`      | Filter by severity: low, medium, high    |
| `--resolution <STATUS>`   | Filter by resolution status              |
| `--open`                  | Show only open alerts                    |
| `--closed`                | Show only closed alerts                  |
| `--date-gte <DATE>`       | Filter alerts after this date            |
| `--date-lte <DATE>`       | Filter alerts before this date           |

### Fetch Alert

```
cloudapps alerts fetch <id>
```

### Close Alert

```
cloudapps alerts close --close-as <CLOSE_AS> <IDS>... [--comment <TEXT>]
```

Close type (`--close-as`): `benign`, `false-positive`, `true-positive`

The close API is a bulk endpoint. Multiple alert IDs can be specified at once. The request body uses `filters.id.eq` to specify target alert IDs.

#### CLI Examples

```
cloudapps alerts close --close-as true-positive ALERT_ID1 ALERT_ID2
cloudapps alerts close --close-as benign ALERT_ID --comment "reason"
cloudapps alerts close --close-as false-positive ALERT_ID
```

Request body example:

```json
{
  "filters": {
    "id": {
      "eq": ["alert-id-1", "alert-id-2"]
    }
  },
  "comment": "optional comment"
}
```

### Mark Read/Unread

```
cloudapps alerts mark-read <id>
cloudapps alerts mark-unread <id>
```

## Response Properties

| Property            | Type    | Description                              |
|---------------------|---------|------------------------------------------|
| _id                 | int     | Alert type identifier                    |
| timestamp           | long    | Timestamp when the alert was raised      |
| entities            | list    | Entities related to the alert            |
| title               | string  | Alert title                              |
| description         | string  | Alert description                        |
| statusValue         | int     | 0: Unread, 1: Read, 2: Archived         |
| severityValue       | int     | 0: Low, 1: Medium, 2: High, 3: Informational |
| resolutionStatusValue| int    | 0: Open, 1: Dismissed, 2: Resolved, 3: False Positive, 4: Benign, 5: True Positive |
| stories             | list    | Risk categories                          |
| evidence            | list    | Brief descriptions of alert key parts    |
| intent              | list    | Kill chain related intent (MITRE ATT&CK) |

## Available Filters

| Filter             | Type      | Operators          |
|--------------------|-----------|--------------------|
| entity.entity      | entity pk | eq, neq            |
| entity.ip          | string    | eq, neq            |
| entity.service     | integer   | eq, neq            |
| entity.instance    | integer   | eq, neq            |
| entity.policy      | string    | eq, neq            |
| entity.file        | string    | eq, neq            |
| alertOpen          | boolean   | eq                 |
| severity           | integer   | eq, neq            |
| resolutionStatus   | integer   | eq, neq            |
| read               | boolean   | eq                 |
| date               | timestamp | lte, gte, range, lte_ndays, gte_ndays |
| resolutionDate     | timestamp | lte, gte, range    |
| risk               | integer   | eq, neq            |
| alertType          | integer   | eq, neq            |
| id                 | string    | eq, neq            |
| source             | string    | eq                 |
