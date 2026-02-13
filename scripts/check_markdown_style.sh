#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_markdown_style.sh [--repo <path>]

Checks basic markdown hygiene:
- no tabs
- no trailing whitespace
USAGE
}

repo="."
while [[ $# -gt 0 ]]; do
    case "$1" in
        --repo)
            if [[ -z "${2:-}" ]]; then
                echo "--repo requires a non-empty path argument" >&2
                usage >&2
                exit 2
            fi
            repo="$2"
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
if [[ "$repo" = /* ]]; then
    REPO_PATH="$repo"
else
    REPO_PATH="$ROOT_DIR/$repo"
fi

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

status=0

if rg -n '\t' "$REPO_PATH/README.md" "$REPO_PATH/CHANGELOG.md" "$REPO_PATH/docs" --glob '*.md' >/dev/null 2>&1; then
    echo "Markdown style check failed: tab characters found." >&2
    rg -n '\t' "$REPO_PATH/README.md" "$REPO_PATH/CHANGELOG.md" "$REPO_PATH/docs" --glob '*.md' || true
    status=1
fi

if rg -n ' +$' "$REPO_PATH/README.md" "$REPO_PATH/CHANGELOG.md" "$REPO_PATH/docs" --glob '*.md' >/dev/null 2>&1; then
    echo "Markdown style check failed: trailing whitespace found." >&2
    rg -n ' +$' "$REPO_PATH/README.md" "$REPO_PATH/CHANGELOG.md" "$REPO_PATH/docs" --glob '*.md' || true
    status=1
fi

if [[ "$status" -ne 0 ]]; then
    exit "$status"
fi

echo "Markdown style check passed."
