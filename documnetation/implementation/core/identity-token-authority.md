# Identity, Tool Tokens, And Project Authority

Mandate is the authority boundary for humans, tools, AI agents, and project
work. Clients do not receive database credentials and do not bypass Mandate.

## Core Model

Mandate separates four ideas:

```text
human user
tool / agent surface
token
project-scoped permissions
```

Example:

```text
human user: Fred Smith
tool: Codex
token: xyz123abc
projects token can access: Axiom, Sentinel, Foodchoo
```

Codex is not duplicated into a fake user for each human. Codex is a tool
surface. Fred's Codex token is the thing that proves Fred authorised Codex to
act through Mandate.

The same applies to Sentinel, Kimi, CLI, API, and the web UI.

## Users

`identity.stbl_user` represents accountable Mandate users.

Human users are normal accountable users:

```text
christo
fred-smith
jenny-lee
```

Service users may exist for operator-owned automation, but they should not be
used to hide which human authorised project work.

## Tool Surfaces

Tool surfaces describe where an action came from:

```text
web
cli
api
codex
kimi
sentinel
```

Tool surfaces are not project-specific users. They are stable labels used on
tokens and audit events.

## Tokens

Tokens are PAT-style credentials. Raw token values are shown once and only
token hashes are persisted.

A token belongs to an accountable user and names the tool surface:

```text
user_id: Fred Smith
agent_surface: codex
token_hash: ...
token_prefix: ...
label: Fred's Codex token
```

For a shared tool such as Sentinel:

```text
user_id: Fred Smith
agent_surface: sentinel
label: Fred's Sentinel token
```

Fred creates or rolls his Sentinel token in Mandate, then stores that token in
Sentinel. Sentinel must present that Mandate token when submitting findings.

## Project Access

A token being valid is not enough. It must be allowed to act on the project and
must have the right scope for the command.

The preferred rule is:

```text
token + project + scope
```

Mandate stores per-token grants in `identity.stbl_token_scope`. User scopes are
still the maximum authority for the accountable user; token scopes narrow that
authority for a specific token.

During the v1 transition, tokens with no token-scope rows continue to use user
scopes. Once a token has token-scope rows, the token must have the required
scope as well as the user.

Examples:

```text
Fred's Codex token on Axiom:
  finding:read
  repair:read
  repair:evidence

Fred's Sentinel token on Axiom:
  finding:create
  finding:read

Fred's Sentinel token on Foodchoo:
  finding:create
  finding:read
```

Sentinel is always findings-only. A Sentinel token must not receive plan,
scaffold, build, repair, admin, link, policy, or audit scopes. Repairs or
upgrades to Sentinel itself must be performed by another tool surface such as
Codex, Kimi, web, or a human operator token.

Do not make token rights universal across all projects unless the token is an
operator-owned, time-limited administrative token.

## User Level

`user_level` is project membership authority. It is not a replacement for
scopes.

Use `user_level` to decide whether a project member can manage other members or
grant authority inside that project.

Use scopes to decide whether a token or user can run a command.

```text
membership: is this user attached to this project?
user_level: can this user manage project authority?
scope: can this actor perform this action?
```

Core commands:

```text
user token create <user_id> <agent_surface> <label> [authority_user_id] [project_id]
token-scope grant <token_id> <scope_code> [project_id] [granted_by]
token-scope list <token_id>
user-scope grant <user_id> <scope_code> [project_id] [granted_by]
```

## Audit

Audit should preserve all parts of the identity story:

```text
actor_user_id: Fred Smith's user id
actor_token_id: Fred's Codex token id
agent_surface: codex
project_id: Axiom
command: repair evidence ...
```

For Sentinel:

```text
actor_user_id: Fred Smith's user id
actor_token_id: Fred's Sentinel token id
agent_surface: sentinel
source_tool: Sentinel
source_run_id: sentinel-run-123
project_id: Axiom
```

This tells reviewers:

- which human is accountable
- which tool performed the action
- which token was used
- which project boundary applied
- which command and scope were checked

## Break-Glass Database Access

Normal users, AI agents, and external tools never receive raw storage access.

Emergency database access is operator-only break-glass maintenance. It is not a
normal Mandate interaction path and must not become a client integration method.
