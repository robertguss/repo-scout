#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_rust_perf_guardrails.sh [--repo <path>] [--fixture <path>] [--record]

Options:
  --repo <path>      Repository root to measure (default: .)
  --fixture <path>   Rust production fixture repo to measure
                     (default: tests/fixtures/phase11/rust_production/corpus)
  --record           Print measured timings without threshold pass/fail gating
  -h, --help         Show this help text
USAGE
}

repo="."
fixture="tests/fixtures/phase11/rust_production/corpus"
record_mode=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="${2:-}"
            shift 2
            ;;
        --fixture)
            fixture="${2:-}"
            shift 2
            ;;
        --record)
            record_mode=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

resolve_path() {
    local candidate="$1"
    if [[ "$candidate" = /* ]]; then
        printf '%s\n' "$candidate"
    else
        printf '%s\n' "$ROOT_DIR/$candidate"
    fi
}

REPO_PATH="$(resolve_path "$repo")"
FIXTURE_PATH="$(resolve_path "$fixture")"

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

if [[ ! -d "$FIXTURE_PATH" ]]; then
    echo "fixture path does not exist: $FIXTURE_PATH" >&2
    exit 2
fi

run_guardrail() {
    local label="$1"
    local threshold="$2"
    shift 2

    local timer_file
    timer_file="$(mktemp)"
    if ! /usr/bin/time -p "$@" >/dev/null 2>"$timer_file"; then
        cat "$timer_file" >&2
        rm -f "$timer_file"
        echo "FAIL [$label] command execution failed" >&2
        exit 1
    fi

    local elapsed
    elapsed="$(awk '/^real / { print $2 }' "$timer_file")"
    rm -f "$timer_file"

    if [[ -z "$elapsed" ]]; then
        echo "FAIL [$label] unable to parse elapsed time" >&2
        exit 1
    fi

    if [[ "$record_mode" -eq 1 ]]; then
        printf 'MEASURE [%s] real=%ss threshold=%ss\n' "$label" "$elapsed" "$threshold"
        return
    fi

    if awk -v elapsed="$elapsed" -v threshold="$threshold" 'BEGIN { exit !(elapsed <= threshold) }'; then
        printf 'PASS [%s] real=%ss <= threshold=%ss\n' "$label" "$elapsed" "$threshold"
    else
        printf 'FAIL [%s] real=%ss > threshold=%ss\n' "$label" "$elapsed" "$threshold" >&2
        exit 1
    fi
}

pushd "$ROOT_DIR" >/dev/null
echo "Building release binary for stable warm-cache timing..."
cargo build --release >/dev/null

BINARY_PATH="$ROOT_DIR/target/release/repo-scout"
if [[ ! -x "$BINARY_PATH" ]]; then
    echo "release binary not found at $BINARY_PATH" >&2
    exit 1
fi

run_guardrail "repo_index" "15.0" "$BINARY_PATH" index --repo "$REPO_PATH"
run_guardrail "repo_find_json" "2.0" "$BINARY_PATH" find run --repo "$REPO_PATH" --json
run_guardrail "repo_refs_json" "2.0" "$BINARY_PATH" refs run --repo "$REPO_PATH" --json
run_guardrail "fixture_index" "2.0" "$BINARY_PATH" index --repo "$FIXTURE_PATH"
run_guardrail "fixture_impact_json" "2.0" "$BINARY_PATH" impact helper --repo "$FIXTURE_PATH" --json
run_guardrail "fixture_diff_impact_json" "2.0" "$BINARY_PATH" diff-impact --changed-file src/util/mod.rs --repo "$FIXTURE_PATH" --json

if [[ "$record_mode" -eq 0 ]]; then
    echo "Rust performance guardrails passed."
fi
popd >/dev/null
