# Architecture Context Document: Data Source Registry

## Purpose

Batchworks uses the Core table/record architecture across projects:

```text
SQL table/view -> ManagedTable-backed *Table -> *Rec -> operational code
```

Application code must not bypass the table's `*Rec`. The `*Rec` is the
cleanliness and correctness boundary for both persisted fields and view/virtual
fields.

The database can have many views for a single table concept. Those views are not
Rust record types. They are read surfaces for the same table-backed `*Rec`.

## Problem

Hardcoding every allowed view in Rust does not scale.

In a 100-table project with several views per table, a Rust list of every view
becomes a second manually maintained database catalog. That violates the lazy
doctrine: database object inventory belongs to the database.

## Decision

Database source inventory is stored in `setting.tlkp_data_source`.

Rust warms that table and uses it to resolve:

- the write table for a record concept
- the default read view
- allowed alternate views
- caller-facing view aliases

Rust still owns field metadata because the `*Rec` needs type/default rules for
clean hydration. The database owns object inventory because SQL objects can be
added, renamed, disabled, or aliased without editing Rust source.

The initial warmup fetch still goes through the data layer:

```text
DataSourceTable -> DataSourceRec -> DataSourceRegistry
```

That keeps bootstrap inside the same table/record discipline as the rest of the
project.

## Rules

- Every application-consumed table/view goes through a `ManagedTable`-backed
  `*Table` wrapper and a `*Rec`.
- A consumed view belongs to the same `Rec` as its table concept.
- `Rec` structs are supersets: persisted fields plus virtual/view fields.
- Hydration is default-first and present-column-only.
- Missing columns keep the `Rec` default.
- Database-owned source inventory decides whether a view is allowed.
- Adding a view should normally require SQL + data-source metadata, not Rust.
- Adding a new field exposed by a view still requires field/record support.

## Data Model

`setting.tlkp_data_source` stores one row per core-managed table/view source:

- `record_key`: stable Rust/core concept key, such as `error_lookup` or `log`
- `schema_name`: PostgreSQL schema
- `object_name`: PostgreSQL table/view name
- `object_type`: `table` or `view`
- `source_role`: `table`, `default_view`, or `view`
- `alias_key`: short caller key for alternate views
- `actv` / `dltd`: active/deleted controls
- `metadata`: optional deployment/runtime metadata

Example rows for log:

```text
record_key | schema | object       | type  | role         | alias
log        | log    | tbl_log      | table | table        |
log        | log    | vw_log       | view  | default_view |
log        | log    | vw_log_warn  | view  | view         | warn
log        | log    | vw_log_info  | view  | view         | info
log        | log    | vw_log_debug | view  | view         | debug
```

## Multi-View Managers

The old hardcoded pattern:

```rust
ManagedTableConfig::new("log.tbl_log", "log.vw_log")
    .with_view_alias("warn", "log.vw_log_warn")
    .with_view_alias("info", "log.vw_log_info")
    .with_view_alias("debug", "log.vw_log_debug")
```

is transitional only.

The target pattern is:

```rust
let sources = registry.record("log")?;
let config = ManagedTableConfig::from_record_sources(sources)?;
```

or equivalent construction through a warmup registry.

The table manager still stays thin. It should not manually know every view once
the data-source registry is wired in.

## Hydration

Views may return different columns. That must not break a `Rec`.

The `Rec` starts from `Default`, then hydrates only columns present in the row.
Fields absent from the selected view remain at their default values.

This preserves the WinDev/Core behavior:

```text
declare fields with defaults
fill what exists
missing values remain clean defaults
```

## Field Metadata

Rust `FieldDef` owns the shape and default behavior of fields:

- field name
- field type
- persisted or virtual source
- nullable/required
- max length
- default
- primary-key marker

Field metadata is not a view registry. Views are database objects and are
registered in `setting.tlkp_data_source`.

## Foundational Seed

The bootstrap seed is:

```text
documentation/deployment/database/seed/data-source.sql
```

It registers the first four managed record concepts:

- `data_source`
- `error_lookup`
- `error`
- `log`

The seed inserts eleven rows because the registry stores one row per database
source: table, default view, and each alternate view alias.
