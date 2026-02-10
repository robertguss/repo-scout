# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

### Added
- Phase 17 documentation-consistency planning artifact:
  `agents/plans/repo-scout-phase17-execplan.md`.
- Milestone coverage for docs consistency gates:
  `tests/milestone71_docs_status_alignment.rs`,
  `tests/milestone72_docs_consistency_gate.rs`.
- Documentation consistency automation surface:
  `scripts/check_docs_consistency.sh`, `just docs-consistency`, and CI wiring in
  `.github/workflows/contract-gates.yml`.

### Changed
- Documentation truth-sync updates for current post-Phase-16 closure posture in:
  `README.md`, `docs/architecture.md`, `agents/plans/repo-scout-phase9-execplan.md`,
  `agents/plans/README.md`, and `docs/dogfood-log.md`.

## [0.1.0] - 2026-02-10

First production-ready release of `repo-scout`.

### Added
- Production-ready language support for Rust, Go, Python, and TypeScript.
- Cross-language convergence fixtures and validation pack under `tests/fixtures/phase15/convergence_pack`.
- Phase 16 High-Bar/GA hardening gates:
  - `scripts/check_phase16_deterministic_replay.sh`
  - `scripts/check_phase16_benchmark_pack.sh`
  - `scripts/check_phase16_known_issues_budget.sh`
  - `scripts/check_phase16_large_repo_benchmark.sh`
  - `scripts/check_phase16_release_checklist.sh`
  - `scripts/check_phase16_large_repo_replay.sh`
- `just` workflows for convergence and GA hardening checks.
- Phase-specific execplans and roadmap closure artifacts for phases 10 through 16.

### Changed
- Documentation and operator workflows were consolidated for production/GA posture.
- Known-issues closure posture tightened to zero deferred issues for Phase 16.

### Quality
- Full integration suite, clippy, formatting, TDD cycle validator, and evidence validator are green at release cut.

[Unreleased]: https://github.com/robertguss/repo-scout/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/robertguss/repo-scout/releases/tag/v0.1.0
