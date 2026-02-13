# Quickstart Tutorial

This tutorial is optimized for first-time users and agent operators.

## Prerequisites

- Rust stable installed
- local clone of `repo-scout`
- a target repository to analyze

## Step 1: build and inspect CLI

```bash
cargo build
cargo run -- --help
```

## Step 2: index a repository

```bash
cargo run -- index --repo /path/to/target-repo
```

Expected output includes fields like:

- `index_path`
- `schema_version`
- `indexed_files`

## Step 3: run core symbol navigation

```bash
cargo run -- find main --repo /path/to/target-repo
cargo run -- refs main --repo /path/to/target-repo
cargo run -- impact main --repo /path/to/target-repo
```

## Step 4: get JSON for automation

```bash
cargo run -- find main --repo /path/to/target-repo --json
cargo run -- refs main --repo /path/to/target-repo --json
```

## Step 5: use planning commands for change work

```bash
cargo run -- verify-plan --changed-file src/lib.rs --repo /path/to/target-repo
cargo run -- diff-impact --changed-file src/lib.rs --repo /path/to/target-repo
```

## Step 6: orientation and structure commands

```bash
cargo run -- orient --repo /path/to/target-repo
cargo run -- summary --repo /path/to/target-repo
cargo run -- tree --repo /path/to/target-repo
```

## Suggested workflow for edits

1. `index`
2. `find` + `refs` for target symbols
3. edit code
4. `index` again
5. `impact` / `diff-impact` / `verify-plan`
6. run tests

## Next

- [Agent Workflows](./agent-workflows.md)
- [CLI Reference](./cli-reference.md)
