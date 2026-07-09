# Code Bricks

Code bricks are reusable data-layer functions for repeated field, getter, record, or transformation logic.

A brick only knows what is passed into it. If a value is not passed in, the brick does not know about it.

This is the hard boundary:

- bricks do not read from the database
- bricks do not write to the database
- bricks do not fetch lookup rows
- bricks do not read from warmup/global caches
- bricks do not instantiate or own table structs
- bricks do not instantiate or own Record structs
- bricks do not own ManagedTable or ManagedRecord behavior
- bricks do not secretly discover missing data

If a brick needs extra context, the caller must fetch or build that context first and pass it in explicitly.

## Purpose

Use a brick when the same task is repeated by multiple fields within one Record struct, multiple getters within one Record struct, or multiple Record structs.

Common brick jobs:

- display formatting
- normalisation
- deterministic calculations
- validation helpers
- JSON/message rendering from values already provided
- small domain transformations used by more than one Rec or data-layer caller

Examples:

- `display_amount(icon, amount)`
- `display_entity_name(first_name, last_name, business_name, entity_type)`
- `render_error_message(template, details)`
- `normalise_required_string(value, field_name, source)`

## Record Boundary

The Record struct decides which fields are passed to the brick.

For example, a currency display brick can format an icon and amount:

```rust
pub fn display_amount(icon: &str, amount: &Decimal) -> String {
    format!("{}{}", icon, amount.round_dp(2))
}
```

The Rec decides how that applies to its fields:

```rust
pub fn display_subtotal(&self) -> String {
    display_amount(&self.ccy_icon, &self.amount_sub)
}

pub fn display_total(&self) -> String {
    display_amount(&self.ccy_icon, &self.amount_total)
}
```

The brick does not know there is an invoice, a table, or a database. It only knows `icon` and `amount`.

This is what lets records be living schemas with super powers. A project may
have many decimal amount fields across many tables, but the formatting rule
still lives once:

```rust
pub fn display_amount(icon: &str, value: &Decimal) -> String {
    format!("{}{}", icon, value.round_dp(2))
}
```

Each `*Rec` then exposes the flat record methods that make sense for its own
fields:

```rust
pub fn display_total(&self) -> String {
    display_amount(&self.ccy_icon, &self.total)
}

pub fn display_subtotal(&self) -> String {
    display_amount(&self.ccy_icon, &self.subtotal)
}
```

Operational code uses `rec.display_total()`. It does not call the brick directly
unless it is deliberately doing a pure value transform outside a record.

## Wrong Shape

This is not a brick:

```rust
pub async fn display_entity_name(entity_id: i64) -> Result<String> {
    let entity = EntityTable::new().fetch_by_id(entity_id).await?;
    Ok(format_entity_name(entity))
}
```

That function performs database work. It belongs in a manager, service, operational flow, or explicit orchestration layer, not in `data/bricks`.

This is the brick shape:

```rust
pub fn display_entity_name(
    first_name: &str,
    last_name: &str,
    business_name: &str,
    entity_type: &str,
) -> String {
    if entity_type == "business" {
        business_name.trim().to_string()
    } else {
        format!("{} {}", first_name.trim(), last_name.trim()).trim().to_string()
    }
}
```

The caller is responsible for providing the names and type.

## Grouping

Related functions can live in one brick module when they share a clear domain purpose.

Good grouping:

- `entity_display`: display name, formal name, phone display, address-line display
- `amount_display`: currency display, percent display, signed amount display
- `error_message`: template rendering, placeholder extraction, detail flattening

Bad grouping:

- generic junk drawer helpers
- functions grouped only because they were written at the same time
- functions that require hidden database or warmup state

## Rust Rule

In Rust, bricks should usually be plain modules under `src/data/bricks`.

They should expose focused functions, not classes:

```text
src/data/bricks/display_amount.rs
src/data/bricks/entity_display.rs
src/data/bricks/render_error_message.rs
```

Tests should be easy because bricks are pure: pass values in, assert values out.
