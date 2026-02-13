# repo-scout

`repo-scout` is a local CLI for repository indexing, code navigation, and structural analysis.
It is built for agent-driven workflows (Codex, Claude Code) and for human maintainers who need deterministic command output.
Phase 16 High-Bar/GA hardening is complete.

## Why repo-scout

- Fast local index in `<repo>/.repo-scout/index.db`
- Symbol search and reference lookup across Rust, Go, Python, and TypeScript
- Impact analysis and verification planning for code changes
- Deterministic terminal and JSON output for automation and CI use

## Install

Prerequisites:

- Rust stable toolchain

Build from source:

```bash
cargo build --release
```

Run from source during development:

```bash
cargo run -- --help
```

## 5-minute quickstart

Index a repository and run core navigation commands:

```bash
# 1) index
cargo run -- index --repo /path/to/target-repo

# 2) find symbol definitions
cargo run -- find main --repo /path/to/target-repo

# 3) find symbol references
cargo run -- refs main --repo /path/to/target-repo

# 4) inspect blast radius
cargo run -- impact main --repo /path/to/target-repo

# 5) machine-readable output
cargo run -- refs main --repo /path/to/target-repo --json
```

## Quickstart for AI-agent workflows

If you want an agent to use `repo-scout` first (before broad file scanning), use a policy like:

```text
When exploring or editing code in this repository:
1. Run `repo-scout index --repo .`
2. Run `repo-scout find <symbol> --repo . --json`
3. Run `repo-scout refs <symbol> --repo . --json`
4. Use results to choose files, then read/edit only those files
5. After changes, rerun index/find/refs and tests
```

Detailed playbooks:

- Codex: `docs/agent-playbook-codex.md`
- Claude Code: `docs/agent-playbook-claude-code.md`
- Agent-agnostic workflow: `docs/agent-workflows.md`

## Commands

Primary commands:

- `index`, `status`
- `find`, `refs`, `impact`, `context`, `tests-for`
- `verify-plan`, `diff-impact`, `explain`, `snippet`
- `summary`, `outline`, `tree`, `orient`, `health`
- `callers`, `callees`, `call-path`, `related`, `deps`, `hotspots`, `circular`

See full command reference in `docs/cli-reference.md`.

## Documentation

This repository uses mdBook docs in `docs/`.

- Docs entry: `docs/introduction.md`
- Summary/nav: `docs/SUMMARY.md`
- Build locally: `just docs-build`

## Contributor workflow

Minimum local checks:

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Contract/TDD validation:

```bash
bash scripts/validate_tdd_cycle.sh --base origin/main
bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md
```

## Contract system and governance

This repository follows Contract System v2 with strict TDD/evidence policy.

- Core contracts: `contracts/core/`
- Active language contract: `contracts/languages/RUST_CODING_CONTRACT.md`
- Templates: `templates/`
- Checklists: `checklists/`
- Validators: `scripts/validate_tdd_cycle.sh`, `scripts/validate_evidence_packet.sh`

## Versioning and changelog

- Versioning: Semantic Versioning
- Changelog format: Keep a Changelog
- Current package version: `0.1.0` (`Cargo.toml`)
- Changelog: `CHANGELOG.md`
