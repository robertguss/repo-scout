# 05 - Process, CI, And Documentation Compliance Report And Plan

## Quick Status

- Local engineering quality gates are healthy (`just check` passes).
- Contract-gate automation exists and runs in CI.
- Major compliance gaps are in enforcement depth and historical/process consistency.

## Findings

### F-PROC-01 (P1): CI workflow does not enforce Rust contract required gates

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:154-166`

Evidence:

- `.github/workflows/contract-gates.yml:33-65` runs only shell lint + TDD/evidence validators.
- It does not run required language gates:
  - `cargo fmt --all -- --check`
  - strict `cargo clippy ... -D clippy::unwrap_used -D clippy::expect_used -D clippy::undocumented_unsafe_blocks`
  - `cargo test --workspace --all-features`

Required modification:

- Extend CI workflow with explicit Rust contract gate steps (or add a dedicated quality-gates workflow).

### F-PROC-02 (P1): TDD validator under-enforces contract claims

Contract references:

- `contracts/core/TDD_ENFORCEMENT_CONTRACT.md`
- `contracts/languages/RUST_CODING_CONTRACT.md:114-116`

Evidence:

- `scripts/validate_tdd_cycle.sh:60-63` returns success when no commits are in range.
- `scripts/validate_tdd_cycle.sh:84-131` only requires at least one RED/GREEN/REFACTOR in total range, not per feature slice.
- It validates commit subject prefixes, but not whether a failing test was observed before implementation.

Required modification:

- Tighten validator semantics to support per-slice evidence checks and stronger red-first guarantees.

### F-PROC-03 (P1): Evidence validator checks headings only, not evidence quality requirements

Contract reference:

- `contracts/core/EVIDENCE_REQUIREMENTS.md:31-63`

Evidence:

- `scripts/validate_evidence_packet.sh:63-98` verifies required headings and placeholder tokens only.
- It does not validate minimum detail quality for Red/Green/Refactor sections (commands, failure summary, expected reason, etc.).

Required modification:

- Add semantic checks for required section content quality, not only heading existence.

### F-PROC-04 (P1): Historical commit taxonomy is largely non-compliant

Contract reference:

- `contracts/core/TDD_ENFORCEMENT_CONTRACT.md:16-28`

Evidence:

- Full-history prefix scan result: `ok=3`, `bad=77`.
- Example failing commit subject from full-history validation:
  - `Add initial cargo project files and planning docs`

Required modification:

- Decide explicit policy for pre-adoption history:
  - do not retroactively enforce before adoption commit, or
  - rewrite history if that is acceptable.
- Document that policy in process docs and validator invocation guidance.

### F-PROC-05 (P2): ExecPlan contract integration is inconsistent across phases

Contract references:

- `agents/PLANS.md` (Contract Inputs, AGENTS constraints, risk-tier sections)

Evidence:

- `agents/repo-scout-phase1-execplan.md` through `agents/repo-scout-phase8-execplan.md` do not include the full contract integration section set.
- `agents/repo-scout-phase9-execplan.md` includes:
  - `## Contract Inputs`
  - `## AGENTS.md Constraints`
  - `## Risk Tier and Required Controls`

Required modification:

- Either:
  - backfill prior phase plans with a conformance addendum, or
  - mark them explicitly as pre-Tiger-adoption artifacts and exclude them from conformance expectations.

### F-PROC-06 (P2): Required task/test/evidence artifact workflow is not concretely materialized in repo artifacts

Contract references:

- `contracts/core/AI_AGENT_CORE_CONTRACT.md:81-97`

Evidence:

- Templates exist in `templates/`, but repository-level examples/artifacts are not consistently materialized.
- `.evidence/` directory is currently missing.
- Process relies on PR body headings, but checklist/test-plan artifact completion is not automated.

Required modification:

- Establish and document one canonical artifact strategy:
  - PR-body-only with strict checklist attestation and validator support, or
  - committed artifact files per change (`.evidence/`, task packet, test plan).

### F-PROC-07 (P2): PR template drift weakens checklist and exception signaling

Evidence:

- `.github/pull_request_template.md:74-94` includes dogfooding/docs checks only.
- Upstream-style checklist completion and exception attestation are not present.

Required modification:

- Merge checklist + dogfooding + docs/plans expectations into one comprehensive PR template.

## Implementation Plan

### Phase A: CI and validator hardening

1. Add Rust quality gates to CI workflow.
2. Extend TDD validator for stronger sequencing semantics.
3. Extend evidence validator for required content-depth checks.

Acceptance:

- CI blocks on contract-required Rust quality gates and evidence quality.

### Phase B: Process policy normalization

1. Define adoption-boundary policy for historical commit prefix compliance.
2. Define canonical artifact policy (PR body vs committed files) and encode in docs.
3. Reconcile PR template to include both checklist and repo-specific dogfooding obligations.

Acceptance:

- Policy is explicit and actionable; no hidden process assumptions remain.

### Phase C: Planning-document conformance pass

1. Add a short conformance note to phase1-8 ExecPlans or mark them as pre-adoption legacy artifacts.
2. Ensure future ExecPlans include contract/risk/AGENTS integration sections by default.

Acceptance:

- Planning artifacts are internally consistent with Tiger conventions for future work.

