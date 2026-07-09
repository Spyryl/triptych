# Structures (`clx*`)

## Purpose

A `clx*` structure is an allowed-shape contract.

It is the list of fields that are permitted for a small data shape. It exists to
enforce discipline at boundaries where the code would otherwise become loose,
especially around JSONB, request payloads, response payloads, workflow data, and
small repeated option/result rows.

The simplest rule:

```text
If a field is not in the clx* structure, it is not allowed.
```

## Kitchen Drawer Analogy

JSONB without a structure is the kitchen drawer where everything gets thrown:

```text
menus
appliance instructions
loose screws
old batteries
random mini cook-books
something that might be useful someday
```

You can put anything in it, but finding the correct thing later is painful.

A `clx*` structure is the molded cutlery tray:

```text
knives go here
forks go here
spoons go here
```

The point is not that the tray stores more things. The point is that it prevents
the drawer from becoming random.

For JSONB, the `clx*` says:

```text
metadata may contain size, shape, and colour.
metadata may not contain window.height just because someone felt like adding it.
```

## What A Structure Is

A structure is a typed whitelist.

Use structures for:

- JSONB shape contracts
- API request bodies
- API response bodies
- small option rows such as key/value pairs
- workflow payloads
- calculated result shapes
- nested payload members
- UI/display data shapes
- criteria/result objects that are not table-backed records

Examples:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClxDictionary {
    pub key: String,
    pub vlu: String,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClxErrorLookupMetadata {
    pub placeholders: Vec<String>,
}
```

The `deny_unknown_fields` attribute is important. It is the Rust/Serde version
of rejecting random fields that are not part of the contract.

## What A Structure Is Not

A `clx*` structure is not a table record.

It must not:

- implement `ManagedRecord`
- own `ManagedTable`
- import `tokio_postgres::Row`
- perform table fetches or saves
- own dirty tracking / ghost snapshots

A `clx*` structure is also not where heavy formatting and persistence behavior
belongs.

## Relationship To `*Rec`

The `*Rec` owns the database row and persistence boundary.

The `clx*` owns the allowed shape for one structured value used by that record.

Example:

```rust
pub struct ErrorLookupRec {
    pub metadata: Value,
}
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClxErrorLookupMetadata {
    pub placeholders: Vec<String>,
}
```

The record bridges between raw JSON storage and the structure:

```rust
pub fn metadata_clx(&self) -> Result<ClxErrorLookupMetadata> {
    Ok(serde_json::from_value(self.metadata.clone())?)
}

pub fn set_metadata_clx(&mut self, value: ClxErrorLookupMetadata) -> Result<()> {
    self.metadata = serde_json::to_value(value)?;
    Ok(())
}
```

If the metadata field changes, the `*Rec` dirty tracking sees the JSONB field
changed and `ManagedTable` can persist it.

## Normalization Boundary

Structures define what is allowed. They should not become the place where record
business behavior gets hidden.

The `*Rec` or virtual record that uses the structure owns:

- defaults used by operational code
- formatting
- trimming
- uppercase/lowercase rules
- validation messages
- managed errors
- GET/SET-style convenience methods

Example:

```rust
pub fn placeholder_count(&self) -> Result<usize> {
    let clx = self.metadata_clx()?;
    Ok(clx.placeholders.len())
}
```

or:

```rust
pub fn set_placeholders(&mut self, value: Vec<String>) -> Result<()> {
    let clx = ClxErrorLookupMetadata { placeholders: value };
    self.set_metadata_clx(clx)
}
```

The structure says `placeholders` is allowed. The record decides how that value
is read, defaulted, normalized, validated, or persisted.

## Relationship To Virtual Records

A virtual record is a behavior-bearing non-table object. It can have getters,
setters, calculated values, internal maps, and workflow-oriented behavior.

A structure is usually simpler: it is the allowed field shape.

Use `clx*` when the main question is:

```text
What fields are allowed here?
```

Use `vrt*` when the main question is:

```text
What behavior does this non-table object provide?
```

## API Contracts

If a caller asks what fields to send to a POST endpoint, the structure should be
able to answer that.

Example:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClxCreateProjectRequest {
    pub code: String,
    pub name: String,
    pub repo_path: String,
}
```

The endpoint or record using that structure can normalize/validate the values,
but the structure lists the permitted fields.

Unknown fields should be rejected. Silently accepting random payload fields
undermines the contract.

## Structured Field Assignment

Operational code must not assign raw object literals, raw payload fragments, or
generic maps into persisted structured fields.

Avoid:

```rust
rec.metadata = payload.metadata.clone().unwrap_or_default();
rec.metadata = json!({ "reason": reason, "source": source });
```

Prefer record-owned methods that map only approved fields:

```rust
rec.apply_approval_metadata(input.metadata)?;
rec.set_approval_reason_code(input.reason_code);
```

The named method may accept a loose input shape when that is useful at a
boundary, but it must explicitly map only fields approved by the relevant
`clx*` structure. Unknown fields disappear because no approved mapping reads
them.

The owning `*Rec` and `clx*` then own:

- normalization
- whitelisting
- required-field validation
- dirty tracking through the persisted JSON/JSONB field
- final row-safe output

This rule applies to any persisted structured field, not only a column literally
named `metadata`.

## API Response Views

API or CLI response structures may use `clx*` when the boundary needs a stable,
explicit output shape.

That keeps handlers from returning raw records and leaking internal fields.

Response `clx*` structures still follow the same rules:

- explicit fields
- safe defaults
- unknown fields rejected on input where deserialization is used
- no workflow logic
- no database access

## JSONB Rule

Every JSONB field that has meaningful application data should have a `clx*`
structure for each supported shape.

JSONB is storage. It is not permission to avoid schema discipline.

Allowed:

```text
metadata -> ClxErrorLookupMetadata
```

Not allowed:

```text
metadata.window.height added because someone needed somewhere to put it
```

If the application needs `window.height`, create or update the relevant
structure deliberately and review whether that data belongs in this JSONB field
at all.

## Summary

- `clx*` means structure.
- A structure is a typed whitelist / allowed-shape contract.
- Unknown fields should be rejected.
- Structures are not table records and are not managers.
- Structures can be used for JSONB, API payloads, workflow payloads, option
  rows, and calculated result shapes.
- `*Rec` / `vrt*` owns formatting, cleaning, validation, and convenience getters
  or setters.
- JSONB fields should use structures to avoid becoming junk drawers.
