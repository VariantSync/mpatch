# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

mpatch is a Rust CLI tool and library for patching files based on a matching between the source
and target of the patch (an alternative to Unix `patch`). It's a single crate (no workspace) —
binary at `src/bin/mpatch.rs`, library root at `src/lib.rs` with modules `diffs`, `error`, `patch`
(submodules `alignment`, `application`, `filtering`, `matching`), and a private `io` module.

Input diffs are expected to be generated with Unix `diff -Naur` — the matching/patching logic
assumes that format.

## Build, test, lint

```
cargo build --verbose
cargo test --verbose
cargo clippy
cargo fmt
```

CI (`.github/workflows/rust.yml`) only runs `cargo build` and `cargo test` — it installs the
clippy component but never invokes it, and never runs `cargo fmt --check`. Still, run `cargo
clippy` and `cargo fmt` locally before considering work done; don't rely on CI to catch lint/format
issues.

Run a single test with `cargo test <test_name>` or `cargo test --test <file_stem> <test_name>`
(e.g. `cargo test --test matching`).

**Tests must be run from the repository root.** Integration tests (`tests/*.rs`) and the doctest
in `src/lib.rs` load fixtures via relative paths (e.g. `tests/samples/source_variant/version-0/...`),
so running `cargo test` from elsewhere breaks fixture loading.

## Git workflow

Feature branches are cut from `develop`; PRs target `develop`, and `main` only receives merges
from `develop`. Commit messages follow a loose Conventional Commits style: `feat:`, `fix:`,
`refactor:`, `refactor(tests):`, `tests:`, `docs:`, `config:`, `chore:`, `ci:`.
