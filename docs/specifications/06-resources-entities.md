# 06 - Resource: Entities

## API Endpoints

| Action     | Method | Path                              |
|------------|--------|-----------------------------------|
| list       | POST   | `/api/v1/entities/`               |
| fetch      | GET    | `/api/v1/entities/{id}/`          |
| fetch-tree | GET    | `/api/v1/entities/{id}/tree/`     |

## CLI Commands

### List Entities

```
cloudapps entities list [options]
```

Options:

| Option              | Description                              |
|---------------------|------------------------------------------|
| `--limit <N>`       | Maximum number of results (default: 100) |
| `--skip <N>`        | Number of results to skip                |
| `--all`             | Fetch all results with auto-pagination   |
| `--filter <JSON>`   | Raw JSON filter                          |
| `--type <TYPE>`     | Filter by entity type                    |
| `--is-admin`        | Filter admin entities only               |
| `--is-external`     | Filter external entities only            |
| `--domain <DOMAIN>` | Filter by domain                         |
| `--status <STATUS>` | Filter by status                         |

### Fetch Entity

```
cloudapps entities fetch <id>
```

### Fetch Entity Tree

```
cloudapps entities fetch-tree <id>
```

## Available Filters

| Filter       | Type      | Operators                         |
|--------------|-----------|-----------------------------------|
| type         | string    | eq, neq                           |
| isAdmin      | string    | eq                                |
| entity       | entity pk | eq, neq                           |
| userGroups   | string    | eq, neq                           |
| app          | integer   | eq, neq                           |
| instance     | integer   | eq, neq                           |
| isExternal   | boolean   | eq                                |
| domain       | string    | eq, neq, isset, isnotset          |
| organization | string    | eq, neq, isset, isnotset          |
| status       | string    | eq, neq                           |

### Status Values

| Value | Label    |
|-------|----------|
| 0     | N/A      |
| 1     | Staged   |
| 2     | Active   |
| 3     | Suspended|
| 4     | Deleted  |

## Notes

- This API is not available in Microsoft 365 Cloud App Security.
