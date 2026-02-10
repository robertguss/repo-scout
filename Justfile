set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

# Contributor workflows
build:
    cargo build

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

check: fmt-check clippy test

# Contract System v2 validators
# Usage: just contract-lint
contract-lint:
    bash -n scripts/validate_tdd_cycle.sh
    bash -n scripts/validate_evidence_packet.sh

# Usage: just contract-tdd [base]
contract-tdd base="origin/main":
    bash scripts/validate_tdd_cycle.sh --base "{{base}}"

# Usage: just contract-evidence-pr [pr_body]
contract-evidence-pr pr_body=".github/pull_request_template.md":
    bash scripts/validate_evidence_packet.sh --pr-body "{{pr_body}}"

# Usage: just contract-check [base] [pr_body]
contract-check base="origin/main" pr_body=".github/pull_request_template.md": contract-lint
    bash scripts/validate_tdd_cycle.sh --base "{{base}}"
    bash scripts/validate_evidence_packet.sh --pr-body "{{pr_body}}"

# Usage: just docs-consistency [repo]
docs-consistency repo=".":
    bash scripts/check_docs_consistency.sh --repo "{{repo}}"

# Usage: just phase18-maintenance-pack [repo]
phase18-maintenance-pack repo=".":
    bash scripts/check_phase18_maintenance_pack.sh --repo "{{repo}}"

# Usage: just phase18-docs-freshness [repo] [doc]
phase18-docs-freshness repo="." doc="docs/maintenance-cadence-phase18.md":
    bash scripts/check_phase18_docs_freshness.sh --repo "{{repo}}" --doc "{{doc}}"

# Dogfood loops
# Usage: just dogfood-pre <symbol> [repo]
dogfood-pre symbol repo=".":
    cargo run -- index --repo "{{repo}}"
    cargo run -- find "{{symbol}}" --repo "{{repo}}" --json
    cargo run -- refs "{{symbol}}" --repo "{{repo}}" --json

# Usage: just dogfood-post <symbol> [repo]
dogfood-post symbol repo=".":
    cargo run -- index --repo "{{repo}}"
    cargo run -- find "{{symbol}}" --repo "{{repo}}"
    cargo run -- refs "{{symbol}}" --repo "{{repo}}"
    cargo test

# TDD helpers
# Usage: just tdd-red <test_name>
tdd-red test_name:
    @echo "Running red step for {{test_name}} (failure expected)."
    -cargo test "{{test_name}}" -- --nocapture

# Usage: just tdd-green <test_name>
tdd-green test_name:
    cargo test "{{test_name}}" -- --nocapture

tdd-refactor:
    cargo test

# User convenience wrappers
# Usage: just index [repo]
index repo=".":
    cargo run -- index --repo "{{repo}}"

# Usage: just status [repo]
status repo=".":
    cargo run -- status --repo "{{repo}}"

# Usage: just find <symbol> [repo]
find symbol repo=".":
    cargo run -- find "{{symbol}}" --repo "{{repo}}"

# Usage: just find-json <symbol> [repo]
find-json symbol repo=".":
    cargo run -- find "{{symbol}}" --repo "{{repo}}" --json

# Usage: just refs <symbol> [repo]
refs symbol repo=".":
    cargo run -- refs "{{symbol}}" --repo "{{repo}}"

# Usage: just refs-json <symbol> [repo]
refs-json symbol repo=".":
    cargo run -- refs "{{symbol}}" --repo "{{repo}}" --json

# Usage: just impact <symbol> [repo]
impact symbol repo=".":
    cargo run -- impact "{{symbol}}" --repo "{{repo}}"

# Usage: just impact-json <symbol> [repo]
impact-json symbol repo=".":
    cargo run -- impact "{{symbol}}" --repo "{{repo}}" --json

# Usage: just context "<task>" [repo] [budget]
context task repo="." budget="1200":
    cargo run -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}"

# Usage: just context-json "<task>" [repo] [budget]
context-json task repo="." budget="1200":
    cargo run -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}" --json

# Usage: just tests-for <symbol> [repo]
tests-for symbol repo=".":
    cargo run -- tests-for "{{symbol}}" --repo "{{repo}}"

# Usage: just tests-for-json <symbol> [repo]
tests-for-json symbol repo=".":
    cargo run -- tests-for "{{symbol}}" --repo "{{repo}}" --json

# Usage: just verify-plan <changed_file> [repo]
verify-plan changed_file repo=".":
    cargo run -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}"

# Usage: just verify-plan-json <changed_file> [repo]
verify-plan-json changed_file repo=".":
    cargo run -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}" --json

# Performance guardrails
# Usage: just perf-baseline-core [symbol] [repo]
perf-baseline-core symbol="run" repo=".":
    /usr/bin/time -p cargo run --release -- index --repo "{{repo}}"
    /usr/bin/time -p cargo run --release -- find "{{symbol}}" --repo "{{repo}}" --json
    /usr/bin/time -p cargo run --release -- refs "{{symbol}}" --repo "{{repo}}" --json

# Usage: just perf-baseline-full [symbol] [changed_file] [task] [repo] [budget]
perf-baseline-full symbol="run" changed_file="src/query/mod.rs" task="update run and verify refs behavior" repo="." budget="1200":
    /usr/bin/time -p cargo run --release -- index --repo "{{repo}}"
    /usr/bin/time -p cargo run --release -- impact "{{symbol}}" --repo "{{repo}}" --json
    /usr/bin/time -p cargo run --release -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}" --json
    /usr/bin/time -p cargo run --release -- tests-for "{{symbol}}" --repo "{{repo}}" --json
    /usr/bin/time -p cargo run --release -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}" --json

# Usage: just perf-rust-guardrails [repo] [fixture]
perf-rust-guardrails repo="." fixture="tests/fixtures/phase11/rust_production/corpus":
    bash scripts/check_rust_perf_guardrails.sh --repo "{{repo}}" --fixture "{{fixture}}"

# Usage: just perf-rust-record [repo] [fixture]
perf-rust-record repo="." fixture="tests/fixtures/phase11/rust_production/corpus":
    bash scripts/check_rust_perf_guardrails.sh --repo "{{repo}}" --fixture "{{fixture}}" --record

# Usage: just phase15-convergence-pack [repo] [fixtures]
phase15-convergence-pack repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase15_convergence_pack.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

# Usage: just phase16-deterministic-replay [repo] [fixtures]
phase16-deterministic-replay repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase16_deterministic_replay.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

# Usage: just phase16-benchmark-pack [repo] [fixtures]
phase16-benchmark-pack repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase16_benchmark_pack.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

# Usage: just phase16-known-issues-budget [repo] [doc]
phase16-known-issues-budget repo="." doc="docs/known-issues-budget-phase16.md":
    bash scripts/check_phase16_known_issues_budget.sh --repo "{{repo}}" --doc "{{doc}}"

# Usage: just phase16-large-repo-benchmark [repo]
phase16-large-repo-benchmark repo=".":
    bash scripts/check_phase16_large_repo_benchmark.sh --repo "{{repo}}"

# Usage: just phase16-release-checklist [repo] [doc]
phase16-release-checklist repo="." doc="docs/release-checklist-phase16.md":
    bash scripts/check_phase16_release_checklist.sh --repo "{{repo}}" --doc "{{doc}}"

# Usage: just phase16-large-repo-replay [repo]
phase16-large-repo-replay repo=".":
    bash scripts/check_phase16_large_repo_replay.sh --repo "{{repo}}"
