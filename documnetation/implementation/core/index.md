# Core Implementation Docs

Architecture context:

- `../../CTO/ACD-CORE.md`: Rust core architecture context and boundaries.
- `../../CTO/ACD-DATA-SOURCE-REGISTRY.md`: database-owned table/view inventory and runtime source resolution.
- `../../CTO/WINDEV-TO-RUST-CORE-MAP.md`: mapping from WinDev and NodeJS concepts to Rust.

Supporting contracts:

- `data-source-registry.md`: warmed runtime registry for table/default-view/alternate-view sources.
- `database-naming-conventions.md`: table/view prefixes and schema naming intent.
- `field-source-doctrine.md`: Rust field definitions plus database-owned table/view source inventory.
- `foundational-data-records.md`: `DataSourceRec`, `ErrorLookupRec`, `ErrorRec`, and `LogRec`.
- `lazy-vs-lego-doctrine.md`: ownership and composition doctrine for avoiding replayed pipelines.
- `operational-code-sovereignty.md`: operational layer rules, side-effect boundaries, and import direction.
- `operational-cli.md`: local CLI control-loop commands and repair gate inspection.
- `operational-naming-conventions.md`: function/file prefixes that expose intent and side effects.
- `number-sequence-allocation.md`: locked allocation owner for managed reference numbers.
- `planning-scaffold-build-doctrine.md`: database-backed plan, scaffold, build contracts and proof/deviation rules.
- `record-doctrine.md`: `*Rec` as the table-backed living schema and flat behavior surface.
- `record-scaffold-generator.md`: generated `fields + Rec + Table` spine for reducing mechanical Rust boilerplate.
- `legacy-source-migration.md`: read-only legacy source migration planning and table mapping.
- `table-record-view-field-rules.md`: persisted fields, view fields, dirty tracking, and read/write rules.
- `bricks.md`: pure reusable data-layer helper functions.
- `structures.md`: `clx*` allowed-shape contracts.
- `virtual-fields.md`: view-only, runtime-only, and computed fields on record-like objects.
- `virtual-records.md`: non-persisted record-like payloads and workflow data.
- `error-warmup-catalog.md`: error lookup/catalog behavior.
