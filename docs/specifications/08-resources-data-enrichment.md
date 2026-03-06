# 08 - Resource: Data Enrichment (Subnets)

## API Endpoints

| Action | Method | Path                        |
|--------|--------|-----------------------------|
| list   | GET    | `/api/subnet/`              |
| create | POST   | `/api/subnet/create_rule/`  |
| update | POST   | `/api/subnet/{id}/update_rule/` |
| delete | DELETE | `/api/subnet/{id}/`         |

## CLI Commands

### List IP Ranges

```
cloudapps data-enrichment list [options]
```

Options:

| Option              | Description                              |
|---------------------|------------------------------------------|
| `--limit <N>`       | Maximum number of results (default: 100) |
| `--skip <N>`        | Number of results to skip                |
| `--all`             | Fetch all results with auto-pagination   |
| `--filter <JSON>`   | Raw JSON filter                          |
| `--category <CAT>`  | Filter by category                       |
| `--tag <TAG_ID>`    | Filter by tag ID                         |
| `--builtin`         | Show only built-in ranges                |
| `--custom`          | Show only custom ranges                  |

### Create IP Range

```
cloudapps data-enrichment create [options]
```

Options:

| Option                   | Description                    |
|--------------------------|--------------------------------|
| `--name <NAME>`          | Name of the IP range (required)|
| `--subnets <SUBNETS>`    | Comma-separated list of subnets (required) |
| `--category <CATEGORY>`  | Category (required)            |
| `--organization <ORG>`   | Registered ISP                 |
| `--tags <TAG_IDS>`       | Comma-separated list of tag IDs|

### Update IP Range

```
cloudapps data-enrichment update <id> [options]
```

Options: Same as `create`.

### Delete IP Range

```
cloudapps data-enrichment delete <id>
```

## Response Properties

| Property     | Type   | Description                              |
|--------------|--------|------------------------------------------|
| _id          | string | Unique ID of the IP range                |
| name         | string | Unique name of the range                 |
| subnets      | list   | Array of masks, IP addresses, and original strings |
| location     | string | Object with name, latitude, longitude, country code, country name |
| organization | string | Registered ISP                           |
| tags         | list   | Array of tag objects                     |
| category     | int    | IP range category                        |
| lastModified | long   | Timestamp of last modification           |

### Category Values

| Value | Label          |
|-------|----------------|
| 1     | Corporate      |
| 2     | Administrative |
| 3     | Risky          |
| 4     | VPN            |
| 5     | Cloud Provider |
| 6     | Other          |

## Available Filters

| Filter   | Type    | Operators |
|----------|---------|-----------|
| category | integer | eq, neq   |
| tags     | string  | eq, neq   |
| builtIn  | bool    | eq        |
