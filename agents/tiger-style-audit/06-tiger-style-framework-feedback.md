# 06 - Tiger Style Framework Feedback (Upstream Fix Candidates)

Target upstream repository:

- `/Users/robertguss/Projects/programming_tiger_style`

These issues were discovered while applying Tiger style to `repo-scout`. They are framework-level concerns, not only local repo issues.

## Framework Issues

### TS-01 (P1): Contract claims and CI workflow baseline are inconsistent

Evidence:

- Rust contract requires CI to run strict Rust gates (`contracts/languages/RUST_CODING_CONTRACT.md:154-166`).
- Distributed baseline workflow (`.github/workflows/contract-gates.yml`) does not run those language gates; it runs only script validators.

Risk:

- Repositories can appear contract-compliant while not enforcing required language quality gates in CI.

Recommended upstream fix:

- Update baseline workflow template to include language-agnostic core checks plus language-specific gates.
- Provide a modular workflow pattern (core + rust/python/typescript job snippets).

### TS-02 (P1): Required inputs/outputs in AI core contract conflict with tier table optionality

Evidence:

- AI core contract says each task must provide task packet and test plan (`contracts/core/AI_AGENT_CORE_CONTRACT.md:81-97`).
- Risk tier table says task packet/test plan optional for Tier 0 (`contracts/core/RISK_TIER_POLICY.md:21-23`).

Risk:

- Teams cannot tell whether Tier 0 can skip those artifacts.

Recommended upstream fix:

- Choose one canonical rule and align all docs:
  - either always required, or
  - tier-gated with explicit exception text in AI core contract.

### TS-03 (P1): Validator scripts under-enforce stated contract rigor

Evidence:

- TDD validator checks commit-prefix sequencing but not red-failure evidence semantics.
- Evidence validator checks heading presence and placeholders only, not required detail quality.

Risk:

- “Compliant” signals can be produced without meeting contract intent.

Recommended upstream fix:

- Add semantic validation rules:
  - per-slice red/green/refactor proof requirements
  - section content checks for commands and failure/pass summaries
  - checklist attestation checks

### TS-04 (P2): Recursion prohibition is operationally rigid for parser/AST-heavy codebases

Evidence:

- Rust contract blanket rule: no recursion unless design-note approved (`contracts/languages/RUST_CODING_CONTRACT.md:45`).
- Real-world AST traversal commonly uses recursion and can remain safe with clear bounds.

Risk:

- Teams may create noisy “approval churn” or awkward iterative rewrites with lower readability.

Recommended upstream fix:

- Reframe rule to “recursion requires explicit boundedness and rationale,” with examples of acceptable recursive traversal patterns.

### TS-05 (P2): 70-line hard limit and 100-column strictness are valuable but need practical implementation guidance

Evidence:

- Hard limits exist (`contracts/languages/RUST_CODING_CONTRACT.md:83`, `:121`) but template guidance does not include migration patterns for legacy code.

Risk:

- First-time adopters get high-volume violations without a staged adoption model.

Recommended upstream fix:

- Add “legacy adoption mode” guidance:
  - enforce on touched/new code first
  - ratchet thresholds over milestones
  - provide refactor playbook patterns

### TS-06 (P2): Language-contract activation model is not explicit for partial installs

Evidence:

- Adoption can copy full `contracts/languages/`, but downstream repos may intentionally keep a subset.
- No manifest pattern defines active language contracts explicitly.

Risk:

- Ambiguous expectations for mixed repos and future maintainers.

Recommended upstream fix:

- Introduce an explicit activation manifest (for example `contracts/ACTIVE_LANGUAGE_CONTRACTS.md`) referenced by AGENTS template.

### TS-07 (P3): Test-scope wording around `unwrap/expect` is ambiguous

Evidence:

- Scope includes tests/tooling, but `unwrap/expect/panic` prohibition is stated for production paths.

Risk:

- Reviewers and teams may enforce inconsistent standards for test code.

Recommended upstream fix:

- Add explicit “test code allowances” subsection with allowed and disallowed patterns.

## Suggested Upstream Execution Order

1. Align contract text conflicts (TS-01, TS-02, TS-03).
2. Publish adoption-mode guidance and explicit language activation model (TS-05, TS-06).
3. Clarify recursion/test allowances with concrete examples (TS-04, TS-07).

