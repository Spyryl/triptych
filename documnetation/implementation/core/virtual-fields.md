# Virtual Fields

Virtual fields are values carried by a record-like object that are not persisted
to the base table.

Mandate has two related concepts:

- view fields on table-backed `*Rec` structs
- runtime fields on records or virtual records

Both must stay out of table writes.

## View Fields

A view field comes from a database view, commonly through a join or calculated
read surface. It belongs on the same `*Rec` as the base table concept, but it is
not included in `persistable_columns()`.

Example shape:

```rust
pub struct EntityRec {
    pub id: i64,
    pub country_code: String,

    // View-only fields.
    pub country_name: Option<String>,
    pub ccy: Option<String>,
}
```

The record can hydrate view fields in `from_row()`, but `current_columns()` and
`persistable_columns()` must only include persisted table columns.

## Runtime Fields

Runtime fields exist only in memory. Examples:

- dirty tracking state
- UI or CLI selection state
- temporary workflow markers
- cached derived values that are not database truth

Runtime fields must not be emitted by `persistable_columns()`.

For persisted records, audit-safe update behavior is owned by
`ManagedRecordState` and `modified_columns()`. Do not invent parallel dirty
tracking unless the record has a clear runtime-only need.

## Computed Getters

Computed getters are derived from other fields. They are not persisted unless
the computed value is deliberately copied into a persisted column by the owning
record or flow.

Rust convention:

```rust
impl InvoiceRec {
    pub fn amount_total(&self) -> Decimal {
        self.amount_base + self.amount_tax
    }

    pub fn amount_total_display(&self) -> String {
        display_amount(&self.ccy_icon, &self.amount_total())
    }
}
```

The record owns the computed behavior. Operational code should not repeat the
same calculation or formatting.

## Hydration Rule

Views may omit persisted fields or expose additional view fields.

Safe hydration is:

```text
Rec::default()
apply every returned column with try_get
leave omitted columns at defaults
scrub
mark_clean
```

This allows normal views to omit fields like `actv` and `dltd` when the view
predicate already defines their values.

## Persistence Rule

`persistable_columns()` returns only real table columns.

Never persist:

- view-only fields
- runtime-only fields
- computed getters
- diagnostic or selection state
- child vectors or RAM-side workflow collections

If a value must be written, it should be an explicit persisted field with a field
constant and record-owned normalization.

## Virtual Data Owners

Use `vrt*` types when the object is not table-backed but does own behavior.

Common forms:

- `Vrt*Payload`: endpoint/API ingress owner that normalizes raw input.
- `Vrt*Data`: internal runtime data owner for repeated RAM-side data.
- `Vrt*Rec` / `Vrt*Record`: record-like object with no direct table manager.
- `VrtPassData`: controlled typed bucket for flexible scalar pass-through with
  named getters/setters.

Virtual records must not implement `ManagedRecord`, own a `ManagedTable`, or save
themselves.

## Summary

| Field Type | Source | Persisted |
| --- | --- | --- |
| Table field | Base table | Yes |
| View field | Database view | No |
| Runtime field | Code only | No |
| Computed getter | Derived | No |

The owning record or virtual record decides how virtual values are normalized,
defaulted, displayed, and exposed.
