#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase16_known_issues_budget.sh [--repo <path>] [--doc <path>] [--record]

Options:
  --repo <path>       Repository root (default: .)
  --doc <path>        Known-issues budget document path relative to --repo
                      (default: docs/known-issues-budget-phase16.md)
  --record            Print parsed counts and thresholds without pass/fail gating
  -h, --help          Show this help text
USAGE
}

repo="."
doc="docs/known-issues-budget-phase16.md"
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
    echo "known-issues budget doc does not exist: $DOC_PATH" >&2
    exit 2
fi

parse_threshold() {
    local key="$1"
    local value
    value="$(
        grep -E "^- ${key}:[[:space:]]*[0-9]+$" "$DOC_PATH" \
            | head -n 1 \
            | sed -E "s/^- ${key}:[[:space:]]*([0-9]+)$/\\1/"
    )"

    if [[ -z "$value" ]]; then
        echo "missing or invalid threshold key in doc: ${key}" >&2
        exit 1
    fi

    printf '%s\n' "$value"
}

MAX_OPEN="$(parse_threshold "max_open")"
MAX_DEFERRED="$(parse_threshold "max_deferred")"
MAX_UNOWNED="$(parse_threshold "max_unowned")"

read -r TOTAL_ISSUES OPEN_ISSUES DEFERRED_ISSUES UNOWNED_ISSUES < <(
    awk -F'|' '
    function trim(s) {
        gsub(/^[ \t]+|[ \t]+$/, "", s);
        return s;
    }
    /^\| PH16-/ {
        total += 1;
        decision = tolower(trim($5));
        owner = trim($6);
        owner_lc = tolower(owner);

        if (decision == "open") {
            open += 1;
        }
        if (decision == "deferred") {
            deferred += 1;
        }
        if (owner == "" || owner_lc == "tbd" || owner_lc == "unassigned" || owner_lc == "none") {
            unowned += 1;
        }
    }
    END {
        printf "%d %d %d %d\n", total + 0, open + 0, deferred + 0, unowned + 0;
    }
    ' "$DOC_PATH"
)

if [[ "$TOTAL_ISSUES" -eq 0 ]]; then
    echo "no Phase 16 issue rows found in $DOC_PATH (expected '| PH16-... |' rows)" >&2
    exit 1
fi

if [[ "$record_mode" -eq 1 ]]; then
    printf 'MEASURE [known_issues_total] count=%s\n' "$TOTAL_ISSUES"
    printf 'MEASURE [open] count=%s threshold=%s\n' "$OPEN_ISSUES" "$MAX_OPEN"
    printf 'MEASURE [deferred] count=%s threshold=%s\n' "$DEFERRED_ISSUES" "$MAX_DEFERRED"
    printf 'MEASURE [unowned] count=%s threshold=%s\n' "$UNOWNED_ISSUES" "$MAX_UNOWNED"
    exit 0
fi

if [[ "$OPEN_ISSUES" -le "$MAX_OPEN" ]]; then
    printf 'PASS [open] count=%s <= threshold=%s\n' "$OPEN_ISSUES" "$MAX_OPEN"
else
    printf 'FAIL [open] count=%s > threshold=%s\n' "$OPEN_ISSUES" "$MAX_OPEN" >&2
    exit 1
fi

if [[ "$DEFERRED_ISSUES" -le "$MAX_DEFERRED" ]]; then
    printf 'PASS [deferred] count=%s <= threshold=%s\n' "$DEFERRED_ISSUES" "$MAX_DEFERRED"
else
    printf 'FAIL [deferred] count=%s > threshold=%s\n' "$DEFERRED_ISSUES" "$MAX_DEFERRED" >&2
    exit 1
fi

if [[ "$UNOWNED_ISSUES" -le "$MAX_UNOWNED" ]]; then
    printf 'PASS [unowned] count=%s <= threshold=%s\n' "$UNOWNED_ISSUES" "$MAX_UNOWNED"
else
    printf 'FAIL [unowned] count=%s > threshold=%s\n' "$UNOWNED_ISSUES" "$MAX_UNOWNED" >&2
    exit 1
fi

echo "Phase 16 known-issues budget check passed."
