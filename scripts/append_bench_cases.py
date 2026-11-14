#!/usr/bin/env python3
"""Appends new bench cases to the published baseline and contract set.

The house rule (ADR-0011): every criterion case in the workspace is
published in `benchmarks/baselines/baselines.json` and gated in
`benchmarks/contracts.json`. This helper appends *only* the named new
cases — measured medians come from this host's fresh `target/criterion`
data — leaving the published snapshot's existing entries untouched.

Usage:
    python3 scripts/append_bench_cases.py trail/reader_announce_100_1k trail_ring/fill_drain_1024

Steps per case:
  1. baselines.json  — append the distilled entry, bump bench_count.
  2. contracts.json  — append the same-host regression gate (the
     uncontracted-benchmark rule fails CI otherwise).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRITERION = ROOT / "target" / "criterion"
BASELINE = ROOT / "benchmarks" / "baselines" / "baselines.json"
CONTRACTS = ROOT / "benchmarks" / "contracts.json"


def collect_case(bench_id: str) -> dict:
    estimates_path = CRITERION / bench_id / "new" / "estimates.json"
    if not estimates_path.exists():
        print(f"no fresh criterion data for {bench_id}; run its bench first", file=sys.stderr)
        raise SystemExit(2)
    estimates = json.loads(estimates_path.read_text(encoding="utf-8"))
    sample_path = CRITERION / bench_id / "new" / "sample.json"
    samples = 0
    if sample_path.exists():
        sample = json.loads(sample_path.read_text(encoding="utf-8"))
        iters = sample.get("iters", 0)
        samples = len(iters) if isinstance(iters, list) else int(iters)
    return {
        "median_ns": estimates["median"]["point_estimate"],
        "mean_ns": estimates["mean"]["point_estimate"],
        "median_abs_dev_ns": estimates["median_abs_dev"]["point_estimate"],
        "samples": samples,
    }


def main() -> int:
    ids = sys.argv[1:]
    if not ids:
        print("usage: append_bench_cases.py <bench_id> [bench_id ...]", file=sys.stderr)
        return 1

    baseline = json.loads(BASELINE.read_text(encoding="utf-8"))
    contracts = json.loads(CONTRACTS.read_text(encoding="utf-8"))
    gates = {gate["baseline"] for gate in contracts["regressions"]}

    for bench_id in ids:
        entry = collect_case(bench_id)
        baseline["benches"][bench_id] = entry
        if bench_id not in gates:
            contracts["regressions"].append(
                {
                    "id": bench_id.replace("/", "_"),
                    "baseline": bench_id,
                    "scope": "same-host",
                }
            )
        print(
            f"appended {bench_id}: median {entry['median_ns']:.0f} ns "
            f"({entry['samples']} samples)"
        )

    baseline["bench_count"] = len(baseline["benches"])
    BASELINE.write_text(json.dumps(baseline, indent=2) + "\n", encoding="utf-8")
    CONTRACTS.write_text(json.dumps(contracts, indent=2) + "\n", encoding="utf-8")
    print(f"baseline now publishes {baseline['bench_count']} cases")
    return 0


if __name__ == "__main__":
    sys.exit(main())
