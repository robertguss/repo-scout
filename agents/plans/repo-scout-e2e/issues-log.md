# E2E Issues Log

This is the rolling, human-readable record of findings for the e2e release matrix.

## Open Findings

- None currently.

## Resolved Findings

- `E2E-RUNNER-001` (resolved): JSON escaping used a GNU sed label loop that is not portable on macOS/BSD sed.
  - reproduction: `bash scripts/run_e2e_release_matrix.sh --repo . --mode smoke --record`
  - observed error: `sed: ... unused label 'a;N;$!ba;s/\n/\\n/g'`
  - resolution: switched to portable newline escaping pipeline using `tr '\n' '\r'` plus `sed 's/\r/\\n/g'`.

- `E2E-RUNNER-002` (resolved): issue-log command/args formatting used backticks in double-quoted `echo`, triggering command substitution.
  - reproduction: `bash scripts/run_e2e_release_matrix.sh --repo . --mode smoke --record`
  - observed error: `diff-impact: command not found` while writing run-history details.
  - resolution: replaced backtick formatting with `printf` placeholder output for command and args fields.

- `E2E-RUNNER-003` (resolved): `diff-impact` matrix default test mode passed the literal token `default`, causing invalid CLI arguments.
  - reproduction: `bash scripts/run_e2e_release_matrix.sh --repo . --mode smoke --record`
  - observed error: `error: unexpected argument 'default' found`
  - resolution: changed default test-mode token from `"default"` to empty string in the matrix.

- `E2E-ENV-002` (resolved): local environment missing `pytest` in PATH.
  - reproduction: `pytest --version`
  - resolution: issue acknowledged for local setup profile; not a repository defect.

## Waived Findings

- `E2E-ENV-001` (waived): Codex shell snapshot warning appears before some bash runs.
  - reproduction: execute some bash commands with login shell in Codex desktop context.
  - waiver rationale: environment/tooling wrapper behavior does not impact command outcomes.
  - owner: `@repo-scout-maintainers`
  - expiry: `2026-06-30`

## Run History
- [run-20260210T105041Z] result=FAIL status=open lane=matrix-diff-impact corpus=workspace followup_id=E2E-FAIL-matrix-diff-impact
  - command: \
  - args: \
  - reproduction: run the command exactly as listed above
  - details: error: unexpected argument 'default' found

Usage: repo-scout diff-impact [OPTIONS] --changed-file <CHANGED_FILES> --repo <REPO>

For more information, try '--help'.
- [run-20260210T105154Z] result=FAIL status=open lane=matrix-diff-impact corpus=workspace followup_id=E2E-FAIL-matrix-diff-impact
  - command: `diff-impact`
  - args: `--changed-file src/query/mod.rs --repo /Users/robertguss/Projects/experiments/repo-scout/. --max-distance 0 default --include-imports --exclude-changed --changed-line src/query/mod.rs:1:1 --changed-symbol select_full_suite_command  --json`
  - reproduction: run the command exactly as listed above
  - details: error: unexpected argument 'default' found

Usage: repo-scout diff-impact [OPTIONS] --changed-file <CHANGED_FILES> --repo <REPO>

For more information, try '--help'.
- [run-20260210T105223Z] result=WARN status=waived lane=info-python corpus=workspace followup_id=E2E-INFO-PYTHON
  - command: `tool-check`
  - args: `mypy missing`
  - reproduction: run the command exactly as listed above
  - details: Python inactive lane tool missing in local environment
- [run-20260210T105223Z] result=WARN status=waived lane=info-python corpus=workspace followup_id=E2E-INFO-PYTHON
  - command: `tool-check`
  - args: `pytest missing`
  - reproduction: run the command exactly as listed above
  - details: Python inactive lane tool missing in local environment
- [run-20260210T105223Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `tsc`
  - args: `--noEmit`
  - reproduction: run the command exactly as listed above
  - details: 
[41m                                                                               [0m
[41m[37m                This is not the tsc command you are looking for                [0m
[41m                                                                               [0m

To get access to the TypeScript compiler, [34mtsc[0m, from the command line either:

- Use [1mnpm install typescript[0m to first add TypeScript to your project [1mbefore[0m using npx
- Use [1myarn[0m to avoid accidentally running code from un-installed packages
- [run-20260210T105223Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `eslint`
  - args: `--max-warnings 0`
  - reproduction: run the command exactly as listed above
  - details: npm warn exec The following package was not found and will be installed: eslint@10.0.0

Oops! Something went wrong! :(

ESLint: 10.0.0

ESLint couldn't find an eslint.config.(js|mjs|cjs) file.

From ESLint v9.0.0, the default configuration file is now eslint.config.js.
If you are using a .eslintrc.* file, please follow the migration guide
to update your configuration file to the new format:

https://eslint.org/docs/latest/use/configure/migration-guide

If you still have problems after following the migration guide, please stop by
https://eslint.org/chat/help to chat with the team.
- [run-20260210T105223Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `prettier`
  - args: `--check`
  - reproduction: run the command exactly as listed above
  - details: Checking formatting...
[warn] .github/pull_request_template.md
[warn] .github/workflows/contract-gates.yml
[warn] AGENTS.md
[warn] agents/PLANS.md
[warn] agents/plans/README.md
[warn] agents/plans/repo-scout-e2e/command-matrix.md
[warn] agents/plans/repo-scout-e2e/gates-and-tooling-matrix.md
[warn] agents/plans/repo-scout-e2e/issues-log.md
[warn] agents/plans/repo-scout-e2e/language-corpus-matrix.md
[warn] agents/plans/repo-scout-e2e/README.md
[warn] agents/plans/repo-scout-phase10-execplan.md
[warn] agents/plans/repo-scout-phase11-execplan.md
[warn] agents/plans/repo-scout-phase12-execplan.md
[warn] agents/plans/repo-scout-phase13-execplan.md
[warn] agents/plans/repo-scout-phase14-execplan.md
[warn] agents/plans/repo-scout-phase15-execplan.md
[warn] agents/plans/repo-scout-phase16-execplan.md
[warn] agents/plans/repo-scout-phase17-execplan.md
[warn] agents/plans/repo-scout-phase18-execplan.md
[warn] agents/plans/repo-scout-phase9-execplan.md
[warn] agents/plans/repo-scout-roadmap-to-production-and-ga.md
[warn] agents/tiger-style-audit/01-source-of-truth-and-method.md
[warn] agents/tiger-style-audit/02-contract-installation-drift.md
[warn] agents/tiger-style-audit/03-src-compliance-report-and-plan.md
[warn] agents/tiger-style-audit/04-tests-compliance-report-and-plan.md
[warn] agents/tiger-style-audit/05-process-ci-docs-compliance-report-and-plan.md
[warn] agents/tiger-style-audit/06-tiger-style-framework-feedback.md
[warn] agents/tiger-style-audit/07-appendix-evidence-snapshots.md
[warn] agents/tiger-style-audit/08-implementation-session-prompt.md
[warn] agents/tiger-style-audit/README.md
[warn] CHANGELOG.md
[warn] checklists/PR_CONTRACT_CHECKLIST.md
[warn] contracts/core/AI_AGENT_CORE_CONTRACT.md
[warn] docs/architecture.md
[warn] docs/cli-reference.md
[warn] docs/contract-artifact-policy.md
[warn] docs/dogfood-log.md
[warn] docs/json-output.md
[warn] docs/known-issues-budget-phase16.md
[warn] docs/maintenance-backlog-phase18.md
[warn] docs/maintenance-cadence-phase18.md
[warn] docs/performance-baseline.md
[warn] docs/performance-thresholds-rust.md
[warn] docs/release-checklist-phase16.md
[warn] README.md
[warn] tests/fixtures/phase10/go_find/src/app.ts
[warn] tests/fixtures/phase14/typescript_production/README.md
[warn] Code style issues found in 47 files. Run Prettier with --write to fix.
- [run-20260210T105223Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `npm-test`
  - args: `test`
  - reproduction: run the command exactly as listed above
  - details: npm error code ENOENT
npm error syscall open
npm error path /Users/robertguss/Projects/experiments/repo-scout/package.json
npm error errno -2
npm error enoent Could not read package.json: Error: ENOENT: no such file or directory, open '/Users/robertguss/Projects/experiments/repo-scout/package.json'
npm error enoent This is related to npm not being able to find a file.
npm error enoent
npm error A complete log of this run can be found in: /Users/robertguss/.npm/_logs/2026-02-10T10_53_07_209Z-debug-0.log
- [run-20260210T105223Z] summary: pass=975 warn=6 fail=0 info=1 unresolved=0 mode=smoke record=1
- [run-20260210T105804Z] result=WARN status=waived lane=info-python corpus=workspace followup_id=E2E-INFO-PYTHON
  - command: `tool-check`
  - args: `mypy missing`
  - reproduction: run the command exactly as listed above
  - details: Python inactive lane tool missing in local environment
- [run-20260210T105804Z] result=WARN status=waived lane=info-python corpus=workspace followup_id=E2E-INFO-PYTHON
  - command: `tool-check`
  - args: `pytest missing`
  - reproduction: run the command exactly as listed above
  - details: Python inactive lane tool missing in local environment
- [run-20260210T105804Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `tsc`
  - args: `--noEmit`
  - reproduction: run the command exactly as listed above
  - details: 
[41m                                                                               [0m
[41m[37m                This is not the tsc command you are looking for                [0m
[41m                                                                               [0m

To get access to the TypeScript compiler, [34mtsc[0m, from the command line either:

- Use [1mnpm install typescript[0m to first add TypeScript to your project [1mbefore[0m using npx
- Use [1myarn[0m to avoid accidentally running code from un-installed packages
- [run-20260210T105804Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `eslint`
  - args: `--max-warnings 0`
  - reproduction: run the command exactly as listed above
  - details: 
Oops! Something went wrong! :(

ESLint: 10.0.0

ESLint couldn't find an eslint.config.(js|mjs|cjs) file.

From ESLint v9.0.0, the default configuration file is now eslint.config.js.
If you are using a .eslintrc.* file, please follow the migration guide
to update your configuration file to the new format:

https://eslint.org/docs/latest/use/configure/migration-guide

If you still have problems after following the migration guide, please stop by
https://eslint.org/chat/help to chat with the team.
- [run-20260210T105804Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `prettier`
  - args: `--check`
  - reproduction: run the command exactly as listed above
  - details: Checking formatting...
[warn] .github/pull_request_template.md
[warn] .github/workflows/contract-gates.yml
[warn] AGENTS.md
[warn] agents/PLANS.md
[warn] agents/plans/README.md
[warn] agents/plans/repo-scout-e2e/command-matrix.md
[warn] agents/plans/repo-scout-e2e/gates-and-tooling-matrix.md
[warn] agents/plans/repo-scout-e2e/issues-log.md
[warn] agents/plans/repo-scout-e2e/language-corpus-matrix.md
[warn] agents/plans/repo-scout-e2e/README.md
[warn] agents/plans/repo-scout-phase10-execplan.md
[warn] agents/plans/repo-scout-phase11-execplan.md
[warn] agents/plans/repo-scout-phase12-execplan.md
[warn] agents/plans/repo-scout-phase13-execplan.md
[warn] agents/plans/repo-scout-phase14-execplan.md
[warn] agents/plans/repo-scout-phase15-execplan.md
[warn] agents/plans/repo-scout-phase16-execplan.md
[warn] agents/plans/repo-scout-phase17-execplan.md
[warn] agents/plans/repo-scout-phase18-execplan.md
[warn] agents/plans/repo-scout-phase9-execplan.md
[warn] agents/plans/repo-scout-roadmap-to-production-and-ga.md
[warn] agents/tiger-style-audit/01-source-of-truth-and-method.md
[warn] agents/tiger-style-audit/02-contract-installation-drift.md
[warn] agents/tiger-style-audit/03-src-compliance-report-and-plan.md
[warn] agents/tiger-style-audit/04-tests-compliance-report-and-plan.md
[warn] agents/tiger-style-audit/05-process-ci-docs-compliance-report-and-plan.md
[warn] agents/tiger-style-audit/06-tiger-style-framework-feedback.md
[warn] agents/tiger-style-audit/07-appendix-evidence-snapshots.md
[warn] agents/tiger-style-audit/08-implementation-session-prompt.md
[warn] agents/tiger-style-audit/README.md
[warn] CHANGELOG.md
[warn] checklists/PR_CONTRACT_CHECKLIST.md
[warn] contracts/core/AI_AGENT_CORE_CONTRACT.md
[warn] docs/architecture.md
[warn] docs/cli-reference.md
[warn] docs/contract-artifact-policy.md
[warn] docs/dogfood-log.md
[warn] docs/json-output.md
[warn] docs/known-issues-budget-phase16.md
[warn] docs/maintenance-backlog-phase18.md
[warn] docs/maintenance-cadence-phase18.md
[warn] docs/performance-baseline.md
[warn] docs/performance-thresholds-rust.md
[warn] docs/release-checklist-phase16.md
[warn] README.md
[warn] tests/fixtures/phase10/go_find/src/app.ts
[warn] tests/fixtures/phase14/typescript_production/README.md
[warn] Code style issues found in 47 files. Run Prettier with --write to fix.
- [run-20260210T105804Z] result=WARN status=waived lane=info-typescript corpus=workspace followup_id=E2E-INFO-info-typescript
  - command: `npm-test`
  - args: `test`
  - reproduction: run the command exactly as listed above
  - details: npm error code ENOENT
npm error syscall open
npm error path /Users/robertguss/Projects/experiments/repo-scout/package.json
npm error errno -2
npm error enoent Could not read package.json: Error: ENOENT: no such file or directory, open '/Users/robertguss/Projects/experiments/repo-scout/package.json'
npm error enoent This is related to npm not being able to find a file.
npm error enoent
npm error A complete log of this run can be found in: /Users/robertguss/.npm/_logs/2026-02-10T10_58_56_552Z-debug-0.log
- [run-20260210T105804Z] summary: pass=1306 warn=6 fail=0 info=1 unresolved=0 mode=full record=0
