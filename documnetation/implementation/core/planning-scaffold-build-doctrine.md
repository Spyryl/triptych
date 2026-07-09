# Planning, Scaffold, And Build Doctrine

Mandate uses planning, scaffold, and build records to turn intent into controlled
work. These lanes are not markdown replacements. They are database-backed work
contracts that must stay queryable, auditable, and small enough for agents and
humans to inspect without dragging large JSON blobs through every command.

## Planning

A plan records the intended outcome and the boundaries of the work before
scaffold, build, finding, or repair rows carry the execution detail.

Every non-trivial plan should be able to answer these questions:

- Objective: what outcome is the work trying to achieve?
- Scope: what is included and what is excluded?
- Current state: what exists now and what is missing?
- Architecture decision: what shape is chosen and why?
- Work slices: what small phases can be built and verified independently?
- Acceptance criteria: what must be true before the plan is considered done?
- Evidence required: which docs, commands, reports, tests, or smoke outputs are
  needed?
- Risks: what could go wrong or create false confidence?
- Non-goals: what is explicitly not being done in this plan?
- Rollback/compatibility: how existing behaviour remains safe while changing?
- Ownership boundaries: which actor or system is allowed to do what?

Planning may describe unknowns. It must not pretend unknowns are evidence.

## Scaffold

Scaffold records describe the planned shape of the work before implementation.
They are the contract the build should follow or deliberately correct.

The scaffold source of truth belongs in database tables, not markdown files.
Old markdown scaffolds were useful as a thinking tool, but they do not scale for
Mandate because they are hard to query, hard to diff by responsibility, expensive
for AI context, and weak as an audit trail.

Scaffold rows should be normalized around first-class child records:

- planned files/modules
- planned symbols/functions/classes
- planned dependencies/imports/callers
- planned LOC forecasts and limits
- planned ownership/layer classification
- planned evidence and doctrine requirements
- unresolved blockers or questions
- project-neutral child contract detail: spec/body, owner process, cache or
  state scope, failure behavior, and verification command
- explicit child-to-child dependency rows for ordering and build readiness

Small JSON metadata is allowed only for flexible extras. Primary scaffold
contract data must be in columns and rows so Mandate can answer targeted
questions such as:

- which planned files are still unresolved?
- which symbols are over forecast?
- which scaffold rows have no build proof?
- which build deviated from the planned file path?
- which planned caller/import was rejected or superseded?
- which planned child still depends on an unresolved child?

Scaffold child kinds are controlled lookup rows. The base catalogue lives in
`setting.tlkp_lookup` as `scaffold_child_kind`; the typed read surface is
`setting.vw_lookup_scaffold_child_kind`. This keeps new kinds such as `table`,
`record`, `view`, `logger`, or `runtime-setting` as data changes rather than
schema changes.

## Build

A build records execution against a plan or scaffold. A build may implement the
planned scaffold exactly, or it may prove that the scaffold was wrong and record
the corrected shape.

Build proof should reference specific scaffold child rows wherever possible.
The build result must be able to say:

- implemented as planned
- implemented at a different path
- merged into another file or symbol
- split into more rows
- rejected as the wrong shape
- deferred with a reason
- blocked by missing evidence or unresolved design

This makes completion auditable. A build should not close merely because code
changed. It closes because the relevant scaffold/build result rows and evidence
show what happened.

## JSON Boundary

JSON fields are for small metadata only. They must not become the primary store
for scaffold or build contracts.

Large stuffed JSON is forbidden for core work-control state because it:

- burns AI context tokens;
- hides child lifecycle;
- weakens audit and completion trails;
- makes UI forms parse blobs instead of records;
- makes targeted list views and unresolved filters difficult;
- encourages agents to treat opaque blobs as unverified truth.

If a value needs status, ownership, timestamps, lifecycle, proof, or links, it
needs a row.

## Lane Boundary

Planning owns intent.

Scaffold owns planned shape.

Build owns execution and proof against that shape.

Repair owns defect or change slices.

Finding owns claims that may or may not become work.

Sentinel and other analyzers may create or sync findings. They do not own
Mandate plans, scaffolds, builds, repairs, or links unless explicitly operating
as a non-Sentinel project actor with appropriate scopes.

## Task-Local Lineage

Scaffold and build child rows are always interpreted in relation to the task
they belong to. They are not global claims about a file forever.

Example:

```text
plan
  -> scaffold
    -> scaffold child: runner-rs/src/cli.rs owns CLI parsing
    -> scaffold child: runner-rs/src/rules/no_secret_literals.rs owns secret literal rule
  -> build
    -> build child/evidence: implemented runner-rs/src/main.rs only as temporary slice
    -> deviation: cli/rules modules not split yet
```

In this example, the scaffold says what the task expected. The build says what
actually happened for that task. The deviation is not a vague note buried in a
blob; it is a recordable result against the planned child rows.

This lets Mandate answer:

- what did this task plan to create or modify?
- which planned child rows were implemented?
- which planned child rows were deferred, rejected, merged, split, or moved?
- why did the build differ from the scaffold?
- what follow-up repair/build should own the unresolved difference?
