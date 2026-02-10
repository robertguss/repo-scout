#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase16_release_checklist.sh [--repo <path>] [--doc <path>] [--record]

Options:
  --repo <path>       Repository root (default: .)
  --doc <path>        Release checklist doc path relative to --repo
                      (default: docs/release-checklist-phase16.md)
  --record            Print parsed gate statuses without pass/fail gating
  -h, --help          Show this help text
USAGE
}

repo="."
doc="docs/release-checklist-phase16.md"
record_mode=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="${2:-}"
            shift 2
            ;;
        --doc)
            doc="${2:-}"
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
DOC_PATH="$(resolve_path "$REPO_PATH" "$doc")"

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

if [[ ! -f "$DOC_PATH" ]]; then
    echo "release checklist doc does not exist: $DOC_PATH" >&2
    exit 2
fi

parse_gate_status() {
    local key="$1"
    local value
    value="$(
        grep -E "^- ${key}:[[:space:]]*(pass|fail)$" "$DOC_PATH" \
            | head -n 1 \
            | sed -E "s/^- ${key}:[[:space:]]*(pass|fail)$/\\1/"
    )"

    if [[ -z "$value" ]]; then
        echo "missing or invalid gate key in doc: ${key}" >&2
        exit 1
    fi

    printf '%s\n' "$value"
}

QUALITY_GATE="$(parse_gate_status "quality_gate")"
EVIDENCE_GATE="$(parse_gate_status "evidence_gate")"
ROLLBACK_PLAN="$(parse_gate_status "rollback_plan")"
DOCS_GATE="$(parse_gate_status "docs_gate")"
CI_GATE="$(parse_gate_status "ci_gate")"

check_gate() {
    local label="$1"
    local status="$2"

    if [[ "$record_mode" -eq 1 ]]; then
        printf 'MEASURE [%s] status=%s\n' "$label" "$status"
        return
    fi

    if [[ "$status" == "pass" ]]; then
        printf 'PASS [%s] status=%s\n' "$label" "$status"
    else
        printf 'FAIL [%s] status=%s (expected pass)\n' "$label" "$status" >&2
        exit 1
    fi
}

check_gate "quality_gate" "$QUALITY_GATE"
check_gate "evidence_gate" "$EVIDENCE_GATE"
check_gate "rollback_plan" "$ROLLBACK_PLAN"
check_gate "docs_gate" "$DOCS_GATE"
check_gate "ci_gate" "$CI_GATE"

if [[ "$record_mode" -eq 0 ]]; then
    echo "Phase 16 release checklist check passed."
fi
