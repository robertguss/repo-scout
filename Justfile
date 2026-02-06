set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

# Format Rust sources.
fmt:
    cargo fmt

# Run full test suite.
test:
    cargo test

# Dogfood loop before editing a feature slice.
dogfood-pre symbol:
    cargo run -- index --repo .
    cargo run -- find "{{symbol}}" --repo . --json
    cargo run -- refs "{{symbol}}" --repo . --json

# Dogfood loop after editing a feature slice.
dogfood-post symbol:
    cargo run -- index --repo .
    cargo run -- find "{{symbol}}" --repo .
    cargo run -- refs "{{symbol}}" --repo .
    cargo test

# TDD red step: run a feature-slice test and expect failure.
tdd-red test_name:
    @echo "Running red step for {{test_name}} (failure expected)."
    -cargo test "{{test_name}}" -- --nocapture

# TDD green step: run the same feature-slice test and expect pass.
tdd-green test_name:
    cargo test "{{test_name}}" -- --nocapture

# TDD refactor step: full-suite pass gate.
tdd-refactor:
    cargo test

# Lightweight local performance guardrail.
perf-baseline symbol:
    /usr/bin/time -p cargo run --release -- index --repo .
    /usr/bin/time -p cargo run --release -- find "{{symbol}}" --repo . --json
    /usr/bin/time -p cargo run --release -- refs "{{symbol}}" --repo . --json
