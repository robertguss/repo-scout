## Summary

Describe the change in 2-5 sentences.

## What Changed

- 

## Strict TDD Evidence (Required)

- [ ] Every feature slice started with a failing test (red).
- [ ] Every feature slice was implemented with minimal code to pass (green).
- [ ] Every feature slice finished with full-suite pass after refactor.
- [ ] Red/green/refactor transcripts are included below or linked from planning artifacts.

Red/green/refactor transcripts or links:

- 

## Dogfooding Evidence (Required)

- [ ] Ran pre-edit dogfood loop:
  - `cargo run -- index --repo .`
  - `cargo run -- find <symbol> --repo . --json`
  - `cargo run -- refs <symbol> --repo . --json`
- [ ] Ran post-edit dogfood loop:
  - `cargo run -- index --repo .`
  - `cargo run -- find <symbol> --repo .`
  - `cargo run -- refs <symbol> --repo .`
  - `cargo test`
- [ ] Added or updated at least one entry in `docs/dogfood-log.md` if any issue was found.

Dogfood transcripts or links:

- 

## Validation

- [ ] `cargo test`
- [ ] Additional milestone or targeted commands:

Commands/output:

- 

## Docs and Plans

- [ ] Updated relevant docs (`README.md`, `docs/`, or both).
- [ ] Updated relevant plan artifacts under `agents/` when behavior changed.
