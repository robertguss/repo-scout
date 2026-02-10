#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase16_large_repo_replay.sh [--repo <path>] [--record]

Options:
  --repo <path>       Repository root for building/running repo-scout (default: .)
  --record            Print deterministic replay measurements for each command
  -h, --help          Show this help text

Environment:
  REPO_SCOUT_BIN      Optional prebuilt repo-scout binary path. If unset, builds
                      target/release/repo-scout from --repo.
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

    if [[ "$record_mode" -eq 1 ]]; then
        echo "MEASURE [$label] deterministic=pass"
    else
        echo "PASS [$label]"
    fi
}

pushd "$REPO_PATH" >/dev/null

if [[ -n "${REPO_SCOUT_BIN:-}" ]]; then
    BINARY_PATH="$(resolve_path "$REPO_PATH" "$REPO_SCOUT_BIN")"
else
    echo "Building release binary for phase16 large-repo replay..."
    cargo build --release >/dev/null
    BINARY_PATH="$REPO_PATH/target/release/repo-scout"
fi

if [[ ! -x "$BINARY_PATH" ]]; then
    echo "repo-scout binary not executable: $BINARY_PATH" >&2
    exit 1
fi

SYMBOL="select_full_suite_command"
CHANGED_FILE="src/query/mod.rs"
CONTEXT_TASK="update select_full_suite_command behavior and verify refs stability"

"$BINARY_PATH" index --repo "$REPO_PATH" >/dev/null
run_replay_check "workspace find" "$BINARY_PATH" find "$SYMBOL" --repo "$REPO_PATH" --json
run_replay_check "workspace refs" "$BINARY_PATH" refs "$SYMBOL" --repo "$REPO_PATH" --json
run_replay_check "workspace tests-for" "$BINARY_PATH" tests-for "$SYMBOL" --repo "$REPO_PATH" --json
run_replay_check "workspace verify-plan" "$BINARY_PATH" verify-plan --changed-file "$CHANGED_FILE" --repo "$REPO_PATH" --json
run_replay_check "workspace diff-impact" "$BINARY_PATH" diff-impact --changed-file "$CHANGED_FILE" --repo "$REPO_PATH" --json
run_replay_check "workspace context" "$BINARY_PATH" context --task "$CONTEXT_TASK" --repo "$REPO_PATH" --budget 1200 --json

if [[ "$record_mode" -eq 1 ]]; then
    echo "Phase 16 large-repo deterministic replay measurements recorded."
else
    echo "Phase 16 large-repo deterministic replay validation passed."
fi

popd >/dev/null
