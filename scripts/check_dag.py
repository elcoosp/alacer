#!/usr/bin/env python3
"""Enforces the crate layering DAG (ADR-9).

The workspace layering is normative (architecture section 4.2). Cargo makes
dependency cycles impossible, but this script enforces the *layer order*:
no crate may depend on a crate above its layer, and no layer may skip the
facade for public re-exports.

Exits non-zero with a diff-friendly report on violations.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

# Layer order: bottom of the stack first. A crate may only depend on crates
# in its own layer or LOWER layers. The devtools observe the core seam only;
# the harness consumes them for the M4 exit suite (ADR-0010).
LAYERS: dict[str, list[str]] = {
    "foundation": ["alacer-core", "alacer-macro", "alacer-view"],
    "surfaces": ["alacer-layout", "alacer-text", "alacer-render", "alacer-a11y"],
    "widgets": ["alacer-widgets"],
    "platform": ["alacer-platform"],
    "devtools": ["alacer-inspect"],
    "harness": ["alacer-testing"],
    "facade": ["alacer"],
}

# Internal crate -> declared path deps it is ALLOWED to use. Extra deps fail.
ALLOWED: dict[str, set[str]] = {
    # Foundation: core depends on nothing internal; the proc-macro is a
    # pure leaf (generated code references ::alacer_view paths at the call
    # site — no internal dependency needed, the dioxus-rsx pattern).
    "alacer-core": set(),
    "alacer-macro": set(),
    "alacer-view": {"alacer-core"},
    # Surfaces depend on foundation only.
    "alacer-layout": {"alacer-core"},
    "alacer-text": {"alacer-core"},
    "alacer-render": {"alacer-core"},
    "alacer-a11y": {"alacer-core"},
    # Widgets consume the core seam only (ADR-9 M1 tightening: the text
    # services reach widgets through alacer-core's TextSeam, installed at
    # assembly sites).
    # Widgets register the default element set into the view registry
    # (ADR-9 M1: the view machinery stays in the foundation; the concrete
    # element set registers itself where the widget types live).
    "alacer-widgets": {"alacer-core", "alacer-view"},
    # Platform drives the whole pipeline.
    "alacer-platform": {
        "alacer-core",
        "alacer-layout",
        "alacer-text",
        "alacer-render",
        "alacer-a11y",
        "alacer-widgets",
    },
    # Harness drives everything except the facade (which re-exports it).
    # The rsx! macro + view tree are foundation leaves: snapshot scenarios
    # are written in the same view DSL the users get (ADR-10). The M4 exit
    # suite drives the inspector through the real pipeline (ADR-0010).
    "alacer-testing": {
        "alacer-core",
        "alacer-macro",
        "alacer-view",
        "alacer-layout",
        "alacer-text",
        "alacer-render",
        "alacer-a11y",
        "alacer-widgets",
        "alacer-inspect",
    },
    # Devtools observe the core seam only (ADR-0010): the causal vocabulary,
    # the observer sink, and nothing above the foundation.
    "alacer-inspect": {"alacer-core"},
    # Facade re-exports everything.
    "alacer": {
        "alacer-core",
        "alacer-macro",
        "alacer-layout",
        "alacer-text",
        "alacer-render",
        "alacer-a11y",
        "alacer-view",
        "alacer-widgets",
        "alacer-platform",
        "alacer-testing",
        "alacer-inspect",
    },
}

INTERNAL = set(ALLOWED)


def layer_of(crate: str) -> int:
    for order, (name, members) in enumerate(LAYERS.items()):
        if crate in members:
            return order
    raise SystemExit(f"unknown internal crate in layer map: {crate}")


def check(root: Path) -> list[str]:
    violations: list[str] = []
    for manifest in sorted(root.glob("crates/*/Cargo.toml")):
        crate = manifest.parent.name
        if crate not in INTERNAL:
            violations.append(f"unknown internal crate directory: {manifest.parent}")
            continue
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        used: set[str] = set()
        for section in ("dependencies", "dev-dependencies", "build-dependencies"):
            for dep in (data.get(section) or {}):
                if dep in INTERNAL:
                    used.add(dep)
        disallowed = used - ALLOWED[crate]
        for dep in sorted(disallowed):
            violations.append(
                f"{crate}: depends on {dep} — not in the allowed set (ADR-9 DAG)"
            )
        for dep in used:
            if layer_of(dep) > layer_of(crate):
                violations.append(
                    f"{crate}: depends DOWNWARD-inverting on {dep} "
                    f"(layer order violated)"
                )
    return violations


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    violations = check(root)
    if violations:
        print("ADR-9 layering violations:\n")
        for violation in violations:
            print(f"  - {violation}")
        return 1
    allowed_total = sum(len(v) for v in ALLOWED.values())
    print(f"layering OK: {len(INTERNAL)} internal crates, {allowed_total} allowed edges")
    return 0


if __name__ == "__main__":
    sys.exit(main())
