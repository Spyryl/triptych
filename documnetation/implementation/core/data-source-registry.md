# Data Source Registry

## Runtime Contract

`setting.tlkp_data_source` is warmed at startup and becomes the runtime registry
for core-managed database sources.

This registry is source inventory, not field inventory. Rust `data/fields`
modules own field names, types, defaults, and persisted/virtual distinction.
The database registry owns which SQL objects exist for each `record_key`.

The warmed registry answers:

- What table does this record write to?
- What view does this record read by default?
- Is this alternate view allowed?
- What SQL object does this alias resolve to?

## Core Types

Rust core provides:

- `DataObjectType`
- `DataSourceRole`
- `DataSourceDef`
- `RecordDataSources`
- `DataSourceRegistry`

These types model warmed rows from `setting.vw_data_source`.

## Bootstrap Fetch

The registry data itself is fetched through normal managed data access:

```rust
let mut table = DataSourceTable::new(pool)?;
let rows = table.fetch_view(FetchArgs::new()).await?;
let registry = DataSourceRegistry::from_sources(rows.into_iter().map(/* convert */));
```

`DataSourceRec` lives in `src/data/records` because it is a DB-backed `*Rec`.
The warmed `DataSourceRegistry` lives in `src/core/warmup` because it is runtime
infrastructure.

## Table Manager Direction

Current managers may still use hardcoded `ManagedTableConfig` while the project is
being bootstrapped.

The target manager construction should consume warmed source metadata instead of
hardcoded view aliases.

Transitional:

```rust
ManagedTableConfig::new("log.tbl_log", "log.vw_log")
    .with_view_alias("warn", "log.vw_log_warn")
```

Target:

```rust
let sources = registry.record("log")?;
let config = ManagedTableConfig::from_record_sources(sources)?;
```

## Hydration Direction

Records should be default-first:

1. `Rec::default()`
2. apply every column present in the returned row
3. leave absent fields at their defaults
4. scrub
5. mark clean

This allows multiple views with different column sets to hydrate into the same
`Rec` without a Rust view field list for every view.

## Deployment

Add a row to `setting.tlkp_data_source` whenever a table or view becomes a
core-managed application source.

Changing source inventory should be a data/deployment change. Rust changes are
needed only when a view introduces new fields the `Rec` does not yet know how to
hydrate.

Source seed rows should include bounded metadata such as `tier` and `purpose`.
They should be idempotent: insert missing rows without clobbering admin or
maintenance edits to existing source rows.

The foundational seed is:

```text
documentation/deployment/database/seed/data-source.sql
```

It currently registers eleven sources for `data_source`, `error_lookup`, `error`,
and `log`.
