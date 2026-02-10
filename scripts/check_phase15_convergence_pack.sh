#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase15_convergence_pack.sh [--repo <path>] [--fixtures <path>]

Options:
  --repo <path>       Repository root for building/running repo-scout (default: .)
  --fixtures <path>   Phase 15 convergence fixture root
                      (default: tests/fixtures/phase15/convergence_pack)
  -h, --help          Show this help text
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

require_contains() {
    local label="$1"
    local expected="$2"
    local haystack="$3"
    if ! grep -Fq "$expected" <<<"$haystack"; then
        echo "FAIL [$label] expected to find: $expected" >&2
        echo "--- command output begin ---" >&2
        printf '%s\n' "$haystack" >&2
        echo "--- command output end ---" >&2
        exit 1
    fi
    echo "PASS [$label]"
}

run_pack_case() {
    local label="$1"
    local repo_path="$2"
    local symbol="$3"
    local changed_file="$4"
    local expected_target="$5"
    local expected_targeted_step="$6"
    local expected_full_suite_step="$7"
    local binary="$8"

    "$binary" index --repo "$repo_path" >/dev/null

    local tests_for_json
    tests_for_json="$("$binary" tests-for "$symbol" --repo "$repo_path" --json)"
    require_contains "$label tests-for target" "\"target\": \"$expected_target\"" "$tests_for_json"
    require_contains "$label tests-for schema" "\"schema_version\": 2" "$tests_for_json"

    local verify_json
    verify_json="$("$binary" verify-plan --changed-file "$changed_file" --repo "$repo_path" --json)"
    require_contains "$label verify targeted" "\"step\": \"$expected_targeted_step\"" "$verify_json"
    require_contains "$label verify full_suite" "\"step\": \"$expected_full_suite_step\"" "$verify_json"
    require_contains "$label verify schema" "\"schema_version\": 2" "$verify_json"
}

pushd "$REPO_PATH" >/dev/null
echo "Building release binary for phase15 convergence pack..."
cargo build --release >/dev/null

BINARY_PATH="$REPO_PATH/target/release/repo-scout"
if [[ ! -x "$BINARY_PATH" ]]; then
    echo "release binary not found at $BINARY_PATH" >&2
    exit 1
fi

run_pack_case \
    "rust" \
    "$FIXTURES_PATH/rust" \
    "phase63_plan" \
    "src/lib.rs" \
    "tests/phase63_flow.rs" \
    "cargo test --test phase63_flow" \
    "cargo test" \
    "$BINARY_PATH"

run_pack_case \
    "go" \
    "$FIXTURES_PATH/go" \
    "PlanPhase63" \
    "src/service.go" \
    "src/service_test.go" \
    "go test ./src" \
    "go test ./..." \
    "$BINARY_PATH"

run_pack_case \
    "python" \
    "$FIXTURES_PATH/python" \
    "plan_phase63" \
    "src/service.py" \
    "tests/test_service.py" \
    "pytest tests/test_service.py" \
    "pytest" \
    "$BINARY_PATH"

run_pack_case \
    "typescript_vitest" \
    "$FIXTURES_PATH/typescript_vitest" \
    "planPhase63" \
    "src/service.ts" \
    "tests/service.test.ts" \
    "npx vitest run tests/service.test.ts" \
    "npx vitest run" \
    "$BINARY_PATH"

run_pack_case \
    "typescript_jest" \
    "$FIXTURES_PATH/typescript_jest" \
    "planPhase63" \
    "src/service.ts" \
    "src/service.spec.ts" \
    "npx jest --runTestsByPath src/service.spec.ts" \
    "npx jest" \
    "$BINARY_PATH"

echo "Phase 15 convergence pack validation passed."
popd >/dev/null
