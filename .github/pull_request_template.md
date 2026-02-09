## Objective

- Problem solved:
- Intended outcome:

## Risk Tier

- Tier: `0 | 1 | 2 | 3`
- Rationale:

## Scope

- Files/components changed:
- Explicit exclusions:

## Red

- Failing test(s):
- Command(s):
- Expected failure summary:
- Why this failure is expected:

## Green

- Minimal implementation summary:
- Command(s):
- Passing summary:

## Refactor

- Structural improvements:
- Why behavior is unchanged:
- Confirmation commands:

## Invariants

- Invariants added/updated:
- Boundary checks added/updated:

## Security Impact

- Threats considered:
- Mitigations:
- Residual risk:

## Performance Impact

- Baseline:
- Post-change:
- Delta explanation:

## Assumptions

1. Assumption 1

## Open Questions

1. Question 1

## Rollback Plan

- Trigger conditions:
- Rollback steps:

## Validation Commands

```bash
# red

# green

# full validation
```

## Contract Attestations

- [ ] Completed `checklists/PR_CONTRACT_CHECKLIST.md`.
- [ ] Completed `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` for Tier 2/3 work, or N/A.
- [ ] If a contract exception is used, it is documented with:
  - waived rule,
  - justification,
  - risk introduced,
  - compensating controls,
  - expiration/removal condition,
  - explicit reviewer approval.
- [ ] No contract exception was required for this change.

## Dogfooding Evidence (repo-scout required)

- [ ] Ran pre-edit dogfood loop:
  - `cargo run -- index --repo .`
  - `cargo run -- find <symbol> --repo . --json`
  - `cargo run -- refs <symbol> --repo . --json`
- [ ] Ran post-edit dogfood loop:
  - `cargo run -- index --repo .`
  - `cargo run -- find <symbol> --repo .`
  - `cargo run -- refs <symbol> --repo .`
  - `cargo test`
- [ ] Added or updated an entry in `docs/dogfood-log.md` if any issue was found.

Dogfood transcripts or links:

-

## Docs and Plans

- [ ] Updated relevant docs (`README.md`, `docs/`, or both).
- [ ] Updated relevant plan artifacts under `agents/` when behavior changed.
