# Contributing to Alacer

## The rules, in one page

1. **Conventional Commits.** `feat(core): …`, `fix(layout): …`,
   `perf(render): …`, `test: …`, `bench: …`, `docs: …`, `chore: …`. The
   changelog and GitHub Releases are generated from the history (git-cliff).
2. **TDD.** Tests come first; every module carries its unit tests inline and
   cross-crate behavior lives in `alacer-testing/tests/`. The M0 exit
   criteria are executable — keep them green.
3. **No lint suppression.** Clippy `pedantic` is denied workspace-wide, and
   there are no `#[allow]` attributes. If a cast is genuinely necessary, put
   it in `alacer-core/src/num.rs` with a reason-carrying `#[expect]`.
4. **One invalidation currency.** A feature that cannot express itself as a
   `Change` mask is rejected or redesigned (axiom A1). Effects never mutate
   widgets; they queue masks.
5. **Caches have eviction policies and tests that prove them** (§14.3).
6. **Zero unsafe.** `unsafe_code = forbid` workspace-wide.
7. **Upstream-first.** Forking a core dependency needs an ADR with a sunset
   plan (A5/ADR-1 of the architecture).
8. **Benchmarks for hot paths.** New engines or passes ship with criterion
   coverage; regressions against saved baselines are CI failures (A6).

## Local gates (all must pass before push)

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps --all-features   # RUSTDOCFLAGS=-D warnings
python3 scripts/check_dag.py
cargo bench --workspace --no-run
```

## The perf contracts (M5, ADR-0011)

A new benchmark case is not done until it is *published*: run
`cargo bench`, then `python3 scripts/collect_baseline.py`, then add its
regression gate to `benchmarks/contracts.json` — the checker fails on an
uncontracted benchmark, and the M5 exit suite enforces the
contracts ↔ baseline bijection. Budget ratios (the architecture's gates,
e.g. W1 ≥ 2×) are enforced everywhere; same-host absolute gates fire on
your machine against the published baseline, and regenerate the dashboard
(`python3 scripts/perf_dashboard.py`) when the baseline moves.

## Snapshot updates

Snapshot drift means *either* a real regression or an intended visual change.
Rerun with `ALACER_BLESS=1 cargo test -p alacer-testing`, review the PNG diff
in `crates/alacer-testing/blessed/`, and commit it — the diff is the review.
