# Phase 2 Fixtures

These fixtures are pre-implementation scaffolding for Phase 2 milestone tests.
They intentionally capture representative source patterns for lifecycle cleanup,
richer Rust symbol extraction, graph edges, and test recommendation heuristics.

Layout:

- `lifecycle/delete_case/`: file deletion scenarios.
- `lifecycle/rename_case/`: file rename scenarios.
- `rust_symbols/`: Rust syntax shapes for richer AST extraction.
- `graph/`: simple call and containment relationships.
- `validation/`: code + tests layout for `tests-for` and `verify-plan` planning.
