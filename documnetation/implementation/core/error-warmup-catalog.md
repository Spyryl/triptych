# Error Warmup Catalog

## Purpose

`log.tlkp_error` is the canonical lookup table for reusable managed-error
definitions. Operational code should not repeat the same message, severity,
status code, and description every time it raises the same error.

The warmup catalog loads those lookup records once, keeps them in memory, and
lets runtime code build structured errors from a code plus call-site context.

## Database Sources

The stable database objects are:

```text
log.tlkp_error
log.vw_error_lookup
```

`log.tlkp_error` includes persisted control fields such as `actv` and `dltd`.
The default view may filter by those fields without returning them. See
`documentation/implementation/core/table-record-view-field-rules.md` for the
general record/view rule.

The `metadata` field should be governed by a `clx*` structure, not treated as an
open junk drawer. See `documentation/implementation/core/structures.md`.

The deployment files are:

```text
documentation/deployment/database/log/tables/error-lookup.sql
documentation/deployment/database/log/views/error-lookup.sql
```

The source inventory seed that makes `ErrorLookupTable` discoverable is:

```text
documentation/deployment/database/seed/data-source.sql
```

Reusable error-code seed rows will live in a separate error-lookup seed once the
initial catalog is ready. When a new reusable error definition is needed, add it
to that seed/admin path, rerun it, then reload warmup.

## Hydration Model

The WinDev-shaped flow is:

```text
records is ErrorLookupTable
records.fetchView()
warmup[errors] = records.arrRec
```

The Rust-shaped flow is:

```rust
let mut records = ErrorLookupTable::new(pool)?;
let errors = records.fetch_view(FetchArgs::new()).await?;
warmup.set_errors(errors);
```

Warmup must keep the hydrated records, not separate field-level maps.

Preferred internal shape:

```rust
pub struct Warmup {
    errors: ErrorWarmup,
}

pub struct ErrorWarmup {
    arr_rec: Vec<ErrorLookupRec>,
    by_code: HashMap<String, ErrorLookupRec>,
}
```

This preserves the table/record pattern while giving fast lookup.

## Lookup Model

Runtime code should be able to fetch the full lookup record in one in-memory
call:

```text
rec is ErrorLookupRec = warmup[errors][code = "REQUIRED_FIELD_MISSING"]
```

Rust should expose that as a safe method:

```rust
let rec = warmup.errors().by_code("REQUIRED_FIELD_MISSING")?;
```

Avoid direct indexing that can panic when a code is missing. Missing codes should
produce a managed error using `ERROR_CODE_NOT_REGISTERED`.

## Error Construction

The lookup record owns the reusable default values:

```text
code
description
message_template
severity
status_code
domain
metadata
```

The call site owns the local context:

```text
source
details
context
cause/path
optional overrides
```

Common required-field errors should be one-liners at the call site:

```rust
return Err(error_manager.required_field(
    "ProjectRec.is_valid",
    "code",
));
```

Equivalent expanded intent:

```rust
return Err(error_manager
    .from_code("REQUIRED_FIELD_MISSING")
    .source("ProjectRec.is_valid")
    .details(json!({ "field": "code" })));
```

The final managed error should be built from the lookup record plus call-site
details. For `REQUIRED_FIELD_MISSING`, the message template:

```text
You must enter data in the [%field%] field
```

with:

```json
{ "field": "code" }
```

renders as:

```text
You must enter data in the code field
```

## Override Rule

Lookup defaults are defaults, not a prison.

Operational code may override message, severity, status code, or details when a
specific failure needs more precision. The original code should usually remain
the same so reporting/grouping still works.

Example intent:

```rust
return Err(error_manager
    .from_code("REQUIRED_FIELD_MISSING")
    .source("ImportProjectRec.is_valid")
    .message("Project import row is missing code")
    .details(json!({ "field": "code", "row": row_number })));
```

## Path Rule

Warmup lookup records do not know how the program reached the failure.

The error manager/core error layer must keep the path by enriching errors at
boundaries:

```rust
save_project(rec).await.map_err(|err| {
    err.enrich(
        "ProjectFlow.save",
        "PROJECT_SAVE_FAILED",
        None,
        None,
        Some(json!({ "project_code": rec.code })),
        None,
    )
})?;
```

Each boundary should add only the useful context it owns. Do not replace the
original error with a vague outer error.

## Runtime Rules

- Warmup fetches `log.vw_error_lookup` once during startup.
- Error creation must not query PostgreSQL.
- The full `Vec<ErrorLookupRec>` should remain available for diagnostics/admin
  tooling.
- A `HashMap<String, ErrorLookupRec>` should be built once for fast code lookup.
- Missing code is a managed error, not a panic.
- Seed rows are added as reusable errors are discovered.
- Prefer reusable codes such as `REQUIRED_FIELD_MISSING` over field-specific
  copies such as `PROJECT_CODE_REQUIRED` unless the error truly has distinct
  business meaning.

## Build Order

1. Create `ErrorLookupRec`.
2. Create `ErrorLookupTable` for `log.tlkp_error` / `log.vw_error_lookup`.
3. Create `Warmup` and `ErrorWarmup`.
4. Load `ErrorLookupTable.fetch_view(FetchArgs::new())` into warmup.
5. Add `ErrorManager` methods that build `ManagedError` from warmup records.
6. Replace repeated record validation errors with lookup-backed helpers.
