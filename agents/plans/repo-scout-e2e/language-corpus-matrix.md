# Language And Corpus Matrix

This matrix defines corpus coverage and per-language checks.

## Required Corpora

- Workspace corpus: `.`
- Rust corpus: `tests/fixtures/phase15/convergence_pack/rust`
- Go corpus: `tests/fixtures/phase15/convergence_pack/go`
- Python corpus: `tests/fixtures/phase15/convergence_pack/python`
- TypeScript corpus (Vitest): `tests/fixtures/phase15/convergence_pack/typescript_vitest`
- TypeScript corpus (Jest): `tests/fixtures/phase15/convergence_pack/typescript_jest`

## Required Language Coverage

- rust
- go
- python
- typescript (`.ts`)
- typescript (`.tsx`)

## TSX Coverage Rule

The repository has no committed `.tsx` fixture corpus today, so TSX coverage is generated dynamically during matrix runs.

Generated TSX corpus requirements:

- one `.tsx` source file
- one `.test.tsx` test-like file
- minimal `package.json` runner signal
- index/find/refs/impact/context/tests-for/verify-plan/diff-impact/explain checks executed and logged

Generated artifacts are temporary and deleted after the run.
