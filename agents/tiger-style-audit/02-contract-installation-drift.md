# 02 - Contract Installation Drift

## Summary

The Tiger-style installation is mostly in sync with upstream. Drift is limited to language-contract coverage choices and PR template customization.

## Findings

### F-DRIFT-01 (P2): Missing Python and TypeScript language contract files relative to upstream package

Evidence:

- Missing in `repo-scout` relative to upstream snapshot:
  - `contracts/languages/PYTHON_CODING_CONTRACT.md`
  - `contracts/languages/TYPESCRIPT_CODING_CONTRACT.md`
- Present and active locally:
  - `contracts/languages/RUST_CODING_CONTRACT.md`
  - `AGENTS.md:93` explicitly declares Rust as active language contract.

Why this matters:

- It is ambiguous whether this is an intentional scoping choice or partial installation drift.
- Without explicit policy, future maintainers may assume multi-language contract coverage that is not actually installed.

Required modification:

- Decide and document one explicit posture:
  - Option A: Rust-only contract scope for this repository.
  - Option B: Keep all upstream language contracts installed for consistency.
- Record this decision in `AGENTS.md` and repository docs so enforcement expectations are unambiguous.

### F-DRIFT-02 (P1): Local PR template diverges from upstream checklist expectations

Evidence:

- Upstream drift detected only in `.github/pull_request_template.md`.
- Local template includes dogfooding/docs sections (`.github/pull_request_template.md:74-94`) but no explicit checklist items for:
  - completed `checklists/PR_CONTRACT_CHECKLIST.md`
  - completed `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` when tier requires
  - exception documentation/approval

Why this matters:

- Core contract outputs require checklist completion (`contracts/core/AI_AGENT_CORE_CONTRACT.md:94-97`).
- Template no longer prompts those outputs directly, increasing process miss risk.

Required modification:

- Reintroduce checklist/exception prompts in PR template while preserving dogfooding additions.
- Keep one merged template instead of forking intent between upstream and local conventions.

## Implementation Plan

### Step 1: Freeze installation intent

- Add a short “Contract Installation Policy” section to `AGENTS.md` and `README.md` defining language-contract scope.

### Step 2: Normalize PR template

- Update `.github/pull_request_template.md` to include:
  - Tiger checklist completion attestation
  - adversarial checklist condition
  - exception documentation attestation
  - existing dogfood/docs plan checks

### Step 3: Add drift check automation

- Add a lightweight script (or just target) that compares local contract assets against upstream source path and reports:
  - missing files
  - modified files

Acceptance criteria:

- Contract asset policy is explicit in docs.
- PR template covers both Tiger checklist expectations and repo-scout dogfooding expectations.
- Drift status is machine-checkable.

