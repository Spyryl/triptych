# WinDev To Rust Core Map

## Why This Exists

The Rust implementation is a translation of the architecture, not a line-by-line
port of WinDev or NodeJS.

The goal is to preserve:

- table/record separation
- living record schemas
- view-first reads
- table writes
- lazy ownership
- reusable core infrastructure

while using Rust's strengths:

- explicit types
- explicit errors
- ownership and borrowing
- trait-based contracts
- parameterized database access

## Concept Map

| WinDev / WLanguage | NodeJS | Rust | Meaning |
| --- | --- | --- | --- |
| `WXManagerClass` | `ManagedTable<T>` | `ManagedTable<R>` | Shared table/view database owner |
| `InvoiceClass` | `InvoiceClass` | `InvoiceTable` / `InvoiceManager` | Thin table-specific manager wrapper |
| `InvoiceRec` | `InvoiceRec` | `InvoiceRec` | One row with field behavior |
| `arrRec` | manager record array | `Vec<R>` inside manager | Last fetched collection |
| `pkFieldName` | `pkField` | `pk_field_name` | Primary key column |
| `DBFileName` | `tableName` | `table_name` | Base table write target |
| `DBViewName` | `viewName` | `view_name` | Default read view |
| `aliasFileName` | `defaultAliasName` | `alias_file_name` | Query alias |
| `SaveRecord()` | `saveRec()` | `save_rec()` | Insert/update decision owner |
| `SaveTxnRecord()` | `saveTxnRec()` | `save_txn_rec()` | Transaction-specific save |
| `PROCEDURE GET x()` | `get x()` | `x()` / `x_display()` | Computed getter |
| `PROCEDURE SET x()` | `set x()` | `set_x(value)` | Normalizing setter |
| child memory mode | staged write graph | explicit vectors/owners | Build in RAM, persist through owner |

## The Big Mental Shift

WinDev lets dynamic objects carry a lot of behavior implicitly. Rust makes the
contracts explicit.

That means:

- a record implements `ManagedRecord`
- a manager is generic over `R: ManagedRecord`
- persistence values are emitted as `FieldValue`
- failures return `Result<T>`
- dirty tracking is stored in `ManagedRecordState`
- table/view names are validated before SQL is generated

The result is more upfront ceremony, but fewer hidden runtime surprises.

## Records In Rust

A Rust `*Rec` should be a normal struct:

```rust
pub struct InvoiceRec {
    state: ManagedRecordState,
    pub id: Option<i64>,
    pub amount: Decimal,
    pub tax: Decimal,
    pub currency_icon: String,
}
```

Record-owned helpers become methods:

```rust
impl InvoiceRec {
    pub fn amount_display(&self) -> String {
        display_amount(&self.currency_icon, &self.amount)
    }

    pub fn tax_display(&self) -> String {
        display_amount(&self.currency_icon, &self.tax)
    }
}
```

The record still owns formatting. Operational code should not rebuild these
strings each time it needs them.

## Managers In Rust

A table-specific manager should usually be thin:

```rust
pub struct InvoiceTable {
    manager: ManagedTable<InvoiceRec>,
}
```

Its constructor configures:

- write table
- default view
- primary key
- aliases
- allowed alternate views

If `ManagedTable` can perform the fetch or save, the table-specific manager should
not duplicate that logic.

## View-First In Rust

The default read path is:

```rust
invoice_table.fetch_view(args).await?;
```

The write path is:

```rust
invoice_table.save_rec(rec).await?;
```

This is the same architectural rule as the WinDev/NodeJS systems:

- read from view
- write to table

## Dirty Tracking

Rust records need an explicit snapshot helper because updates should not write
every field just because the struct has every field.

The expected pattern:

1. `from_row()` hydrates the record.
2. `scrub()` normalizes it.
3. `mark_clean()` stores the persisted snapshot.
4. operational code changes fields.
5. `modified_columns()` compares current values to the snapshot.
6. `ManagedTable` writes only the changed columns.

This is what keeps omitted view columns safe.

## Bricks

WinDev private functions and repeated field helpers usually become Rust bricks
when they are pure value transformations.

Good brick:

```rust
pub fn display_amount(icon: &str, amount: &Decimal) -> String
```

Bad brick:

```rust
pub async fn display_amount_for_invoice(invoice_id: i64) -> Result<String>
```

The second example fetches data. It belongs in a manager, service, or flow owner,
not in `data/bricks`.

## Practical Rules For Learning Rust Through Core

- Start with structs and methods before traits.
- Add traits when multiple records need the same manager contract.
- Prefer `Result<T>` over panics.
- Prefer `Option<T>` for nullable values.
- Prefer `Decimal` for money.
- Prefer `i64` for database integer IDs unless the schema requires otherwise.
- Let records normalize values; do not scatter normalization through callers.
- Let managers persist values; do not add `rec.save()`.

## First Build Order

1. Define a small real `*Rec`.
2. Implement `ManagedRecord` for it.
3. Embed `ManagedRecordState`.
4. Define its table columns and view columns.
5. Add record-owned getters/setters.
6. Create a thin table manager wrapper.
7. Fetch through the view.
8. Save through the table.
9. Add validation.
10. Only then add broader abstractions.
