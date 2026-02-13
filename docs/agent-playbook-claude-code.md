# Claude Code Playbook

This playbook is for teams using Claude Code agents.

## Recommended instruction fragment

```text
Use repo-scout as the first repository navigation layer.
Before large reads or edits, run:
- repo-scout index --repo .
- repo-scout find <target_symbol> --repo . --json
- repo-scout refs <target_symbol> --repo . --json
Use those results to scope reads and edits.
After edits, rerun index/find/refs and run tests.
```

## Suggested command flow

```bash
repo-scout index --repo .
repo-scout find <symbol> --repo . --json
repo-scout refs <symbol> --repo . --json
repo-scout impact <symbol> --repo . --json
repo-scout verify-plan --changed-file <file> --repo . --json
```

## Review loop

- capture query outputs in PR notes
- include pre/post query diffs for risky changes
- keep command transcripts deterministic and reproducible
