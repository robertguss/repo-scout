#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: run_e2e_release_matrix.sh [--repo <path>] [--mode <full|smoke>] [--record]

Options:
  --repo <path>         Repository root (default: .)
  --mode <full|smoke>   Execution mode (default: full)
  --record              Record-only mode (never fails due to unresolved findings)
  -h, --help            Show this help text
USAGE
}

repo="."
mode="full"
record_mode=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="${2:-}"
            shift 2
            ;;
        --mode)
            mode="${2:-}"
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

if [[ "$mode" != "full" && "$mode" != "smoke" ]]; then
    echo "invalid --mode value: $mode (expected full or smoke)" >&2
    exit 2
fi

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

ARTIFACT_DIR="$REPO_PATH/agents/plans/repo-scout-e2e"
ISSUES_LOG="$ARTIFACT_DIR/issues-log.md"
OBSERVATION_LOG="$ARTIFACT_DIR/observations.jsonl"
RUN_ID="run-$(date -u +%Y%m%dT%H%M%SZ)"

for required_file in "$ISSUES_LOG" "$OBSERVATION_LOG"; do
    if [[ ! -f "$required_file" ]]; then
        echo "required artifact missing: $required_file" >&2
        exit 2
    fi
done

json_escape() {
    printf '%s' "$1" \
        | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' \
        | tr '\n' '\r' \
        | sed -e 's/\r/\\n/g'
}

unresolved_count=0
warning_count=0
pass_count=0
fail_count=0
info_count=0

ensure_run_history_section() {
    if ! grep -Fq "## Run History" "$ISSUES_LOG"; then
        {
            echo
            echo "## Run History"
        } >>"$ISSUES_LOG"
    fi
}

log_observation() {
    local lane="$1"
    local corpus="$2"
    local command="$3"
    local args="$4"
    local result="$5"
    local severity="$6"
    local status="$7"
    local owner="$8"
    local followup_id="$9"
    local timestamp
    timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    case "$result" in
        PASS) pass_count=$((pass_count + 1)) ;;
        WARN) warning_count=$((warning_count + 1)) ;;
        FAIL) fail_count=$((fail_count + 1)) ;;
        INFO) info_count=$((info_count + 1)) ;;
    esac

    if [[ "$status" == "open" ]]; then
        unresolved_count=$((unresolved_count + 1))
    fi

    printf '{"run_id":"%s","timestamp":"%s","lane":"%s","corpus":"%s","command":"%s","args":"%s","result":"%s","severity":"%s","status":"%s","owner":"%s","followup_id":"%s"}\n' \
        "$(json_escape "$RUN_ID")" \
        "$(json_escape "$timestamp")" \
        "$(json_escape "$lane")" \
        "$(json_escape "$corpus")" \
        "$(json_escape "$command")" \
        "$(json_escape "$args")" \
        "$(json_escape "$result")" \
        "$(json_escape "$severity")" \
        "$(json_escape "$status")" \
        "$(json_escape "$owner")" \
        "$(json_escape "$followup_id")" \
        >>"$OBSERVATION_LOG"
}

append_issue_entry() {
    local result="$1"
    local status="$2"
    local lane="$3"
    local corpus="$4"
    local command="$5"
    local args="$6"
    local followup_id="$7"
    local details="$8"

    ensure_run_history_section
    {
        printf -- '- [%s] result=%s status=%s lane=%s corpus=%s followup_id=%s\n' \
            "$RUN_ID" "$result" "$status" "$lane" "$corpus" "$followup_id"
        printf '  - command: `%s`\n' "$command"
        printf '  - args: `%s`\n' "$args"
        printf '  - reproduction: run the command exactly as listed above\n'
        printf '  - details: %s\n' "$details"
    } >>"$ISSUES_LOG"
}

run_command() {
    local lane="$1"
    local corpus="$2"
    local command_name="$3"
    local args="$4"
    local command="$5"
    local severity="$6"
    local enforce_mode="$7" # required | informational

    local output
    if output="$(eval "$command" 2>&1)"; then
        log_observation "$lane" "$corpus" "$command_name" "$args" "PASS" "$severity" "resolved" "@repo-scout-maintainers" "none"
        return 0
    fi

    if [[ "$enforce_mode" == "informational" ]]; then
        log_observation "$lane" "$corpus" "$command_name" "$args" "WARN" "$severity" "waived" "@repo-scout-maintainers" "E2E-INFO-$lane"
        append_issue_entry "WARN" "waived" "$lane" "$corpus" "$command_name" "$args" "E2E-INFO-$lane" "$output"
        return 0
    fi

    log_observation "$lane" "$corpus" "$command_name" "$args" "FAIL" "$severity" "open" "@repo-scout-maintainers" "E2E-FAIL-$lane"
    append_issue_entry "FAIL" "open" "$lane" "$corpus" "$command_name" "$args" "E2E-FAIL-$lane" "$output"
    return 1
}

run_expected_failure() {
    local lane="$1"
    local corpus="$2"
    local command_name="$3"
    local args="$4"
    local command="$5"

    if eval "$command" >/tmp/repo-scout-e2e-negative.out 2>/tmp/repo-scout-e2e-negative.err; then
        log_observation "$lane" "$corpus" "$command_name" "$args" "FAIL" "P2" "open" "@repo-scout-maintainers" "E2E-NEG-$lane"
        append_issue_entry "FAIL" "open" "$lane" "$corpus" "$command_name" "$args" "E2E-NEG-$lane" "expected failure but command succeeded"
        return 1
    fi

    log_observation "$lane" "$corpus" "$command_name" "$args" "PASS" "P3" "resolved" "@repo-scout-maintainers" "none"
    return 0
}

run_replay_check() {
    local lane="$1"
    local corpus="$2"
    local command_name="$3"
    local args="$4"
    local command="$5"

    local first_run second_run
    first_run="$(mktemp)"
    second_run="$(mktemp)"

    if ! eval "$command" >"$first_run" 2>&1; then
        log_observation "$lane" "$corpus" "$command_name" "$args" "FAIL" "P1" "open" "@repo-scout-maintainers" "E2E-REPLAY-$lane"
        append_issue_entry "FAIL" "open" "$lane" "$corpus" "$command_name" "$args" "E2E-REPLAY-$lane" "first replay run failed"
        rm -f "$first_run" "$second_run"
        return 1
    fi

    if ! eval "$command" >"$second_run" 2>&1; then
        log_observation "$lane" "$corpus" "$command_name" "$args" "FAIL" "P1" "open" "@repo-scout-maintainers" "E2E-REPLAY-$lane"
        append_issue_entry "FAIL" "open" "$lane" "$corpus" "$command_name" "$args" "E2E-REPLAY-$lane" "second replay run failed"
        rm -f "$first_run" "$second_run"
        return 1
    fi

    if cmp -s "$first_run" "$second_run"; then
        log_observation "$lane" "$corpus" "$command_name" "$args" "PASS" "P3" "resolved" "@repo-scout-maintainers" "none"
        rm -f "$first_run" "$second_run"
        return 0
    fi

    log_observation "$lane" "$corpus" "$command_name" "$args" "FAIL" "P1" "open" "@repo-scout-maintainers" "E2E-REPLAY-$lane"
    append_issue_entry "FAIL" "open" "$lane" "$corpus" "$command_name" "$args" "E2E-REPLAY-$lane" "output mismatch across deterministic replay runs"
    rm -f "$first_run" "$second_run"
    return 1
}

prepare_binary() {
    run_command "build" "workspace" "cargo-build-release" "--release" "cd \"$REPO_PATH\" && cargo build --release" "P2" "required"
}

make_tsx_corpus() {
    local tsx_root
    tsx_root="$(mktemp -d)"
    mkdir -p "$tsx_root/src" "$tsx_root/tests"

    cat >"$tsx_root/src/widget.tsx" <<'TSX'
export function RenderPhase79(): string {
  return "phase79";
}
TSX

    cat >"$tsx_root/tests/widget.test.tsx" <<'TSXTEST'
import { RenderPhase79 } from "../src/widget";

void RenderPhase79;
TSXTEST

    cat >"$tsx_root/package.json" <<'PKG'
{
  "name": "repo-scout-generated-tsx",
  "private": true,
  "devDependencies": {
    "vitest": "^1.6.0"
  }
}
PKG

    printf '%s\n' "$tsx_root"
}

run_find_refs_matrix() {
    local label="$1"
    local repo_arg="$2"
    local symbol="$3"

    local json_modes=("" "--json")
    local code_modes=("" "--code-only")
    local exclude_modes=("" "--exclude-tests")
    local max_modes=("" "--max-results 0" "--max-results 1" "--max-results 5")

    if [[ "$mode" == "full" ]]; then
        max_modes+=("--max-results 25")
    fi

    for command_name in find refs; do
        for json_flag in "${json_modes[@]}"; do
            for code_flag in "${code_modes[@]}"; do
                for exclude_flag in "${exclude_modes[@]}"; do
                    for max_flag in "${max_modes[@]}"; do
                        local args="$symbol --repo $repo_arg $json_flag $code_flag $exclude_flag $max_flag"
                        run_command "matrix-$command_name" "$label" "$command_name" "$args" "$BINARY $command_name $symbol --repo \"$repo_arg\" $json_flag $code_flag $exclude_flag $max_flag" "P2" "required"
                    done
                done
            done
        done

        run_replay_check "replay-$command_name" "$label" "$command_name" "$symbol --repo $repo_arg --json" "$BINARY $command_name $symbol --repo \"$repo_arg\" --json"
    done
}

run_context_matrix() {
    local label="$1"
    local repo_arg="$2"
    local symbol="$3"

    local budgets=(1 200 1200)
    local code_modes=("" "--code-only")
    local exclude_modes=("" "--exclude-tests")

    if [[ "$mode" == "full" ]]; then
        budgets+=(5000)
    fi

    for budget in "${budgets[@]}"; do
        for code_flag in "${code_modes[@]}"; do
            for exclude_flag in "${exclude_modes[@]}"; do
                local args="--task update-${symbol}-behavior --repo $repo_arg --budget $budget --json $code_flag $exclude_flag"
                run_command "matrix-context" "$label" "context" "$args" "$BINARY context --task \"update $symbol behavior\" --repo \"$repo_arg\" --budget $budget --json $code_flag $exclude_flag" "P2" "required"
            done
        done
    done

    run_replay_check "replay-context" "$label" "context" "--task update-${symbol}-behavior --repo $repo_arg --budget 1200 --json" "$BINARY context --task \"update $symbol behavior\" --repo \"$repo_arg\" --budget 1200 --json"
}

run_verify_diff_matrix() {
    local label="$1"
    local repo_arg="$2"
    local symbol="$3"
    local changed_file="$4"

    local changed_line="$changed_file:1:1"

    local verify_max_values=("" "--max-targeted 0" "--max-targeted 1" "--max-targeted 8")
    if [[ "$mode" == "full" ]]; then
        verify_max_values+=("--max-targeted 20")
    fi

    for max_flag in "${verify_max_values[@]}"; do
        run_command "matrix-verify-plan" "$label" "verify-plan" "--changed-file $changed_file --repo $repo_arg --json $max_flag" "$BINARY verify-plan --changed-file \"$changed_file\" --repo \"$repo_arg\" --json $max_flag" "P2" "required"
        run_command "matrix-verify-plan" "$label" "verify-plan" "--changed-file $changed_file --changed-line $changed_line --changed-symbol $symbol --repo $repo_arg --json $max_flag" "$BINARY verify-plan --changed-file \"$changed_file\" --changed-line \"$changed_line\" --changed-symbol \"$symbol\" --repo \"$repo_arg\" --json $max_flag" "P2" "required"
    done

    local max_distance_values=(0 1 2)
    if [[ "$mode" == "full" ]]; then
        max_distance_values+=(3)
    fi

    local test_modes=("" "--include-tests" "--exclude-tests")
    local max_results_values=("" "--max-results 0" "--max-results 1" "--max-results 10")
    if [[ "$mode" == "full" ]]; then
        max_results_values+=("--max-results 50")
    fi

    for max_distance in "${max_distance_values[@]}"; do
        for test_mode in "${test_modes[@]}"; do
            for max_results in "${max_results_values[@]}"; do
                local args="--changed-file $changed_file --repo $repo_arg --max-distance $max_distance $test_mode --include-imports --exclude-changed --changed-line $changed_line --changed-symbol $symbol $max_results --json"
                run_command "matrix-diff-impact" "$label" "diff-impact" "$args" "$BINARY diff-impact --changed-file \"$changed_file\" --repo \"$repo_arg\" --max-distance $max_distance $test_mode --include-imports --exclude-changed --changed-line \"$changed_line\" --changed-symbol \"$symbol\" $max_results --json" "P2" "required"
            done
        done
    done

    run_replay_check "replay-verify-plan" "$label" "verify-plan" "--changed-file $changed_file --repo $repo_arg --json" "$BINARY verify-plan --changed-file \"$changed_file\" --repo \"$repo_arg\" --json"
    run_replay_check "replay-diff-impact" "$label" "diff-impact" "--changed-file $changed_file --repo $repo_arg --json" "$BINARY diff-impact --changed-file \"$changed_file\" --repo \"$repo_arg\" --json"
}

run_corpus_matrix() {
    local label="$1"
    local repo_arg="$2"
    local symbol="$3"
    local changed_file="$4"
    local explain_symbol="$5"

    run_command "matrix-index" "$label" "index" "--repo $repo_arg" "$BINARY index --repo \"$repo_arg\"" "P1" "required"
    run_command "matrix-status" "$label" "status" "--repo $repo_arg" "$BINARY status --repo \"$repo_arg\"" "P2" "required"

    run_find_refs_matrix "$label" "$repo_arg" "$symbol"

    run_command "matrix-impact" "$label" "impact" "$symbol --repo $repo_arg" "$BINARY impact \"$symbol\" --repo \"$repo_arg\"" "P2" "required"
    run_command "matrix-impact" "$label" "impact" "$symbol --repo $repo_arg --json" "$BINARY impact \"$symbol\" --repo \"$repo_arg\" --json" "P2" "required"
    run_replay_check "replay-impact" "$label" "impact" "$symbol --repo $repo_arg --json" "$BINARY impact \"$symbol\" --repo \"$repo_arg\" --json"

    run_context_matrix "$label" "$repo_arg" "$symbol"

    run_command "matrix-tests-for" "$label" "tests-for" "$symbol --repo $repo_arg" "$BINARY tests-for \"$symbol\" --repo \"$repo_arg\"" "P2" "required"
    run_command "matrix-tests-for" "$label" "tests-for" "$symbol --repo $repo_arg --json" "$BINARY tests-for \"$symbol\" --repo \"$repo_arg\" --json" "P2" "required"
    run_command "matrix-tests-for" "$label" "tests-for" "$symbol --repo $repo_arg --include-support --json" "$BINARY tests-for \"$symbol\" --repo \"$repo_arg\" --include-support --json" "P2" "required"
    run_replay_check "replay-tests-for" "$label" "tests-for" "$symbol --repo $repo_arg --json" "$BINARY tests-for \"$symbol\" --repo \"$repo_arg\" --json"

    run_verify_diff_matrix "$label" "$repo_arg" "$symbol" "$changed_file"

    run_command "matrix-explain" "$label" "explain" "$explain_symbol --repo $repo_arg" "$BINARY explain \"$explain_symbol\" --repo \"$repo_arg\"" "P2" "required"
    run_command "matrix-explain" "$label" "explain" "$explain_symbol --repo $repo_arg --json" "$BINARY explain \"$explain_symbol\" --repo \"$repo_arg\" --json" "P2" "required"
    run_command "matrix-explain" "$label" "explain" "$explain_symbol --repo $repo_arg --json --include-snippets" "$BINARY explain \"$explain_symbol\" --repo \"$repo_arg\" --json --include-snippets" "P2" "required"
    run_replay_check "replay-explain" "$label" "explain" "$explain_symbol --repo $repo_arg --json" "$BINARY explain \"$explain_symbol\" --repo \"$repo_arg\" --json"
}

run_negative_checks() {
    run_expected_failure "negative" "workspace" "missing-args" "find" "$BINARY find"
    run_expected_failure "negative" "workspace" "missing-args" "refs" "$BINARY refs"
    run_expected_failure "negative" "workspace" "invalid-repo" "index --repo /definitely/missing/path" "$BINARY index --repo /definitely/missing/path"
    run_expected_failure "negative" "workspace" "verify-plan-invalid-changed-line" "--changed-file src/query/mod.rs --changed-line src/query/mod.rs:0 --repo ." "$BINARY verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:0 --repo ."
    run_expected_failure "negative" "workspace" "diff-impact-flag-conflict" "--changed-file src/query/mod.rs --include-tests --exclude-tests --repo ." "$BINARY diff-impact --changed-file src/query/mod.rs --include-tests --exclude-tests --repo ."
    run_expected_failure "negative" "workspace" "unknown-subcommand" "unknown-cmd" "$BINARY unknown-cmd"
}

run_required_gate_suite() {
    local commands=(
        "just check"
        "just docs-consistency ."
        "just phase18-docs-freshness ."
        "just phase18-maintenance-pack ."
        "just phase15-convergence-pack ."
        "just phase16-deterministic-replay ."
        "just phase16-benchmark-pack ."
        "just phase16-known-issues-budget ."
        "just phase16-release-checklist ."
        "just phase16-large-repo-benchmark ."
        "just phase16-large-repo-replay ."
    )

    if [[ "$mode" == "smoke" ]]; then
        commands=("just check" "just docs-consistency .")
    fi

    for command in "${commands[@]}"; do
        run_command "gates" "workspace" "$command" "$command" "cd \"$REPO_PATH\" && $command" "P1" "required"
    done
}

run_informational_tooling_lanes() {
    local python_tools=(ruff mypy pytest)
    local missing_python=0
    for tool in "${python_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            log_observation "info-python" "workspace" "tool-check" "$tool missing" "WARN" "P3" "waived" "@repo-scout-maintainers" "E2E-INFO-PYTHON"
            append_issue_entry "WARN" "waived" "info-python" "workspace" "tool-check" "$tool missing" "E2E-INFO-PYTHON" "Python inactive lane tool missing in local environment"
            missing_python=1
        fi
    done

    if [[ "$missing_python" -eq 0 ]]; then
        run_command "info-python" "workspace" "ruff-format" "format --check" "cd \"$REPO_PATH\" && ruff format --check ." "P3" "informational"
        run_command "info-python" "workspace" "ruff-check" "check" "cd \"$REPO_PATH\" && ruff check . --output-format=full" "P3" "informational"
        run_command "info-python" "workspace" "mypy" "." "cd \"$REPO_PATH\" && mypy ." "P3" "informational"
        run_command "info-python" "workspace" "pytest" "-q" "cd \"$REPO_PATH\" && pytest -q" "P3" "informational"
    fi

    local ts_tools=(node npm npx)
    local missing_ts=0
    for tool in "${ts_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            log_observation "info-typescript" "workspace" "tool-check" "$tool missing" "WARN" "P3" "waived" "@repo-scout-maintainers" "E2E-INFO-TS"
            append_issue_entry "WARN" "waived" "info-typescript" "workspace" "tool-check" "$tool missing" "E2E-INFO-TS" "TypeScript inactive lane tool missing in local environment"
            missing_ts=1
        fi
    done

    if [[ "$missing_ts" -eq 0 ]]; then
        run_command "info-typescript" "workspace" "tsc" "--noEmit" "cd \"$REPO_PATH\" && npx tsc --noEmit" "P3" "informational"
        run_command "info-typescript" "workspace" "eslint" "--max-warnings 0" "cd \"$REPO_PATH\" && npx eslint . --max-warnings 0" "P3" "informational"
        run_command "info-typescript" "workspace" "prettier" "--check" "cd \"$REPO_PATH\" && npx prettier --check ." "P3" "informational"
        run_command "info-typescript" "workspace" "npm-test" "test" "cd \"$REPO_PATH\" && npm test" "P3" "informational"
    fi
}

write_run_summary() {
    ensure_run_history_section
    {
        echo "- [$RUN_ID] summary: pass=$pass_count warn=$warning_count fail=$fail_count info=$info_count unresolved=$unresolved_count mode=$mode record=$record_mode"
    } >>"$ISSUES_LOG"
}

BINARY="$REPO_PATH/target/release/repo-scout"
log_observation "lifecycle" "workspace" "run-start" "--mode $mode --record $record_mode" "INFO" "P3" "resolved" "@repo-scout-maintainers" "none"
prepare_binary

run_negative_checks

run_corpus_matrix "workspace" "$REPO_PATH" "select_full_suite_command" "src/query/mod.rs" "select_full_suite_command"
run_corpus_matrix "rust" "$REPO_PATH/tests/fixtures/phase15/convergence_pack/rust" "phase63_plan" "src/lib.rs" "phase63_plan"
run_corpus_matrix "go" "$REPO_PATH/tests/fixtures/phase15/convergence_pack/go" "PlanPhase63" "src/service.go" "PlanPhase63"
run_corpus_matrix "python" "$REPO_PATH/tests/fixtures/phase15/convergence_pack/python" "plan_phase63" "src/service.py" "plan_phase63"
run_corpus_matrix "typescript_vitest" "$REPO_PATH/tests/fixtures/phase15/convergence_pack/typescript_vitest" "planPhase63" "src/service.ts" "planPhase63"
run_corpus_matrix "typescript_jest" "$REPO_PATH/tests/fixtures/phase15/convergence_pack/typescript_jest" "planPhase63" "src/service.ts" "planPhase63"

tsx_corpus="$(make_tsx_corpus)"
run_corpus_matrix "typescript_tsx_generated" "$tsx_corpus" "RenderPhase79" "src/widget.tsx" "RenderPhase79"
rm -rf "$tsx_corpus"

run_required_gate_suite
run_informational_tooling_lanes
write_run_summary
log_observation "lifecycle" "workspace" "run-end" "pass=$pass_count warn=$warning_count fail=$fail_count unresolved=$unresolved_count mode=$mode record=$record_mode" "INFO" "P3" "resolved" "@repo-scout-maintainers" "none"

if [[ "$record_mode" -eq 0 && "$unresolved_count" -gt 0 ]]; then
    echo "E2E release matrix detected unresolved findings: $unresolved_count" >&2
    exit 1
fi

echo "E2E release matrix completed: pass=$pass_count warn=$warning_count fail=$fail_count info=$info_count unresolved=$unresolved_count mode=$mode record=$record_mode"
