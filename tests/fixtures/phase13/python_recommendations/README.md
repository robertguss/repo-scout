# Phase 13 Python Recommendation Fixture

This fixture exercises Phase 13 Python production-closure behavior:

- explicit pytest detection via `pyproject.toml`,
- runnable Python test target synthesis for `tests-for` and `verify-plan`,
- Python `*_tests.py` test-like pattern classification,
- relative-import caller attribution for `diff-impact`.

Expected checks:

- `tests-for compute_plan --repo <fixture> --json` includes:
  - `tests/unit/test_service.py`
  - `src/service_tests.py`
- `verify-plan --changed-file src/service.py --repo <fixture> --json` includes:
  - targeted `pytest tests/unit/test_service.py`
  - full-suite `pytest`
- `diff-impact --changed-file src/pkg/util.py --repo <fixture> --json` includes:
  - `called_by` row for symbol `run` in `src/pkg/consumer.py`
