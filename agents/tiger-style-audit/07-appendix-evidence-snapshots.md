# 07 - Appendix: Evidence Snapshots

## Command Outcomes

### Quality Baseline

- `just check`: pass
- `just contract-check`: pass for `origin/main..HEAD` (no commits in range)

### Full-History Prefix Audit

- `bash scripts/validate_tdd_cycle.sh --base <root-commit>`: fail
- First failing subject observed: `Add initial cargo project files and planning docs`
- Full-history scan summary: `ok=3`, `bad=77`

### Contract Asset Drift

Compared upstream Tiger source to local install set:

- Missing local files:
  - `contracts/languages/PYTHON_CODING_CONTRACT.md`
  - `contracts/languages/TYPESCRIPT_CODING_CONTRACT.md`
- Modified local file:
  - `.github/pull_request_template.md`

## Structural Metrics Snapshot

### Production Functions >70 Lines

- `src/indexer/languages/typescript.rs:44` (`extract`, 262)
- `src/indexer/mod.rs:45` (`index_repository`, 257)
- `src/indexer/languages/python.rs:44` (`extract`, 178)
- `src/indexer/languages/python.rs:257` (`collect_call_symbols`, 143)
- `src/store/schema.rs:31` (`bootstrap_schema`, 94)
- `src/indexer/rust_ast.rs:41` (`extract_rust_items`, 93)
- `src/indexer/languages/rust.rs:17` (`extract`, 79)
- `src/indexer/languages/python.rs:486` (`import_bindings`, 77)
- `src/indexer/mod.rs:407` (`resolve_symbol_id_in_tx`, 76)

### Test Functions >70 Lines

- `tests/milestone27_context_scope.rs:184` (102)
- `tests/milestone16_python.rs:160` (101)
- `tests/milestone23_verify_plan_precision.rs:63` (99)
- `tests/milestone12_diff_impact.rs:179` (87)
- `tests/milestone30_query_focus.rs:36` (77)
- `tests/milestone14_adapter.rs:15` (71)
- `tests/milestone30_query_focus.rs:115` (71)

### Recursion Sites

- `src/indexer/rust_ast.rs:179-201` (`collect_call_identifiers`)
- `src/indexer/rust_ast.rs:340-352` (`last_identifier_text`)
- `src/indexer/languages/python.rs:257-399` (`collect_call_symbols`)
- `src/indexer/languages/typescript.rs:374-385` (`collect_type_identifiers`)
- `src/indexer/languages/typescript.rs:389-508` (`collect_call_symbols`)

### Test `unwrap`/`expect` Count

- `tests/` total occurrences: 543

### Line-Length Drift (>100)

Highest counts:

- `tests/milestone27_context_scope.rs`: 10
- `src/indexer/mod.rs`: 9
- `src/output.rs`: 6
- `src/main.rs`: 6
- `src/indexer/rust_ast.rs`: 6

