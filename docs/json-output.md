# JSON Output

Use `--json` to get machine-readable command output.

## Design goals

- deterministic field names and ordering
- explicit command identity
- explicit schema versioning for stable integrations

## Common top-level fields

Most JSON responses include these fields:

- `schema_version`
- `command`
- `query` or command-specific request fields
- `results` (array)

## Integration guidance

- Parse by field names, not by line formatting.
- Validate `schema_version` in your automation.
- Treat unknown fields as forward-compatible additions.

## Recommended agent usage

Prefer JSON mode when agents consume `repo-scout` output:

```bash
repo-scout find <symbol> --repo . --json
repo-scout refs <symbol> --repo . --json
repo-scout diff-impact --changed-file <file> --repo . --json
repo-scout verify-plan --changed-file <file> --repo . --json
```
