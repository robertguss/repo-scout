# Contract Artifact Policy

## Canonical Strategy

Canonical strategy: PR-body-first.

Repository-level evidence is captured in `.github/pull_request_template.md` headings and validated by
repo scripts. Task framing, test intent, and evidence intent should be reflected in ExecPlans and
mapped to:

- `templates/TASK_PACKET_TEMPLATE.md`
- `templates/TEST_PLAN_TEMPLATE.md`
- `templates/EVIDENCE_PACKET_TEMPLATE.md`

## Required Validation Commands

Run these commands before opening or updating a PR:

- `scripts/validate_tdd_cycle.sh --base origin/main`
- `scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`

These validators are the authoritative enforcement mechanism for TDD sequencing and evidence quality
under the PR-body-first model.

## Optional Committed Evidence

Committed evidence files are optional. Use `.evidence/EVIDENCE_PACKET.md` only when a change requires
additional local transcripts that do not fit cleanly in the PR template.
