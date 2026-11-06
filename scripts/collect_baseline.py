#!/usr/bin/env python3
"""Publishes criterion results as the Alacer performance baseline (M5, ADR-0011).

Walks `target/criterion` after a `cargo bench` run and distills every
benchmark case into `benchmarks/baselines/baselines.json` — the published
baseline the perf contracts (`benchmarks/contracts.json`) gate against and
the perf dashboard renders from.

The baseline is a *snapshot*: every entry carries the criterion median, mean,
median absolute deviation and sample count in nanoseconds, plus the
environment that produced it (git commit, toolchain, host fingerprint). The
fingerprint (os/arch/cpu model/count, sha256-truncated) is what
`scripts/check_perf_contracts.py` uses to enforce the `same-host` regression
gates honestly: absolute times only compare on comparable silicon.

Usage:
    cargo bench --workspace            # or per crate
    python3 scripts/collect_baseline.py [--output benchmarks/baselines/baselines.json]

Exit codes: 0 on success, 1 on usage/IO errors, 2 when no criterion data exists.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRITERION = ROOT / "target" / "criterion"
DEFAULT_OUTPUT = ROOT / "benchmarks" / "baselines" / "baselines.json"
SCHEMA = 1


def host_fingerprint() -> tuple[dict[str, str], str]:
    """The host identity and its fingerprint hash (os/arch/cpu model/count)."""
    cpu_model = ""
    try:
        for line in Path("/proc/cpuinfo").read_text(encoding="utf-8").splitlines():
            if line.startswith("model name"):
                cpu_model = line.split(":", 1)[1].strip()
                break
    except OSError:
        cpu_model = ""
    if not cpu_model:
        cpu_model = platform.processor() or "unknown"
    cpu_count = os.cpu_count() or 0
    host = {
        "os": f"{platform.system()} {platform.release()}",
        "arch": platform.machine(),
        "cpu_model": cpu_model,
        "cpu_count": str(cpu_count),
    }
    basis = "|".join(host.values())
    fingerprint = hashlib.sha256(basis.encode("utf-8")).hexdigest()[:16]
    return host, fingerprint


def git_state() -> dict[str, str | bool]:
    """The commit the baseline was measured on, and whether the tree was dirty."""

    def git(*args: str) -> str:
        try:
            return subprocess.run(
                ["git", *args], cwd=ROOT, check=True, capture_output=True, text=True
            ).stdout.strip()
        except (subprocess.CalledProcessError, FileNotFoundError):
            return "unknown"

    commit = git("rev-parse", "HEAD")
    dirty = bool(git("status", "--porcelain"))
    return {"commit": commit, "dirty": dirty}


def toolchain() -> str:
    try:
        out = subprocess.run(
            ["rustc", "--version"], check=True, capture_output=True, text=True
        ).stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        out = "unknown"
    return out


def collect_bench(dir_new: Path) -> dict[str, object]:
    """One benchmark case: statistics from `estimates.json`, samples from `samples.json`."""
    estimates = json.loads((dir_new / "estimates.json").read_text(encoding="utf-8"))
    median = estimates["median"]["point_estimate"]
    mean = estimates["mean"]["point_estimate"]
    mad = estimates["median_abs_dev"]["point_estimate"]
    samples = 0
    sample_path = dir_new / "sample.json"
    if sample_path.exists():
        sample = json.loads(sample_path.read_text(encoding="utf-8"))
        iters = sample.get("iters", 0)
        samples = len(iters) if isinstance(iters, list) else int(iters)
    return {
        "median_ns": median,
        "mean_ns": mean,
        "median_abs_dev_ns": mad,
        "samples": samples,
    }


def collect() -> dict[str, object]:
    if not CRITERION.is_dir():
        print(f"no criterion data at {CRITERION}; run `cargo bench` first", file=sys.stderr)
        raise SystemExit(2)
    benches: dict[str, dict[str, object]] = {}
    for estimates_path in sorted(CRITERION.glob("**/new/estimates.json")):
        bench_dir = estimates_path.parent.parent  # <id>/new/estimates.json
        bench_id = bench_dir.relative_to(CRITERION).as_posix()
        if bench_id in benches:
            print(f"duplicate benchmark id {bench_id}; criterion ids must be unique", file=sys.stderr)
            raise SystemExit(1)
        benches[bench_id] = collect_bench(estimates_path.parent)
    if not benches:
        print(f"no benchmark results under {CRITERION}", file=sys.stderr)
        raise SystemExit(2)
    host, fingerprint = host_fingerprint()
    return {
        "schema": SCHEMA,
        "description": (
            "Published Alacer performance baseline (M5, ADR-0011): one entry per "
            "criterion benchmark case, median/mean/MAD in nanoseconds. Ids are the "
            "criterion result directories under target/criterion. Regenerate with "
            "scripts/collect_baseline.py after `cargo bench`."
        ),
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "toolchain": toolchain(),
        "git": git_state(),
        "host": {**host, "fingerprint": fingerprint},
        "bench_count": len(benches),
        "benches": benches,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output", type=Path, default=DEFAULT_OUTPUT, help="where to write the baseline JSON"
    )
    args = parser.parse_args()
    baseline = collect()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(baseline, indent=2, sort_keys=False) + "\n", encoding="utf-8"
    )
    print(f"published {baseline['bench_count']} benchmark baselines -> {args.output}")
    print(f"host fingerprint: {baseline['host']['fingerprint']} ({baseline['host']['cpu_model']})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
