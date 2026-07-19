---
name: verify
description: Run mpatch's full local check suite (build, test, clippy, fmt check) before considering changes done or opening a PR. Use after making code changes in this repo.
---

Run these commands from the repository root, in order, and report any failures:

```
cargo build --verbose
cargo test --verbose
cargo clippy
cargo fmt --check
```

Notes:
- CI (`.github/workflows/rust.yml`) only runs `cargo build` and `cargo test` — clippy and fmt are
  not enforced there, so this skill is the only thing that catches those issues before a PR.
- If `cargo fmt --check` fails, run `cargo fmt` to fix formatting in place, then re-run this check.
- If `cargo clippy` reports warnings, fix them unless they're clearly false positives — explain
  why before leaving one unaddressed.
- Tests must run from the repo root since integration tests and the `src/lib.rs` doctest load
  fixtures via relative paths (e.g. `tests/samples/...`).
