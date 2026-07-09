# Foundational Data Records

## Purpose

Batchworks has four database-backed data concepts that must exist before normal
business records can be built safely:

- data-source inventory
- reusable error lookup templates
- occurred errors
- operational logs

They are regular data-layer records and table wrappers, even though two of them
feed warmup.

## Files

Fields:

```text
src/data/fields/data_source.rs
src/data/fields/error_lookup.rs
src/data/fields/error.rs
src/data/fields/log.rs
```

Records:

```text
src/data/records/data_source_rec.rs
src/data/records/error_lookup_rec.rs
src/data/records/error_rec.rs
src/data/records/log_rec.rs
```

Tables:

```text
src/data/tables/data_source.rs
src/data/tables/error_lookup.rs
src/data/tables/error.rs
src/data/tables/log.rs
```

## DataSourceRec

`DataSourceRec` is the DB-backed record for `setting.tlkp_data_source`.

Runtime role:

```text
DataSourceTable.fetch_view()
-> Vec<DataSourceRec>
-> DataSourceRegistry
-> warmup source inventory
```

The default view is `setting.vw_data_source`. It omits `actv` and `dltd` because
the view predicate already filters active, non-deleted rows. `DataSourceRec`
keeps those fields as persisted table fields and defaults them during view
hydration.

## ErrorLookupRec

`ErrorLookupRec` is the DB-backed record for `log.tlkp_error`.

It is not an occurred error. It is the reusable template/catalog row:

```text
code
description
message_template
severity
status_code
domain
metadata
```

Runtime role:

```text
ErrorLookupTable.fetch_view()
-> Vec<ErrorLookupRec>
-> warmup error catalog
-> ErrorManager template lookup
```

The default view is `log.vw_error_lookup`. It omits `actv` and `dltd`, matching
the same active/non-deleted lookup-view rule as `DataSourceRec`.

## ErrorRec

`ErrorRec` is an occurred error instance backed by `log.tbl_error`.

It is the database-facing version of a failure occurrence. It can be built from
`DiagnosticEnvelope` once the data layer is available.

`ErrorRec` table and view fields are intentionally the same. Error inspection and
debugging should not lose fields through the normal view.

Errors are written to `log.tbl_error`, not to `log.tbl_log`.

## LogRec

`LogRec` is an operational log instance backed by `log.tbl_log`.

Allowed `log_type` values are:

```text
warn
info
debug
```

Errors are excluded because they have the dedicated `ErrorRec` / `log.tbl_error`
path.

Log views are filter-only surfaces. They all return the same fields:

```text
log.vw_log
log.vw_log_warn
log.vw_log_info
log.vw_log_debug
```

Adding another filtered log view should be a database + data-source seed change,
not a Rust record-shape change, unless it introduces new fields.

## Seed

The source registry seed lives at:

```text
documentation/deployment/database/seed/data-source.sql
```

It registers the foundational sources:

- `data_source`: table + default view
- `error_lookup`: table + default view
- `error`: table + default view
- `log`: table + default view + `warn`/`info`/`debug` aliases

That is eleven rows because the registry stores one row per database source.

The seed is idempotent. It inserts missing rows and does not overwrite existing
maintenance/admin changes.

## Dependency Rule

Core does not import these records.

The dependency direction is:

```text
core/foundation -> DiagnosticEnvelope
core/db         -> ManagedTable / ManagedRecord contracts
core/warmup     -> source registry contracts
data            -> DataSourceRec, ErrorLookupRec, ErrorRec, LogRec
```

`DiagnosticEnvelope` is shared, but its level is not a loose string. Log rows
use `LogLevel` values for `log_type`; error rows use `ErrorSeverity` values for
`severity`.

Warmup can use data-layer table wrappers during application startup, but generic
core warmup contracts must not depend on concrete project records.
