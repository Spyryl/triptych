# Operational CLI

Mandate alpha is local-first. The CLI is the operational surface for proving the
control loop before remote ingestion or API exposure.

Use the repo wrapper:

```bash
bin/mandate help
```

For a system-style command, add the repo `bin/` directory to `PATH` and call
`mandate ...`. While developing, `cargo run -- ...` is equivalent.

## Projects And Policy

```bash
bin/mandate project list
bin/mandate project create <code> <name> <repo_path> [default_branch]
bin/mandate project status <project_id>
bin/mandate policy list [project_id]
```

`policy list` without a project lists all policy rows. `policy list <project_id>`
lists the effective alpha policy for that project: global defaults
(`project_id = 0`) plus project-specific rows.

## Minimal Alpha Lanes

Planning:

```bash
bin/mandate plan create <project_id> <title> [summary]
bin/mandate plan list <project_id>
bin/mandate plan show <plan_id>
bin/mandate plan item-create <plan_id> <item_kind> <title> [body] [parent_item_id]
bin/mandate plan item-list <plan_id>
bin/mandate plan close <plan_id>
```

Plan items hold the cookbook-test planning rows: objective, scope,
current_state, architecture_decision, work_slice, acceptance_criteria,
evidence_required, risk, non_goal, rollback, ownership_boundary, question, and
note.

Scaffold:

```bash
bin/mandate scaffold create <project_id> <title> [plan_id] [target_path]
bin/mandate scaffold list <project_id>
bin/mandate scaffold show <scaffold_id>
bin/mandate scaffold child-create <scaffold_id> <child_kind> <title> [target_path] [parent_child_id]
bin/mandate scaffold child-detail <scaffold_child_id> [body] [owner_process] [cache_scope] [failure_behavior] [verification_command]
bin/mandate scaffold child-list <scaffold_id>
bin/mandate scaffold dependency-create <child_id> <depends_on_child_id> [dependency_kind] [notes]
bin/mandate scaffold dependency-list <scaffold_id>
bin/mandate scaffold ready <scaffold_id>
bin/mandate scaffold block <scaffold_id> <reason>
bin/mandate scaffold close <scaffold_id>
```

Scaffold children are the planned files, modules, symbols, functions,
dependencies, docs, evidence, questions, and other child rows that make the
scaffold queryable. They are task-local; they describe the shape expected for
that scaffold, not a permanent global ownership claim.

`child-detail` adds project-neutral contract detail to a child row: spec/body
text, owner process, cache or state scope, failure behavior, and verification
command. A cookbook project, a financial runtime, and a Rust audit runner can
all use those fields without inheriting Axiom-specific rules.

`dependency-create` records explicit child-to-child dependency edges. Use it
when one planned file, module, symbol, check, or decision must exist before
another can be built or verified. Do not hide dependency order inside large
JSON blobs.

Scaffold child kinds are lookup data, not table schema. They are seeded under
`setting.tlkp_lookup` with lookup type `scaffold_child_kind` and exposed through
`setting.vw_lookup_scaffold_child_kind`. Add new project-neutral kinds by
seeding lookup rows, not by altering `scaffold.tbl_scaffold_child`.

Build:

```bash
bin/mandate build create <project_id> <title> [scaffold_id] [file_path]
bin/mandate build list <project_id>
bin/mandate build show <build_id>
bin/mandate build evidence <build_id> <preflight_status> [--profiles code,code] [evidence_doc] [notes]
bin/mandate build changed-files <build_id> <path,path>
bin/mandate build result-create <build_id> <scaffold_child_id> <result_status> [actual_path] [notes]
bin/mandate build result-list <build_id>
bin/mandate build gate <build_id>
bin/mandate build start <build_id>
bin/mandate build block <build_id> <reason>
bin/mandate build recheck <build_id>
bin/mandate build verify <build_id>
bin/mandate build close <build_id>
```

`plan_id` and `scaffold_id` are optional in create commands. Pass `0` when you
want to skip that optional link while still supplying a later positional value.

## Findings

```bash
bin/mandate finding create <project_id> <title> [summary]
bin/mandate finding list <project_id> [--unresolved]
bin/mandate finding status <finding_id>
bin/mandate finding detail-create <finding_id> <detail_kind> <summary> [path] [detail]
bin/mandate finding detail-list <finding_id>
bin/mandate finding investigate <finding_id> [reason]
bin/mandate finding accept <finding_id> [repair_title]
bin/mandate finding accept-build <finding_id> [build_title]
bin/mandate finding reject <finding_id> <reason>
bin/mandate finding duplicate <finding_id> <canonical_finding_id> [reason]
```

Findings are the intake lane. Investigation either rejects/duplicates the row or
accepts it into a controlled repair/build lane.

Finding details are structured audit rows for locations, evidence,
checker_output, claims, impact, recommendations, duplicate references,
reproduction steps, source reports, and notes. Do not stuff a whole Sentinel
report into one finding summary when the important parts need lifecycle,
filtering, or review.

Finding resolution is derived from linked work. Do not manually pretend a
finding is resolved. Use `finding status <finding_id>` to inspect the raw
finding status, linked repairs/builds, effective status, blockers, and evidence
paths. Use `finding list <project_id> --unresolved` for actionable findings; it
excludes accepted findings whose linked repairs/builds are verified or closed.

## Repairs

```bash
bin/mandate repair create <project_id> <title> [repair_type] [file_path]
bin/mandate repair child <parent_repair_id> <title> [repair_type] [file_path]
bin/mandate repair tree --file <repair-tree.yml>
bin/mandate repair list <project_id>
bin/mandate repair evidence <repair_id> <affected_layer> <preflight_status> [--profiles code,code] [evidence_doc] [notes]
bin/mandate repair changed-files <repair_id> <path,path>
bin/mandate repair gate <repair_id>
bin/mandate repair start <repair_id>
bin/mandate repair block <repair_id> <reason>
bin/mandate repair fixed <repair_id> [commit_sha] [report_path]
bin/mandate repair verify <repair_id> [report_path]
bin/mandate repair close <repair_id> [notes]
```

Tree creation accepts YAML or JSON. To create a new parent with children:

```yaml
title: Parent repair
repair_type: code
file_path: src/example.rs
children:
  - title: Child one
  - title: Child two
    children:
      - title: Child two point one
```

To add children under an existing open repair:

```yaml
parent_repair_id: 22
children:
  - title: Child 22.1
  - title: Child 22.2
```

The tree command inserts rows transactionally, validates the existing parent,
rejects closed parents, and assigns sibling sort order deterministically.

## Gates

Inspect repair policy with:

```bash
bin/mandate repair gate <repair_id>
```

The command returns:

- whether start evidence is required
- whether finish evidence is required
- whether passing evidence exists
- whether start or finish are currently blocked
- target paths from `file_path` and metadata `changed_files`
- required evidence profile codes
- missing evidence profile codes
- matching policy rows
- attached evidence rows and preflight status

Lifecycle commands use the same gate evaluation:

- `repair start` blocks when before-start evidence is required and no passing
  evidence exists.
- `build start` blocks when before-start evidence is required and
  `preflight_status` is not `pass`.
- `repair verify` and `repair close` block when finish evidence is required and
  no passing evidence exists.
- parent repairs cannot be verified or closed while child repairs remain open.

Only `preflight_status = pass` satisfies a gate. When a policy path rule
requires evidence profiles, passing evidence must include the matching profile
codes:

```bash
bin/mandate build evidence 23 pass --profiles rust-error-boundary,mandate-policy-pack-doctrine src/ops/mandate_ops.rs "cargo test passed"
bin/mandate repair evidence 308 core pass --profiles axiom-record-doctrine documentation/concepts-and-requirements/record-construction.md "record doctrine checked"
```

Use changed-file commands when a work item touches more than its original
`file_path`. Gates evaluate all target paths:

```bash
bin/mandate build changed-files 23 src/ops/mandate_ops.rs,src/data/records/build_rec.rs
bin/mandate repair changed-files 308 data/records/order_rec.ts,data/structures/clxOrder.ts
```

## Controlled Links

Inspect the controlled vocabulary with:

```bash
bin/mandate control work-kinds
bin/mandate control relationship-kinds
bin/mandate control allowed-relationships
```

Use `link create` only for relationships allowed by
`setting.stb_allowed_work_relationship`:

```bash
bin/mandate link create <source_kind> <source_id> <target_kind> <target_id> <relationship_kind>
bin/mandate link list <work_kind> <work_id>
```

Same-lane links are allowed when the controlled relationship table permits
them. For example, Sentinel can link a later finding back to an earlier finding:

```bash
bin/mandate link create finding <new_finding_id> finding <old_finding_id> regression_of
bin/mandate link create finding <new_finding_id> finding <old_finding_id> reopened_by
bin/mandate link create finding <new_finding_id> finding <old_finding_id> related_to
bin/mandate link create finding <new_finding_id> finding <old_finding_id> supersedes
```

If a finding is satisfied or caused by repair/build work, link it to that work:

```bash
bin/mandate link create finding <finding_id> repair <repair_id> fixes
bin/mandate link create finding <finding_id> repair <repair_id> partially_fixes
bin/mandate link create finding <finding_id> build <build_id> investigates
bin/mandate link create finding <new_finding_id> repair <repair_id> caused_by_fix_for
```

Mandate does not use database foreign keys for work links. The operational owner
performs the integrity checks instead:

- the relationship kind must be allowed for the source/target work kinds
- metadata must be present when the allowed-relationship row requires it
- supported alpha endpoints must exist before the link is inserted

The `doc` work kind is intentionally treated as external evidence and is not
resolved against a Mandate table in alpha.

## Legacy Migration

Repairs v2 is a legacy source. Mandate owns the import and reads the source with
isolated `MANDATE_IMPORT_*` settings:

```text
MANDATE_IMPORT_DB_HOST
MANDATE_IMPORT_DB_PORT
MANDATE_IMPORT_DB_NAME
MANDATE_IMPORT_DB_USER
MANDATE_IMPORT_DB_PASSWORD
MANDATE_IMPORT_SCHEMA
```

Commands:

```bash
bin/mandate migration legacy-source plan
bin/mandate migration legacy-source import --dry-run
bin/mandate migration legacy-source import --apply
```

Dry-run reports inserts, duplicates, and rejected rows without writing. Apply
imports idempotently and preserves legacy IDs in metadata where relevant.

## Scaffold Generator

```bash
bin/mandate scaffold record <spec_path> [--check] [--write] [--update-mods]
```

The scaffold generator is for Mandate's own `fields + Rec + Table` pattern. It
is not the full future Axiom-style scaffold database.

## Sequences

```bash
bin/mandate sequence list
bin/mandate sequence create <sequence_key> [prefix] [next_value]
bin/mandate sequence next <sequence_key>
```
