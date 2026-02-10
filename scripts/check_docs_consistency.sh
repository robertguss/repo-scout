#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_docs_consistency.sh [--repo <path>] [--record]

Options:
  --repo <path>       Repository root (default: .)
  --record            Print check statuses without pass/fail gating
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

README_PATH="$REPO_PATH/README.md"
ARCH_PATH="$REPO_PATH/docs/architecture.md"
CHANGELOG_PATH="$REPO_PATH/CHANGELOG.md"
PHASE9_PATH="$REPO_PATH/agents/plans/repo-scout-phase9-execplan.md"
PLANS_README_PATH="$REPO_PATH/agents/plans/README.md"
ROADMAP_PATH="$REPO_PATH/agents/plans/repo-scout-roadmap-to-production-and-ga.md"
PHASE16_PLAN_PATH="$REPO_PATH/agents/plans/repo-scout-phase16-execplan.md"

for path in \
    "$README_PATH" \
    "$ARCH_PATH" \
    "$CHANGELOG_PATH" \
    "$PHASE9_PATH" \
    "$PLANS_README_PATH" \
    "$ROADMAP_PATH" \
    "$PHASE16_PLAN_PATH"; do
    if [[ ! -f "$path" ]]; then
        echo "required file does not exist: $path" >&2
        exit 2
    fi
done

check_contains() {
    local file="$1"
    local needle="$2"
    local label="$3"

    if grep -Fq -- "$needle" "$file"; then
        if [[ "$record_mode" -eq 1 ]]; then
            printf 'MEASURE [%s] status=pass\n' "$label"
        else
            printf 'PASS [%s] status=pass\n' "$label"
        fi
    else
        if [[ "$record_mode" -eq 1 ]]; then
            printf 'MEASURE [%s] status=fail\n' "$label"
        else
            printf 'FAIL [%s] missing expected text: %s\n' "$label" "$needle" >&2
            exit 1
        fi
    fi
}

check_not_contains() {
    local file="$1"
    local needle="$2"
    local label="$3"

    if grep -Fq -- "$needle" "$file"; then
        if [[ "$record_mode" -eq 1 ]]; then
            printf 'MEASURE [%s] status=fail\n' "$label"
        else
            printf 'FAIL [%s] found prohibited text: %s\n' "$label" "$needle" >&2
            exit 1
        fi
    else
        if [[ "$record_mode" -eq 1 ]]; then
            printf 'MEASURE [%s] status=pass\n' "$label"
        else
            printf 'PASS [%s] status=pass\n' "$label"
        fi
    fi
}

check_contains "$README_PATH" "Phase 16 High-Bar/GA hardening is complete" \
    "readme_phase16_closure_status"
check_not_contains "$README_PATH" "Phase 16 is now in progress" \
    "readme_no_phase16_in_progress_text"

check_contains "$ARCH_PATH" "as of Phase 16 closure" "architecture_post_phase16_framing"
check_not_contains "$ARCH_PATH" "after Phase 14" "architecture_no_phase14_framing"

check_contains "$CHANGELOG_PATH" "## [Unreleased]" "changelog_unreleased_section"

check_contains "$PHASE9_PATH" "Superseded Status" "phase9_superseded_heading"
check_contains "$PHASE9_PATH" "closed via later implemented phases" \
    "phase9_superseded_closure_statement"
check_not_contains "$PHASE9_PATH" "- [ ] Milestone 42" "phase9_no_open_milestones"

check_contains "$PLANS_README_PATH" "Phase 9 Superseded Status" \
    "plans_inventory_phase9_policy"

check_contains "$ROADMAP_PATH" "Phase 16 (Completed): High-Bar / GA Hardening" \
    "roadmap_phase16_completed_state"
check_contains "$PHASE16_PLAN_PATH" "Phase 16 closure is complete." \
    "phase16_execplan_closure_statement"

if [[ "$record_mode" -eq 0 ]]; then
    echo "Docs consistency check passed."
fi
