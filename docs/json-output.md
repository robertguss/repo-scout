# JSON Output

Use `--json` to get machine-readable command output.

## Design goals

- deterministic field names and ordering
- explicit command identity
- explicit schema IDs for stable integrations
- explicit success/error envelopes for agents

## Agent-first envelope

Phase 20 commands (`status`, `find`, `refs`, `schema`, `resolve`, `refactor-plan`) use:

- `schema` (for example `repo-scout/find@v1`)
- `command`
- `ok` (boolean)
- `meta` (repo/index lifecycle context)
- `data` (command payload)

Example:

```json
{
  "schema": "repo-scout/find@v1",
  "command": "find",
  "ok": true,
  "meta": {
    "repo": ".",
    "index": {
      "schema_version": 4,
      "indexed_at": "1739442375123",
      "head_sha": "abc123",
      "stale": false
    }
  },
  "data": {
    "query": "run",
    "results": []
  }
}
```

Error envelope:

```json
{
  "schema": "repo-scout/error@v1",
  "command": "find",
  "ok": false,
  "error": {
    "code": "INDEX_STALE",
    "message": "Index is stale relative to repository files",
    "details": {
      "suggested_fix": "repo-scout index --repo ."
    }
  }
}
```

## Integration guidance

- Parse by field names, not by line formatting.
- Validate `schema` in your automation.
- Treat unknown fields as forward-compatible additions.

## Recommended agent usage

Prefer JSON mode when agents consume `repo-scout` output:

```bash
repo-scout find <symbol> --repo . --json
repo-scout refs <symbol> --repo . --json
repo-scout schema --repo . --json
repo-scout resolve <symbol> --repo . --json
repo-scout refactor-plan <target> --repo . --json
repo-scout diff-impact --changed-file <file> --repo . --json
repo-scout verify-plan --changed-file <file> --repo . --json
```
