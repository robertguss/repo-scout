# 04 - Tests Compliance Report And Plan (`tests/`)

## Quick Status

Green signals:

- Test suite passes currently (`cargo test`).
- Naming convention mostly aligns with milestone pattern in `AGENTS.md`.
- Integration-style end-to-end behavior coverage is strong.

Main gaps are style-contract alignment and test ergonomics under Tiger rules.

## Findings

### F-TEST-01 (P2): Multiple test functions exceed 70-line function limit

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:83-85`

Evidence (function start line, computed length):

- `tests/milestone27_context_scope.rs:184` (102 lines)
- `tests/milestone16_python.rs:160` (101 lines)
- `tests/milestone23_verify_plan_precision.rs:63` (99 lines)
- `tests/milestone12_diff_impact.rs:179` (87 lines)
- `tests/milestone30_query_focus.rs:36` (77 lines)
- `tests/milestone14_adapter.rs:15` (71 lines)
- `tests/milestone30_query_focus.rs:115` (71 lines)

Required modification:

- Decompose large tests into smaller scenario-focused test cases with shared helper setup.

### F-TEST-02 (P3): Significant line-length drift over 100 columns in tests

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:121`

Evidence:

- Over-100-column line counts are high in multiple files, including:
  - `tests/milestone27_context_scope.rs` (10)
  - `tests/milestone7_rust_symbols.rs` (5)
  - `tests/milestone6_schema_migration.rs` (4)
  - `tests/milestone6_lifecycle.rs` (4)

Required modification:

- Reflow long assertions/fixture literals and extract helper constants.

### F-TEST-03 (P2): Heavy `unwrap`/`expect` usage in tests creates policy ambiguity under installed contract language

Contract references:

- Scope includes tests/tooling: `contracts/languages/RUST_CODING_CONTRACT.md:24-26`
- Error rule is production-only wording: `contracts/languages/RUST_CODING_CONTRACT.md:68-72`

Evidence:

- `unwrap`/`expect` occurrences in `tests/`: 543 total.
- Representative examples:
  - `tests/milestone27_context_scope.rs:29`
  - `tests/milestone28_verify_plan_scope.rs:25`
  - `tests/milestone23_verify_plan_precision.rs:40`

Why this matters:

- Current contract wording is ambiguous about strictness for test code.
- If interpreted strictly for all Rust scope, this repository is far from compliant.
- If interpreted as production-only, current pattern is acceptable but should be explicitly documented.

Required modification:

- Define explicit repository policy in `AGENTS.md`:
  - whether `unwrap`/`expect` is allowed in tests,
  - and if allowed, in what limited patterns.

### F-TEST-04 (P3): Test helper panic usage should be explicitly policy-approved

Evidence:

- `tests/common/mod.rs:37` uses `panic!` for terminal failure in wait-loop behavior.

Required modification:

- Either keep with explicit test-policy exemption, or replace with richer error propagation in helper APIs.

## Implementation Plan

### Phase A: Clarify contract interpretation for tests

1. Add a short test-policy section to `AGENTS.md` clarifying:
   - allowed assertion/error idioms in tests,
   - whether test `unwrap`/`expect` is accepted,
   - any required alternatives.

Acceptance:

- Test policy is unambiguous and version-controlled.

### Phase B: Structural cleanup of oversized tests

1. Split each >70-line test into focused scenarios.
2. Move repeated fixture setup into `tests/common/mod.rs` helper functions.
3. Keep deterministic ordering and current contract behavior unchanged.

Acceptance:

- No test functions exceed 70 lines.
- Full test suite remains green.

### Phase C: Style normalization pass

1. Reflow lines >100 columns in test files.
2. Introduce concise helper variables/constants for long command arrays and assertion chains.

Acceptance:

- Tests comply with 100-column line-length requirement.

