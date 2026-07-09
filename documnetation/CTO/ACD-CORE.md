# Architecture Context Document: Rust Core

## Purpose

The Rust core is the platform layer for Batchworks data access and record
behavior. It carries forward the WinDev `WXManagerClass` / `*Rec` split and the
NodeJS table/record architecture, but expresses it in Rust terms.

Core owns infrastructure:

- PostgreSQL table access and transaction boundaries
- view-first reads
- table-only writes
- SQL builder guardrails
- record hydration and persistence contracts
- dirty tracking and persisted snapshots
- structured errors
- reusable foundational helpers

Core must not know business entities. It can know `ManagedRecord`, `FieldValue`,
`ManagedTable`, and `ManagedRecordState`; it must not know `InvoiceRec`,
`EntityRec`, or any domain-specific table.

## Doctrine

### Table And Record

Batchworks keeps two data object types:

- `*Table` or `*Manager`: collection/table behavior, fetches, saves, deletes,
  transactions, and view selection.
- `*Rec`: one row, its field values, validation, normalization, computed
  properties, display helpers, and row-owned behavior.

The original phrase still applies: records are living schemas with super powers.

### View-First Reads, Table Writes

Operational reads default to views. Writes always target base tables.

Every persisted table should have a normal operational view, even when the first
version is only:

```sql
SELECT ...
FROM schema.tbl_name
WHERE actv
  AND NOT dltd;
```

`ManagedTable` owns this split:

- `fetch_view`, `build_view`, `find_by_id_from_view`: read through views.
- `insert_rec`, `update_rec`, `save_rec`, `save_txn_rec`: write to tables.

### Lazy, Not Replayed

Lazy means using the truthful owner instead of repeating work.

If record normalization belongs to a `*Rec`, operational code calls the record.
If SQL construction belongs to `ManagedTable`, table structs do not duplicate it.
If a flow owner already creates a downstream outcome, callers do not replay that
pipeline around it.

Repeated formatting, parsing, query building, persistence, or mapping logic is an
ownership failure. Promote it into the right owner once.

### Rust-Native, Not A Blind Port

Rust has no WinDev-style dynamic class inheritance or `GET field` attributes.
The architecture is preserved through:

- traits for contracts
- structs for records and managed tables
- enum values for typed SQL parameters
- explicit `Result<T>` errors
- owned values and snapshots for dirty tracking
- modules for pure bricks

We keep the philosophy. We do not force Rust to pretend it is WLanguage.

## Core Types

### `FieldValue`

`FieldValue` is the shared typed value carrier between records and managed
tables. A record emits `FieldValue`s when it describes columns to persist. `ManagedTable`
binds those values as PostgreSQL parameters.

`FieldValue` exists to avoid string-built SQL values and to keep table writes
parameterized.

### `ManagedRecord`

Every persisted `*Rec` implements `ManagedRecord`.

Required responsibilities:

- hydrate from a PostgreSQL row
- expose its primary key
- list table columns
- list default view columns while ManagedTable still uses the transitional column-list path
- emit persistable columns
- emit modified columns
- scrub and validate itself
- mark itself clean after fetch/save

Records own one-row behavior. They do not save themselves and they do not fetch
other records.

### `ManagedRecordState`

`ManagedRecordState` is the persisted snapshot/dirty tracking helper embedded by
`*Rec` structs.

It supports the view-first architecture:

- a view can omit table control columns such as `actv` and `dltd`
- the record hydrates safe defaults for omitted fields
- `mark_clean()` stores the fetched state as the ghost snapshot
- later updates write only changed fields

This prevents accidental writes of defaulted fields that were not intentionally
changed by operational code.

### `ManagedTable<R>`

`ManagedTable<R>` is the generic table/view access owner for any `R:
ManagedRecord`.

It owns:

- configured table name
- default view name
- primary key field
- default alias
- allowed view names
- view alias mapping
- record collection memory
- read query construction
- insert/update/save/delete
- transaction save variants

Table structs should normally be constructor-only wrappers around
`ManagedTableConfig`. Extra table methods require a clear reason: ManagedTable
cannot express the operation cleanly, or the method is a deliberately authorized
special case.

The long-term source inventory owner is `setting.tlkp_data_source`. Hardcoded
view aliases in `ManagedTableConfig` are a bootstrap bridge until the runtime
warmup registry is wired into table construction.

## Layer Boundaries

### Core

Core contains reusable infrastructure only, split into physical tiers:

- `core/foundation`: diagnostics, errors, config, secrets, debug logging,
  field definitions, normalization, number/sort/time helpers.
- `core/db`: `ManagedTable`, `ManagedTableConfig`, `ManagedRecord`,
  `ManagedRecordState`, `FieldValue`, locking, and database error conversions.
- `core/warmup`: source registry contracts loaded after the PostgreSQL pool is
  available.

Core must not import from `data`.

### Data

Data contains project-owned record/table definitions and reusable data helpers:

- `data/records`: persisted `*Rec` and table wrapper types
- `data/fields`: field metadata and shared field constants
- `data/tables`: table manager wrappers
- `data/bricks`: pure helper functions
- `data/structures`: `clx*` allowed-shape contracts
- `data/virtual_records`: non-persisted record-like payloads and workflow data

The first foundational records are:

- `DataSourceRec` / `DataSourceTable`: bootstrap reads for
  `setting.tlkp_data_source` and `setting.vw_data_source`.
- `ErrorLookupRec` / `ErrorLookupTable`: warmup reads for reusable managed-error
  templates from `log.tlkp_error`.
- `ErrorRec` / `ErrorTable`: occurred error sink backed by `log.tbl_error`.
- `LogRec` / `LogTable`: operational warning/info/debug sink backed by
  `log.tbl_log`.

Both records can be built from a core `DiagnosticEnvelope`, but they read
different level families. `ErrorRec` uses `ErrorSeverity` for the `severity`
column; `LogRec` uses `LogLevel` for the partitioning `log_type` column.

`DataSourceRec` and `ErrorLookupRec` are warmup-owned at runtime, but their
initial fetch still uses normal `ManagedTable` access so bootstrap follows the
same table/record discipline as the rest of the project.

### Operational Code

Operational code composes outcomes. It should receive shaped facts from records,
virtual records, structures, and table managers. It should not parse endpoint JSON
or normalize raw database values repeatedly.

## Contracts

### Record Contract

A persisted record should:

- map persisted table columns 1:1 to fields
- keep view-only values out of `persistable_columns`
- implement idempotent `scrub()`
- implement `is_valid()` for required invariants
- expose computed helpers as ordinary Rust methods
- use `ManagedRecordState` for dirty tracking
- call `mark_clean()` after hydration and successful persistence

Records may have public fields when final normalization happens in `scrub()`.
Use setter methods when assigning a field needs immediate normalization.

Getter/setter convention:

```rust
rec.set_order_id(value);
let order_id = rec.order_id();
let display = rec.amount_display();
```

### ManagedTable Contract

A managed table should:

- default reads to the configured view
- allow alternate views only through allowlisted names or aliases
- write only to the configured base table
- call `scrub()` and `is_valid()` before persistence
- update only modified columns
- never put record-specific formatting or one-row business behavior in the
  manager

### SQL Contract

`ManagedTable`-generated SQL must use parameter binding for values.

Trusted SQL fragments such as `join` and `where_clause` are allowed only through
manager builder arguments and must stay inside the guardrails. Arbitrary raw SQL
is an exception path, not a normal feature.

### Error Contract

Rust code should return `crate::core::error::Result<T>`.

Use `CoreError::validation_at`, `CoreError::invalid_id_at`, or a structured
`ManagedError` for business and validation failures. Avoid unstructured errors
where the caller needs code/source/status details.

## Naming

Use clear Rust module names, while preserving architectural intent.

Examples:

- `managed_table.rs`
- `managed_record.rs`
- `entity_rec.rs`
- `entity_table.rs`
- `vrt_order_payload.rs`
- `clx_api_body.rs`
- `display_amount.rs`

Rust files should use snake case. Types should use PascalCase.

## Forbidden Patterns

- records saving themselves
- routine table reads in operational code when a view exists
- writes through views
- arbitrary view names from callers
- string interpolation for SQL values
- record-specific display or formatting in managers
- database access inside bricks
- hidden lookup/cache/database reads inside setters
- repeated normalization in operational code
- inventing routing facts such as country, wallet, account, mapping, or company
  defaults outside the owning data object or explicit policy owner

## Related Docs

- `documentation/implementation/core/table-record-view-field-rules.md`
- `documentation/implementation/core/bricks.md`
- `documentation/implementation/core/virtual-records.md`
- `documentation/implementation/core/structures.md`
- `documentation/implementation/core/error-warmup-catalog.md`
