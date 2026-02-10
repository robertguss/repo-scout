# Runbook

## Baseline Run

Use the strict full matrix for baseline:

- `bash scripts/run_e2e_release_matrix.sh --repo . --mode full`

## Ongoing Cadence

- Per change: smoke run
  - `bash scripts/run_e2e_release_matrix.sh --repo . --mode smoke`
- Scheduled full run: weekly full matrix
- Pre-release full run: strict full matrix + required gate suite

## Modes

- `smoke`: fast representative coverage for routine validation.
- `full`: exhaustive cross-product validation and full gate execution.

## Sign-off Workflow

- Update `issues-log.md` and `observations.jsonl`.
- Confirm there are zero unresolved findings.
- If a warning/failure is deferred, move it to waived with owner, rationale, and expiry.

Keywords: baseline, ongoing, smoke, full, sign-off.
