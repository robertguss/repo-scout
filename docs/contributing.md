# Contributing

## Development prerequisites

- Rust stable
- `just` (recommended)

## Standard local loop

```bash
just build
just fmt-check
just clippy
just test
```

## Strict TDD and evidence policy

Production changes follow Red -> Green -> Refactor.

Required validators before PR:

```bash
bash scripts/validate_tdd_cycle.sh --base origin/main
bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md
```

## Risk tiers

Declare a risk tier before implementation (`0 | 1 | 2 | 3`).

Reference policy:

- `contracts/core/RISK_TIER_POLICY.md`

For Tier 2/3 work, complete:

- `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`

## Dogfooding requirement

When implementing feature slices in this repository:

Pre-implementation:

```bash
cargo run -- index --repo .
cargo run -- find <target_symbol> --repo . --json
cargo run -- refs <target_symbol> --repo . --json
```

Post-implementation:

```bash
cargo run -- index --repo .
cargo run -- find <target_symbol> --repo .
cargo run -- refs <target_symbol> --repo .
cargo test
```
