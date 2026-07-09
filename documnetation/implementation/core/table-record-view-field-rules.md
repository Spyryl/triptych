# Table, Record, And View Field Rules

## Core Rule

A table-backed `*Rec` represents the persisted table record, not only one view.

The fields folder helps keep the distinction clear:

- persisted fields describe the table write surface
- view fields describe read-only facts that may arrive from views
- virtual/runtime fields describe record-owned facts that must not be written
  directly to the base table

`setting.tlkp_data_source` handles the other half of the problem: the database
source inventory. Rust fields say what the record knows how to carry; the source
registry says which table/view objects can hydrate or persist that record
concept.

That means persisted table fields such as `actv` and `dltd` belong in the `*Rec`
even when normal operational views do not expose them.

The `*Rec` must be able to insert and update those fields through the table:

```rust
pub struct ErrorLookupRec {
    pub actv: bool,
    pub dltd: bool,
    // ...
}
```

and:

```rust
fn persistable_columns(&self) -> Vec<(&'static str, FieldValue)> {
    vec![
        ("actv", FieldValue::Bool(self.actv)),
        ("dltd", FieldValue::Bool(self.dltd)),
        // ...
    ]
}
```

Views are read surfaces. Tables are write surfaces.

The `*Rec` is allowed to be richer than either surface. It can expose getters,
setters, and computed helpers that operational code treats as ordinary record
facts, while `persistable_columns()` still writes only real table columns.

## Default View Rule

Default operational views should return valid records, not necessarily every
persisted control field.

For active/non-deleted records, this is valid:

```sql
CREATE OR REPLACE VIEW log.vw_error_lookup AS
SELECT
    id,
    code,
    description,
    message_template,
    severity,
    status_code,
    domain,
    metadata,
    created_at,
    updated_at
FROM log.tlkp_error
WHERE actv
  AND NOT dltd;
```

The view can omit `actv` and `dltd` because the view predicate already defines
that returned rows are active and not deleted.

## Why Saves Stay Safe

Saving does not write every field. `ManagedTable` updates only fields that changed
against the record's ghost snapshot.

If a view omits `actv`, the record hydrates `actv` to its default:

```text
actv = true
```

Then `mark_clean()` stores that value in the record's ghost snapshot:

```text
original.actv = true
```

If operational code changes some other field and saves:

```text
actv: true -> true
description: old -> new
```

only `description` is written to the `UPDATE`.

So omitting `actv` / `dltd` from a view does not accidentally write those default
values back to the table, as long as dirty tracking compares against the ghost
snapshot.

## Insert Rule

New records still need persisted control fields in the `*Rec`.

For new records:

```rust
impl Default for ErrorLookupRec {
    fn default() -> Self {
        Self {
            actv: true,
            dltd: false,
            // ...
        }
    }
}
```

Then:

```rust
records.save_rec(rec)
```

inserts the intended default values, or Postgres applies equivalent table
defaults when the field is omitted from the insert.

## Intentional State Changes

If operational code intentionally changes a control field:

```rust
rec.actv = false;
records.save_rec(rec).await?;
```

dirty tracking sees:

```text
actv: true -> false
```

and `ManagedTable` includes only `actv` in the `UPDATE` when that is the only
changed field.

## View Shape Rule

When a view omits persisted fields, the `*Rec` must tolerate that shape without
requiring a Rust definition for every SQL view.

There are two safe options for the SQL itself.

Option 1: include the omitted fields in the view.

Use this when the consumer needs to inspect or compare those field values.

Option 2: omit the fields from the view and hydrate defaults in `from_row()`.

Use this when the view predicate already defines the field values.

The target Rust behavior is:

```text
Rec::default()
apply every returned column
leave omitted columns at their defaults
scrub
mark clean
```

That means `from_row()` must default fields that a view may not return:

```rust
actv: row.try_get("actv").unwrap_or(true),
dltd: row.try_get("dltd").unwrap_or(false),
```

Do not call `row.get("actv")` when the view does not return `actv`; that will
fail at runtime.

`view_columns()` is transitional. It exists because the current ManagedTable still
asks a `ManagedRecord` for a default view field list. It is not the long-term
answer for a multi-view project. Do not create a Rust column list for every SQL
view.

The long-term manager direction is to select the resolved source with:

```sql
SELECT me.*
FROM resolved_schema.resolved_view AS me
```

and let default-first `Rec` hydration handle the returned shape.

## Source Inventory Rule

View names are database object inventory. They belong in
`setting.tlkp_data_source`, not in each Rust table file.

For a multi-view table such as `log`, the old pattern is:

```rust
ManagedTableConfig::new("log.tbl_log", "log.vw_log")
    .with_view_alias("warn", "log.vw_log_warn")
    .with_view_alias("info", "log.vw_log_info")
    .with_view_alias("debug", "log.vw_log_debug")
```

That is bootstrap-only.

The target pattern is:

```rust
let sources = registry.record("log")?;
let config = ManagedTableConfig::from_record_sources(sources)?;
```

The database rows decide the table, default view, and alternate view aliases.
The `LogRec` still owns all known fields and defaults.

## When To Expose `actv`

Expose `actv` in a view only when the consumer must inspect or manage active
state.

Examples:

```text
admin screens
maintenance tools
mixed active/inactive reports
```

An inactive-specific view may still omit `actv` if the view name and predicate
make the state explicit:

```sql
WHERE NOT actv
  AND NOT dltd
```

## When To Expose `dltd`

Normal views should not expose deleted records.

Deleted records should require an explicit recovery/audit view. This avoids
ordinary operational code accidentally treating deleted records as valid data.

Examples:

```text
log.vw_error_lookup_deleted
control.vw_project_deleted
```

Those views should only be allowlisted on table structs that genuinely need
recovery/audit access.

## Rust Naming Rule

Use Rust-native names for table managers instead of carrying the WinDev `*Class`
suffix forward.

The file and type pattern is:

```text
src/data/tables/error.rs        -> ErrorTable
src/data/records/error_rec.rs   -> ErrorRec
src/data/fields/flds_error.rs   -> ErrorFields

src/data/tables/analysis_run.rs       -> AnalysisRunTable
src/data/records/analysis_run_rec.rs  -> AnalysisRunRec
src/data/fields/flds_analysis_run.rs  -> AnalysisRunFields
```

Rules:

- table files use the plain snake_case table concept: `project.rs`,
  `analysis_run.rs`
- table types use `PascalCaseTable`: `ProjectTable`, `AnalysisRunTable`
- record files and types keep the explicit `Rec` suffix
- field files keep the `flds_` prefix, but field types use `*Fields`
- avoid naming a table type only `Error`, because Rust already has `Error`
  traits/types and the intent becomes muddy

## Summary

- `actv` and `dltd` are persisted table fields.
- Persisted fields belong in the `*Rec`.
- Default views may filter by `actv` / `dltd` without returning those fields.
- Saves remain safe because updates use ghost dirty tracking.
- If a view omits fields, `from_row()` must default them safely.
- `view_columns()` is transitional and must not become one list per SQL view.
- View/source inventory belongs in `setting.tlkp_data_source`.
- Include `actv` / `dltd` in a view only when the consumer must inspect or
  manage those flags.
- Rust table manager types use `*Table`, not `*Class`.
