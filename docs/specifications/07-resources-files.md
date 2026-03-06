# 07 - Resource: Files

## API Endpoints

| Action | Method | Path                       |
|--------|--------|----------------------------|
| list   | POST   | `/api/v1/files/`           |
| fetch  | GET    | `/api/v1/files/{id}/`      |

## CLI Commands

### List Files

```
cloudapps files list [options]
```

Options:

| Option                 | Description                              |
|------------------------|------------------------------------------|
| `--limit <N>`          | Maximum number of results (default: 100) |
| `--skip <N>`           | Number of results to skip                |
| `--all`                | Fetch all results with auto-pagination   |
| `--filter <JSON>`      | Raw JSON filter                          |
| `--service <ID>`       | Filter by service app ID                 |
| `--filetype <TYPE>`    | Filter by file type                      |
| `--filename <NAME>`    | Filter by file name                      |
| `--extension <EXT>`    | Filter by file extension                 |
| `--sharing <LEVEL>`    | Filter by sharing level                  |
| `--owner <ENTITY>`     | Filter by owner entity                   |

### Fetch File

```
cloudapps files fetch <id>
```

## Available Filters

| Filter                  | Type      | Operators                         |
|-------------------------|-----------|-----------------------------------|
| service                 | integer   | eq, neq                           |
| instance                | integer   | eq, neq                           |
| fileType                | integer   | eq, neq                           |
| allowDeleted            | boolean   | eq                                |
| policy                  | string[]  | cabinetmatchedrulesequals, neq, isset, isnotset |
| filename                | string    | eq                                |
| modifiedDate            | timestamp | lte, gte, range, lte_ndays, gte_ndays |
| createdDate             | timestamp | lte, gte, range                   |
| collaborator.entity     | entity pk | eq, neq                           |
| collaborator.domains    | string    | eq, neq                           |
| collaborator.groups     | string    | eq, neq                           |
| collaborator.withDomain | string    | eq, neq, deq                      |
| owner.entity            | entity pk | eq, neq                           |
| owner.orgUnit           | string    | eq, neq                           |
| sharing                 | integer   | eq, neq                           |
| fileId                  | string    | eq, neq                           |
| fileLabels              | string    | eq, neq, isset, isnotset          |
| fileScanLabels          | string    | eq, neq, isset, isnotset          |
| extension               | string    | eq, neq                           |
| mimeType                | string    | eq, neq                           |
| trashed                 | boolean   | eq                                |
| parentFolder            | folder    | eq, neq                           |
| folder                  | boolean   | eq                                |
| quarantined             | boolean   | eq                                |
| snapshotLastModifiedDate| timestamp | lte, gte, range                   |

### File Type Values

| Value | Label         |
|-------|---------------|
| 0     | Other         |
| 1     | Document      |
| 2     | Spreadsheet   |
| 3     | Presentation  |
| 4     | Text          |
| 5     | Image         |
| 6     | Folder        |

### Sharing Level Values

| Value | Label              |
|-------|--------------------|
| 0     | Private            |
| 1     | Internal           |
| 2     | External           |
| 3     | Public             |
| 4     | Public (Internet)  |

## Notes

- This API is not available in Microsoft 365 Cloud App Security.
