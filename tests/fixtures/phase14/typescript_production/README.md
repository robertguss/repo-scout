# Phase 14 TypeScript Production Fixture

This fixture corpus exercises Phase 14 TypeScript production-closure behavior:

- strict runner-aware recommendation detection for `vitest` and `jest` contexts,
- strict fallback behavior when both Node runners are signaled (ambiguous context),
- TypeScript directory import resolution (`./util` -> `./util/index.ts`) for `diff-impact` caller attribution.
