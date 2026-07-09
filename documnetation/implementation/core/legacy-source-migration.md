# Legacy Source Migration

Mandate legacy-source migration is explicit and read-only until an apply command
exists. The first supported command is a source inspection plan:

```bash
mandate migration legacy-source plan
```

The planner connects to the configured legacy source database and reports:

- expected legacy source tables
- Mandate target tables
- import phase order
- source row counts
- missing required source tables
- whether the source is ready for a later apply command

Default source connection:

```text
host: 127.0.0.1
port: 5432
database: legacy_source
user: mandate_import
schema: legacy
```

Override with:

```text
MANDATE_IMPORT_DB_HOST
MANDATE_IMPORT_DB_PORT
MANDATE_IMPORT_DB_NAME
MANDATE_IMPORT_DB_USER
MANDATE_IMPORT_DB_PASSWORD
MANDATE_IMPORT_SCHEMA
```

The plan command does not write Mandate rows. A future apply/import command must
remain separate and explicit.

## Mapping

| Legacy source | Mandate target | Phase |
| --- | --- | --- |
| `<schema>.tbl_project` | `control.tbl_project` | 1 |
| `<schema>.tbl_repair` | `repair.tbl_repair` | 2 |
| `<schema>.stb_repair_documentation` | `repair.stb_repair_doc` | 3 |
| `<schema>.stb_repair_event` | `repair.stb_repair_event` | 4 |
| `<schema>.tbl_finding` | `finding.tbl_finding` | 5 |
| `<schema>.stb_finding_repair` | `control.stb_work_link` | 6 |

Finding rows remain quarantine data. Accepted finding/repair relationships become
controlled work links rather than implicit repair ownership.
