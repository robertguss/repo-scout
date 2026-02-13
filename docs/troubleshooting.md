# Troubleshooting

## `repo-scout` cannot find symbols

- Ensure index exists and is up to date:

```bash
repo-scout index --repo <REPO>
```

- Retry in JSON mode to inspect full metadata:

```bash
repo-scout find <SYMBOL> --repo <REPO> --json
```

## Command results feel noisy

Use scope controls on `find`/`refs`:

```bash
repo-scout refs <SYMBOL> --repo <REPO> --code-only --exclude-tests --max-results 20
```

## Changed files but stale impact output

Re-index after edits:

```bash
repo-scout index --repo <REPO>
```

Then rerun `diff-impact` / `verify-plan`.

## Docs build issues

Run:

```bash
just docs-build
just docs-check
```
