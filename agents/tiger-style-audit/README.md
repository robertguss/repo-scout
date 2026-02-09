# Tiger Style Audit Index (2026-02-09)

## Purpose

This folder contains an audit-only deep-dive of `repo-scout` against the Tiger style contracts and conventions that are now installed in this repository.

This audit does not implement code changes. It documents gaps and an implementation plan for a follow-up execution session.

## Source of Truth Used

- Installed Tiger assets inside this repository:
  - `contracts/`
  - `templates/`
  - `checklists/`
  - `scripts/`
  - `.github/workflows/contract-gates.yml`
  - `.github/pull_request_template.md`
  - `AGENTS.md`
- Upstream reference repository for drift and convention intent:
  - `/Users/robertguss/Projects/programming_tiger_style`

## Audit Files

1. `agents/tiger-style-audit/01-source-of-truth-and-method.md`
2. `agents/tiger-style-audit/02-contract-installation-drift.md`
3. `agents/tiger-style-audit/03-src-compliance-report-and-plan.md`
4. `agents/tiger-style-audit/04-tests-compliance-report-and-plan.md`
5. `agents/tiger-style-audit/05-process-ci-docs-compliance-report-and-plan.md`
6. `agents/tiger-style-audit/06-tiger-style-framework-feedback.md`
7. `agents/tiger-style-audit/07-appendix-evidence-snapshots.md`

## Baseline Validation Snapshot

- `just check`: pass (fmt/clippy/tests green)
- `just contract-check`: pass for `origin/main..HEAD` (no commits in range, evidence headings pass)
- `bash scripts/validate_tdd_cycle.sh --base <root-commit>`: fail (historical commit subject prefix non-compliance)

## Top Priority Themes

- Large-function and monolithic-flow violations versus the 70-line Rust contract limit.
- Recursion usage that is only partially design-approved.
- Public API option-shape drift from contract guidance (`bool` flags and `usize` at boundaries).
- Process enforcement gaps: CI gate and validators currently under-enforce Tiger requirements.
- Planning/documentation artifacts before Tiger adoption are structurally inconsistent with the new contract integration expectations.
