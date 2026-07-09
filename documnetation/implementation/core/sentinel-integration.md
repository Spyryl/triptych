# Sentinel Integration

Sentinel and Mandate should stay separate.

```text
Sentinel = audit engine
Mandate = work-control and finding system of record
```

Sentinel runs audits. Mandate records the durable outcome of those audits as
findings, evidence references, finding provenance, and audit provenance.

## Boundary

Sentinel owns:

```text
audit scripts
rule packs
model/panel review logic
raw scan artifacts
large generated reports
project-specific analyzer configuration
Sentinel runtime/database
```

Mandate owns:

```text
projects
users
tokens
project-scoped scopes
findings
repair/build lifecycle
evidence gates
audit trail
```

Do not merge the audit engine into Mandate. Mandate should know that Sentinel
ran and what Sentinel found; it should not become Sentinel's runtime.

## Authentication

Sentinel is a shared tool surface. It exists once as `agent_surface =
sentinel`.

Each user who wants Sentinel to submit findings must create a Mandate token for
Sentinel:

```text
human user: Fred Smith
tool surface: sentinel
token: Fred's Sentinel token
project access: Axiom, Foodchoo
scopes: project:read, finding:create, finding:read
```

Fred then stores that token in Sentinel. Sentinel must include it when calling
Mandate.

Sentinel must not write findings to Mandate without a valid Mandate token.

## Finding Submission

When Sentinel submits a finding, Mandate should receive enough provenance to
reconstruct where it came from:

```text
project_id
title
summary
source_tool = Sentinel
source_run_id
source_report_path or source_report_url
evidence excerpt
claimed files / lines
severity / confidence
```

The Mandate token identifies the accountable user and the tool surface:

```text
actor_user_id = token.user_id
actor_token_id = token.id
agent_surface = sentinel
```

## Results Belong In Mandate

Sentinel findings belong in Mandate because findings become part of the work
control story:

```text
finding -> investigation -> accepted/rejected/duplicate
finding -> repair/build
repair/build -> evidence -> verified/closed
later Sentinel run -> new finding linked to prior finding/repair
```

Mandate should store enough history for a later Sentinel run to say:

```text
prior finding: 123
linked repair: 456
repair claimed to fix X by doing Y
current analysis still sees X because Z
new finding references 123 and 456 in its provenance
```

## Audit Runs

Mandate may eventually add a small `analysis_run` lane/table, but only as a
provenance record:

```text
id
project_id
source_tool
source_run_id
started_by_user_id
started_by_token_id
summary counts
report path / URL
created_at
```

That table should not import Sentinel's execution engine, rule system, or raw
artifact store.

## Required Scopes

A normal Sentinel token should usually have:

```text
project:read
finding:create
finding:read
```

Grant those scopes to the Sentinel token, not to a fake Sentinel user:

```text
token-scope grant <token_id> project:read <project_id>
token-scope grant <token_id> finding:create <project_id>
token-scope grant <token_id> finding:read <project_id>
```

It must not have:

```text
plan:create
scaffold:create
build:create
repair:lifecycle
build:lifecycle
admin:user
admin:scope
link:create
```

Sentinel creates claims. Humans or authorised implementation agents investigate
and move work through repair/build lifecycle.

If Sentinel is working on Sentinel itself, the implementation work must use a
non-Sentinel token such as Codex, Kimi, web UI, or a human operator token.
`agent_surface = sentinel` remains findings-only even on the Sentinel project.

## UI Implications

Mandate's UI needs a user-facing token area where a human can:

- create a Sentinel token
- see the token once
- copy it into Sentinel
- grant project-scoped scopes
- revoke or roll the token
- inspect which projects Sentinel can submit findings for

The UI should present this as:

```text
Tool tokens
  Codex
  Kimi
  Sentinel
  CLI/API
```

not as fake per-tool users.
