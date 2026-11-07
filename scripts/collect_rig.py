#!/usr/bin/env python3
"""Declares the benchmark rig as committed data (M7, ADR-0013).

Writes `benchmarks/rig.json`: the host identity (the same
sha256-truncated fingerprint basis `collect_baseline.py` uses, so a rig
and a baseline cross-reference by fingerprint), the toolchain, the git
state, the measurement protocol (§14.1: 5-run medians), the backend mode,
and the dependency versions of every measured framework, read from the
lockfiles. The declared rig is the environment the committed
`benchmarks/comparisons/results.json` was collected on; rerunning this
script on another machine re-declares that machine — the runner refuses
to mix fingerprints.

Usage:
    python3 scripts/collect_rig.py [--output benchmarks/rig.json]

Exit codes: 0 on success, 1 on usage/IO errors.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUTPUT = ROOT / "benchmarks" / "rig.json"
SCHEMA = 1

# The measured frameworks and the lockfile that pins each one: the
# workspace lock for the Alacer side, the competitor projects' own
# lockfiles for the competitors (ADR-0013 §2: standalone projects).
LOCKFILES = {
    "workspace": ROOT / "Cargo.lock",
    "egui": ROOT / "benchmarks" / "competitors" / "egui_workloads" / "Cargo.lock",
    "masonry": ROOT / "benchmarks" / "competitors" / "masonry_workloads" / "Cargo.lock",
}
FRAMEWORKS = ("egui", "masonry")
# Extra workspace crates worth declaring (the load-bearing adopted cores).
WORKSPACE_DEPS = ("winit", "wgpu", "taffy", "parley", "accesskit", "reactive_graph")


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
    """The commit the rig was declared on, and whether the tree was dirty."""

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


def lock_versions(lock_path: Path, packages: tuple[str, ...] | None) -> dict[str, str]:
    """Pinned versions for `packages` from one Cargo.lock ({} when absent)."""
    if not lock_path.is_file():
        return {}
    text = lock_path.read_text(encoding="utf-8")
    names = packages if packages is not None else None
    versions: dict[str, str] = {}
    for match in re.finditer(r'^name = "([^"]+)"\s*\nversion = "([^"]+)"', text, re.M):
        name, version = match.group(1), match.group(2)
        if names is None:
            if name in FRAMEWORKS:
                versions[name] = version
        elif name in names:
            versions[name] = version
    return versions


def collect() -> dict[str, object]:
    host, fingerprint = host_fingerprint()
    workspace_versions = lock_versions(LOCKFILES["workspace"], WORKSPACE_DEPS)
    frameworks: dict[str, dict[str, str]] = {}
    for framework in FRAMEWORKS:
        versions = lock_versions(LOCKFILES[framework], None)
        frameworks[framework] = versions or {
            "note": f"lockfile not found at {LOCKFILES[framework].relative_to(ROOT)}; "
            "build the competitor project first (scripts/run_comparison.py --build)"
        }
    return {
        "schema": SCHEMA,
        "description": (
            "Declared benchmark rig (M7, ADR-0013): the environment the committed "
            "comparison results were collected on, the 5-run median protocol, and "
            "the measured frameworks' pinned versions. Regenerate with "
            "scripts/collect_rig.py; the runner refuses to mix fingerprints."
        ),
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "host": {**host, "fingerprint": fingerprint},
        "toolchain": toolchain(),
        "git": git_state(),
        # The reference desktop of §14.2 (Vulkan + llvmpipe parity) is the
        # portfolio step; this rig is the declared collection environment,
        # labeled honestly. Headless CPU measurement: vsync has no meaning
        # here — a labeled non-applicable, not an omitted fact.
        "backend_mode": "headless-cpu (software measurement path; no GPU device)",
        "vsync": "not-applicable-headless",
        "protocol": {
            "runs": 5,
            "estimator": "median-of-medians (criterion median per run, median across runs)",
            "sample_size": 30,
            "measurement_time_ms": 3000,
            "warm_up_ms": 500,
        },
        "dependencies": {"workspace": workspace_versions, "frameworks": frameworks},
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output", type=Path, default=DEFAULT_OUTPUT, help="where to write the rig JSON"
    )
    args = parser.parse_args()
    rig = collect()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(rig, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"declared rig -> {args.output}")
    print(f"host fingerprint: {rig['host']['fingerprint']} ({rig['host']['cpu_model']})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
