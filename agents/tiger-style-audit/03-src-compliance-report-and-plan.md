# 03 - Source Code Compliance Report And Plan (`src/`)

## Quick Status

Green signals:

- No `unsafe` blocks in `src/`.
- No `todo!`/`unimplemented!` in `src/`.
- Baseline quality gates currently pass (`just check`).

Primary gaps are structural and contract-shape oriented, not immediate runtime breakage.

## Findings

### F-SRC-01 (P1): Multiple production functions exceed the 70-line hard limit

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:83-85`

Evidence (function start line, computed length):

- `src/indexer/languages/typescript.rs:44` (`extract`, 262 lines)
- `src/indexer/mod.rs:45` (`index_repository`, 257 lines)
- `src/indexer/languages/python.rs:44` (`extract`, 178 lines)
- `src/indexer/languages/python.rs:257` (`collect_call_symbols`, 143 lines)
- `src/store/schema.rs:31` (`bootstrap_schema`, 94 lines)
- `src/indexer/rust_ast.rs:41` (`extract_rust_items`, 93 lines)
- `src/indexer/languages/rust.rs:17` (`extract`, 79 lines)
- `src/indexer/languages/python.rs:486` (`import_bindings`, 77 lines)
- `src/indexer/mod.rs:407` (`resolve_symbol_id_in_tx`, 76 lines)

Why this matters:

- This concentrates parsing, SQL persistence, ranking, and edge resolution logic in large procedures, increasing regression risk and reducing review quality.

Required modification:

- Refactor each long function into smaller, single-responsibility helpers while preserving behavior with strict red-green-refactor slices.

### F-SRC-02 (P2): Monolithic orchestration violates architecture modularity intent

Contract reference:

- `contracts/core/ARCHITECTURE_CONTRACT.md:10-24`

Evidence:

- `src/indexer/mod.rs:45-301` combines file iteration, hashing decisions, DB deletes/inserts, symbol-id reuse, and deferred-edge replay in one flow.
- `src/query/mod.rs:165-424` combines changed-symbol seed extraction, bounded traversal, test-target attachment, filtering, sorting, and cap logic.
- `src/query/mod.rs:965-1089` combines keyword extraction, direct relevance scoring, neighbor expansion, filtering, sort, and budget cap.

Why this matters:

- Contract expects small composable units with explicit boundaries; current modules are operationally correct but structurally over-coupled.

Required modification:

- Introduce explicit submodules and pure helper layers (for ranking/scoring/filtering) to separate DB access from ranking logic.

### F-SRC-03 (P2): Recursion usage is only partially documented as approved

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:45`

Evidence:

- Recursive helpers:
  - `src/indexer/rust_ast.rs:179-201` (`collect_call_identifiers`)
  - `src/indexer/rust_ast.rs:340-352` (`last_identifier_text`)
  - `src/indexer/languages/typescript.rs:374-385` (`collect_type_identifiers`)
  - `src/indexer/languages/python.rs:257-399` (`collect_call_symbols`)
  - `src/indexer/languages/typescript.rs:389-508` (`collect_call_symbols`)
- Design-note approval exists for call-symbol recursion in phase 8:
  - `agents/repo-scout-phase8-execplan.md:112-116`
- No equivalent design-note evidence was found for `collect_call_identifiers`, `last_identifier_text`, or `collect_type_identifiers`.

Required modification:

- Either:
  - add explicit design-note approvals for all recursive helpers, or
  - convert remaining unapproved recursion to iterative traversal.

### F-SRC-04 (P2): Public API shape uses many behavior booleans where enums are preferred

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:124`

Evidence:

- CLI flags and option structs include behavior booleans:
  - `src/cli.rs:50-52`, `src/cli.rs:65-67`, `src/cli.rs:83-85`, `src/cli.rs:127-134`
- Query options mirror the same boolean pattern:
  - `src/query/mod.rs:146-153`, `src/query/mod.rs:1470-1489`

Why this matters:

- Multiple booleans create invalid/ambiguous state combinations and spread branch complexity.

Required modification:

- Replace behavior flag clusters with explicit enums or mode structs (for scope, test-target inclusion mode, import-seed mode).

### F-SRC-05 (P2): `usize` is exposed at API and serialized boundaries

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:89-91`

Evidence:

- CLI and query boundary fields:
  - `src/cli.rs:54`, `src/cli.rs:69`, `src/cli.rs:81`, `src/cli.rs:108`, `src/cli.rs:126`
  - `src/query/mod.rs:153`, `src/query/mod.rs:158`, `src/query/mod.rs:163`
- JSON output path accepts `usize` budget:
  - `src/output.rs:213`

Required modification:

- Standardize boundary-facing count/budget fields on fixed-width integers (`u32` or `u64`) and convert internally where needed.

### F-SRC-06 (P3): Contract expectations for invariant assertions and `#[must_use]` are under-applied

Contract reference:

- Assertions/invariants: `contracts/languages/RUST_CODING_CONTRACT.md:61-64`
- `#[must_use]`: `contracts/languages/RUST_CODING_CONTRACT.md:105`

Evidence:

- Production modules currently have few explicit runtime invariant assertions.
- No `#[must_use]` annotations detected in `src/` on public-returning APIs.

Required modification:

- Add targeted pre/postcondition checks in critical scoring/resolution boundaries.
- Add `#[must_use]` where ignored outputs could hide defects.

### F-SRC-07 (P3): Source line-length drift above 100 columns exists in key modules

Contract reference:

- `contracts/languages/RUST_CODING_CONTRACT.md:121`

Evidence:

- Over-100-column lines are concentrated in:
  - `src/indexer/mod.rs` (9)
  - `src/output.rs` (6)
  - `src/main.rs` (6)
  - `src/indexer/rust_ast.rs` (6)

Required modification:

- Reflow long SQL/doc/comment/println lines during refactor passes.

## Implementation Plan

### Phase A: Boundary and API-shape correction

1. Replace public `usize` boundary fields with fixed-width integer types.
2. Introduce explicit enums for behavior mode selection currently represented by boolean clusters.
3. Add conversion helpers at clap/query boundaries.

Acceptance:

- No `usize` in CLI/query/output public boundary fields.
- No invalid flag combinations represented as free booleans.

### Phase B: Function decomposition and boundary extraction

1. Split `index_repository` into staged helpers:
   - file state decision
   - per-file DB refresh
   - edge resolution pass
2. Split `diff_impact_for_changed_files`, `context_matches_scoped`, and `verify_plan_for_changed_files` into:
   - DB row acquisition
   - pure scoring/filtering
   - deterministic sort/cap stage
3. Split adapter `extract` flows by concern (definition extraction, call/reference extraction, import-edge extraction, normalization/dedup).

Acceptance:

- All production functions are <= 70 lines.
- Integration tests remain green with unchanged behavior.

### Phase C: Recursion governance and contract hardening

1. For each recursive helper, choose and document one path:
   - approved-by-design-note, or
   - iterative rewrite.
2. Add targeted invariant checks and `#[must_use]` annotations where meaningful.
3. Reflow over-100-column lines during refactor-only commits.

Acceptance:

- Recursion is either documented or removed.
- Critical invariants are asserted in hot logic boundaries.
- Source line-length drift is eliminated.

