# CLI Reference

Top-level help:

```bash
repo-scout --help
```

## Core indexing and navigation

### `index`

Build/update the local index.

```bash
repo-scout index --repo <REPO>
```

### `status`

Show index status and health metadata.

```bash
repo-scout status --repo <REPO>
```

### `find`

Find symbol definitions.

```bash
repo-scout find <SYMBOL> --repo <REPO> [--json] [--code-only] [--exclude-tests] [--max-results <N>] [--compact]
```

### `refs`

Find references to a symbol.

```bash
repo-scout refs <SYMBOL> --repo <REPO> [--json] [--code-only] [--exclude-tests] [--max-results <N>] [--compact]
```

### `impact`

Show direct dependents/impact for a symbol.

```bash
repo-scout impact <SYMBOL> --repo <REPO> [--json]
```

### `context`

Rank files/symbols relevant to a task description.

```bash
repo-scout context --task <TASK> --repo <REPO> [--budget <N>] [--json] [--code-only] [--exclude-tests]
```

### `tests-for`

Suggest tests related to a symbol.

```bash
repo-scout tests-for <SYMBOL> --repo <REPO> [--json] [--include-support]
```

## Change analysis

### `verify-plan`

Generate test/verification plan from changed files/lines/symbols.

```bash
repo-scout verify-plan --repo <REPO> [--changed-file <PATH>] [--changed-line <SPEC>] [--changed-symbol <SYMBOL>] [--since <REV>] [--unstaged] [--max-targeted <N>] [--json]
```

### `diff-impact`

Compute change blast radius from file or symbol deltas.

```bash
repo-scout diff-impact --repo <REPO> [--changed-file <PATH>] [--changed-line <SPEC>] [--changed-symbol <SYMBOL>] [--since <REV>] [--unstaged] [--max-distance <N>] [--max-results <N>] [--no-limit] [--include-tests] [--exclude-tests] [--include-imports] [--exclude-changed] [--json]
```

## Deep inspection

### `explain`

Detailed symbol dossier.

```bash
repo-scout explain <SYMBOL> --repo <REPO> [--json] [--include-snippets] [--compact]
```

### `snippet`

Extract source snippet for a symbol.

```bash
repo-scout snippet <SYMBOL> --repo <REPO> [--json] [--context <LINES>]
```

### `outline`

Show file structure/signatures.

```bash
repo-scout outline <FILE> --repo <REPO> [--json]
```

### `summary`

Whole-repo structural overview.

```bash
repo-scout summary --repo <REPO>
```

## Graph and relationship commands

### `callers`

```bash
repo-scout callers <SYMBOL> --repo <REPO> [--json]
```

### `callees`

```bash
repo-scout callees <SYMBOL> --repo <REPO> [--json]
```

### `deps`

```bash
repo-scout deps <FILE> --repo <REPO> [--json]
```

### `hotspots`

```bash
repo-scout hotspots --repo <REPO> [--limit <N>] [--json]
```

### `call-path`

```bash
repo-scout call-path <FROM> <TO> --repo <REPO> [--max-depth <N>] [--json]
```

### `related`

```bash
repo-scout related <SYMBOL> --repo <REPO> [--json]
```

### `health`

```bash
repo-scout health --repo <REPO> [--top <N>] [--threshold <N>] [--large-files] [--large-functions] [--json]
```

### `circular`

```bash
repo-scout circular --repo <REPO> [--max-length <N>] [--json]
```

### `tree`

```bash
repo-scout tree --repo <REPO> [--depth <N>] [--no-deps] [--focus <PATH>] [--symbols] [--json]
```

### `orient`

```bash
repo-scout orient --repo <REPO> [--depth <N>] [--top <N>] [--json]
```

## Practical defaults

For automation, use `--json` and parse command output strictly.

For interactive use:

- start with `index`, then `find` and `refs`
- move to `impact` / `diff-impact` for risk assessment
- use `verify-plan` before running full test suites
