#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase16_benchmark_pack.sh [--repo <path>] [--fixtures <path>] [--record]

Options:
  --repo <path>       Repository root for build and workspace checks (default: .)
  --fixtures <path>   Cross-language benchmark fixture root
                      (default: tests/fixtures/phase15/convergence_pack)
  --record            Print measured timings without threshold pass/fail gating
  -h, --help          Show this help text
USAGE
}

repo="."
fixtures="tests/fixtures/phase15/convergence_pack"
record_mode=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="${2:-}"
            shift 2
            ;;
        --fixtures)
            fixtures="${2:-}"
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
    local base="$1"
    local candidate="$2"
    if [[ "$candidate" = /* ]]; then
        printf '%s\n' "$candidate"
    else
        printf '%s\n' "$base/$candidate"
    fi
}

REPO_PATH="$(resolve_path "$ROOT_DIR" "$repo")"
FIXTURES_PATH="$(resolve_path "$REPO_PATH" "$fixtures")"

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

if [[ ! -d "$FIXTURES_PATH" ]]; then
    echo "fixture path does not exist: $FIXTURES_PATH" >&2
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

run_language_case() {
    local label="$1"
    local repo_path="$2"
    local symbol="$3"
    local changed_file="$4"
    local binary="$5"
    local index_threshold="$6"
    local query_threshold="$7"

    run_guardrail "$label index" "$index_threshold" "$binary" index --repo "$repo_path"
    run_guardrail "$label find" "$query_threshold" "$binary" find "$symbol" --repo "$repo_path" --json
    run_guardrail "$label refs" "$query_threshold" "$binary" refs "$symbol" --repo "$repo_path" --json
    run_guardrail "$label tests-for" "$query_threshold" "$binary" tests-for "$symbol" --repo "$repo_path" --json
    run_guardrail "$label verify-plan" "$query_threshold" "$binary" verify-plan --changed-file "$changed_file" --repo "$repo_path" --json
    run_guardrail "$label diff-impact" "$query_threshold" "$binary" diff-impact --changed-file "$changed_file" --repo "$repo_path" --json
}

pushd "$REPO_PATH" >/dev/null
echo "Building release binary for phase16 benchmark pack..."
cargo build --release >/dev/null

BINARY_PATH="$REPO_PATH/target/release/repo-scout"
if [[ ! -x "$BINARY_PATH" ]]; then
    echo "release binary not found at $BINARY_PATH" >&2
    exit 1
fi

run_guardrail "workspace index" "25.0" "$BINARY_PATH" index --repo "$REPO_PATH"
run_guardrail "workspace find" "4.0" "$BINARY_PATH" find select_full_suite_command --repo "$REPO_PATH" --json
run_guardrail "workspace refs" "4.0" "$BINARY_PATH" refs select_full_suite_command --repo "$REPO_PATH" --json

run_language_case "rust" "$FIXTURES_PATH/rust" "phase63_plan" "src/lib.rs" "$BINARY_PATH" "3.0" "2.5"
run_language_case "go" "$FIXTURES_PATH/go" "PlanPhase63" "src/service.go" "$BINARY_PATH" "3.0" "2.5"
run_language_case "python" "$FIXTURES_PATH/python" "plan_phase63" "src/service.py" "$BINARY_PATH" "3.0" "2.5"
run_language_case "typescript_vitest" "$FIXTURES_PATH/typescript_vitest" "planPhase63" "src/service.ts" "$BINARY_PATH" "3.0" "2.5"
run_language_case "typescript_jest" "$FIXTURES_PATH/typescript_jest" "planPhase63" "src/service.ts" "$BINARY_PATH" "3.0" "2.5"

if [[ "$record_mode" -eq 0 ]]; then
    echo "Phase 16 benchmark pack guardrails passed."
fi
popd >/dev/null
