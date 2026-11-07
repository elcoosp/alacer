#!/usr/bin/env python3
"""The cross-framework comparison runner (M7, ADR-0013).

Builds the competitor harness projects (`benchmarks/competitors/`), runs
each beat workload's criterion case five times (§14.1's 5-run median
protocol: criterion median per run, median across runs), reads the Alacer
side from the published baseline (same-host by fingerprint), and distills
everything into `benchmarks/comparisons/results.json` — one labeled entry
per (workload, framework), plus the beat verdict.

Everything is labeled: framework, pinned version, criterion case id, run
count, and the measured scope (ADR-0013 §3's asymmetry rule — the
framework's scope is a subset of what every competitor measures in the
same row). `--validate` checks the committed artifacts cheaply; that is
the mode the M7 exit suite and CI call.

Usage:
    python3 scripts/run_comparison.py --build          # compile competitor benches
    python3 scripts/run_comparison.py [--runs 5] [--remeasure-alacer]
    python3 scripts/run_comparison.py --validate

Exit codes: 0 on success, 1 on validation failure or usage errors, 2 when
a bench run fails.
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
COMPETITORS = ROOT / "benchmarks" / "competitors"
RIG = ROOT / "benchmarks" / "rig.json"
BASELINE = ROOT / "benchmarks" / "baselines" / "baselines.json"
CONTRACTS = ROOT / "benchmarks" / "contracts.json"
DASHBOARD = ROOT / "benchmarks" / "dashboard" / "index.html"
DEFAULT_OUTPUT = ROOT / "benchmarks" / "comparisons" / "results.json"
SCHEMA = 1

# The beat workloads (§18's kill-criterion trio): the Alacer side is the
# published M6 baseline case; the competitor side names each project's
# bench file and criterion case id. The scope notes are the ADR-0013 §3
# asymmetry rule, verbatim: the Alacer scope is a subset of every
# competitor scope in its row.
WORKLOADS: list[dict[str, object]] = [
    {
        "id": "w1_churn_10k",
        "cite": "architecture §14.2 W1",
        "alacer_baseline": "w1/incremental_churn_10pct",
        "alacer_scope": (
            "incremental relayout of the churned frame through the persistent "
            "taffy tree (layout only — the framework's own published bench)"
        ),
        "alacer_bench": ("alacer-layout", "w1_churn"),
        "competitors": {
            "egui": {
                "bench_file": "w1_churn",
                "case_id": "egui/w1_churn_full_10k",
                "scope": (
                    "full immediate-mode frame: widget walk + layout + galley "
                    "shaping + tessellation over the whole 10k tree; row "
                    "virtualization deliberately not taken (mechanism under "
                    "test — see the project README)"
                ),
            },
            "masonry": {
                "bench_file": "w1_churn",
                "case_id": "masonry/w1_churn_10k",
                "scope": (
                    "full pass pipeline through redraw(): update + layout + "
                    "compose + paint-scene build + a11y tree update over the "
                    "whole 10k tree; vello rasterization excluded (no GPU "
                    "device on this rig — labeled, conservative)"
                ),
            },
        },
    },
    {
        "id": "w2_text_jitter_1k",
        "cite": "architecture §14.2 W2",
        "alacer_baseline": "w2/jitter_incremental_1k",
        "alacer_scope": (
            "incremental text relayout under the ±5% width jitter through the "
            "persistent taffy tree (layout only — the framework's own bench)"
        ),
        "alacer_bench": ("alacer-layout", "w2_text"),
        "competitors": {
            "egui": {
                "bench_file": "w2_text",
                "case_id": "egui/w2_text_jitter_1k",
                "scope": (
                    "full immediate-mode frame at the jittered width: widget "
                    "walk + layout + galley re-shaping (the galley cache's "
                    "real behavior) + tessellation; virtualization not taken"
                ),
            },
            "masonry": {
                "bench_file": "w2_text",
                "case_id": "masonry/w2_text_jitter_1k",
                "scope": (
                    "full pass pipeline through redraw() at the jittered "
                    "width: parley re-breaking on changed constraints + "
                    "layout + paint-scene build + a11y tree update; vello "
                    "rasterization excluded (labeled, conservative)"
                ),
            },
        },
    },
    {
        "id": "w3_animation_1k",
        "cite": "architecture §14.2 W3",
        "alacer_baseline": "w3/animation_frame_1k",
        "alacer_scope": (
            "one animation frame through the real ten-pass pipeline INCLUDING "
            "the damaged software raster (the framework's own bench)"
        ),
        "alacer_bench": ("alacer-testing", "w3_animation"),
        "competitors": {
            "egui": {
                "bench_file": "w3_animation",
                "case_id": "egui/w3_animation_1k",
                "scope": (
                    "full immediate-mode frame: widget walk + per-frame paint "
                    "ops (1k rects, 10% fading) + tessellation; pixel "
                    "rasterization excluded (as for every competitor row)"
                ),
            },
            "masonry": {
                "bench_file": "w3_animation",
                "case_id": "masonry/w3_animation_1k",
                "scope": (
                    "full pass pipeline through redraw() with compose-level "
                    "transforms on 1k nodes (translation-only path); no fade "
                    "churn and no vello rasterization (labeled, conservative)"
                ),
            },
        },
    },
]

FRAMEWORKS = ("egui", "masonry")


def log(message: str) -> None:
    print(message, flush=True)


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def project_dir(framework: str) -> Path:
    return COMPETITORS / f"{framework}_workloads"


def framework_version(framework: str) -> str:
    """The pinned version of a competitor framework, from its lockfile."""
    lock = project_dir(framework) / "Cargo.lock"
    if lock.is_file():
        text = lock.read_text(encoding="utf-8")
        for match in re.finditer(r'^name = "([^"]+)"\s*\nversion = "([^"]+)"', text, re.M):
            if match.group(1) == framework:
                return match.group(2)
    return "unknown"


def estimates_path_for(competitor: str, case_id: str) -> Path:
    """target/criterion/<group>/<case>/new/estimates.json under the project."""
    project = project_dir(competitor)
    group, case = case_id.split("/", 1)
    return project / "target" / "criterion" / group / case / "new" / "estimates.json"


def read_median(estimates_path: Path) -> float:
    estimates = json.loads(estimates_path.read_text(encoding="utf-8"))
    return float(estimates["median"]["point_estimate"])


def run_competitor_case(framework: str, bench_file: str, case_id: str, runs: int) -> list[float]:
    """`runs` process-level runs of one criterion case; median per run."""
    medians: list[float] = []
    for run in range(runs):
        log(f"  {framework} {case_id}: run {run + 1}/{runs}")
        result = subprocess.run(
            ["cargo", "bench", "--bench", bench_file],
            cwd=project_dir(framework),
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            log(result.stdout[-2000:])
            log(result.stderr[-2000:])
            raise SystemExit(2)
        medians.append(read_median(estimates_path_for(framework, case_id)))
    return medians


def run_alacer_case(package: str, bench_file: str, case_id: str, runs: int) -> list[float]:
    medians: list[float] = []
    group, case = case_id.split("/", 1)
    estimates_path = ROOT / "target" / "criterion" / group / case / "new" / "estimates.json"
    for run in range(runs):
        log(f"  alacer {package}/{bench_file}: run {run + 1}/{runs}")
        result = subprocess.run(
            ["cargo", "bench", "-p", package, "--bench", bench_file],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            log(result.stdout[-2000:])
            log(result.stderr[-2000:])
            raise SystemExit(2)
        medians.append(read_median(estimates_path))
    return medians


def build_competitors() -> None:
    for framework in FRAMEWORKS:
        log(f"building {framework} benches (release)...")
        result = subprocess.run(
            ["cargo", "build", "--release", "--benches"],
            cwd=project_dir(framework),
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            log(result.stdout[-3000:])
            log(result.stderr[-3000:])
            raise SystemExit(2)


def collect(runs: int, remeasure_alacer: bool) -> dict[str, object]:
    rig = load_json(RIG)
    if rig["schema"] != SCHEMA:
        raise SystemExit(f"rig schema {rig['schema']} is not {SCHEMA}; run scripts/collect_rig.py")
    rig_fingerprint = rig["host"]["fingerprint"]
    protocol = rig["protocol"]
    if int(protocol["runs"]) != runs:
        log(
            f"note: the declared protocol runs {protocol['runs']}; collecting "
            f"{runs} runs as requested"
        )

    baseline = load_json(BASELINE)
    baseline_fingerprint = baseline["host"]["fingerprint"]
    same_host = baseline_fingerprint == rig_fingerprint
    if not same_host:
        log(
            f"note: the published baseline's host ({baseline_fingerprint}) is not "
            f"the declared rig ({rig_fingerprint}) — with --remeasure-alacer the "
            "comparison stays same-host; without it the Alacer rows are advisory"
        )

    build_competitors()

    workloads: list[dict[str, object]] = []
    for workload in WORKLOADS:
        workload_id = str(workload["id"])
        log(f"measuring {workload_id}...")
        if remeasure_alacer:
            package, bench_file = workload["alacer_bench"]  # type: ignore[misc]
            run_medians = run_alacer_case(
                package, bench_file, str(workload["alacer_baseline"]), runs
            )
            alacer_median = statistics.median(run_medians)
            alacer_source = "remeasured"
        else:
            alacer_median = float(
                baseline["benches"][str(workload["alacer_baseline"])]["median_ns"]
            )
            alacer_source = "published-baseline"

        competitors: list[dict[str, object]] = []
        for framework in FRAMEWORKS:
            spec = workload["competitors"][framework]  # type: ignore[index]
            run_medians = run_competitor_case(
                framework, str(spec["bench_file"]), str(spec["case_id"]), runs
            )
            competitors.append(
                {
                    "framework": framework,
                    "version": framework_version(framework),
                    "case": spec["case_id"],
                    "median_ns": statistics.median(run_medians),
                    "runs": runs,
                    "run_medians_ns": run_medians,
                    "scope": spec["scope"],
                }
            )

        workloads.append(
            {
                "id": workload_id,
                "cite": workload["cite"],
                "alacer": {
                    "baseline_id": workload["alacer_baseline"],
                    "median_ns": alacer_median,
                    "source": alacer_source,
                    "scope": workload["alacer_scope"],
                },
                "competitors": competitors,
            }
        )

    won = []
    for row in workloads:
        alacer_median = float(row["alacer"]["median_ns"])  # type: ignore[index]
        best_competitor = min(
            float(competitor["median_ns"])  # type: ignore[index]
            for competitor in row["competitors"]  # type: ignore[union-attr]
        )
        if alacer_median < best_competitor:
            won.append(row["id"])

    return {
        "schema": SCHEMA,
        "description": (
            "Published cross-framework comparison (M7, ADR-0013): the §18 "
            "beat workloads, 5-run medians per framework, everything labeled "
            "with its measured scope. Regenerate with "
            "scripts/run_comparison.py; validate with --validate."
        ),
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "rig_fingerprint": rig_fingerprint,
        "same_host": same_host,
        "advisory": not same_host,
        "protocol": {
            "runs": runs,
            "estimator": "median-of-medians (criterion median per run, median across runs)",
        },
        "beat": {
            "definition": (
                "alacer median < every competitor median on the same host, "
                "per workload (§1: beats both egui and the best audited "
                "retained stack in at least one shared workload)"
            ),
            "workloads_won": won,
        },
        "workloads": workloads,
    }


def validate() -> int:
    """The committed artifacts, checked cheaply (the exit suite calls this)."""
    failures: list[str] = []

    def check(condition: bool, message: str) -> None:
        if not condition:
            failures.append(message)

    rig = load_json(RIG)
    check(rig["schema"] == SCHEMA, f"rig schema is {rig['schema']}, expected {SCHEMA}")
    rig_fingerprint = str(rig["host"]["fingerprint"])
    check(
        re.fullmatch(r"[0-9a-f]{16}", rig_fingerprint) is not None,
        f"rig fingerprint {rig_fingerprint!r} is not a sha256-truncated hash",
    )
    check(int(rig["protocol"]["runs"]) == 5, "the rig protocol declares the 5-run median")

    results = load_json(DEFAULT_OUTPUT)
    check(results["schema"] == SCHEMA, "results schema is 1")
    check(
        results["rig_fingerprint"] == rig_fingerprint,
        "the results' rig fingerprint matches the declared rig",
    )
    check(int(results["protocol"]["runs"]) == 5, "the results carry 5 runs per case")

    baseline = load_json(BASELINE)
    published = baseline["benches"]
    workload_ids = {str(workload["id"]) for workload in WORKLOADS}

    rows = results["workloads"]
    check(
        {str(row["id"]) for row in rows} == workload_ids,  # type: ignore[union-attr]
        "the results cover exactly the beat workload set",
    )
    for row in rows:  # type: ignore[union-attr]
        workload_id = str(row["id"])
        alacer = row["alacer"]  # type: ignore[index]
        check(
            str(alacer["baseline_id"]) in published,  # type: ignore[index]
            f"{workload_id}: the alacer baseline id is published",
        )
        check(
            float(alacer["median_ns"]) > 0.0,  # type: ignore[index]
            f"{workload_id}: the alacer median is measured",
        )
        check(
            bool(str(alacer["scope"]).strip()),  # type: ignore[index]
            f"{workload_id}: the alacer scope is documented",
        )
        frameworks_seen = set()
        for competitor in row["competitors"]:  # type: ignore[union-attr]
            framework = str(competitor["framework"])
            frameworks_seen.add(framework)
            check(
                float(competitor["median_ns"]) > 0.0,  # type: ignore[index]
                f"{workload_id}/{framework}: the median is measured",
            )
            check(
                int(competitor["runs"]) == 5,  # type: ignore[index]
                f"{workload_id}/{framework}: 5 runs",
            )
            check(
                len(competitor["run_medians_ns"]) == 5,  # type: ignore[index]
                f"{workload_id}/{framework}: the run medians are recorded",
            )
            check(
                bool(str(competitor["scope"]).strip()),  # type: ignore[index]
                f"{workload_id}/{framework}: the scope is documented",
            )
            check(
                competitor["version"] != "unknown",  # type: ignore[index]
                f"{workload_id}/{framework}: the version is pinned",
            )
        check(
            frameworks_seen == set(FRAMEWORKS),
            f"{workload_id}: both competitors measured",
        )

    beat = results["beat"]
    won = [str(item) for item in beat["workloads_won"]]  # type: ignore[index]
    check(bool(won), "the committed comparison wins at least one workload")
    check(set(won) <= workload_ids, "the won set names only beat workloads")

    contracts = load_json(CONTRACTS)
    gates = contracts["test_enforced_gates"]
    check(
        any(gate["id"] == "comparison_corpus_published" for gate in gates),
        "contracts.json publishes the comparison corpus gate",
    )
    dashboard = DASHBOARD.read_text(encoding="utf-8")
    check("Cross-framework comparison" in dashboard, "the dashboard publishes the comparison")
    for workload_id in sorted(workload_ids):
        check(workload_id in dashboard, f"the dashboard publishes {workload_id}")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print(
        f"comparison: OK ({len(workload_ids)} workloads × {len(FRAMEWORKS)} "
        f"competitors; beat: {', '.join(won)})"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build", action="store_true", help="compile the competitor benches only")
    parser.add_argument("--runs", type=int, default=5, help="process-level runs per case (default 5)")
    parser.add_argument(
        "--remeasure-alacer",
        action="store_true",
        help="re-run the Alacer side instead of reading the published baseline",
    )
    parser.add_argument("--validate", action="store_true", help="check the committed artifacts")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()

    if args.validate:
        return validate()
    if args.build:
        build_competitors()
        return 0
    if not RIG.is_file():
        print("benchmarks/rig.json missing; run scripts/collect_rig.py first", file=sys.stderr)
        return 1
    results = collect(args.runs, args.remeasure_alacer)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(results, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    won = results["beat"]["workloads_won"]  # type: ignore[index]
    print(f"published {len(results['workloads'])} workload comparisons -> {args.output}")  # type: ignore[index]
    print(f"beat: {', '.join(won) if won else 'none — the claim does not hold on this rig'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
