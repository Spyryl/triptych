# Architecture Context Document: Rust Core Tiers

## Purpose

Batchworks core is not a flat utility folder. It has boot order and dependency
direction.

The Rust layout makes those tiers physical so bootstrap code, database access,
warmup, diagnostics, and project data do not become circular.

## Tier 0: Foundation

Folder:

```text
src/core/foundation/
```

Foundation is available before database connections, warmup caches, or project
records exist.

Foundation owns:

- `DiagnosticEnvelope`
- `ManagedError` / `CoreError`
- config and secrets loading
- diagnostic and debug logging primitives
- field definitions
- normalization, number, sort, and time helpers

Foundation must not depend on:

- `src/data`
- `ErrorRec`
- `LogRec`
- `ErrorLookupRec`
- warmed lookup caches
- database manager construction

If anything fails during bootstrap, foundation must still be able to shape the
failure as a `DiagnosticEnvelope`.

## Tier 1: DB

Folder:

```text
src/core/db/
```

DB owns generic PostgreSQL-facing contracts and table access:

- `ManagedTable`
- `ManagedTableConfig`
- `ManagedRecordState`
- `ManagedRecord`
- `FieldValue`
- locking helpers
- database error conversions into `CoreError`

DB may depend on foundation. DB must not depend on warmup or project data.

This keeps generic table access usable before `setting.tlkp_data_source` and
`log.tlkp_error` have been warmed.

## Tier 2: Warmup

Folder:

```text
src/core/warmup/
```

Warmup owns runtime registries loaded after the PostgreSQL pool exists:

- `DataSourceRegistry`
- `RecordDataSources`
- `DataSourceDef`
- registry bridges such as `ManagedTableConfig::from_record_sources`

Warmup may depend on foundation and DB. Warmup must not depend on project
records directly unless the module is explicitly a project warmup module outside
generic core.

## Project Data

Folder:

```text
src/data/
```

Project data owns DB-backed records and table wrappers:

- `DataSourceRec`
- `ErrorLookupRec`
- `ErrorRec`
- `LogRec`
- business `*Rec`
- business `*Table`
- fields, bricks, structures, virtual records

Project data may convert a `DiagnosticEnvelope` into `ErrorRec` or `LogRec`.
Project warmup may use `DataSourceTable` and `ErrorLookupTable` to fill runtime
caches. Generic core must not import those records.

## Diagnostics

`DiagnosticEnvelope` is the lowest-tier diagnostic shape.

It is not a `Rec` and not a `clx`. It is the core transport envelope used when
anything needs to report what happened.

It has one common shape, but its level is intentionally split by diagnostic
kind:

- log diagnostics use `LogLevel`: `warn`, `info`, `debug`
- error diagnostics use `ErrorSeverity`: `error`, `critical`, `fatal`

This keeps shared fields such as source, code, message, context, target table,
and created time in one envelope while preventing log partition values from
being confused with error severities.

Preferred runtime flow:

```text
build DiagnosticEnvelope
try database sink through ErrorRec / LogRec
fall back to dated JSONL file when database write is unavailable
```

Database availability changes the sink, not the diagnostic shape.

## Errors

`ErrorLookupRec` is the warmed template/catalog row from `log.tlkp_error`.

`ErrorRec` is the actual occurred error row for `log.tbl_error`.

Foundation `CoreError` does not use `ErrorLookupRec`. It creates disciplined
structured errors before warmup exists. After warmup, higher-level error
management can use `ErrorLookupRec` templates to render standardized messages
and then write `ErrorRec` through the database-first/file-second sink chain.
