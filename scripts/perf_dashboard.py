#!/usr/bin/env python3
"""Renders the Alacer performance dashboard (M5, ADR-0011).

A single self-contained HTML file — no external assets, no JavaScript —
published at `benchmarks/dashboard/index.html`. It renders:

- the environment the published baseline was measured on (commit, toolchain,
  host fingerprint);
- the budget contracts (the architecture's falsifiable gates) with the
  measured ratio and PASS/FAIL;
- the same-host regression table (published median vs the freshest criterion
  run, if one exists next to the repo) with delta bars;
- the test-enforced gates (the budgets criterion cannot express), each with
  its enforcement site.

The dashboard is regenerated (`python3 scripts/perf_dashboard.py`) whenever
baselines change; CI re-generates and uploads it on main.

Usage:
    python3 scripts/perf_dashboard.py [--output benchmarks/dashboard/index.html]
"""

from __future__ import annotations

import argparse
import html
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRITERION = ROOT / "target" / "criterion"
CONTRACTS = ROOT / "benchmarks" / "contracts.json"
BASELINE = ROOT / "benchmarks" / "baselines" / "baselines.json"
DEFAULT_OUTPUT = ROOT / "benchmarks" / "dashboard" / "index.html"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def human_ns(value: float) -> str:
    value = float(value)
    for unit in ("ns", "µs", "ms", "s"):
        if abs(value) < 1000.0 or unit == "s":
            return f"{value:,.1f} {unit}" if unit != "ns" else f"{value:,.0f} ns"
        value /= 1000.0
    return f"{value:.1f} s"


def fresh_medians() -> dict[str, float]:
    if not CRITERION.is_dir():
        return {}
    fresh: dict[str, float] = {}
    for estimates_path in sorted(CRITERION.glob("**/new/estimates.json")):
        bench_id = estimates_path.parent.parent.relative_to(CRITERION).as_posix()
        estimates = json.loads(estimates_path.read_text(encoding="utf-8"))
        fresh[bench_id] = estimates["median"]["point_estimate"]
    return fresh


def esc(value: object) -> str:
    return html.escape(str(value), quote=True)


STYLE = """
:root { color-scheme: dark; }
* { box-sizing: border-box; }
body { margin: 0; background: #101418; color: #d7dde3;
       font: 14px/1.5 ui-sans-serif, system-ui, "Segoe UI", sans-serif; }
header { padding: 28px 32px 20px; border-bottom: 1px solid #232b33; }
h1 { margin: 0 0 4px; font-size: 22px; color: #f2f5f7; letter-spacing: 0.2px; }
h1 span { color: #55b46a; }
h2 { font-size: 15px; color: #9fb0bd; margin: 28px 0 10px; text-transform: uppercase;
     letter-spacing: 1.2px; }
main { padding: 8px 32px 40px; max-width: 1100px; }
p.sub { margin: 0; color: #7d8b97; }
table { border-collapse: collapse; width: 100%; margin-top: 6px; }
th, td { text-align: left; padding: 7px 10px; border-bottom: 1px solid #1d252c; }
th { color: #8395a3; font-weight: 600; font-size: 12px; text-transform: uppercase;
     letter-spacing: 0.8px; }
td.num, th.num { text-align: right; font-variant-numeric: tabular-nums; }
.badge { display: inline-block; padding: 1px 9px; border-radius: 999px;
         font-size: 12px; font-weight: 600; }
.pass  { background: #173425; color: #6fd28a; }
.fail  { background: #3c1a1e; color: #ef7285; }
.skip  { background: #33301a; color: #d9c36b; }
.meta  { color: #8b98a4; font-size: 13px; }
code   { background: #1a2129; padding: 1px 6px; border-radius: 4px;
         font: 12.5px ui-monospace, "Cascadia Code", Menlo, monospace; color: #b7c4cf; }
.bar { position: relative; height: 6px; background: #1a2129; border-radius: 3px;
       min-width: 120px; overflow: hidden; }
.bar i { position: absolute; inset: 0 auto 0 0; border-radius: 3px; }
footer { padding: 18px 32px; border-top: 1px solid #232b33; color: #66727d;
         font-size: 12.5px; }
"""

PAGE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Alacer performance dashboard</title>
<style>{style}</style>
</head>
<body>
<header>
  <h1>Alacer <span>perf contracts</span> — baselines as contracts</h1>
  <p class="sub">M5 (architecture §18, ADR-0011): every criterion benchmark is a published
     baseline; the budget gates are the architecture's falsifiable numbers.</p>
</header>
<main>
  <p class="meta">Baseline measured on <code>{commit}</code>{dirty} · {toolchain} ·
     host <code>{fingerprint}</code> · {cpu_model} × {cpu_count} · generated {generated_at}.</p>

  <h2>Budget contracts (host-independent)</h2>
  <table>
    <tr><th>Contract</th><th class="num">Baseline ratio</th><th class="num">Latest ratio</th><th>Gate</th><th>Status</th><th>Cite</th></tr>
{budget_rows}
  </table>

  <h2>Published baselines — same-host regression gates</h2>
  <p class="meta">Gate: fresh median ≤ published median × (1 + {tolerance:.0%}) on the baseline's host
     fingerprint; foreign hosts (CI runners) skip absolute gates by design and enforce the
     budget contracts instead.</p>
  <table>
    <tr><th>Benchmark</th><th class="num">Published median</th><th class="num">Latest median</th><th class="num">Δ</th><th>vs gate</th><th>Status</th></tr>
{regression_rows}
  </table>

  <h2>Test-enforced gates (outside criterion's model)</h2>
  <table>
    <tr><th>Gate</th><th>Cite</th><th>Enforced by</th></tr>
{gate_rows}
  </table>
</main>
<footer>Generated by <code>scripts/perf_dashboard.py</code> from
<code>benchmarks/contracts.json</code> + <code>benchmarks/baselines/baselines.json</code>.
Regenerate after re-benching; the M5 exit suite checks that every contract is published here.</footer>
</body>
</html>
"""

BAR_COLORS = {"PASS": "#4caf7d", "FAIL": "#e05d78", "SKIP": "#c9b458"}


def budget_rows(
    contracts: dict, published: dict[str, float], fresh: dict[str, float]
) -> tuple[str, int]:
    """Rendered rows plus the number of failing budget contracts."""
    rows = []
    failures = 0
    for budget in contracts["budgets"]:
        num, den = budget["numerator"], budget["denominator"]
        if num in published and den in published:
            base_ratio = published[num] / published[den]
        else:
            base_ratio = float("nan")
        if num in fresh and den in fresh:
            fresh_ratio = fresh[num] / fresh[den]
        else:
            fresh_ratio = base_ratio
        limit, op = float(budget["budget"]), budget["op"]
        ok = fresh_ratio >= limit if op == ">=" else fresh_ratio <= limit
        status = "PASS" if ok else "FAIL"
        failures += 0 if ok else 1
        delta_pct = (fresh_ratio / base_ratio - 1.0) if base_ratio == base_ratio else 0.0
        width = max(5.0, min(100.0, fresh_ratio / (base_ratio if base_ratio else 1.0) * 50.0))
        rows.append(
            f'    <tr><td><code>{esc(budget["id"])}</code></td>'
            f'<td class="num">{base_ratio:.2f}×</td>'
            f'<td class="num">{fresh_ratio:.2f}× <small class="meta">({delta_pct:+.1%})</small></td>'
            f'<td class="num">{op} {limit:g}×</td>'
            f'<td><div class="bar"><i style="width:{width:.0f}%;background:{BAR_COLORS[status]}"></i></div></td>'
            f'<td><span class="badge {status.lower()}">{status}</span></td>'
            f'<td class="meta">{esc(budget["cite"])}</td></tr>'
        )
    return "\n".join(rows), failures


def regression_rows(contracts: dict, published: dict[str, float], fresh: dict[str, float]) -> str:
    default_tolerance = float(contracts["tolerance_default"])
    rows = []
    for gate in contracts["regressions"]:
        bench_id = gate["baseline"]
        tolerance = float(gate.get("tolerance", default_tolerance))
        base = published.get(bench_id)
        if base is None:
            continue
        fresh_value = fresh.get(bench_id)
        if fresh_value is None:
            latest_html, delta_html, status = '<span class="meta">no fresh run</span>', "—", "SKIP"
            bar = 5.0
        else:
            delta = fresh_value / base - 1.0
            ok = delta <= tolerance
            status = "PASS" if ok else "FAIL"
            latest_html, delta_html = human_ns(fresh_value), f"{delta:+.1%}"
            bar = max(3.0, min(100.0, (fresh_value / base) * 40.0))
        rows.append(
            f'    <tr><td><code>{esc(bench_id)}</code></td>'
            f'<td class="num">{human_ns(base)}</td>'
            f'<td class="num">{latest_html}</td>'
            f'<td class="num">{delta_html}</td>'
            f'<td><div class="bar"><i style="width:{bar:.0f}%;background:{BAR_COLORS[status]}"></i></div></td>'
            f'<td><span class="badge {status.lower()}">{status}</span></td></tr>'
        )
    return "\n".join(rows)


def gate_rows(contracts: dict) -> str:
    return "\n".join(
        f'    <tr><td>{esc(gate["gate"])}</td>'
        f'<td class="meta">{esc(gate["cite"])}</td>'
        f'<td><code>{esc(gate["enforced_by"])}</code></td></tr>'
        for gate in contracts["test_enforced_gates"]
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    contracts = load_json(CONTRACTS)
    baseline = load_json(BASELINE)
    published = {k: v["median_ns"] for k, v in baseline["benches"].items()}
    fresh = fresh_medians()
    budget_html, budget_failures = budget_rows(contracts, published, fresh)
    host = baseline["host"]
    dirty = " (dirty tree)" if baseline.get("git", {}).get("dirty") else ""
    page = PAGE.format(
        style=STYLE,
        commit=baseline.get("git", {}).get("commit", "unknown")[:12],
        dirty=dirty,
        toolchain=esc(baseline.get("toolchain", "unknown")),
        fingerprint=host.get("fingerprint", "unknown"),
        cpu_model=esc(host.get("cpu_model", "unknown")),
        cpu_count=esc(host.get("cpu_count", "?")),
        generated_at=esc(baseline.get("generated_at", "")),
        budget_rows=budget_html,
        regression_rows=regression_rows(contracts, published, fresh),
        gate_rows=gate_rows(contracts),
        tolerance=float(contracts["tolerance_default"]),
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(page, encoding="utf-8")
    print(
        f"dashboard: {len(contracts['regressions'])} baselines, "
        f"{len(contracts['budgets'])} budgets "
        f"({budget_failures} failing), {len(contracts['test_enforced_gates'])} "
        f"test-enforced gates -> {args.output}"
    )
    return 1 if budget_failures else 0


if __name__ == "__main__":
    sys.exit(main())
