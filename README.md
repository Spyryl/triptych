# Triptych

Triptych compiles Markdown doctrine and evidence documents into deterministic
YAML capsules for Sentinel.

Triptych does not interpret doctrine with AI. It preserves compact, structured
evidence so Sentinel can reason over stable capsule files instead of repeatedly
loading raw Markdown.

## Sentinel Command Contract

```bash
triptych sentinel build \
  --project-root <project> \
  --cache-root <cache> \
  --evidence <file>
```

Arguments:

- `--project-root <project>` is required. Evidence files must resolve inside
  this directory.
- `--cache-root <cache>` is required. Triptych writes YAML capsules under this
  directory.
- `--evidence <file>` adds one Markdown evidence file. It is repeatable.
- `--evidence-list <file>` adds newline-delimited evidence paths. Blank lines
  and lines starting with `#` are ignored. This option is stable for Sentinel
  integration.

Only `.md` evidence files are supported in this version.

## Exit Codes

- `0`: all requested capsules were created, reused, or updated.
- `1`: command, argument, path, unsupported evidence, cache, parse, or IO error.

Triptych fails hard on missing files, unsupported file types, and evidence paths
outside `--project-root`. It does not currently return skipped entries.

## Stdout

Successful `sentinel build` commands write machine-readable JSON to stdout:

```json
{
  "ok": true,
  "capsules": [
    {
      "source": "documentation/implementation/core/index.md",
      "capsule": ".sentinel/triptych-cache/documentation/implementation/core/index.yml",
      "status": "created"
    }
  ]
}
```

`status` is one of:

- `created`: no capsule existed before this run.
- `reused`: the existing capsule matched the source freshness metadata.
- `updated`: a capsule existed, but source freshness metadata changed.

On failure, Triptych writes a JSON error object to stderr:

```json
{"code":"EVIDENCE_NOT_FOUND","message":"evidence file does not exist: ..."}
```

## Stderr

Triptych-owned errors are safe for telemetry/logging and contain no tokens by
design. Build-tool output from `cargo run` is not part of the Triptych CLI
contract; Sentinel should call the compiled `triptych` binary directly.

## Cache Freshness

Capsule paths mirror each evidence path relative to `--project-root`:

```text
documentation/implementation/core/index.md
-> <cache-root>/documentation/implementation/core/index.yml
```

Triptych reuses an existing capsule when the capsule's recorded `mtime_unix_ms`
and `size` match the current source file. If either value differs, Triptych
regenerates the capsule and records the current SHA-256.

SHA-256 is evidence in the capsule schema. It is not the cheap cache freshness
gate in this version.

## Capsule Schema

Capsules are YAML with these stable top-level fields:

- `schema_version`: currently `1`.
- `generator`: currently `triptych-sentinel`.
- `source`: source path and fingerprint metadata.
- `doc`: title and Markdown headings.
- `rules`: conservative rule-like line extraction.

`source` fields:

- `absolute_path`
- `project_relative_path`
- `mtime_unix_ms`
- `size`
- `sha256`

`doc` fields:

- `title`
- `headings[]`

Each heading contains:

- `level`
- `text`
- `line`

`rules` fields:

- `must[]`
- `must_not[]`
- `should[]`
- `flag_if[]`

Each rule contains:

- `text`
- `evidence[]`

Each evidence entry contains:

- `line`

## Sentinel Example

```bash
triptych sentinel build \
  --project-root /Volumes/Development/Projects/NodeJS/Foodchoo-Financial \
  --cache-root /Volumes/Development/Projects/NodeJS/Foodchoo-Financial/.sentinel/triptych-cache \
  --evidence /Volumes/Development/Projects/NodeJS/Foodchoo-Financial/documentation/implementation/core/lazy-vs-lego-doctrine.md
```

For Axiom-style documentation, Sentinel should pass the Axiom project root as
`--project-root`, a Sentinel-owned cache directory as `--cache-root`, and either
repeat `--evidence` for each selected Markdown document or pass a stable
`--evidence-list`.

## Versioning Promise

For `schema_version: 1`, Sentinel can rely on:

- JSON stdout shape for successful `sentinel build` runs.
- JSON stderr shape for Triptych-owned failures.
- `--evidence-list` support.
- cache path mirroring from project-relative Markdown path to `.yml`.
- capsule fields listed in this README.
- status values `created`, `reused`, and `updated`.

Triptych may add fields without changing `schema_version`. It must not remove or
rename the fields above without introducing a new schema version.
