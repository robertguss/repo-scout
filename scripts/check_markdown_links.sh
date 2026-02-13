#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage: check_markdown_links.sh [--repo <path>]

Checks relative Markdown links in README.md, CHANGELOG.md, and docs/*.md.
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
if [[ "$repo" = /* ]]; then
    REPO_PATH="$repo"
else
    REPO_PATH="$ROOT_DIR/$repo"
fi

if [[ ! -d "$REPO_PATH" ]]; then
    echo "repo path does not exist: $REPO_PATH" >&2
    exit 2
fi

failures=0

check_file() {
    local file="$1"
    local dir
    dir="$(dirname "$file")"

    while IFS= read -r link; do
        local target="$link"

        target="${target%%#*}"
        [[ -z "$target" ]] && continue

        case "$target" in
            http://*|https://*|mailto:*|tel:*)
                continue
                ;;
        esac

        local resolved
        if [[ "$target" = /* ]]; then
            resolved="$REPO_PATH/$target"
        else
            resolved="$dir/$target"
        fi

        if [[ ! -e "$resolved" ]]; then
            echo "BROKEN LINK: $file -> $target" >&2
            failures=$((failures + 1))
        fi
    done < <(sed -nE 's/.*\[[^][]+\]\(([^)]+)\).*/\1/p' "$file")
}

while IFS= read -r md; do
    check_file "$md"
done < <(cd "$REPO_PATH" && {
    [[ -f README.md ]] && echo "$REPO_PATH/README.md"
    [[ -f CHANGELOG.md ]] && echo "$REPO_PATH/CHANGELOG.md"
    rg --files docs -g '*.md' | sed "s#^#$REPO_PATH/#"
})

if [[ "$failures" -ne 0 ]]; then
    echo "Markdown link check failed with $failures broken link(s)." >&2
    exit 1
fi

echo "Markdown link check passed."
