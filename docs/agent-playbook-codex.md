# Codex Playbook

This playbook is for teams using Codex-style coding agents.

## Recommended system instruction fragment

```text
In this repository, use repo-scout before broad file scanning.
Run:
1) repo-scout index --repo .
2) repo-scout find <target_symbol> --repo . --json
3) repo-scout refs <target_symbol> --repo . --json
Then inspect only the files returned by those commands.
After code edits, rerun index/find/refs and run tests.
```

## Task loop

1. Identify target symbol or changed file.
2. Run `find` and `refs` in JSON mode.
3. Read only relevant files.
4. Edit and validate.
5. Re-index and rerun targeted queries.
6. Run repository tests.

## Useful command set

```bash
repo-scout index --repo .
repo-scout find <symbol> --repo . --json
repo-scout refs <symbol> --repo . --json
repo-scout diff-impact --changed-file <file> --repo . --json
repo-scout verify-plan --changed-file <file> --repo . --json
```
