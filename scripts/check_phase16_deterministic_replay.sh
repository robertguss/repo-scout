#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase16_deterministic_replay.sh [--repo <path>] [--fixtures <path>]

Options:
  --repo <path>       Repository root for building/running repo-scout (default: .)
  --fixtures <path>   Cross-language fixture root
                      (default: tests/fixtures/phase15/convergence_pack)
  -h, --help          Show this help text

Environment:
  REPO_SCOUT_BIN      Optional prebuilt repo-scout binary path. If unset, builds
                      target/release/repo-scout from --repo.
USAGE
}

repo="."
fixtures="tests/fixtures/phase15/convergence_pack"

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

run_replay_check() {
    local label="$1"
    shift
    local first_run
    local second_run
    first_run="$(mktemp)"
    second_run="$(mktemp)"
    "$@" >"$first_run"
    "$@" >"$second_run"
    if ! cmp -s "$first_run" "$second_run"; then
        echo "FAIL [$label] output mismatch across replay runs" >&2
        diff -u "$first_run" "$second_run" >&2 || true
        rm -f "$first_run" "$second_run"
        exit 1
    fi
    rm -f "$first_run" "$second_run"
    echo "PASS [$label]"
}

run_language_case() {
    local label="$1"
    local repo_path="$2"
    local symbol="$3"
    local changed_file="$4"
    local binary="$5"

    "$binary" index --repo "$repo_path" >/dev/null
    run_replay_check "$label find" "$binary" find "$symbol" --repo "$repo_path" --json
    run_replay_check "$label refs" "$binary" refs "$symbol" --repo "$repo_path" --json
    run_replay_check "$label tests-for" "$binary" tests-for "$symbol" --repo "$repo_path" --json
    run_replay_check "$label verify-plan" "$binary" verify-plan --changed-file "$changed_file" --repo "$repo_path" --json
    run_replay_check "$label diff-impact" "$binary" diff-impact --changed-file "$changed_file" --repo "$repo_path" --json
}

pushd "$REPO_PATH" >/dev/null

if [[ -n "${REPO_SCOUT_BIN:-}" ]]; then
    BINARY_PATH="$(resolve_path "$REPO_PATH" "$REPO_SCOUT_BIN")"
else
    echo "Building release binary for phase16 deterministic replay..."
    cargo build --release >/dev/null
    BINARY_PATH="$REPO_PATH/target/release/repo-scout"
fi

if [[ ! -x "$BINARY_PATH" ]]; then
    echo "repo-scout binary not executable: $BINARY_PATH" >&2
    exit 1
fi

run_language_case "rust" "$FIXTURES_PATH/rust" "phase63_plan" "src/lib.rs" "$BINARY_PATH"
run_language_case "go" "$FIXTURES_PATH/go" "PlanPhase63" "src/service.go" "$BINARY_PATH"
run_language_case "python" "$FIXTURES_PATH/python" "plan_phase63" "src/service.py" "$BINARY_PATH"
run_language_case "typescript_vitest" "$FIXTURES_PATH/typescript_vitest" "planPhase63" "src/service.ts" "$BINARY_PATH"
run_language_case "typescript_jest" "$FIXTURES_PATH/typescript_jest" "planPhase63" "src/service.ts" "$BINARY_PATH"

echo "Phase 16 deterministic replay validation passed."
popd >/dev/null
