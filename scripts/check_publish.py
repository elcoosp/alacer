#!/usr/bin/env python3
"""Publish-readiness pre-flight: proves the 0.17.0 cut packages cleanly.

The release workflow (.github/workflows/release.yml) verifies, builds the
changelog, and publishes on a `v*` tag — but nothing before that tag ever
proves the twelve workspace crates actually `cargo package` in the order
crates.io demands. This script is that pre-flight.

The mechanics it has to model (discovered the hard way, on cargo 1.99):

`cargo package` resolves the *published* manifest of each crate — paths
stripped, workspace inheritance materialized — against a registry. A first
cut's siblings are by definition not on crates.io yet, so a bare loop over
`cargo package -p <crate>` cannot resolve any internal dependency. The
script therefore simulates the registry fill that the real sequential
publish performs:

1. `cargo vendor` unpacks every locked external crate into
   `target/publish-vendor/` (the locked versions, exactly).
2. A transient `.cargo/config.toml` replaces the `crates.io` source with
   that directory for the duration of the run (restored byte-identically
   — the file is removed in a `finally`).
3. In topological order (runtime edges), each crate runs
   `cargo package -p <crate> --allow-dirty`; on success its `.crate`
   tarball is unpacked into the vendor directory — the registry fill.
4. When a crate's dependency closure (runtime *and* dev edges — package
   resolution covers dev-dependencies too) names a sibling whose package
   step has not run yet, that sibling is seeded into the vendor directory
   from workspace source with a materialized manifest. This happens for
   the `alacer-a11y` ↔ `alacer-testing` dev-cycle, which cannot be
   published in *any* order against a real empty registry — the seed is
   the simulation's stand-in for "the sibling is already published", and
   the report calls the gap out.

The result: every one of the twelve `.crate` tarballs the cut will upload
is created and verified in isolation against a registry whose internal
content is the cut's own earlier tarballs (plus the documented seeds).
`--allow-dirty` is passed because the handoff regime leaves the tree
intentionally dirty; the tarball content is what a tag publish ships.

Disk management: between crates the script removes the extracted
verification directories under `target/package/` (keeping the `.crate`
files) — the builds are the sandbox's pressure point. `--dry-run` prints
the order and staging plan only.

Run with `python3 scripts/check_publish.py [--no-simulate] [--dry-run]
[--root R]` from anywhere.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
from pathlib import Path

# Free space below which the next verification build is refused (the
# sandbox that motivated the regime holds 9.9 GB).
DEFAULT_THRESHOLD_GIB = 4.0

# The vendor tree lives OUTSIDE target/ so the between-crate `cargo clean`
# (target/debug is where verification builds actually accumulate — they
# share the main workspace target) never touches the registry fill. It is
# transient: removed in the finally, along with the source config.
VENDOR_DIR = ".publish-vendor"
CONFIG_REL = ".cargo/config.toml"

# The manifest tables whose `key.workspace = true` inheritance the seed
# manifests materialize.
DEP_SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")


def parse_members(root: Path) -> list[str]:
    """The workspace member list, reduced to crate names, sorted."""
    manifest = root / "Cargo.toml"
    if not manifest.is_file():
        raise SystemExit(f"no workspace manifest at {manifest}")
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    members = data.get("workspace", {}).get("members", [])
    return sorted(member.removeprefix("crates/") for member in members)


def build_graph(root: Path, members: set[str], include_dev: bool = False) -> dict[str, set[str]]:
    """Internal edges per crate from the crate manifests.

    Runtime edges gate the publish order. With `include_dev`, dev-only
    edges join in (package resolution covers dev-dependencies, so a
    crate's *staging* closure needs them). A dep key rides the
    `.workspace = true` shorthand or names the crate directly.
    """
    sections = DEP_SECTIONS if include_dev else ("dependencies", "build-dependencies")
    graph: dict[str, set[str]] = {}
    for crate in sorted(members):
        manifest = root / "crates" / crate / "Cargo.toml"
        if not manifest.is_file():
            raise SystemExit(f"missing crate manifest: {manifest}")
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        edges: set[str] = set()
        for section in sections:
            for dep in data.get(section) or {}:
                name = dep.removesuffix(".workspace")
                if name in members:
                    edges.add(name)
        graph[crate] = edges
    return graph


def topological_order(graph: dict[str, set[str]]) -> list[str]:
    """Kahn's sort, alphabetical among ready crates; refuses a cycle."""
    remaining = {crate: set(deps) for crate, deps in graph.items()}
    order: list[str] = []
    while remaining:
        ready = sorted(crate for crate, deps in remaining.items() if not deps)
        if not ready:
            cycle = ", ".join(sorted(remaining))
            raise SystemExit(f"dependency cycle among: {cycle}")
        for crate in ready:
            order.append(crate)
            del remaining[crate]
        for deps in remaining.values():
            deps.difference_update(ready)
    return order


def transitive_closure(graph: dict[str, set[str]], crate: str) -> set[str]:
    """Every internal crate reachable from `crate` (edges as given)."""
    seen: set[str] = set()
    frontier = [crate]
    while frontier:
        current = frontier.pop()
        for dep in graph.get(current, ()):
            if dep not in seen:
                seen.add(dep)
                frontier.append(dep)
    seen.discard(crate)
    return seen


def plan_staging(order: list[str], closures: dict[str, set[str]]) -> dict[str, set[str]]:
    """Seeds each crate needs at its turn: closure minus already-packaged.

    Everything earlier in the order packages first and stages its real
    tarball; anything later in the order (dev-cycle siblings) is seeded
    from workspace source for resolution only.
    """
    plan: dict[str, set[str]] = {}
    for index, crate in enumerate(order):
        packaged = set(order[:index])
        plan[crate] = set(closures[crate]) - packaged
    return plan


def expand_seed_universe(
    candidates: set[str], graph_all: dict[str, set[str]], real: set[str]
) -> set[str]:
    """Grows the seed set until closed under the internal edges.

    A seed's manifest names its own dependencies, so seeding one crate
    drags its whole (runtime + dev) closure in — including the crate
    currently being packaged, which its dev-cycle seed may reference.
    Anything already staged as a real tarball stays out.
    """
    universe: set[str] = set()
    frontier = list(candidates)
    while frontier:
        current = frontier.pop()
        if current in universe or current in real:
            continue
        universe.add(current)
        frontier.extend(graph_all.get(current, ()))
    return universe


def package_command(crate: str) -> list[str]:
    """The per-crate proof command. `--allow-dirty` because the handoff
    regime leaves the tree dirty by design (see module docstring)."""
    return ["cargo", "package", "-p", crate, "--allow-dirty"]


def _emit_value(value: object) -> str:
    """Renders a TOML scalar/array for the materialized manifests."""
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        escaped = value.replace("\\", "\\\\").replace('"', '\\"')
        return f'"{escaped}"'
    if isinstance(value, list):
        return "[" + ", ".join(_emit_value(item) for item in value) + "]"
    raise SystemExit(f"unsupported TOML value in workspace inheritance: {value!r}")


def _emit_dep(entry: object) -> str:
    """Renders one dependency entry: a version string or an inline table
    with the `path` (workspace-local) key stripped."""
    if isinstance(entry, str):
        return _emit_value(entry)
    if isinstance(entry, dict):
        fields = {key: value for key, value in entry.items() if key != "path"}
        body = ", ".join(f"{key} = {_emit_value(value)}" for key, value in fields.items())
        return "{ " + body + " }"
    raise SystemExit(f"unsupported dependency entry: {entry!r}")


def materialize_manifest(root: Path, crate: str) -> str:
    """The published shape of a member manifest: workspace inheritance
    resolved to literals, `path` keys stripped, `[lints]` dropped.

    Line surgery rather than a full TOML re-emit, so everything the
    script doesn't understand stays exactly as written.
    """
    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    package_fields = workspace.get("workspace", {}).get("package", {})
    workspace_deps = workspace.get("workspace", {}).get("dependencies", {})
    source = (root / "crates" / crate / "Cargo.toml").read_text(encoding="utf-8")

    out: list[str] = []
    dropped_table: str | None = None
    current_table = ""
    for line in source.splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            current_table = stripped[1:-1]
            if current_table == "lints":
                dropped_table = current_table
                continue
            dropped_table = None
            out.append(line)
            continue
        if dropped_table == "lints":
            continue
        key, equals, value = stripped.partition("=")
        if not equals:
            out.append(line)
            continue
        key = key.strip()
        bare = key.removesuffix(".workspace")
        if current_table == "package" and key.endswith(".workspace") and stripped == f"{key} = true":
            if bare not in package_fields:
                raise SystemExit(f"{crate}: inherits unset workspace package field `{bare}`")
            out.append(f"{bare} = {_emit_value(package_fields[bare])}")
            continue
        if current_table in DEP_SECTIONS and key.endswith(".workspace") and stripped == f"{key} = true":
            if bare not in workspace_deps:
                raise SystemExit(f"{crate}: inherits unset workspace dependency `{bare}`")
            out.append(f"{bare} = {_emit_dep(workspace_deps[bare])}")
            continue
        if current_table in DEP_SECTIONS and value.strip() == "{ workspace = true, optional = true }":
            if bare not in workspace_deps:
                raise SystemExit(f"{crate}: inherits unset workspace dependency `{bare}`")
            rendered = _emit_dep(workspace_deps[bare])
            if rendered.startswith('"'):
                rendered = "{ version = " + rendered + ", optional = true }"
            else:
                rendered = rendered[:-2] + ", optional = true }"
            out.append(f"{bare} = {rendered}")
            continue
        out.append(line)
    return "\n".join(out) + "\n"


def _write_checksums(crate_dir: Path, crate_file: Path | None) -> None:
    """The directory-source checksum manifest for a staged crate."""
    files: dict[str, str] = {}
    for file in sorted(crate_dir.rglob("*")):
        if file.is_file() and file.name != ".cargo-checksum.json":
            relative = file.relative_to(crate_dir).as_posix()
            files[relative] = hashlib.sha256(file.read_bytes()).hexdigest()
    payload: dict[str, object] = {"files": files, "package": None}
    if crate_file is not None:
        payload["package"] = hashlib.sha256(crate_file.read_bytes()).hexdigest()
    (crate_dir / ".cargo-checksum.json").write_text(
        json.dumps(payload), encoding="utf-8"
    )


def seed_crate(root: Path, crate: str, vendor: Path) -> Path:
    """Seeds a not-yet-packaged sibling from workspace source: the crate
    directory copied into the vendor tree, its manifest materialized to
    the published shape. The stand-in for "already on the registry"."""
    target = vendor / crate
    if target.exists():
        return target
    shutil.copytree(root / "crates" / crate, target)
    (target / "Cargo.toml").write_text(materialize_manifest(root, crate), encoding="utf-8")
    _write_checksums(target, None)
    return target


def stage_real_crate(root: Path, crate: str, vendor: Path, crate_file: Path) -> Path:
    """Stages a successfully packaged crate: its `.crate` tarball unpacked
    into the vendor tree (the registry fill), replacing any seed."""
    target = vendor / crate
    if target.exists():
        shutil.rmtree(target)
    with tarfile.open(crate_file, "r:gz") as archive:
        archive.extractall(vendor, filter="data")
    staged = vendor / crate_file.stem.removesuffix(".crate")
    if staged != target:
        staged.rename(target)
    _write_checksums(target, crate_file)
    return target


def _run(command: list[str], cwd: str) -> int:
    """The real runner: subprocess exit code, output inherited (streamed)."""
    return subprocess.run(command, cwd=cwd).returncode


def wipe_verification_artifacts(root: Path, runner=_run) -> None:
    """The between-crate disk valve: a full `cargo clean`.

    Verification builds share the main workspace `target/debug` (which is
    why they are fast), so that directory is what actually accumulates —
    and the vendor tree sits outside `target/`, where it survives.
    """
    runner(["cargo", "clean"], cwd=str(root))


def needs_clean(free_bytes: int, threshold_bytes: int) -> bool:
    """Whether free space is under the clean threshold."""
    return free_bytes < threshold_bytes


def write_source_config(root: Path, vendor: Path) -> None:
    """The transient source replacement: crates.io reads the vendor tree."""
    cargo_dir = root / ".cargo"
    config = cargo_dir / "config.toml"
    if config.exists():
        raise SystemExit(f"{config} already exists — move it aside before the proof")
    cargo_dir.mkdir(exist_ok=True)
    config.write_text(
        "[source.crates-io]\n"
        'replace-with = "alacer-cut"\n'
        "\n"
        "[source.alacer-cut]\n"
        f'directory = "{vendor.relative_to(root).as_posix()}"\n',
        encoding="utf-8",
    )


def remove_source_config(root: Path) -> None:
    """Restores the tree: the transient config goes away byte-clean."""
    cargo_dir = root / ".cargo"
    config = cargo_dir / "config.toml"
    if config.is_file():
        config.unlink()
    try:
        cargo_dir.rmdir()
    except OSError:
        pass


def run_proof(
    root: Path,
    runner=_run,
    simulate: bool = True,
    wipe_between: bool = True,
    threshold_bytes: int = int(DEFAULT_THRESHOLD_GIB * 1024**3),
    keep_vendor: bool = False,
) -> tuple[int, int]:
    """Packages every crate in publish order; returns (passed, failed).

    Stops at the first failing crate. With `simulate`, the vendor tree
    stands in for the registry (see the module docstring). Between crates
    a full `cargo clean` runs (`wipe_between`) — verification builds share
    the main `target/debug`, which is what fills the disk; the threshold
    refuses to START a crate below it, before a build can ENOSPC. The
    vendor tree and the source config are transient; `keep_vendor` leaves
    the fill in place for inspection.
    """
    order = topological_order(build_graph(root, set(parse_members(root))))
    print(f"publish order ({len(order)} crates): {', '.join(order)}")
    real_staged: set[str] = set()
    if simulate:
        whole = build_graph(root, set(parse_members(root)), include_dev=True)
        closures = {crate: transitive_closure(whole, crate) for crate in order}
        plan = plan_staging(order, closures)
        for crate, seeds in plan.items():
            if seeds:
                print(f"  seeds for {crate}: {', '.join(sorted(seeds))}")
    else:
        plan = {crate: set() for crate in order}

    vendor = root / VENDOR_DIR
    config_written = False
    passed = 0
    failed = 0
    try:
        if simulate:
            print("--- vendoring locked externals (the local registry floor) ---")
            if runner(["cargo", "vendor", VENDOR_DIR], cwd=str(root)) != 0:
                print("FAILED: cargo vendor")
                return 0, 1
            write_source_config(root, vendor)
            config_written = True
        for index, crate in enumerate(order, start=1):
            free = shutil.disk_usage(root).free
            if index > 1 and wipe_between:
                print(f"--- cargo clean between crates ({free // 1024**3} GiB free) ---")
                wipe_verification_artifacts(root, runner)
            free = shutil.disk_usage(root).free
            if needs_clean(free, threshold_bytes):
                print(f"REFUSED: {free // 1024**3} GiB free is below the "
                      f"{threshold_bytes // 1024**3} GiB threshold before {crate}")
                failed += 1
                break
            if simulate:
                universe = expand_seed_universe(plan[crate], whole, real_staged)
                for dep in sorted(universe):
                    seeded = seed_crate(root, dep, vendor)
                    print(f"  seeded {dep} -> {seeded.relative_to(root)}")
            print(f"--- packaging {crate} ({index}/{len(order)}) ---")
            if runner(package_command(crate), cwd=str(root)) == 0:
                passed += 1
                if simulate:
                    crate_file = root / "target" / "package" / f"{crate}-*.crate"
                    matches = sorted(crate_file.parent.glob(crate_file.name))
                    if not matches:
                        raise SystemExit(f"cargo package reported success but no .crate for {crate}")
                    staged = stage_real_crate(root, crate, vendor, matches[-1])
                    print(f"  staged {matches[-1].name} -> {staged.relative_to(root)}")
                real_staged.add(crate)
            else:
                print(f"FAILED: cargo package -p {crate}")
                failed += 1
                break
    finally:
        if config_written:
            remove_source_config(root)
        if simulate and not keep_vendor and vendor.exists():
            shutil.rmtree(vendor, ignore_errors=True)
    return passed, failed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument(
        "--no-simulate",
        action="store_true",
        help="bare cargo package per crate (fails on interdependent first cuts)",
    )
    parser.add_argument(
        "--no-wipe",
        action="store_true",
        help="keep the extracted verification builds between crates",
    )
    parser.add_argument(
        "--threshold-gib",
        type=float,
        default=DEFAULT_THRESHOLD_GIB,
        help="refuse when free space drops below this many GiB",
    )
    parser.add_argument("--dry-run", action="store_true", help="print the order and plan only")
    parser.add_argument(
        "--keep-vendor",
        action="store_true",
        help="leave the transient vendor tree in place for inspection",
    )
    args = parser.parse_args()

    order = topological_order(build_graph(args.root, set(parse_members(args.root))))
    if args.dry_run:
        whole = build_graph(args.root, set(parse_members(args.root)), include_dev=True)
        plan = plan_staging(order, {c: transitive_closure(whole, c) for c in order})
        print(f"publish order ({len(order)} crates): {', '.join(order)}")
        for crate, seeds in plan.items():
            if seeds:
                print(f"  seeds for {crate}: {', '.join(sorted(seeds))}")
        return 0

    passed, failed = run_proof(
        args.root,
        simulate=not args.no_simulate,
        wipe_between=not args.no_wipe,
        threshold_bytes=int(args.threshold_gib * 1024**3),
        keep_vendor=args.keep_vendor,
    )
    total = len(order)
    if failed:
        print(f"publish proof FAILED: {passed}/{total} packaged")
        return 1
    print(f"publish proof OK: {passed}/{total} crates packaged in publish order")
    return 0


if __name__ == "__main__":
    sys.exit(main())
