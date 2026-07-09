# Operational Naming Conventions

Operational function names should encode intent and side-effect level.

This keeps phase boundaries visible during review and makes lazy ownership
failures easier to spot.

## Canonical Prefixes

### `compose*`

Orchestrates a flow end-to-end.

Allowed:

- call `fetch*`, `resolve*`, `validate*`, `prepare*`, `build*`, `create*`,
  `persist*`, `emit*`, and external integration owners
- sequence phases and policy decisions

Avoid:

- low-level SQL
- raw payload parsing
- record-local normalization
- manual replay of work already owned by called `create*` or `persist*` owners

### `create*`

Creates a complete meaningful artifact or outcome.

Allowed:

- call lower owners to build, apply, persist, and emit the outcome
- call another `create*` only when the nested creation is a genuinely separate
  business outcome

Avoid:

- thin pass-through wrappers
- child-record phase names that are really `build*` or `persist*`

### `fetch*`

Owns database reads.

Input should usually be ids or criteria. Output should usually be `*Rec` values
or explicit context.

Forbidden:

- database writes
- posting
- queueing
- orchestration

### `resolve*`

Derives facts from already-fetched records or context.

Forbidden:

- database reads
- database writes
- network calls

If `resolve*` needs more database facts, create or call a `fetch*` owner first.

### `prepare*`

Assembles RAM-side drafts, plans, or record graphs from known context.

Forbidden:

- database reads
- database writes
- network calls

### `build*`

Performs pure local transformations.

Prefer `build*` for shape conversion and deterministic construction that does
not need records as living owners.

### `apply*`

Applies in-memory mutation to a record, virtual record, or plan.

Database writes belong in `persist*`, not ordinary `apply*`.

### `persist*`

Owns database writes for known prepared artifacts.

Allowed:

- inserts
- updates
- deletes
- transactional persistence
- batched writes

Avoid:

- fetching broad context
- composing entire flows
- calculating record-owned fields locally

### `validate*` / `assert*`

Use `validate*` for user/data rule checks that return structured errors.

Use `assert*` for internal invariants and preconditions.

### `acquire*`

Acquires resource guards, idempotency claims, or locks.

It may write guard state when that is the narrow purpose of the owner.

### `ensure*`

Enforces required state idempotently.

It may read or write only to satisfy one invariant and must be safe to call
repeatedly.

### `emit*`

Emits events, metrics, logs, or diagnostics.

### `call*` / `request*` / `send*`

Owns external integration I/O such as HTTP, API, queue, or transport calls.

Prefer these over ambiguous `post*` naming unless the domain specifically uses
`post*` for accounting or ledger posting.

## File Match Rule

For operational runtime files, the primary exported function should match the
file stem.

Example:

```text
compose_project_flow.rs -> compose_project_flow
fetch_project_context.rs -> fetch_project_context
```

Compatibility aliases should be short-lived migration debt.

## Names To Avoid

Avoid new `process*` functions. The name hides whether the function reads,
writes, validates, transforms, or orchestrates.

Avoid generic `handle*` unless the function is truly an adapter or route
boundary.

## Neutral Contract Files

Contract files may use names such as `*_contracts.rs` when they contain only
side-effect-free types, constants, or protocol definitions.

They must not perform:

- database reads
- database writes
- orchestration
- normalization fallback chains
- runtime policy resolution

If behavior appears, split it into the correct operational owner.

## Core Discipline

These names complement the core rules:

- `*Rec` normalizes, scrubs, validates, and exposes record-local behavior.
- `ManagedTable` owns generic persistence mechanics.
- `fetch*` is where database facts enter operational code.
- `resolve*`, `prepare*`, and `build*` should be RAM-only.
- `persist*` concentrates database writes for transaction forensics.
- `compose*` reads like an instruction manual, not a data-shaping workshop.
