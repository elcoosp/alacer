# ADR-0006: Release automation — git-cliff + GitHub Releases

- **Status:** Accepted (2026-11-05)

## Decision

Conventional Commits are policy (feat/fix/perf/test/bench/doc/chore). On a
`v*` tag: the verify job gates on clippy + tests + DAG; git-cliff generates
per-tag release notes and a refreshed `CHANGELOG.md` (committed back);
release builds of the counter and scrolling-list examples for Linux, macOS,
and Windows attach to a GitHub Release via softprops/action-gh-release.

## Actions pinned (latest majors as of 2026-11)

`actions/checkout@v7`, `actions/upload-artifact@v7`, `actions/download-
artifact@v8`, `Swatinem/rust-cache@v2`, `EmbarkStudios/cargo-deny-action@v2`,
`taiki-e/install-action@v2`, `softprops/action-gh-release@v2`.

## Consequences

- The changelog is generated, never hand-written; the bot commit is marked
  `[skip ci]`.
- cargo-deny denies AGPL/GPL-family licenses outright (§20 provenance rules).
