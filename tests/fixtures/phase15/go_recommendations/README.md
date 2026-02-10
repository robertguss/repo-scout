# Phase 15 Go Recommendation Fixture

This fixture captures a minimal Go source + `_test.go` pairing for recommendation convergence.

It is used to validate:

- `tests-for` default runnable-target classification for Go test files,
- `verify-plan` targeted command synthesis (`go test ./<package_dir>`),
- `verify-plan` full-suite gate selection (`go test ./...`) for Go-only changed scope.
