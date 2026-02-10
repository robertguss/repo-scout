#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase18_docs_freshness.sh [--repo <path>] [--doc <path>] [--record]

Options:
  --repo <path>       Repository root (default: .)
  --doc <path>        Cadence document path relative to --repo
                      (default: docs/maintenance-cadence-phase18.md)
  --record            Print measurements without pass/fail gating
  -h, --help          Show this help text
USAGE
}

repo="."
doc="docs/maintenance-cadence-phase18.md"
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
    echo "docs freshness policy doc does not exist: $DOC_PATH" >&2
    exit 2
fi

parse_refresh_interval() {
    local value
    value="$(
        grep -E "^- refresh_interval_days:[[:space:]]*[0-9]+$" "$DOC_PATH" \
            | head -n 1 \
            | sed -E 's/^- refresh_interval_days:[[:space:]]*([0-9]+)$/\1/'
    )"

    if [[ -z "$value" ]]; then
        echo "missing or invalid cadence key: refresh_interval_days" >&2
        exit 1
    fi

    printf '%s\n' "$value"
}

REFRESH_INTERVAL_DAYS="$(parse_refresh_interval)"

if ! grep -Fq -- "| doc | last_reviewed | next_review_due | reviewer | status |" "$DOC_PATH"; then
    echo "missing required marker header row in $DOC_PATH" >&2
    exit 1
fi

read -r TRACKED_ROWS MISSING_FIELDS INVALID_DATES INVALID_WINDOWS STALE_ROWS < <(
    awk -F'|' '
    function trim(s) {
        gsub(/^[ \t]+|[ \t]+$/, "", s);
        return s;
    }

    /^\| docs\// {
        tracked += 1;
        last_reviewed = trim($3);
        next_review_due = trim($4);
        reviewer = trim($5);
        status = tolower(trim($6));

        if (last_reviewed == "" || next_review_due == "" || reviewer == "" || status == "") {
            missing_fields += 1;
        }

        if (last_reviewed !~ /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/ ||
            next_review_due !~ /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/) {
            invalid_dates += 1;
        } else if (next_review_due < last_reviewed) {
            invalid_windows += 1;
        }

        if (status != "current") {
            stale_rows += 1;
        }
    }

    END {
        printf "%d %d %d %d %d\n",
            tracked + 0,
            missing_fields + 0,
            invalid_dates + 0,
            invalid_windows + 0,
            stale_rows + 0;
    }
    ' "$DOC_PATH"
)

if [[ "$TRACKED_ROWS" -eq 0 ]]; then
    echo "no docs freshness rows found in $DOC_PATH (expected '| docs/... |' rows)" >&2
    exit 1
fi

if [[ "$record_mode" -eq 1 ]]; then
    printf 'MEASURE [refresh_interval_days] value=%s\n' "$REFRESH_INTERVAL_DAYS"
    printf 'MEASURE [tracked_rows] count=%s\n' "$TRACKED_ROWS"
    printf 'MEASURE [missing_fields] count=%s\n' "$MISSING_FIELDS"
    printf 'MEASURE [invalid_dates] count=%s\n' "$INVALID_DATES"
    printf 'MEASURE [invalid_windows] count=%s\n' "$INVALID_WINDOWS"
    printf 'MEASURE [stale_rows] count=%s\n' "$STALE_ROWS"
    exit 0
fi

printf 'PASS [%s] value=%s\n' "refresh_interval_days" "$REFRESH_INTERVAL_DAYS"
printf 'PASS [%s] count=%s\n' "tracked_rows" "$TRACKED_ROWS"

if [[ "$MISSING_FIELDS" -eq 0 ]]; then
    printf 'PASS [%s] count=%s\n' "missing_fields" "$MISSING_FIELDS"
else
    printf 'FAIL [%s] count=%s\n' "missing_fields" "$MISSING_FIELDS" >&2
    exit 1
fi

if [[ "$INVALID_DATES" -eq 0 ]]; then
    printf 'PASS [%s] count=%s\n' "invalid_dates" "$INVALID_DATES"
else
    printf 'FAIL [%s] count=%s\n' "invalid_dates" "$INVALID_DATES" >&2
    exit 1
fi

if [[ "$INVALID_WINDOWS" -eq 0 ]]; then
    printf 'PASS [%s] count=%s\n' "invalid_windows" "$INVALID_WINDOWS"
else
    printf 'FAIL [%s] count=%s\n' "invalid_windows" "$INVALID_WINDOWS" >&2
    exit 1
fi

if [[ "$STALE_ROWS" -eq 0 ]]; then
    printf 'PASS [%s] count=%s\n' "stale_rows" "$STALE_ROWS"
else
    printf 'FAIL [%s] count=%s\n' "stale_rows" "$STALE_ROWS" >&2
    exit 1
fi

echo "Phase 18 docs freshness check passed."
