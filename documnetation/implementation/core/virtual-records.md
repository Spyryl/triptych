# Virtual Records

Virtual records are RAM-side data owners with record-like behavior.

They are not fallback DTOs and they are not table records. They exist when data
needs the same disciplined access pattern as a `*Rec`, but the object is not
persisted directly through `ManagedTable`.

Think of `vrt*` as the non-table sibling of `*Rec`:

- it can own getters and setters
- it can normalize input
- it can expose flat domain names over nested or bucketed storage
- it can group header and child records into one operational object
- it can replace anonymous local structs used only because the language needed a
  temporary shape

Do not confuse virtual records with `clx*` structures. A `clx*` structure is an
allowed-shape contract / typed whitelist. A virtual record is a non-table object
with behavior. See `documentation/implementation/core/structures.md`.

Use virtual records for:

- operational payloads
- query/build criteria
- temporary IDs and workflow context
- calculated values
- flexible pass-through data such as `VrtPassData`
- header-plus-child record aggregates such as `VrtInvoiceRec`
- repeated RAM-side shapes that would otherwise become loose one-off structs

Do not use virtual records for direct table persistence. If a struct is fetched
or saved through `ManagedTable`, it is a persisted `*Rec`, not a virtual record.

## Getter / Setter Naming

Rust does not support WinDev-style `GET field` / `SET field` attributes that can
both be used as `rec.field`. The project convention is:

- getter: `field_name()`
- setter: `set_field_name(value)`
- derived getter: `field_name_upper()`, `field_name_display()`, or another clear suffix

The getter and setter must keep the same field name after `set_`.

Preferred:

```rust
vrt.set_order_id(order_id);
let order_id = vrt.order_id();

vrt.set_qty_banana(qty);
let qty = vrt.qty_banana();
```

Avoid name inversion:

```rust
// Avoid this pairing.
vrt.set_banana_qty(qty);
let qty = vrt.qty_banana();
```

The point of this rule is to keep operational code flat and obvious while
keeping normalization at the data-object boundary.

In WinDev, this shows up as:

```text
vrt.order_id = value
order_id = vrt.order_id
```

In Rust, the same doctrine is expressed with methods:

```rust
vrt.set_order_id(value);
let order_id = vrt.order_id();
```

Operational code should still feel flat: use the named accessor, not the
underlying bucket, map, child vector, or raw payload shape.

## Normalization Boundary

Operational code should not repeat formatting/normalization logic.

Avoid:

```rust
rec.x = raw_x.trim().to_string();
rec.id = raw_id.trim().parse::<i64>().unwrap_or_default();
```

Prefer:

```rust
rec.set_x(x);
rec.set_id(id);
```

The setter owns normalization:

```rust
pub fn set_order_id(&mut self, value: impl Into<String>) {
    let value = value.into();
    self.s_vlu.insert(
        "order_id".to_string(),
        normalise_string(&value),
    );
}
```

The getter owns defaulting and presentation:

```rust
pub fn order_id(&self) -> String {
    self.s_vlu.get("order_id").cloned().unwrap_or_default()
}

pub fn order_id_upper(&self) -> String {
    self.order_id().to_uppercase()
}
```

## Associative Storage

Rust associative-array equivalents are:

- `HashMap<K, V>` for fast average lookup
- `BTreeMap<K, V>` for stable key ordering

For virtual records and debug-friendly payloads, prefer `BTreeMap<String, T>`.

Example:

```rust
pub struct VrtPassData {
    id: BTreeMap<String, i64>,
    x_vlu: BTreeMap<String, Decimal>,
    n_vlu: BTreeMap<String, i64>,
    s_vlu: BTreeMap<String, String>,
    b_vlu: BTreeMap<String, bool>,
}
```

## `VrtPassData`

`VrtPassData` is the controlled flexible bucket.

Its job is to replace loose temporary shapes with typed buckets plus named
getters/setters. The buckets are implementation detail; operational code should
prefer the named accessors.

Canonical buckets:

- `id`: database identifiers and other id-like integers
- `x_vlu`: decimal values such as money, percentages, and decimal quantities
- `n_vlu`: whole-number values such as counts and integer quantities
- `s_vlu`: strings
- `b_vlu`: booleans

Example intent:

```rust
impl VrtPassData {
    pub fn order_id(&self) -> String {
        self.s_vlu.get("order_id").cloned().unwrap_or_default()
    }

    pub fn set_order_id(&mut self, value: impl Into<String>) {
        let value = value.into();
        self.s_vlu.insert("order_id".to_string(), normalise_string(&value));
    }

    pub fn sender_id(&self) -> i64 {
        self.id.get("sender").copied().unwrap_or_default()
    }

    pub fn set_sender_id(&mut self, value: i64) {
        self.id.insert("sender".to_string(), value.max(0));
    }
}
```

The point is not that every caller should manipulate `id["sender"]` directly.
The point is that a flexible storage owner can expose disciplined names such as
`sender_id()`, `receiver_id()`, `entity_id()`, `raised_txn_id()`, or
`amount_sub()`.

If operational code starts inventing many local one-off shapes with the same
fields, promote the shape into `VrtPassData` accessors or a more specific
`Vrt*Data` / `Vrt*Rec` owner.

## `Vrt*Payload`

`Vrt*Payload` owns endpoint or external-input ingress.

Its job is to gather and normalize request facts once:

- headers
- parameters
- body/raw data
- authenticated user/account facts
- source IP or caller context
- boundary errors or diagnostics

Endpoint handlers should not repeatedly parse the raw request shape. They should
construct a payload owner and then use named accessors:

```rust
let payload = VrtInboundPayload::from_request(request)?;
let user_id = payload.user_id();
let order_id = payload.order_id();
let auth_passed = payload.auth_passed();
```

This is the same doctrine as `VrtPassData`: the backing storage can be flexible,
but operational code uses the disciplined vocabulary exposed by the `vrt*`
owner.

Payload owners may produce `*Rec`, `VrtPassData`, `clx*`, or other `vrt*` values
for downstream work. They must not become persistence owners.

## Aggregate Virtual Records

A `Vrt*Rec` can wrap a header record and child record arrays so operational code
can manipulate one record-like object.

Example shape:

```rust
pub struct VrtInvoiceRec {
    pub rec: InvoiceRec,
    pub lines: Vec<TransactionLineRec>,
    pub header_tax_breakdown: Vec<TransactionTaxBreakdownRec>,
    pub line_tax_breakdown: Vec<TransactionTaxBreakdownRec>,
    pub addresses: Vec<TransactionAddressRec>,
}
```

The aggregate owns record-level convenience:

```rust
impl VrtInvoiceRec {
    pub fn id(&self) -> Result<i64> {
        self.rec.id_or_throw("VrtInvoiceRec.id")
    }

    pub fn country_code(&self) -> Result<String> {
        self.rec.country_code_or_throw("VrtInvoiceRec.country_code")
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn tax_breakdown(&self) -> Vec<&TransactionTaxBreakdownRec> {
        self.header_tax_breakdown
            .iter()
            .chain(self.line_tax_breakdown.iter())
            .collect()
    }
}
```

This keeps operational code at the record level:

```rust
let invoice_id = invoice.id()?;
let line_count = invoice.line_count();
let taxes = invoice.tax_breakdown();
```

instead of making every flow know how invoice headers, lines, addresses, links,
tax breakdowns, paylegs, notes, and other child records are physically grouped.

Aggregate virtual records may hold real `*Rec` values, but the aggregate itself
does not implement `ManagedRecord` and does not save itself. Persistence still
belongs to table managers or operational `persist*` owners.

This is the Rust expression of the old RAM-side child-management idea: build and
manipulate a header-plus-children graph in memory, then let an explicit
`persist*` owner flush the graph through the proper table managers.

## Header Record Bridging

An aggregate virtual record may also attach child arrays back onto its header
record when the header `*Rec` provides record-level child helpers.

That pattern is allowed when it preserves the flat record interface:

```text
VrtInvoiceRec owns the invoice aggregate
InvoiceRec owns invoice-local accessors over its child values
operational code asks the aggregate/header for named facts
```

The danger to avoid is leaking physical child storage everywhere. If every flow
has to remember which vector contains which child data, the `vrt*` owner is not
doing enough work.

See `record-doctrine.md` for the matching `*Rec` side of this rule.

## Persisted Records

The `set_` convention can also be used on persisted records when field
normalization is needed, but persisted fields may remain public when the record's
`scrub()` method performs final normalization before save.

For persisted records, audit-safe update behavior is still owned by
`ManagedRecordState` plus `ManagedRecord::modified_columns()`, not by setters.
