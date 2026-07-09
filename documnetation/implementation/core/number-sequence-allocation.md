# Number Sequence Allocation

`setting.tbl_number_sequence` is the source of truth for managed reference
numbers. Callers must not read `next_value`, increment it in RAM, and save it
outside the sequence owner.

Use:

```text
mandate sequence list
mandate sequence create <sequence_key> [prefix] [next_value]
mandate sequence next <sequence_key>
```

Operational code should use `NumberSequenceTable::allocate_next`. The table
owner opens a transaction, takes a transaction-scoped advisory lock for the
sequence key, locks the matching row with `FOR UPDATE`, returns the current
value, advances `next_value`, and commits.

CLI orchestration goes through `MandateOps`:

- `list_sequences`
- `create_sequence`
- `allocate_sequence`

The returned payload contains:

- `sequence_key`
- `value`
- `label`
- `next_value`

`label` is the record prefix plus the allocated value. The pure allocation rule
lives on `NumberSequenceRec`, so the table owner only handles serialization and
persistence.
