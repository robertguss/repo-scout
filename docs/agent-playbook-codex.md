# Codex Playbook

This playbook is for teams using Codex-style coding agents.

## Recommended system instruction fragment

```text
In this repository, use repo-scout before broad file scanning.
Run:
1) repo-scout index --repo .
2) repo-scout schema --repo . --json
3) repo-scout find <target_symbol> --repo . --json
4) repo-scout refs <target_symbol> --repo . --json
Then inspect only the files returned by those commands.
After code edits, rerun index/find/refs and run tests.
```

## Task loop

1. Identify target symbol or changed file.
2. Run `schema` once to pin command contracts.
3. Run `find` and `refs` in JSON mode.
4. If symbol is ambiguous, run `resolve`.
5. Read only relevant files.
6. Edit and validate.
7. Re-index and rerun targeted queries.
8. For major edits, run `refactor-plan`.
9. Run repository tests.

## Useful command set

```bash
repo-scout index --repo .
repo-scout schema --repo . --json
repo-scout find <symbol> --repo . --json
repo-scout refs <symbol> --repo . --json
repo-scout resolve <symbol> --repo . --json
repo-scout query --repo . --format jsonl --input <batch.jsonl>
repo-scout refactor-plan <target> --repo . --json
repo-scout diff-impact --changed-file <file> --repo . --json
repo-scout verify-plan --changed-file <file> --repo . --json
```
