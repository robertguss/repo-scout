# Phase 8 semantic precision fixture

This fixture stresses duplicate-name alias imports across TypeScript and Python.

It includes four caller paths per language:

- namespace/module alias call to `pkg_a`/`util_a`
- namespace/module alias call to `pkg_b`/`util_b`
- direct alias-import call to `pkg_a`/`util_a`
- direct alias-import call to `pkg_b`/`util_b`

`diff-impact --changed-file src/util_a.ts` and `diff-impact --changed-file src/pkg_a/util.py` should
include only the `*_a` callers (not `*_b`).
