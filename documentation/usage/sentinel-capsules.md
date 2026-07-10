# Sentinel Capsules

Triptych's Sentinel rail is a deterministic evidence compiler.

It reads markdown files supplied by Sentinel, writes small YAML capsules under a
Sentinel-provided cache root, and returns a JSON report describing each capsule.
It does not call an LLM and it does not depend on Docent.

## Command

```bash
triptych sentinel build \
  --project-root /path/to/project \
  --cache-root /path/to/project/.sentinel/doctrine-cache \
  --cache-format yml \
  --evidence /path/to/project/documentation/architecture/core.md
```

Use `--evidence` more than once, or pass a newline-delimited file:

```bash
triptych sentinel build \
  --project-root /path/to/project \
  --cache-root /path/to/project/.sentinel/doctrine-cache \
  --cache-format json \
  --evidence-list /tmp/sentinel-evidence.txt
```

`--cache-format` is optional and defaults to `yml`. Supported values are `yml`,
`yaml`, and `json`. CLI stdout is JSON regardless of cache format.

## Cache Contract

Capsule paths mirror the evidence file path relative to `--project-root`.

```text
documentation/implementation/core/index.md
-> <cache-root>/documentation/implementation/core/index.yml
```

With `--cache-format json`, Triptych writes:

```text
documentation/implementation/core/index.md
-> <cache-root>/documentation/implementation/core/index.json
```

Each capsule records:

- source absolute path
- source project-relative path
- modified time in Unix milliseconds
- file size
- SHA-256
- markdown title and headings
- explicit rule-like lines with source line numbers
- immediate child bullets under rule stems
- short fenced examples labelled as examples, good, bad, allowed, or not allowed

Triptych reuses an existing capsule when recorded `mtime + size` match the
current source file. If either value differs, Triptych regenerates the capsule
and records the new SHA-256.

## Extraction Rule

V1 extraction is conservative. Triptych classifies explicit rule language such
as `must`, `must not`, `should`, `do not`, and `never` from Markdown paragraphs
and list items. It joins wrapped Markdown blocks before classification so
assertions are not chopped at source line wraps, but it must not invent doctrine
from surrounding prose.

When a rule-like line is a stem, Triptych attaches the immediate bullet or
numbered-list children beneath that stem. When a short fenced code block is
introduced by a label such as `Good`, `Bad`, `Example`, `Allowed`, or
`Not allowed`, or by a rule-like stem such as `Operational code should read
like:`, Triptych preserves it under `doc.examples`.

Sentinel remains responsible for selecting relevant capsules and using them in
review prompts.
