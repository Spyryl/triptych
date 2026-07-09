# Field And Source Doctrine

Rust does not give this project WinDev-style record fields, field attributes, or
GET/SET properties.

So Mandate uses two explicit layers:

- `data/fields`: Rust-owned field definitions for a record concept.
- `setting.tlkp_data_source`: database-owned source inventory for tables and
  views.

Together they preserve the lazy core pattern without making Rust carry every
database object and field shape in ad hoc code.

## Field Definitions

`data/fields/<record>.rs` owns the field vocabulary for a `*Rec`.

Field definitions identify:

- persisted table columns
- virtual/view fields where the record needs a named field contract
- field type
- nullable/required behavior
- length limits
- defaults
- primary-key marker

Example shape:

```rust
pub const ID: &str = "id";
pub const RECORD_KEY: &str = "record_key";
pub const ACTV: &str = "actv";

pub const ID_DEF: FieldDef = FieldDef::bigint_pk(ID);
pub const RECORD_KEY_DEF: FieldDef = FieldDef::required_text(RECORD_KEY, 100);
pub const ACTV_DEF: FieldDef = FieldDef::persisted(
    ACTV,
    FieldType::Bool,
    false,
    true,
    None,
    FieldDefault::Bool(true),
    false,
);
```

The goal is not to create paperwork. The goal is to put field truth in one
place so `*Rec` can be a living schema with super powers instead of a loose
struct full of magic strings.

## Persisted Vs Virtual

`FieldSource::Persisted` means the field is part of the table write contract.

`FieldSource::Virtual` means the field can be carried or exposed by the record,
but must not be written directly to the base table.

Virtual fields may come from:

- a view join
- a calculated read surface
- runtime state
- metadata-backed getters/setters
- child data attached by a `Vrt*Rec` aggregate

Only persisted fields belong in `persistable_columns()`.

## Source Inventory

`setting.tlkp_data_source` owns database object inventory.

It answers:

- What table does this `record_key` write to?
- What view does it read by default?
- Which alternate views are allowed?
- Which caller-facing alias resolves to each alternate view?

It stores one row per table or view source:

```text
record_key | schema_name | object_name | object_type | source_role  | alias_key
log        | log         | tbl_log     | table       | table        |
log        | log         | vw_log      | view        | default_view |
log        | log         | vw_log_warn | view        | view         | warn
```

This keeps view inventory in the database, where database object inventory
belongs.

## Seed Pattern

Source seeds are idempotent. They insert missing rows only, so maintenance/admin
changes are not clobbered by re-running deployment scripts.

The seed shape is:

```sql
WITH seed (
    record_key,
    schema_name,
    object_name,
    object_type,
    source_role,
    alias_key,
    metadata
) AS (
    VALUES
        ('log', 'log', 'tbl_log',      'table', 'table',        NULL,    '{"tier":"diagnostic","purpose":"operational log sink"}'::jsonb),
        ('log', 'log', 'vw_log',       'view',  'default_view', NULL,    '{"tier":"diagnostic","purpose":"all operational logs"}'::jsonb),
        ('log', 'log', 'vw_log_warn',  'view',  'view',         'warn',  '{"tier":"diagnostic","purpose":"warning logs"}'::jsonb)
)
INSERT INTO setting.tlkp_data_source (...)
SELECT ...
WHERE NOT EXISTS (...);
```

Metadata should stay bounded and descriptive: tier, purpose, compatibility
notes, deployment hints. It is not a junk drawer for business state.

## Record Key Rule

`record_key` is the stable Rust/core concept key.

It is not necessarily the SQL table name. It names the record concept:

```text
error_lookup
log
quote_line
product_component
delivery_stop
```

The `*Rec` owns fields and behavior for that record concept. The source registry
owns the SQL objects that can hydrate or persist it.

## Adding A View

Adding an alternate view normally requires:

1. SQL view definition.
2. `setting.tlkp_data_source` seed/admin row with `source_role = 'view'`.
3. A caller-facing `alias_key`.

Rust changes are needed only when the view exposes fields the `*Rec` does not
yet know how to hydrate or use.

Do not create a new Rust record type for every SQL view. A view is a read
surface for the same table-backed record concept.

## Adding A Field

Adding a field normally requires:

1. Database migration when the field is persisted.
2. A field constant and `FieldDef` in `data/fields`.
3. `*Rec` storage and default/hydration support.
4. `persistable_columns()` support only when persisted.
5. Getter/setter/computed behavior when operational code needs a named fact.

View-only fields skip table persistence but still need record support when
operational code uses them.

## What Belongs Where

`data/fields` owns:

- field names
- field types
- defaults
- persisted/virtual distinction
- field-level constraints needed by records

`setting.tlkp_data_source` owns:

- schema name
- table/view object name
- source role
- view aliases
- active/deleted source availability
- source-level metadata

`*Rec` owns:

- hydration
- scrub/validation
- getters/setters
- computed/display behavior
- persistence shape

`ManagedTable` owns:

- source resolution
- SQL generation
- parameter binding
- insert/update/fetch mechanics

## Summary

- Rust field metadata lives in `data/fields`.
- Database source inventory lives in `setting.tlkp_data_source`.
- A `*Rec` can be hydrated from many views of the same table concept.
- Missing view fields are handled by default-first hydration and record
  getters/setters.
- `persistable_columns()` writes only persisted fields.
- Adding a view is usually SQL + source metadata, not a new Rust record.
