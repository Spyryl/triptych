# Record Doctrine

`*Rec` is the main living schema owner for table-backed data.

It is not only a row-shaped struct. It is the place where a persisted row becomes
safe and useful to operational code.

The old phrase is still the best one:

```text
records are living schemas with super powers
```

The "schema" part is the table/view field contract. The "super powers" are
virtual fields, getters/setters, metadata-backed attributes, computed behavior,
and brick-backed helpers that let operational code stay flat.

## Core Shape

A `*Rec` owns:

- persisted table fields
- view-only fields
- runtime-only fields that belong to that record instance
- virtual fields exposed through getters/setters
- metadata/JSONB-backed accessors
- normalization and scrub behavior
- validation
- computed getters
- display helpers
- row-local collection helpers
- dirty tracking through `ManagedRecordState`

The table manager owns persistence. The record owns one-row behavior.

## Fields Folder And Field Reality

`data/fields` separates field reality from record behavior.

Use field modules to identify:

- persisted table columns
- default view columns while the transitional `view_columns()` path exists
- view-only fields
- runtime/virtual field names where shared metadata is useful

That split keeps the table write contract honest while allowing the `*Rec` to
present a richer interface than the base table.

The record can be:

- the record of a base table
- the record hydrated from a default view of that table
- the record hydrated from an alternate view of that table

Missing fields are acceptable when defaults and getters/setters can produce a
valid record surface.

## Flat Operational Access

Operational code should be able to use a record through named facts:

```rust
let full_name = rec.full_name();
let email = rec.email();
let customer_category = rec.customer_category();
rec.set_staff_rank(rank);
rec.add_allergen(allergen);
```

Operational code should not need to know that those values came from:

- a base table column
- a view join
- a JSONB metadata field
- a nested metadata array
- a calculated value
- a runtime child collection

That storage detail belongs inside the `*Rec`.

## Virtual Setters

A setter does not have to map one input to one physical field.

It may set one or more backing fields when that named record fact carries more
meaning than the physical storage.

Example intent:

```rust
impl CustomerRec {
    pub fn set_is_customer(&mut self, value: bool) {
        self.tp = if value { "customer" } else { "not_customer" }.to_string();
    }

    pub fn set_important_customer(&mut self, value: bool) {
        self.tp = if value { "customer" } else { "not_customer" }.to_string();
        self.state = if value { "important" } else { "not_important" }.to_string();
    }
}
```

Operational code still gets the flat interface:

```rust
rec.set_important_customer(true);
let is_customer = rec.is_customer();
```

The caller does not need to know that the value is backed by `tp`, `state`, or
another storage decision.

The discipline is that the setter must remain record-local. It may normalize and
update fields on the record; it must not fetch hidden context or persist the
record.

## Metadata-Backed Attributes

If operational code needs a meaningful value stored inside JSONB/metadata, the
record should expose a named getter/setter or named method.

Avoid:

```rust
rec.metadata["staff"]["rank"] = json!(rank);
let value = rec.metadata["delivery-times"].clone();
```

Prefer:

```rust
rec.set_staff_rank(rank);
let delivery_times = rec.delivery_times()?;
```

The record and its `clx*` metadata contract own the path, defaults,
normalization, and validation.

## View Fields

View fields belong on the same `*Rec` as the base table concept when they are
read facts about that record.

Examples:

```rust
pub struct CustomerRec {
    pub id: i64,
    pub company_id: i64,

    // View-only.
    pub primary_contact: Option<String>,
    pub current_balance: Decimal,
}
```

These fields can be hydrated from views, but they must not be returned by
`persistable_columns()`.

## Computed And Display Getters

Computed and display values should be methods on the owning record.

Examples:

```rust
impl CustomerRec {
    pub fn customer_type(&self) -> String {
        title_case_code(&self.tp)
    }

    pub fn terms_display(&self) -> String {
        render_payment_terms(&self.terms)
    }
}
```

Repeated display or formatting logic can delegate to a pure brick, but the
record decides which fields feed the brick.

Example:

```rust
pub fn display_total(&self) -> String {
    display_amount(&self.ccy_icon, &self.total)
}

pub fn display_subtotal(&self) -> String {
    display_amount(&self.ccy_icon, &self.subtotal)
}
```

The brick owns the repeated formatting rule. The record owns which amount fields
use it and how operational code asks for the display value.

## Collection Helpers

A record may expose helpers over record-local collections when those collections
are part of the record's behavior surface.

Examples:

```rust
rec.add_allergen(value);
rec.remove_allergen(value);
rec.add_delivery_range(day, start, finish);
rec.remove_delivery_range(day);
```

The point is the same as scalar getters/setters: callers should use named
record behavior instead of manipulating raw arrays inside metadata.

## Header And Child Data

A persisted `*Rec` still represents one table row. It does not save child rows
itself.

However, it may expose record-local helpers over child values when a
`Vrt*Rec` aggregate or operational fetch owner has attached those child values
for RAM-side use.

The ownership split is:

```text
Vrt*Rec aggregate -> groups header record and child records
header *Rec       -> exposes header-local and child-derived convenience methods
persist* owner    -> saves the prepared header/child graph
ManagedTable      -> writes individual table-backed records
```

This preserves the flat record interface without making `rec.save()` or hidden
child persistence legal.

## Scrub And Validation

Use `scrub()` for idempotent normalization before persistence.

Use `is_valid()` for required invariants.

Setters may normalize immediately when a value must be safe as soon as it enters
the record. Public fields are acceptable when `scrub()` performs final
normalization before save.

## No Hidden Fetches

Record methods must not secretly fetch from the database, warmup caches, or
external services.

Allowed:

- use fields already on the record
- use child values deliberately attached to the record
- use pure bricks
- use core normalization/time/number helpers
- use `clx*` structures for metadata shape

Not allowed:

- table manager construction inside a getter
- database queries inside a setter
- hidden lookup fetches to make a display value work
- saving the record from inside the record

If extra context is needed, a `fetch*`, `resolve*`, `prepare*`, or `Vrt*Rec`
owner should supply it explicitly.

## Table Manager Lineage

The old WinDev pattern was:

```text
WXManagerClass -> configured table class -> arrRec
```

The Rust pattern is:

```text
ManagedTable<R> -> thin *Table wrapper -> Vec<R>
```

The table wrapper configures the source and delegates generic behavior. It
should not duplicate record-local behavior or SQL mechanics already owned by
`ManagedTable`.

## Summary

- `*Rec` is table-backed living schema.
- `*Rec` is a living schema with super powers.
- `*Rec` owns row-local behavior and flat named accessors.
- `*Rec` hides metadata/view/computed storage details from operational code.
- `*Rec` may expose virtual setters that update one or more backing fields.
- `*Rec` may use bricks for repeated display, formatting, and deterministic
  transforms.
- `*Rec` may expose RAM-side child-derived helpers when supplied by an aggregate
  owner.
- `*Rec` does not save itself or fetch hidden context.
- `ManagedTable` and `*Table` own persistence mechanics.
