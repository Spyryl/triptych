# Lazy vs Lego Doctrine

Mandate uses two complementary ideas:

- Lazy: use the truthful owner instead of repeating the same work.
- Lego: build small, composable owners and assemble larger outcomes from them.

The goal is low drift, small files, and changes that can be made once in the
right place.

## Lazy

Lazy does not mean careless or incomplete. It means ownership discipline.

Rules:

- If work already has a truthful owner, call that owner.
- If the same pipeline appears in two or more places, promote it into one owner.
- If a called owner already creates a downstream outcome, callers do not replay
  the downstream pipeline around it.
- If normalization belongs to a `*Rec`, `vrt*`, or `clx*`, operational code does
  not repeat that normalization.
- If SQL construction belongs to `ManagedTable`, table wrappers do not duplicate
  query assembly.
- If a brick can solve a pure repeated value transform, pass the required values
  into the brick and keep it free of hidden context.

The benefit is simple: bugs are fixed once, not copied into many flows.

## Lego

Lego means owners are small enough to compose:

- `fetch*`: read boundary; database facts become `*Rec` or explicit context.
- `resolve*`: derive facts from already-fetched records or context.
- `prepare*` / `build*`: RAM-only assembly and pure transformations.
- `apply*`: in-memory mutation of records, virtual records, or plans.
- `persist*`: database mutation of known prepared artifacts.
- `create*`: complete meaningful outcome owner.
- `compose*`: orchestration owner that sequences phases.
- `emit*`: event, metric, or diagnostic emission.
- `call*` / `request*` / `send*`: external integration I/O.
- `ensure*`: idempotent required-state enforcement.

These names are not decoration. They tell the reader where I/O, mutation, and
business ownership live.

## No Replays

If a downstream pipeline is owned by a called owner, callers do not replay it.

Bad shape:

```text
compose_flow:
  fetch context
  build child records by hand
  persist many unrelated tables by hand
  repeat mapping/normalization locally
  call a create owner that already does some of the same work
```

Good shape:

```text
compose_flow:
  fetch required context
  resolve policy and authority
  prepare top-level records
  call the owner that builds mapped child artifacts
  call the owner that persists the prepared graph
  emit follow-up diagnostics/events
```

When a flow grows into a long list of little repeated steps, treat that as an
ownership problem. Extract the truthful owner instead of expanding the flow.

## Formatting Is Foundational

Operational code should receive clean records and payload owners.

Avoid this shape:

```rust
rec.code = raw_code.trim().to_uppercase();
rec.amount = raw_amount.parse().unwrap_or_default();
```

Prefer this shape:

```rust
rec.set_code(raw_code);
rec.set_amount(raw_amount)?;
```

The record, virtual record, structure, or core helper owns the normalization.
Operational code should compose outcomes, not scatter parsing and fallback
rules.

## Owner Placement

Use the smallest truthful owner:

- `core/foundation`: reusable primitives that exist before database warmup.
- `core/db`: generic table/record persistence contracts.
- `core/warmup`: runtime registries loaded after the database pool exists.
- `data/records`: persisted row behavior.
- `data/tables`: thin table manager wrappers.
- `data/bricks`: pure value transformations.
- `data/structures`: allowed-shape contracts.
- `data/virtual_records`: non-table objects with behavior.
- operational code: applied behavior, orchestration, and assembly.

If an owner needs hidden database access, it is not a brick or structure. If it
saves itself, it is not following the record/table split.

## Triage Checklist

Ask these before creating or expanding code:

1. Is this replaying work that another owner already owns?
2. Is this formatting or parsing that belongs in a `*Rec`, `vrt*`, `clx*`, or
   core helper?
3. Is this a thin wrapper that adds no ownership?
4. Is this mixing reads, RAM-only derivation, and writes under a misleading
   name?
5. Is this local shape important enough to become a `clx*` or `vrt*` owner?
6. Is this file growing because a reusable primitive is missing?

## Rust-Specific Rule

Rust gives us explicit traits, modules, and `Result<T>`. Use those strengths.

Do not blindly port dynamic NodeJS or WinDev patterns. Preserve the ownership
doctrine, then express it through Rust structs, traits, modules, typed values,
and explicit errors.
