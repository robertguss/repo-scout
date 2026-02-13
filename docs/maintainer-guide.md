# Maintainer Guide

## Branching and release hygiene

- Keep changelog current in Keep a Changelog format.
- Use SemVer-compatible release tags.
- Ensure contract validators pass before merge.

## Required quality gates

- formatting: `cargo fmt -- --check`
- linting: `cargo clippy --all-targets --all-features -- -D warnings`
- tests: `cargo test`
- contract validators:
  - `scripts/validate_tdd_cycle.sh`
  - `scripts/validate_evidence_packet.sh`

## Docs governance

- Build docs before release:

```bash
just docs-build
```

- Run docs checks:

```bash
just docs-check
```

## Keeping agent docs accurate

When command flags or output fields change:

1. update `docs/cli-reference.md`
2. update `docs/json-output.md`
3. update `README.md` quickstart examples
4. rerun docs checks
