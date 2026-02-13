# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New mdBook-first documentation architecture with dedicated sections for quickstart, agent workflows, contributor guidance, maintainer operations, and troubleshooting.
- Dedicated agent playbooks for Codex and Claude Code with reusable instruction templates.
- New docs pages:
  - `docs/introduction.md`
  - `docs/quickstart.md`
  - `docs/agent-workflows.md`
  - `docs/agent-playbook-codex.md`
  - `docs/agent-playbook-claude-code.md`
  - `docs/contributing.md`
  - `docs/maintainer-guide.md`
  - `docs/release-process.md`
  - `docs/troubleshooting.md`
  - `legacy gates doc (removed)`

### Changed

- `README.md` rewritten to reflect current command surface and to prioritize first-time onboarding plus agent-driven usage.
- `docs/cli-reference.md`, `docs/json-output.md`, and `docs/architecture.md` rewritten for current behavior and terminology.
- `docs/SUMMARY.md` reorganized into a user-first navigation model with explicit legacy sectioning.
- `Justfile` modernized with a cleaner workflow surface, docs automation commands, and reorganized gate recipes.

### Removed

- Obsolete planning and report docs removed from `docs/`:
  - `docs/plans/` (entire directory)
  - `docs/claude-dogfood-report.md`
  - `docs/dogfood-log.md`
  - `docs/phase7-semantic-precision.md`

## [0.1.0] - 2026-02-10

### Added

- First production-ready release of `repo-scout`.
- Production-ready language support for Rust, Go, Python, and TypeScript.
- Cross-language convergence fixtures and validation pack in `tests/fixtures/phase15/convergence_pack`.
- Phase 16 hardening gates:
  - deterministic replay
  - benchmark pack
  - known-issues budget
  - large-repo benchmark
  - release checklist
  - large-repo replay

### Changed

- Documentation and operator workflows consolidated for production/GA posture.
- Known-issues closure posture tightened to zero deferred issues for Phase 16.

### Quality

- Integration suite, formatting/linting, TDD cycle validation, and evidence validation passed at release cut.

[Unreleased]: https://github.com/robertguss/repo-scout/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/robertguss/repo-scout/releases/tag/v0.1.0
