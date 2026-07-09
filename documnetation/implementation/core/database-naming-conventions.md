# Database Naming Conventions

Database object names should make ownership visible before a developer opens the
table definition.

These conventions are portable across Mandate schemas unless a project-specific
database plan says otherwise.

## Table Prefixes

### `tbl_`

Use `tbl_` for independent operational entities.

Characteristics:

- the row has its own lifecycle
- other tables may reference it
- it is not meaningful only as a child of one parent row

Examples:

```sql
tbl_project
tbl_entity
tbl_job
```

### `stbl_`

Use `stbl_` for dependent child tables.

Characteristics:

- the row belongs to a parent table row
- the row does not make sense without that parent
- the table normally carries a parent id

Examples:

```sql
stbl_project_member
stbl_job_step
stbl_entity_address
```

### `tlkp_`

Use `tlkp_` for lookup, reference, and configuration tables.

Characteristics:

- rows define allowed values, templates, source inventory, or config
- rows are usually seeded or managed through admin tooling
- operational code uses them as reference data

Examples:

```sql
tlkp_data_source
tlkp_error
tlkp_country
```

## Views

Use `vw_` for database views.

Examples:

```sql
vw_project
vw_job_step
vw_error_lookup
```

Views are read surfaces. Table-backed `*Rec` structs represent the base table
concept and may include view-only fields. Writes still target the base table.

## Schemas

Use schemas to separate operational domains and infrastructure boundaries.

Current foundational examples:

- `setting`: source inventory and runtime configuration.
- `log`: error lookup, occurred errors, and operational logs.

Future schemas should have a clear ownership purpose. Do not create a schema as
a junk drawer for unrelated tables.

## Decision Tree

```text
Is this reference/configuration data?
  yes -> tlkp_
  no  -> can the row exist independently?
           yes -> tbl_
           no  -> stbl_
```

Views use `vw_` regardless of whether they read a `tbl_`, `stbl_`, or `tlkp_`.

## Rules

- Lookup tables must be named distinctly from operational tables.
- Every operationally consumed table/view should be registered in the data
  source registry.
- Table/view names are database object inventory, not ad hoc Rust constants in
  every table wrapper.
- A consumed view belongs to the same `*Rec` as its table concept.
- If a table prefix feels awkward, revisit the data model before inventing a new
  prefix.
