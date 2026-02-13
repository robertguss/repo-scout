# Release Process

## Versioning policy

- Semantic Versioning

## Changelog policy

- Keep a Changelog format in `CHANGELOG.md`
- Update `Unreleased` continuously
- Cut a dated release section at tag time

## Release checklist

1. Run full quality gates.
2. Verify docs build/check.
3. Confirm changelog entries.
4. Tag release.

Core commands:

```bash
just check
just contract-check
just docs-check
```
