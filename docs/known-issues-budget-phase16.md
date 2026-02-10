# Phase 16 Known-Issues Budget

This artifact defines the High-Bar known-issues budget and ownership expectations for Phase 16
release hardening.

## Budget Thresholds

- max_open: 0
- max_deferred: 0
- max_unowned: 0

## Tracked Issues

| id | summary | severity | decision | owner | target_phase | notes |
| --- | --- | --- | --- | --- | --- | --- |
| PH16-001 | Deterministic replay gate absent for cross-language command set | P2 | closed | @repo-scout-maintainers | phase16 | Closed by Milestone 64 (`check_phase16_deterministic_replay.sh`). |
| PH16-002 | No benchmark-pack timing guardrail for fixture-based High-Bar checks | P2 | closed | @repo-scout-maintainers | phase16 | Closed by Milestone 65 (`check_phase16_benchmark_pack.sh`). |
| PH16-003 | Larger-repo benchmark budget remains pending beyond fixture-pack coverage | P3 | closed | @repo-scout-maintainers | phase16 | Closed by Milestone 67 (`check_phase16_large_repo_benchmark.sh`) and Milestone 69 (`check_phase16_large_repo_replay.sh`) repository-scale hardening gates. |

## Triage Rules

- `open` issues are not permitted at Phase 16 exit (`max_open: 0`).
- `deferred` issues are not permitted at Phase 16 closure (`max_deferred: 0`).
- every tracked issue row must include an owner (`max_unowned: 0`).
