set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Show available recipes.
default: help

# List all recipes.
help:
    @just --list

# Compile debug build.
build:
    cargo build

# Compile optimized release build.
build-release:
    cargo build --release

# Format Rust code.
fmt:
    cargo fmt

# Check formatting without changing files.
fmt-check:
    cargo fmt -- --check

# Run strict lints.
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run full test suite.
test:
    cargo test

# Run format check + lints + tests.
check: fmt-check clippy test

# Syntax-check contract validator scripts.
contract-lint:
    bash -n scripts/validate_tdd_cycle.sh
    bash -n scripts/validate_evidence_packet.sh

# Validate TDD cycle against base branch.
contract-tdd base="origin/main":
    bash scripts/validate_tdd_cycle.sh --base "{{base}}"

# Validate PR evidence headings/body.
contract-evidence pr_body=".github/pull_request_template.md":
    bash scripts/validate_evidence_packet.sh --pr-body "{{pr_body}}"

# Run all contract checks.
contract-check base="origin/main" pr_body=".github/pull_request_template.md": contract-lint
    bash scripts/validate_tdd_cycle.sh --base "{{base}}"
    bash scripts/validate_evidence_packet.sh --pr-body "{{pr_body}}"

# Internal guard: ensure mdBook is installed.
_docs-require-mdbook:
    @command -v mdbook >/dev/null 2>&1 || (echo "mdbook is required. Install: cargo install mdbook" >&2; exit 1)

# Build docs site.
docs-build: _docs-require-mdbook
    mdbook build docs

# Serve docs locally with live reload.
docs-serve: _docs-require-mdbook
    mdbook serve docs

# Check markdown style in repo/docs.
docs-style repo=".":
    bash scripts/check_markdown_style.sh --repo "{{repo}}"

# Check markdown links in repo/docs.
docs-links repo=".":
    bash scripts/check_markdown_links.sh --repo "{{repo}}"

# Build docs + run style and link checks.
docs-check repo=".":
    just docs-build
    just docs-style "{{repo}}"
    just docs-links "{{repo}}"

# Pre-change dogfood: index/find/refs (JSON).
dogfood-pre symbol repo=".":
    cargo run -- index --repo "{{repo}}"
    cargo run -- find "{{symbol}}" --repo "{{repo}}" --json
    cargo run -- refs "{{symbol}}" --repo "{{repo}}" --json

# Post-change dogfood: index/find/refs + tests.
dogfood-post symbol repo=".":
    cargo run -- index --repo "{{repo}}"
    cargo run -- find "{{symbol}}" --repo "{{repo}}"
    cargo run -- refs "{{symbol}}" --repo "{{repo}}"
    cargo test

# Build or update index for repository.
index repo=".":
    cargo run -- index --repo "{{repo}}"

# Show repository and index status.
status repo=".":
    cargo run -- status --repo "{{repo}}"

# Find symbol definitions/usages (human output).
find symbol repo=".":
    cargo run -- find "{{symbol}}" --repo "{{repo}}"

# Find symbol definitions/usages (JSON output).
find-json symbol repo=".":
    cargo run -- find "{{symbol}}" --repo "{{repo}}" --json

# Find references to a symbol (human output).
refs symbol repo=".":
    cargo run -- refs "{{symbol}}" --repo "{{repo}}"

# Find references to a symbol (JSON output).
refs-json symbol repo=".":
    cargo run -- refs "{{symbol}}" --repo "{{repo}}" --json

# Analyze downstream impact of symbol changes.
impact symbol repo=".":
    cargo run -- impact "{{symbol}}" --repo "{{repo}}"

# Analyze impact of symbol changes (JSON output).
impact-json symbol repo=".":
    cargo run -- impact "{{symbol}}" --repo "{{repo}}" --json

# Build coding context for a task.
context task repo="." budget="1200":
    cargo run -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}"

# Build coding context for a task (JSON output).
context-json task repo="." budget="1200":
    cargo run -- context --task "{{task}}" --repo "{{repo}}" --budget "{{budget}}" --json

# Find tests covering a symbol.
tests-for symbol repo=".":
    cargo run -- tests-for "{{symbol}}" --repo "{{repo}}"

# Verify plan coverage for a changed file.
verify-plan changed_file repo=".":
    cargo run -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}"

# Verify plan coverage for a changed file (JSON output).
verify-plan-json changed_file repo=".":
    cargo run -- verify-plan --changed-file "{{changed_file}}" --repo "{{repo}}" --json

# Analyze likely impact from a changed file.
diff-impact changed_file repo=".":
    cargo run -- diff-impact --changed-file "{{changed_file}}" --repo "{{repo}}"

# Analyze changed-file impact (JSON output).
diff-impact-json changed_file repo=".":
    cargo run -- diff-impact --changed-file "{{changed_file}}" --repo "{{repo}}" --json

# Generate repo orientation summary.
orient repo=".":
    cargo run -- orient --repo "{{repo}}"

# Generate high-level repo summary.
summary repo=".":
    cargo run -- summary --repo "{{repo}}"

# Run project health checks.
health repo=".":
    cargo run -- health --repo "{{repo}}"

# Run phase15 convergence gate (legacy compatibility).
gate-convergence repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase15_convergence_pack.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

# Run deterministic replay gate (legacy compatibility).
gate-deterministic-replay repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase16_deterministic_replay.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

# Run benchmark pack gate (legacy compatibility).
gate-benchmark-pack repo="." fixtures="tests/fixtures/phase15/convergence_pack":
    bash scripts/check_phase16_benchmark_pack.sh --repo "{{repo}}" --fixtures "{{fixtures}}"

# Run large-repo replay gate (legacy compatibility).
gate-large-repo-replay repo=".":
    bash scripts/check_phase16_large_repo_replay.sh --repo "{{repo}}"

# Run e2e release matrix gate suite.
gate-e2e-matrix repo="." mode="full":
    bash scripts/run_e2e_release_matrix.sh --repo "{{repo}}" --mode "{{mode}}"
