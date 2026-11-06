#!/usr/bin/env python3
"""Checks the Alacer performance contracts (M5, ADR-0011).

Compares fresh criterion data (`target/criterion`, or `--candidate file.json`)
against the published baseline (`benchmarks/baselines/baselines.json`) and the
contract set (`benchmarks/contracts.json`):

- **Budgets** — architecture gates expressed as median ratios (the W1 ≥ 2×
  gate). Host-independent: enforced everywhere, always.
- **Regressions** — per-case median gates against the published baseline,
  scoped `same-host`: they compare only when the measuring host's fingerprint
  (os/arch/cpu model/count) matches the baseline's. On a foreign host — a CI
  runner — they are reported as SKIPPED, not failed: absolute nanoseconds
  across different silicon is not a contract, it is noise theatre. CI enforces
  the budgets and criterion's own previous-run comparison; same-host gates
  fire on maintainer machines and pinned-hardware cron runners
  (`--strict-host` turns the skip into a failure for those).

Exit codes: 0 = all applicable contracts pass, 1 = any failure,
2 = missing data (no criterion run, no baseline).
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRITERION = ROOT / "target" / "criterion"
CONTRACTS = ROOT / "benchmarks" / "contracts.json"
BASELINE = ROOT / "benchmarks" / "baselines" / "baselines.json"


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        print(f"missing required file: {path}", file=sys.stderr)
        raise SystemExit(2) from None
    except json.JSONDecodeError as error:
        print(f"invalid JSON in {path}: {error}", file=sys.stderr)
        raise SystemExit(2) from None


def fresh_measurements(candidate: Path | None) -> tuple[dict[str, float], str]:
    """Fresh medians by bench id, and the host fingerprint they were measured on."""
    if candidate is not None:
        data = load_json(candidate)
        benches = data["benches"]
        return {
            bench_id: entry["median_ns"] for bench_id, entry in benches.items()
        }, data["host"]["fingerprint"]
    if not CRITERION.is_dir():
        print(f"no criterion data at {CRITERION}; run `cargo bench` first", file=sys.stderr)
        raise SystemExit(2)
    fresh: dict[str, float] = {}
    for estimates_path in sorted(CRITERION.glob("**/new/estimates.json")):
        bench_id = estimates_path.parent.parent.relative_to(CRITERION).as_posix()
        estimates = json.loads(estimates_path.read_text(encoding="utf-8"))
        fresh[bench_id] = estimates["median"]["point_estimate"]
    if not fresh:
        print(f"no benchmark results under {CRITERION}", file=sys.stderr)
        raise SystemExit(2)
    # The fresh host fingerprint: recompute the same way the collector does.
    sys.path.insert(0, str(ROOT / "scripts"))
    from collect_baseline import host_fingerprint  # noqa: E402  (local import by design)

    _, fingerprint = host_fingerprint()
    return fresh, fingerprint


def human_ns(value: float) -> str:
    for unit in ("ns", "µs", "ms", "s"):
        if abs(value) < 1000.0 or unit == "s":
            return f"{value:.1f} {unit}" if unit != "ns" else f"{value:.0f} {unit}"
        value /= 1000.0
    return f"{value:.1f} s"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--candidate",
        type=Path,
        default=None,
        help="baseline-format JSON of fresh measurements (default: target/criterion)",
    )
    parser.add_argument(
        "--baseline", type=Path, default=BASELINE, help="the published baseline JSON"
    )
    parser.add_argument(
        "--contracts", type=Path, default=CONTRACTS, help="the contract set JSON"
    )
    parser.add_argument(
        "--strict-host",
        action="store_true",
        help="fail (not skip) same-host regression gates when the host differs",
    )
    args = parser.parse_args()

    contracts = load_json(args.contracts)
    baseline = load_json(args.baseline)
    if baseline.get("schema") != 1 or contracts.get("schema") != 1:
        print("contract/baseline schema must be 1", file=sys.stderr)
        raise SystemExit(2)
    published: dict[str, float] = {
        bench_id: entry["median_ns"] for bench_id, entry in baseline["benches"].items()
    }
    fresh, fresh_host = fresh_measurements(args.candidate)
    baseline_host = baseline["host"]["fingerprint"]
    same_host = fresh_host == baseline_host
    default_tolerance = float(contracts["tolerance_default"])

    failures: list[str] = []
    rows: list[tuple[str, str, str, str]] = []

    # --- Budgets: architecture gates as median ratios (host-independent). ---
    for budget in contracts["budgets"]:
        num_id, den_id, op, limit = (
            budget["numerator"],
            budget["denominator"],
            budget["op"],
            float(budget["budget"]),
        )
        missing = [i for i in (num_id, den_id) if i not in fresh or i not in published]
        if missing:
            failures.append(f"budget {budget['id']}: missing benchmark data for {missing}")
            continue
        ratio = fresh[num_id] / fresh[den_id]
        baseline_ratio = published[num_id] / published[den_id]
        ok = ratio >= limit if op == ">=" else ratio <= limit
        if not ok:
            failures.append(
                f"budget {budget['id']}: {num_id} / {den_id} = {ratio:.2f} "
                f"violates {op} {limit} (baseline ratio {baseline_ratio:.2f})"
            )
        rows.append(
            (
                f"budget:{budget['id']}",
                f"{baseline_ratio:.2f}×",
                f"{ratio:.2f}×",
                f"{'PASS' if ok else 'FAIL'} (gate {op} {limit:g}×)",
            )
        )

    # --- Regressions: per-case medians against the published baseline. ---
    for gate in contracts["regressions"]:
        gate_id, bench_id = gate["id"], gate["baseline"]
        tolerance = float(gate.get("tolerance", default_tolerance))
        if gate.get("scope", "same-host") != "same-host":
            failures.append(f"regression {gate_id}: unknown scope {gate.get('scope')!r}")
            continue
        if bench_id not in published:
            failures.append(f"regression {gate_id}: {bench_id} missing from the published baseline")
            continue
        if bench_id not in fresh:
            failures.append(f"regression {gate_id}: {bench_id} not in the fresh run (bench removed or renamed?)")
            continue
        if not same_host:
            if args.strict_host:
                failures.append(
                    f"regression {gate_id}: host mismatch (fresh {fresh_host} vs "
                    f"baseline {baseline_host}) and --strict-host is set"
                )
                continue
            rows.append((f"regression:{gate_id}", human_ns(published[bench_id]), human_ns(fresh[bench_id]), "SKIP (foreign host)"))
            continue
        delta = fresh[bench_id] / published[bench_id] - 1.0
        ok = delta <= tolerance
        if not ok:
            failures.append(
                f"regression {gate_id}: {bench_id} median {human_ns(fresh[bench_id])} is "
                f"{delta:+.1%} vs published {human_ns(published[bench_id])} (tolerance {tolerance:.0%})"
            )
        rows.append(
            (
                f"regression:{gate_id}",
                human_ns(published[bench_id]),
                human_ns(fresh[bench_id]),
                f"{'PASS' if ok else 'FAIL'} ({delta:+.1%}, gate ≤ {tolerance:.0%})",
            )
        )

    # --- Unpublished coverage: every fresh case must be under contract. ---
    contracted = {gate["baseline"] for gate in contracts["regressions"]}
    for bench_id in sorted(fresh):
        if bench_id not in contracted:
            failures.append(
                f"uncontracted benchmark: {bench_id} ran but has no regression gate — "
                f"publish its baseline and add the gate (ADR-0011)"
            )

    print(f"host: fresh {fresh_host} vs baseline {baseline_host} -> {'same' if same_host else 'FOREIGN'}")
    print()
    for row in rows:
        print(f"  {row[0]:<36} {row[1]:>14}  ->  {row[2]:>14}   {row[3]}")

    if failures:
        print(f"\n{len(failures)} contract failure(s):", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    passed = sum(1 for row in rows if row[3].startswith("PASS"))
    skipped = sum(1 for row in rows if "SKIP" in row[3])
    print(f"\nperf contracts OK: {passed} pass, {skipped} skipped (foreign host), 0 failed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
