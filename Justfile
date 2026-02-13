set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default: help

help:
    @just --list

# Build and quality
build:
    cargo build

build-release:
    cargo build --release

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

check: fmt-check clippy test

# Contract validators
contract-lint:
    bash -n scripts/validate_tdd_cycle.sh
    bash -n scripts/validate_evidence_packet.sh

contract-tdd base="origin/main":
    bash scripts/validate_tdd_cycle.sh --base "{{base}}"

contract-evidence pr_body=".github/pull_request_template.md":
    bash scripts/validate_evidence_packet.sh --pr-body "{{pr_body}}"

contract-check base="origin/main" pr_body=".github/pull_request_template.md": contract-lint
    bash scripts/validate_tdd_cycle.sh --base "{{base}}"
    bash scripts/validate_evidence_packet.sh --pr-body "{{pr_body}}"

# Docs
_docs-require-mdbook:
    @command -v mdbook >/dev/null 2>&1 || (echo "mdbook is required. Install: cargo install mdbook" >&2; exit 1)

docs-build: _docs-require-mdbook
    mdbook build docs

docs-serve: _docs-require-mdbook
    mdbook serve docs

docs-style repo=".":
    bash scripts/check_markdown_style.sh --repo "{{repo}}"

docs-links repo=".":
    bash scripts/check_markdown_links.sh --repo "{{repo}}"

docs-check repo=".": docs-build docs-style docs-links

# Dogfooding loops
dogfood-pre symbol repo=".":
    cargo run -- index --repo "{{repo}}"
    cargo run -- find "{{symbol}}" --repo "{{repo}}" --json
    cargo run -- refs "{{symbol}}" --repo "{{repo}}" --json

dogfood-post symbol repo=".":
    cargo run -- index --repo "{{repo}}"
    cargo run -- find "{{symbol}}" --repo "{{repo}}"
    cargo run -- refs "{{symbol}}" --repo "{{repo}}"
    cargo test

# CLI wrappers
index repo=".":
    cargo run -- index --repo "{{repo}}"

status repo=".":
    cargo run -- status --repo "{{repo}}"

find symbol repo=".":
    cargo run -- find "{{symbol}}" --repo "{{repo}}"

find-json symbol repo=".":
    cargo run -- find "{{symbol}}" --repo "{{repo}}" --json

refs symbol repo=".":
    cargo run -- refs "{{symbol}}" --repo "{{repo}}"

refs-json symbol repo=".":
    cargo run -- refs "{{symbol}}" --repo "{{repo}}" --json

impact symbol repo=".":
    cargo run -- impact "{{symbol}}" --repo "{{repo}}"

impact-json symbol repo=".":
    cargo run -- impact "{{symbol}}" --repo "{{repo}}" --json

context task repo="." budget="1200":
    cargo run -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}"

context-json task repo="." budget="1200":
    cargo run -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}" --json

tests-for symbol repo=".":
    cargo run -- tests-for "{{symbol}}" --repo "{{repo}}"

verify-plan changed_file repo=".":
    cargo run -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}"

verify-plan-json changed_file repo=".":
    cargo run -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}" --json

diff-impact changed_file repo=".":
    cargo run -- diff-impact --changed-file "{{changed_file}}" --repo "{{repo}}"

diff-impact-json changed_file repo=".":
    cargo run -- diff-impact --changed-file "{{changed_file}}" --repo "{{repo}}" --json

orient repo=".":
    cargo run -- orient --repo "{{repo}}"

summary repo=".":
    cargo run -- summary --repo "{{repo}}"

health repo=".":
    cargo run -- health --repo "{{repo}}"

# Legacy phase gates (kept for historical compatibility)
gate-convergence repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase15_convergence_pack.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

gate-deterministic-replay repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase16_deterministic_replay.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

gate-benchmark-pack repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase16_benchmark_pack.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

gate-known-issues repo="." doc="docs/known-issues-budget-phase16.md":
    bash scripts/check_phase16_known_issues_budget.sh --repo "{{repo}}" --doc "{{doc}}"

gate-release-checklist repo="." doc="docs/release-checklist-phase16.md":
    bash scripts/check_phase16_release_checklist.sh --repo "{{repo}}" --doc "{{doc}}"

gate-large-repo-benchmark repo=".":
    bash scripts/check_phase16_large_repo_benchmark.sh --repo "{{repo}}"

gate-large-repo-replay repo=".":
    bash scripts/check_phase16_large_repo_replay.sh --repo "{{repo}}"

gate-maintenance repo=".":
    bash scripts/check_phase18_maintenance_pack.sh --repo "{{repo}}"

gate-doc-freshness repo="." doc="docs/maintenance-cadence-phase18.md":
    bash scripts/check_phase18_docs_freshness.sh --repo "{{repo}}" --doc "{{doc}}"

gate-e2e-matrix repo="." mode="full":
    bash scripts/run_e2e_release_matrix.sh --repo "{{repo}}" --mode "{{mode}}"
