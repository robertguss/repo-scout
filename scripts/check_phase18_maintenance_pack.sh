#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_phase18_maintenance_pack.sh [--repo <path>]

Options:
  --repo <path>       Repository root (default: .)
  -h, --help          Show this help text
USAGE
}

repo="."

while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            repo="${2:-}"
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

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

check_exists() {
    local path="$1"
    local label="$2"
    if [[ -f "$path" ]]; then
        printf 'PASS [%s] exists=%s\n' "$label" "yes"
    else
        printf 'FAIL [%s] exists=%s\n' "$label" "no" >&2
        exit 1
    fi
}

check_exists "$REPO_PATH/docs/maintenance-backlog-phase18.md" "maintenance-backlog-phase18.md"
check_exists "$REPO_PATH/scripts/check_docs_consistency.sh" "check_docs_consistency.sh"
check_exists "$REPO_PATH/scripts/check_phase18_docs_freshness.sh" "check_phase18_docs_freshness.sh"

if bash "$REPO_PATH/scripts/check_docs_consistency.sh" --repo "$REPO_PATH" >/dev/null 2>&1; then
    printf 'PASS [%s] status=pass\n' "docs_consistency_gate"
else
    printf 'FAIL [%s] status=fail\n' "docs_consistency_gate" >&2
    exit 1
fi

if bash "$REPO_PATH/scripts/check_phase18_docs_freshness.sh" --repo "$REPO_PATH" >/dev/null 2>&1; then
    printf 'PASS [%s] status=pass\n' "phase18_docs_freshness_gate"
else
    printf 'FAIL [%s] status=fail\n' "phase18_docs_freshness_gate" >&2
    exit 1
fi

echo "Phase 18 maintenance pack check passed."
