# Record Scaffold Generator

Mandate keeps the lazy architecture, but Rust makes `fields + Rec + Table`
mechanically expensive. The scaffold generator removes the boring part without
changing the ownership model.

Dry-run a record scaffold:

```bash
mandate scaffold record documentation/scaffold/examples/note-rec.json
```

Write files if none of the targets exist:

```bash
mandate scaffold record documentation/scaffold/examples/note-rec.json --write
```

Check that files and module updates can be applied without writing:

```bash
mandate scaffold record documentation/scaffold/examples/note-rec.json --check --update-mods
```

Write files and append missing `mod.rs` exports:

```bash
mandate scaffold record documentation/scaffold/examples/note-rec.json --write --update-mods
```

The generator creates:

- `src/data/fields/<module_name>.rs`
- `src/data/records/<module_name>_rec.rs`
- `src/data/tables/<module_name>.rs`
- `documentation/deployment/database/<schema>/tables/<object>.sql`
- `documentation/deployment/database/<schema>/views/<object>.sql`

The dry-run JSON also includes `mod_updates`, which lists the module/export
lines that `--update-mods` will manage.

Spec rules:

- `module_name` and field names must be `snake_case`.
- `rec_name` must be a Rust type name.
- `table_name` and `view_name` must be `schema.object`.
- `pk_field` must exist in `fields`.
- `pk_field` must be persisted.
- Field names must be unique.
- Every field must be persisted, included in the view, or both.
- `max_len` is optional and only valid for `text` / `opt_text` fields.
- `default` is optional and validated by field type.

View-only fields use `FieldDef::virtual_field`, hydrate from view rows, and stay
out of `persistable_columns()`.

Generated SQL is starter DDL for the same `*Rec` contract:

- persisted fields become table columns
- view fields become default view columns
- `actv` / `dltd` produce the default active/non-deleted view predicate
- no foreign keys are generated
- no per-table `updated_at` trigger is generated

Supported defaults:

- `bool`: `true` or `false`
- `i64` / `i32`: an integer literal in range
- `text` / `opt_text`: a string value
- `json`: `{}` or `object`
- `timestamp`: `now` or `now_utc`

It intentionally does not overwrite existing files. `--update-mods` appends only
missing module/export lines to:

- `src/data/fields/mod.rs`
- `src/data/records/mod.rs`
- `src/data/tables/mod.rs`

Before writing, the CLI validates all generated targets. If `--update-mods` is
present, it also validates that the module files can be read before creating
any generated file. `--check` runs the same validation without writing.

Generated files still need review. The generator is for the mechanical spine;
domain-specific virtual fields, lifecycle helpers, custom validation, and
deeper domain behavior are still deliberate code.
