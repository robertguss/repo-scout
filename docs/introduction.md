# Introduction

`repo-scout` is a local-first repository intelligence CLI.

It builds an index for a target repository and answers deterministic code-navigation questions:

- Where is a symbol defined?
- Where is it referenced?
- What is the likely blast radius of a change?
- Which tests should run after a change?

## Who this is for

- Engineers doing code review, refactors, and incident response
- Maintainers who need consistent local and CI checks
- Humans orchestrating AI agents that should inspect code with precision instead of broad scanning

## Core model

1. Build index: `repo-scout index --repo <path>`
2. Query symbols/structure (`find`, `refs`, `impact`, `orient`, etc.)
3. Use JSON output for automation
4. Re-index after edits and rerun critical queries

## Language support

Current parser/indexing focus includes:

- Rust
- Go
- Python
- TypeScript

Text fallback still supports generic file content where AST metadata is unavailable.

## Determinism

`repo-scout` prioritizes deterministic behavior:

- stable ranking for query results
- explicit JSON schema versioning by command family
- reproducible command outputs for scripted workflows

Continue with the hands-on tutorial in [Quickstart Tutorial](./quickstart.md).
