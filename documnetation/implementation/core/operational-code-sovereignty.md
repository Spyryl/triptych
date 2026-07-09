# Operational Code Sovereignty

Operational code is the applied-behavior layer above `core` and `data`.

It must reuse lower-layer owners instead of rebuilding them.

## Layer Position

```text
core        -> reusable infrastructure and primitives
data        -> records, tables, fields, structures, bricks, virtual records
operational -> applied behavior, orchestration, and assembly
```

Operational code should:

- trust `*Rec` for record-owned truth
- use table wrappers and `ManagedTable` for database access
- use core primitives for errors, config, diagnostics, time, numbers, sorting,
  and secrets
- keep flow files focused on sequencing and applied decisions
- avoid direct JSON/payload parsing when a `vrt*Payload`, `vrt*Data`, `vrt*Rec`,
  or `clx*` can own the shape

Bypassing foundational owners is an exception path and needs a concrete reason.

## Prefix Contract

| Prefix | Boundary | Side Effect |
| --- | --- | --- |
| `compose*` | coordinates phases | high |
| `create*` | creates complete outcome | medium/high |
| `fetch*` | database read | read I/O |
| `resolve*` | records/context to facts | none |
| `prepare*` | context to RAM draft | none |
| `build*` | pure shape transform | none |
| `apply*` | in-memory mutation | none |
| `acquire*` | guard/lock/idempotency claim | scoped write |
| `persist*` | draft to database | write I/O |
| `ensure*` | idempotent invariant enforcement | scoped read/write |
| `emit*` | diagnostics/events/metrics | scoped I/O |
| `call*` / `request*` / `send*` | external integration | external I/O |

The label must match the contents.

## Functional Rules

- `create*` must own a complete meaningful outcome, not a thin wrapper.
- `compose*` must not manually replay sub-assembly already owned by called
  owners.
- `resolve*` must not perform database reads or writes.
- `prepare*` and `build*` must not perform database reads, writes, or network
  calls.
- `apply*` is RAM-only by default.
- `persist*` should receive known prepared artifacts and write them.
- `ensure*` may read/write only to enforce one invariant and must be safe to
  call repeatedly.
- Operational files must group by one domain owner or one justified table-family
  boundary, not by same verb alone.
- If a file grows past normal review comfort, look first for a missing reusable
  owner before accepting the size.

## Record-Shaping Boundary

Operational code must not:

- parse endpoint JSON repeatedly
- calculate payload-owned totals from raw payload fragments
- extract nested ids through long payload chains when a payload owner can expose
  accessors
- mutate JSONB fields with raw object literals
- create anonymous local DTOs for shapes that are reused, persisted, or security
  sensitive

Prefer:

- `Vrt*Payload` for endpoint ingress
- `Vrt*Data` for repeated RAM-side data
- `clx*` for allowed structured shapes
- `*Rec` methods for metadata-backed record attributes

## Mandatory Core Usage

Use structured core errors:

```rust
return Err(CoreError::validation_with_details(
    "ProjectRec.is_valid",
    "REQUIRED_FIELD_MISSING",
    "code is required",
    json!({ "field": "code" }),
));
```

Do not replace rich errors with vague outer errors. Enrich at boundaries when
context is useful.

Use central helpers for:

- time
- number normalization
- sorting
- secrets
- diagnostics and debug logging
- SQL/value binding through `ManagedTable` and `FieldValue`

## Exception Policy

Exceptions must be explicit and local:

```rust
// EXCEPTION[TYPE]: reason | auth: name YYYY-MM-DD
```

Missing or vague exception notes should be treated as unresolved design debt.

## Import Direction

Dependency direction matters:

```text
core/foundation -> no data, no db, no warmup
core/db         -> foundation only
core/warmup     -> foundation + db
data            -> core
operational     -> core + data
```

Generic core must not import project data. Bricks and structures must not hide
database access. Records must not save themselves.

## Flow Compression Rule

When a flow grows because missing behavior was restored, follow-up compression
is mandatory:

- push record-shaped work down into `*Rec`
- push allowed-shape work into `clx*`
- push payload normalization into `vrt*Payload`
- push pure repeated transforms into bricks
- push database writes into `persist*`
- keep `compose*` as orchestration

The goal is not tiny files for aesthetics. The goal is truthful ownership.
