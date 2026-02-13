# Agent Workflows

This page defines a practical default policy for running `repo-scout` in AI-assisted coding loops.

## Agent-first discovery policy

When an agent is asked to work in a repository:

1. Index first:

```bash
repo-scout index --repo .
```

2. Resolve target symbols before broad file reads:

```bash
repo-scout find <symbol> --repo . --json
repo-scout refs <symbol> --repo . --json
```

3. Use structural commands for planning:

```bash
repo-scout impact <symbol> --repo . --json
repo-scout verify-plan --changed-file <path> --repo . --json
```

4. Re-run index + key queries after edits.

## Why this policy works

- reduces irrelevant file traversal
- gives deterministic artifacts for discussion and review
- keeps agent behavior auditable in command transcripts

## Prompt template (agent-agnostic)

```text
Use repo-scout as your first navigation tool in this repository.
Before reading many files, run:
- repo-scout index --repo .
- repo-scout find <target_symbol> --repo . --json
- repo-scout refs <target_symbol> --repo . --json
Use results to choose files.
After edits, rerun index/find/refs and then run tests.
```

## Tooling wrappers

Use `just` wrappers for repeatable local flows:

```bash
just dogfood-pre target_symbol .
# ... make changes ...
just dogfood-post target_symbol .
```

See platform-specific playbooks:

- [Codex Playbook](./agent-playbook-codex.md)
- [Claude Code Playbook](./agent-playbook-claude-code.md)
