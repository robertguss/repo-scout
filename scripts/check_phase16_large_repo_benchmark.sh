#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase16_large_repo_benchmark.sh [--repo <path>] [--record]

Options:
  --repo <path>       Repository root to benchmark (default: .)
  --record            Print measured timings without threshold pass/fail gating
  -h, --help          Show this help text
USAGE
}

repo="."
record_mode=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="${2:-}"
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
THRESHOLD_DOC_PATH="$ROOT_DIR/docs/performance-thresholds-phase16-large-repo.md"

resolve_path() {
    local base="$1"
    local candidate="$2"
    if [[ "$candidate" = /* ]]; then
        printf '%s\n' "$candidate"
    else
        printf '%s\n' "$base/$candidate"
    fi
}

REPO_PATH="$(resolve_path "$ROOT_DIR" "$repo")"

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

if [[ ! -f "$THRESHOLD_DOC_PATH" ]]; then
    echo "threshold doc does not exist: $THRESHOLD_DOC_PATH" >&2
    exit 2
fi

parse_threshold() {
    local key="$1"
    local value
    value="$(
        grep -E "^- ${key}:[[:space:]]*[0-9]+(\\.[0-9]+)?$" "$THRESHOLD_DOC_PATH" \
            | head -n 1 \
            | sed -E "s/^- ${key}:[[:space:]]*([0-9]+(\\.[0-9]+)?)$/\\1/"
    )"

    if [[ -z "$value" ]]; then
        echo "missing or invalid threshold key in doc: ${key}" >&2
        exit 1
    fi

    printf '%s\n' "$value"
}

MAX_INDEX_SECONDS="$(parse_threshold "max_index_seconds")"
MAX_FIND_SECONDS="$(parse_threshold "max_find_seconds")"
MAX_REFS_SECONDS="$(parse_threshold "max_refs_seconds")"
MAX_CONTEXT_SECONDS="$(parse_threshold "max_context_seconds")"
MAX_VERIFY_PLAN_SECONDS="$(parse_threshold "max_verify_plan_seconds")"
MAX_DIFF_IMPACT_SECONDS="$(parse_threshold "max_diff_impact_seconds")"

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

pushd "$REPO_PATH" >/dev/null
echo "Building release binary for phase16 large-repo benchmark..."
cargo build --release >/dev/null

BINARY_PATH="$REPO_PATH/target/release/repo-scout"
if [[ ! -x "$BINARY_PATH" ]]; then
    echo "release binary not found at $BINARY_PATH" >&2
    exit 1
fi

run_guardrail "large_repo index" "$MAX_INDEX_SECONDS" \
    "$BINARY_PATH" index --repo "$REPO_PATH"
run_guardrail "large_repo find" "$MAX_FIND_SECONDS" \
    "$BINARY_PATH" find select_full_suite_command --repo "$REPO_PATH" --json
run_guardrail "large_repo refs" "$MAX_REFS_SECONDS" \
    "$BINARY_PATH" refs select_full_suite_command --repo "$REPO_PATH" --json
run_guardrail "large_repo context" "$MAX_CONTEXT_SECONDS" \
    "$BINARY_PATH" context --task "update run and verify refs behavior" --repo "$REPO_PATH" --budget 1200 --json
run_guardrail "large_repo verify-plan" "$MAX_VERIFY_PLAN_SECONDS" \
    "$BINARY_PATH" verify-plan --changed-file src/query/mod.rs --repo "$REPO_PATH" --json
run_guardrail "large_repo diff-impact" "$MAX_DIFF_IMPACT_SECONDS" \
    "$BINARY_PATH" diff-impact --changed-file src/query/mod.rs --repo "$REPO_PATH" --json

if [[ "$record_mode" -eq 0 ]]; then
    echo "Phase 16 large-repo benchmark guardrails passed."
fi
popd >/dev/null
