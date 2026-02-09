#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/validate_evidence_packet.sh [--file <path> | --pr-body <path>]

Validates evidence packet structure and section quality.

Options:
  --file <path>      Path to evidence packet markdown file
  --pr-body <path>   Path to file containing pull request body markdown
USAGE
}

section_body() {
  local heading="$1"
  awk -v heading="## $heading" '
    $0 == heading {capture=1; next}
    /^## / && capture {exit}
    capture {print}
  ' "$NORMALIZED_PATH"
}

section_has_field_with_value() {
  local heading="$1"
  local field_pattern="$2"
  local section
  section="$(section_body "$heading")"

  grep -Eiq "^[[:space:]-]*(${field_pattern})[[:space:]]*:[[:space:]]*[^[:space:]].*$" <<<"$section"
}

require_field_with_value() {
  local heading="$1"
  local field_pattern="$2"
  local human_label="$3"

  if ! section_has_field_with_value "$heading" "$field_pattern"; then
    echo "Missing quality detail in '$heading': $human_label" >&2
    return 1
  fi

  return 0
}

MODE=""
INPUT_PATH=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --file)
      MODE="file"
      INPUT_PATH="${2:-}"
      shift 2
      ;;
    --pr-body)
      MODE="pr"
      INPUT_PATH="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$MODE" ]]; then
  if [[ -f ".evidence/EVIDENCE_PACKET.md" ]]; then
    MODE="file"
    INPUT_PATH=".evidence/EVIDENCE_PACKET.md"
  else
    echo "No input provided and .evidence/EVIDENCE_PACKET.md not found." >&2
    usage
    exit 2
  fi
fi

if [[ -z "$INPUT_PATH" || ! -f "$INPUT_PATH" ]]; then
  echo "Input file not found: $INPUT_PATH" >&2
  exit 2
fi

NORMALIZED_PATH="$(mktemp)"
trap 'rm -f "$NORMALIZED_PATH"' EXIT
tr -d '\r' < "$INPUT_PATH" > "$NORMALIZED_PATH"

REQUIRED_HEADINGS=()
while IFS= read -r heading; do
  REQUIRED_HEADINGS+=("$heading")
done <<'EOF_HEADINGS'
## Objective
## Risk Tier
## Scope
## Red
## Green
## Refactor
## Invariants
## Security Impact
## Performance Impact
## Assumptions
## Open Questions
## Rollback Plan
## Validation Commands
EOF_HEADINGS

missing=0
for heading in "${REQUIRED_HEADINGS[@]}"; do
  if ! grep -qxF "$heading" "$NORMALIZED_PATH"; then
    echo "Missing required heading: $heading" >&2
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  echo "Evidence packet validation failed." >&2
  exit 1
fi

if grep -Eqi '<fill|tbd|todo|replace me>' "$NORMALIZED_PATH"; then
  echo "Evidence packet contains unresolved placeholders (e.g., TBD/TODO/<fill>)." >&2
  exit 1
fi

TEMPLATE_SCAFFOLD_MODE=0
if [[ "$MODE" == "pr" && "$(basename "$INPUT_PATH")" == "pull_request_template.md" ]]; then
  TEMPLATE_SCAFFOLD_MODE=1
fi

if [[ "$TEMPLATE_SCAFFOLD_MODE" -eq 0 ]]; then
  quality_failed=0

  require_field_with_value "Risk Tier" "Tier|Risk Tier" "declared tier" || quality_failed=1
  require_field_with_value "Risk Tier" "Rationale" "tier rationale" || quality_failed=1

  require_field_with_value "Red" "Failing test\\(s\\)|Failing tests" "failing test names" \
    || quality_failed=1
  require_field_with_value "Red" "Command\\(s\\)( used)?" "red-stage command" \
    || quality_failed=1
  require_field_with_value "Red" "Failure summary|Expected failure summary" \
    "failure summary" || quality_failed=1
  require_field_with_value "Red" "Why this failure is expected" "expected failure reason" \
    || quality_failed=1

  require_field_with_value "Green" "Minimal implementation summary" \
    "minimal implementation summary" || quality_failed=1
  require_field_with_value "Green" "Command\\(s\\)( used)?" "green-stage command" \
    || quality_failed=1
  require_field_with_value "Green" "Passing summary" "passing summary" || quality_failed=1

  require_field_with_value "Refactor" "Structural improvements|Structural improvements made" \
    "refactor summary" || quality_failed=1
  require_field_with_value "Refactor" "Why behavior is unchanged" \
    "behavior-preservation rationale" || quality_failed=1
  require_field_with_value "Refactor" "Confirmation commands|Command\\(s\\) used" \
    "post-refactor confirmation command" || quality_failed=1

  if [[ "$quality_failed" -ne 0 ]]; then
    echo "Evidence packet quality validation failed." >&2
    exit 1
  fi
fi

if [[ "$MODE" == "pr" ]]; then
  echo "PR body evidence validation passed."
else
  echo "Evidence packet file validation passed: $INPUT_PATH"
fi
