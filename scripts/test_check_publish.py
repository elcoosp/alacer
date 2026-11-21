#!/usr/bin/env python3
"""Unit suite for `check_publish.py` — the publish-proof pipeline's logic.

The proof run itself (cargo package per crate, plus the vendor fill) is
slow and disk-hungry, so the suite exercises everything around it on
synthetic workspaces: member parsing, both dependency graphs (runtime and
runtime+dev), the deterministic topological sort, cycle refusal, the
staging plan, manifest materialization, seed/tarball staging, the
verification-build wipe, the transient source config, and the command
shape. The cargo invocation is injected, so nothing here shells out.

Run with `python3 -m unittest scripts.test_check_publish` from the repo
root, or `python3 scripts/test_check_publish.py` directly.
"""

from __future__ import annotations

import hashlib
import json
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path

import check_publish

# A synthetic four-crate workspace (a-core < a-view < a-widgets < a) plus
# a dev-only harness crate that must never enter the publish order.
FIXTURE_MEMBERS = ["a", "a-core", "a-testing", "a-view", "a-widgets"]

FIXTURE_MANIFESTS: dict[str, dict[str, str]] = {
    "a": {"dependencies": "a-core.workspace = true\na-view.workspace = true\na-widgets.workspace = true"},
    "a-core": {"dependencies": "bitflags = \"2\""},
    "a-testing": {"dev-dependencies": "a-widgets.workspace = true"},
    "a-view": {"dependencies": "a-core.workspace = true"},
    "a-widgets": {"dependencies": "a-core.workspace = true\na-view.workspace = true"},
}

RUNTIME_ONLY_MEMBERS = ["a", "a-core", "a-view", "a-widgets"]


def make_workspace(
    members: list[str],
    manifests: dict[str, dict[str, str]],
    root_manifest: str | None = None,
) -> Path:
    """Writes a synthetic workspace into a temp dir and returns its root."""
    root = Path(tempfile.mkdtemp(prefix="check-publish-test-"))
    if root_manifest is None:
        lines = ["[workspace]", 'resolver = "3"', "members = ["]
        lines += [f'    "crates/{name}",' for name in members]
        lines += ["]", "", "[workspace.dependencies]"]
        lines += [f'{name} = {{ path = "crates/{name}", version = "0.1.0" }}' for name in members]
        root_manifest = "\n".join(lines)
    (root / "Cargo.toml").write_text(root_manifest, encoding="utf-8")
    for name, sections in manifests.items():
        crate_dir = root / "crates" / name
        crate_dir.mkdir(parents=True)
        body = ["[package]", f'name = "{name}"', 'version = "0.1.0"']
        for section, deps in sections.items():
            body.append(f"[{section}]")
            body.append(deps)
        (crate_dir / "Cargo.toml").write_text("\n".join(body), encoding="utf-8")
        (crate_dir / "src").mkdir()
        (crate_dir / "src" / "lib.rs").write_text("pub fn stub() {}\n", encoding="utf-8")
    return root


class ParseMembersTest(unittest.TestCase):
    """The workspace member list comes from the root manifest only."""

    def test_reads_the_declared_member_list(self) -> None:
        root = make_workspace(FIXTURE_MEMBERS, FIXTURE_MANIFESTS)
        self.assertEqual(check_publish.parse_members(root), sorted(FIXTURE_MEMBERS))

    def test_missing_root_manifest_refuses(self) -> None:
        root = Path(tempfile.mkdtemp(prefix="check-publish-empty-"))
        with self.assertRaises(SystemExit):
            check_publish.parse_members(root)


class BuildGraphTest(unittest.TestCase):
    """Internal edges ride runtime sections; dev-only joins on request."""

    def test_runtime_edges_are_internal_and_directed(self) -> None:
        root = make_workspace(FIXTURE_MEMBERS, FIXTURE_MANIFESTS)
        graph = check_publish.build_graph(root, set(FIXTURE_MEMBERS))
        self.assertEqual(graph["a"], {"a-core", "a-view", "a-widgets"})
        self.assertEqual(graph["a-view"], {"a-core"})
        self.assertEqual(graph["a-widgets"], {"a-core", "a-view"})
        self.assertEqual(graph["a-core"], set())

    def test_dev_only_dependency_excluded_from_runtime_graph(self) -> None:
        root = make_workspace(FIXTURE_MEMBERS, FIXTURE_MANIFESTS)
        graph = check_publish.build_graph(root, set(FIXTURE_MEMBERS))
        self.assertNotIn("a-widgets", graph["a-testing"])

    def test_dev_edges_join_with_include_dev(self) -> None:
        root = make_workspace(FIXTURE_MEMBERS, FIXTURE_MANIFESTS)
        graph = check_publish.build_graph(root, set(FIXTURE_MEMBERS), include_dev=True)
        self.assertEqual(graph["a-testing"], {"a-widgets"})

    def test_build_dependency_is_an_edge(self) -> None:
        members = ["b-core", "b-macro", "b-user"]
        manifests = {
            "b-core": {},
            "b-macro": {},
            "b-user": {"build-dependencies": "b-macro.workspace = true"},
        }
        root = make_workspace(members, manifests)
        graph = check_publish.build_graph(root, set(members))
        self.assertEqual(graph["b-user"], {"b-macro"})


class TopologicalOrderTest(unittest.TestCase):
    """Publish order: dependencies first, alphabetical among equals."""

    def test_dependencies_precede_dependents(self) -> None:
        graph = {
            "a": {"a-core", "a-view", "a-widgets"},
            "a-core": set(),
            "a-testing": set(),
            "a-view": {"a-core"},
            "a-widgets": {"a-core", "a-view"},
        }
        order = check_publish.topological_order(graph)
        position = {name: index for index, name in enumerate(order)}
        self.assertLess(position["a-core"], position["a-view"])
        self.assertLess(position["a-view"], position["a-widgets"])
        self.assertLess(position["a-widgets"], position["a"])

    def test_ties_are_alphabetical_and_deterministic(self) -> None:
        graph = {"b": set(), "a": set(), "c": set()}
        self.assertEqual(check_publish.topological_order(graph), ["a", "b", "c"])

    def test_cycle_refuses(self) -> None:
        graph = {"x": {"y"}, "y": {"x"}}
        with self.assertRaises(SystemExit):
            check_publish.topological_order(graph)


class TransitiveClosureTest(unittest.TestCase):
    """Closures walk the given edges transitively, excluding self."""

    def test_closure_over_runtime_plus_dev_edges(self) -> None:
        graph = {
            "a": {"a-core", "a-widgets"},
            "a-core": set(),
            "a-testing": set(),
            "a-view": {"a-core"},
            "a-widgets": {"a-core", "a-view"},
        }
        closure = check_publish.transitive_closure(graph, "a")
        self.assertEqual(closure, {"a-core", "a-view", "a-widgets"})

    def test_self_excluded_but_reachable_cycle_members_kept(self) -> None:
        graph = {"x": {"y"}, "y": {"x"}}
        self.assertEqual(check_publish.transitive_closure(graph, "x"), {"y"})


class PlanStagingTest(unittest.TestCase):
    """Seeds = the staging closure minus what already packaged."""

    def test_later_closures_are_seeded(self) -> None:
        order = ["a-core", "a-view", "a-widgets", "a"]
        closures = {
            "a-core": set(),
            "a-view": {"a-core"},
            "a-widgets": {"a-core", "a-view"},
            "a": {"a-core", "a-view", "a-widgets"},
        }
        plan = check_publish.plan_staging(order, closures)
        self.assertEqual(plan["a-core"], set())
        self.assertEqual(plan["a-view"], set())
        self.assertEqual(plan["a-widgets"], set())
        self.assertEqual(plan["a"], set())

    def test_dev_cycle_sibling_is_seeded_not_real(self) -> None:
        # a11y packages before widgets/testing in runtime order, but its
        # dev-deps name them — those ride the seed lane.
        order = ["alacer-core", "alacer-a11y", "alacer-widgets", "alacer-testing"]
        closures = {
            "alacer-core": set(),
            "alacer-a11y": {"alacer-core", "alacer-widgets", "alacer-testing"},
            "alacer-widgets": {"alacer-core"},
            "alacer-testing": {"alacer-core", "alacer-widgets", "alacer-a11y"},
        }
        plan = check_publish.plan_staging(order, closures)
        self.assertEqual(plan["alacer-a11y"], {"alacer-widgets", "alacer-testing"})
        self.assertEqual(plan["alacer-widgets"], set())
        self.assertEqual(plan["alacer-testing"], set())


class ExpandSeedUniverseTest(unittest.TestCase):
    """Seeds close under the internal edges; real tarballs stay out."""

    GRAPH = {
        "p-a11y": {"p-core", "p-widgets", "p-testing"},
        "p-core": set(),
        "p-testing": {"p-core", "p-view", "p-widgets", "p-a11y"},
        "p-view": {"p-core"},
        "p-widgets": {"p-core", "p-view"},
    }

    def test_dev_cycle_pulls_in_the_crate_being_packaged(self) -> None:
        universe = check_publish.expand_seed_universe(
            {"p-testing"}, self.GRAPH, real={"p-core"}
        )
        self.assertIn("p-a11y", universe)
        self.assertIn("p-testing", universe)
        self.assertIn("p-view", universe)
        self.assertIn("p-widgets", universe)
        self.assertNotIn("p-core", universe)

    def test_no_candidates_yields_no_universe(self) -> None:
        self.assertEqual(
            check_publish.expand_seed_universe(set(), self.GRAPH, real=set()), set()
        )


class PackageCommandTest(unittest.TestCase):
    """The proof command packages one crate and tolerates the dirty tree.

    `--no-verify` because the tarball's verification is the proof's own
    explicit step (see `VerifyCommandTest`), not cargo's implicit one.
    """

    def test_command_shape(self) -> None:
        self.assertEqual(
            check_publish.package_command("alacer-core"),
            ["cargo", "package", "-p", "alacer-core", "--allow-dirty", "--no-verify"],
        )


class VerifyCommandTest(unittest.TestCase):
    """The verification step is exactly `cargo check --lib`, nothing more.

    The published package is verified in its shipped shape: the library
    compiles. `cargo test` / `cargo check --all-targets` would demand the
    dev-dependency *targets* — local-development furniture the cut's own
    dev-cycle (alacer-a11y ↔ alacer-testing) turns into a resolution
    deadlock the publish step never faces.
    """

    def test_command_shape(self) -> None:
        self.assertEqual(
            check_publish.verify_command(), ["cargo", "check", "--lib"]
        )

    def test_command_never_names_test_targets(self) -> None:
        command = check_publish.verify_command()
        for banned in ("--all-targets", "--tests", "--benches", "--examples", "test"):
            self.assertNotIn(banned, command)


class VerifyUnpackedTest(unittest.TestCase):
    """Each packaged `.crate` is verified extracted, in the shipped shape."""

    def setUp(self) -> None:
        self.root = make_workspace(["a-core"], {"a-core": {}})
        package_dir = self.root / "target" / "package"
        package_dir.mkdir(parents=True)
        self.crate_file = self._build_crate_tarball("a-core", "0.1.0")

    def _build_crate_tarball(self, crate: str, version: str) -> Path:
        staging = Path(tempfile.mkdtemp(prefix="check-publish-verify-"))
        payload = staging / f"{crate}-{version}"
        payload.mkdir()
        (payload / "Cargo.toml").write_text(
            f'[package]\nname = "{crate}"\nversion = "{version}"\n', encoding="utf-8"
        )
        (payload / "src").mkdir()
        (payload / "src" / "lib.rs").write_text("pub fn real() {}\n", encoding="utf-8")
        crate_file = self.root / "target" / "package" / f"{crate}-{version}.crate"
        with tarfile.open(crate_file, "w:gz") as archive:
            archive.add(payload, arcname=payload.name)
        return crate_file

    def test_verifies_the_extracted_crate_in_the_shipped_shape(self) -> None:
        observed: dict[str, object] = {}

        def recording_runner(command: list[str], **kwargs: object) -> int:
            # The extraction is cleaned after the check, so the shipped
            # shape is captured at call time.
            cwd = Path(str(kwargs["cwd"]))
            observed["command"] = command
            observed["cwd"] = str(cwd)
            observed["manifest"] = (cwd / "Cargo.toml").read_text(encoding="utf-8")
            observed["lib"] = (cwd / "src" / "lib.rs").is_file()
            return 0

        code = check_publish.verify_unpacked(
            self.root, "a-core", self.crate_file, runner=recording_runner
        )
        self.assertEqual(code, 0)
        self.assertEqual(
            observed["command"], check_publish.verify_command()
        )
        extracted = Path(str(observed["cwd"]))
        self.assertEqual(extracted.name, "a-core-0.1.0")
        self.assertEqual(observed["manifest"], '[package]\nname = "a-core"\nversion = "0.1.0"\n')
        self.assertTrue(observed["lib"], "the shipped lib is in the extraction")
        self.assertFalse(extracted.exists(), "the extraction is cleaned after")

    def test_nonzero_verification_code_propagates(self) -> None:
        def failing_runner(_command: list[str], **_kwargs: object) -> int:
            return 3

        code = check_publish.verify_unpacked(
            self.root, "a-core", self.crate_file, runner=failing_runner
        )
        self.assertEqual(code, 3)

    def test_vendor_fill_travels_as_the_extraction_config(self) -> None:
        seen: list[str | None] = []

        def recording_runner(_command: list[str], **kwargs: object) -> int:
            # The extraction is cleaned after the check: capture now.
            cwd = Path(str(kwargs["cwd"]))
            config = cwd / ".cargo" / "config.toml"
            seen.append(config.read_text(encoding="utf-8") if config.is_file() else None)
            return 0

        # No vendor fill: the check resolves against the ambient registry.
        check_publish.verify_unpacked(
            self.root, "a-core", self.crate_file, runner=recording_runner
        )
        self.assertIsNone(seen[-1])
        # With the fill present, the source replacement rides a local
        # config pointing at the absolute vendor path.
        vendor = self.root / check_publish.VENDOR_DIR
        vendor.mkdir()
        check_publish.verify_unpacked(
            self.root, "a-core", self.crate_file, runner=recording_runner
        )
        self.assertIn('replace-with = "alacer-cut"', str(seen[-1]))
        self.assertIn(f'directory = "{vendor.resolve().as_posix()}"', str(seen[-1]))

    def test_extraction_base_is_outside_the_workspace_walk_up(self) -> None:
        # The shipped manifest carries no `[workspace]` table, so an
        # extraction inside the repo is claimed by the repo's workspace
        # walk-up; the temp base must not sit under the repo root.
        seen: list[Path] = []

        def recording_runner(_command: list[str], **kwargs: object) -> int:
            seen.append(Path(str(kwargs["cwd"])))
            return 0

        check_publish.verify_unpacked(
            self.root, "a-core", self.crate_file, runner=recording_runner
        )
        self.assertFalse(
            seen[-1].is_relative_to(self.root),
            "the extraction must live outside the repo tree",
        )


class NeedsCleanTest(unittest.TestCase):
    """Disk pressure below the threshold asks for a `cargo clean`."""

    def test_below_threshold_is_true(self) -> None:
        self.assertTrue(check_publish.needs_clean(1024, 4 * 1024**3))

    def test_above_threshold_is_false(self) -> None:
        self.assertFalse(check_publish.needs_clean(8 * 1024**3, 4 * 1024**3))


class MaterializeManifestTest(unittest.TestCase):
    """Seed manifests resolve workspace inheritance to the published shape."""

    ROOT_MANIFEST = """\
[workspace]
resolver = "3"
members = ["crates/m"]

[workspace.package]
version = "0.2.0"
edition = "2021"

[workspace.dependencies]
a-core = { path = "crates/a-core", version = "0.2.0", features = ["x"] }
subsecond = "0.7"
"""

    M_MANIFEST = """\
[package]
name = "m"
version.workspace = true
edition.workspace = true

[lints]
workspace = true

[dependencies]
a-core.workspace = true
subsecond = { workspace = true, optional = true }
"""

    def setUp(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="check-publish-mat-"))
        (self.root / "Cargo.toml").write_text(self.ROOT_MANIFEST, encoding="utf-8")
        crate_dir = self.root / "crates" / "m"
        crate_dir.mkdir(parents=True)
        (crate_dir / "Cargo.toml").write_text(self.M_MANIFEST, encoding="utf-8")

    def test_inheritance_resolved_and_paths_stripped(self) -> None:
        out = check_publish.materialize_manifest(self.root, "m")
        self.assertIn('version = "0.2.0"', out)
        self.assertIn('edition = "2021"', out)
        self.assertIn('a-core = { version = "0.2.0", features = ["x"] }', out)
        self.assertIn('subsecond = { version = "0.7", optional = true }', out)

    def test_lints_table_dropped(self) -> None:
        out = check_publish.materialize_manifest(self.root, "m")
        self.assertNotIn("[lints]", out)

    def test_output_stays_valid_toml(self) -> None:
        out = check_publish.materialize_manifest(self.root, "m")
        parsed = tomllib.loads(out)
        self.assertEqual(parsed["package"]["version"], "0.2.0")
        self.assertEqual(parsed["dependencies"]["a-core"]["version"], "0.2.0")
        self.assertNotIn("path", parsed["dependencies"]["a-core"])


class SeedCrateTest(unittest.TestCase):
    """Seeds copy the crate source in with a materialized manifest."""

    def setUp(self) -> None:
        self.members = ["a-core", "a-view"]
        self.root = make_workspace(self.members, {
            "a-core": {},
            "a-view": {"dependencies": "a-core.workspace = true"},
        })
        self.vendor = self.root / "vendor-dir"
        self.vendor.mkdir()

    def test_seed_copies_source_and_materializes_manifest(self) -> None:
        seeded = check_publish.seed_crate(self.root, "a-view", self.vendor)
        manifest = tomllib.loads((seeded / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(manifest["package"]["name"], "a-view")
        self.assertEqual((seeded / "src" / "lib.rs").exists(), True)
        checksums = json.loads((seeded / ".cargo-checksum.json").read_text(encoding="utf-8"))
        self.assertIn("Cargo.toml", checksums["files"])
        self.assertIn("src/lib.rs", checksums["files"])
        self.assertIsNone(checksums["package"])

    def test_seed_is_idempotent(self) -> None:
        first = check_publish.seed_crate(self.root, "a-core", self.vendor)
        marker = first / "marker.txt"
        marker.write_text("x", encoding="utf-8")
        second = check_publish.seed_crate(self.root, "a-core", self.vendor)
        self.assertEqual(first, second)
        self.assertTrue(marker.exists())


class StageRealCrateTest(unittest.TestCase):
    """Real tarballs replace seeds; the package hash rides the checksums."""

    def setUp(self) -> None:
        self.members = ["a-core"]
        self.root = make_workspace(self.members, {"a-core": {}})
        self.vendor = self.root / "vendor-dir"
        self.vendor.mkdir()
        package_dir = self.root / "target" / "package"
        package_dir.mkdir(parents=True)

    def _build_crate_tarball(self, crate: str, version: str) -> Path:
        staging = Path(tempfile.mkdtemp(prefix="check-publish-tar-"))
        payload = staging / f"{crate}-{version}"
        payload.mkdir()
        (payload / "Cargo.toml").write_text(
            f'[package]\nname = "{crate}"\nversion = "{version}"\n', encoding="utf-8"
        )
        (payload / "src").mkdir()
        (payload / "src" / "lib.rs").write_text("pub fn real() {}\n", encoding="utf-8")
        crate_file = self.root / "target" / "package" / f"{crate}-{version}.crate"
        with tarfile.open(crate_file, "w:gz") as archive:
            archive.add(payload, arcname=payload.name)
        return crate_file

    def test_real_tarball_replaces_seed_and_hashes_package(self) -> None:
        check_publish.seed_crate(self.root, "a-core", self.vendor)
        crate_file = self._build_crate_tarball("a-core", "0.1.0")
        staged = check_publish.stage_real_crate(self.root, "a-core", self.vendor, crate_file)
        self.assertEqual(staged, self.vendor / "a-core")
        self.assertEqual((staged / "src" / "lib.rs").read_text(), "pub fn real() {}\n")
        checksums = json.loads((staged / ".cargo-checksum.json").read_text(encoding="utf-8"))
        self.assertEqual(
            checksums["package"],
            hashlib.sha256(crate_file.read_bytes()).hexdigest(),
        )


class CleanBetweenTest(unittest.TestCase):
    """The between-crate valve is a full `cargo clean` via the runner."""

    def test_wipe_runs_cargo_clean(self) -> None:
        ran: list[list[str]] = []
        check_publish.wipe_verification_artifacts(
            Path(tempfile.mkdtemp(prefix="check-publish-clean-")),
            runner=lambda command, **_kw: ran.append(command) or 0,
        )
        self.assertEqual(ran, [["cargo", "clean"]])


class SourceConfigTest(unittest.TestCase):
    """The transient source replacement is written and removed cleanly."""

    def setUp(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="check-publish-config-"))
        (self.root / "target" / "publish-vendor").mkdir(parents=True)

    def test_write_and_remove_round_trip(self) -> None:
        vendor = self.root / ".publish-vendor"
        vendor.mkdir(parents=True)
        check_publish.write_source_config(self.root, vendor)
        config = self.root / ".cargo" / "config.toml"
        text = config.read_text(encoding="utf-8")
        self.assertIn('replace-with = "alacer-cut"', text)
        self.assertIn('directory = ".publish-vendor"', text)
        check_publish.remove_source_config(self.root)
        self.assertFalse(config.exists())
        self.assertFalse((self.root / ".cargo").exists())

    def test_existing_config_refuses(self) -> None:
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo" / "config.toml").write_text("[x]\n", encoding="utf-8")
        with self.assertRaises(SystemExit):
            check_publish.write_source_config(
                self.root, self.root / ".publish-vendor"
            )


class RunProofTest(unittest.TestCase):
    """The orchestration walks the order, runs cargo, and reports counts."""

    def test_all_packages_pass_in_runtime_order(self) -> None:
        root = make_workspace(RUNTIME_ONLY_MEMBERS, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in RUNTIME_ONLY_MEMBERS
        })
        ran: list[list[str]] = []

        def fake_runner(command: list[str], **_kwargs: object) -> int:
            ran.append(command)
            if command[:2] == ["cargo", "clean"]:
                return 0
            if command == check_publish.verify_command():
                return 0
            crate = command[command.index("-p") + 1]
            crate_file = root / "target" / "package" / f"{crate}-0.1.0.crate"
            crate_file.parent.mkdir(parents=True, exist_ok=True)
            with tarfile.open(crate_file, "w:gz") as archive:
                archive.add(root / "crates" / crate, arcname=f"{crate}-0.1.0")
            return 0

        summary = check_publish.run_proof(
            root, runner=fake_runner, simulate=False, threshold_bytes=0
        )
        packaged = [
            command[command.index("-p") + 1]
            for command in ran
            if "package" in command
        ]
        self.assertEqual(packaged, ["a-core", "a-view", "a-widgets", "a"])
        self.assertEqual(summary, (4, 0))

    def test_failure_stops_and_counts(self) -> None:
        members = ["a-core", "a-view"]
        root = make_workspace(members, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in members
        })

        def failing_runner(_command: list[str], **_kwargs: object) -> int:
            return 101

        summary = check_publish.run_proof(
            root, runner=failing_runner, simulate=False, threshold_bytes=0
        )
        self.assertEqual(summary, (0, 1))

    def test_verification_failure_counts_and_stops(self) -> None:
        # The tarball packages, but the lib-only check on the extraction
        # fails: the crate counts failed, and the proof stops there.
        root = make_workspace(RUNTIME_ONLY_MEMBERS, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in RUNTIME_ONLY_MEMBERS
        })
        commands: list[list[str]] = []

        def flaky_checker(command: list[str], **_kwargs: object) -> int:
            commands.append(command)
            if command == check_publish.verify_command():
                return 7
            if command[:2] == ["cargo", "vendor"]:
                (root / ".publish-vendor").mkdir(parents=True, exist_ok=True)
                return 0
            if command[:2] == ["cargo", "clean"]:
                return 0
            crate = command[command.index("-p") + 1]
            crate_file = root / "target" / "package" / f"{crate}-0.1.0.crate"
            crate_file.parent.mkdir(parents=True, exist_ok=True)
            with tarfile.open(crate_file, "w:gz") as archive:
                archive.add(root / "crates" / crate, arcname=f"{crate}-0.1.0")
            return 0

        summary = check_publish.run_proof(
            root, runner=flaky_checker, simulate=True, threshold_bytes=0
        )
        self.assertEqual(summary, (0, 1))
        self.assertTrue(
            any(command == check_publish.verify_command() for command in commands)
        )
        # The tree is restored even on the verification stop.
        self.assertFalse((root / ".cargo").exists())

    def test_simulated_run_stages_tarballs_and_restores_the_tree(self) -> None:
        root = make_workspace(RUNTIME_ONLY_MEMBERS, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in RUNTIME_ONLY_MEMBERS
        })
        vendor = root / ".publish-vendor"
        commands: list[list[str]] = []

        def fake_runner(command: list[str], **_kwargs: object) -> int:
            commands.append(command)
            if command[:2] == ["cargo", "vendor"]:
                vendor.mkdir(parents=True, exist_ok=True)
                return 0
            if command[:2] == ["cargo", "clean"]:
                return 0
            if command == check_publish.verify_command():
                return 0
            crate = command[command.index("-p") + 1]
            crate_file = root / "target" / "package" / f"{crate}-0.1.0.crate"
            crate_file.parent.mkdir(parents=True, exist_ok=True)
            with tarfile.open(crate_file, "w:gz") as archive:
                archive.add(root / "crates" / crate, arcname=f"{crate}-0.1.0")
            return 0

        summary = check_publish.run_proof(
            root, runner=fake_runner, simulate=True, keep_vendor=True, threshold_bytes=0
        )
        self.assertEqual(summary, (4, 0))
        self.assertEqual(commands[0], ["cargo", "vendor", ".publish-vendor"])
        packaged = [command[command.index("-p") + 1] for command in commands
                    if "package" in command]
        self.assertEqual(packaged, ["a-core", "a-view", "a-widgets", "a"])
        cleans = [command for command in commands if command[:2] == ["cargo", "clean"]]
        self.assertEqual(len(cleans), 3)
        for crate in RUNTIME_ONLY_MEMBERS:
            self.assertTrue((vendor / crate / ".cargo-checksum.json").exists())
        self.assertFalse((root / ".cargo").exists())

    def test_transient_vendor_removed_by_default(self) -> None:
        root = make_workspace(RUNTIME_ONLY_MEMBERS, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in RUNTIME_ONLY_MEMBERS
        })

        def fake_runner(command: list[str], **_kwargs: object) -> int:
            if command[:2] == ["cargo", "vendor"]:
                (root / ".publish-vendor").mkdir(parents=True, exist_ok=True)
                return 0
            if command[:2] == ["cargo", "clean"]:
                return 0
            if command == check_publish.verify_command():
                return 0
            crate = command[command.index("-p") + 1]
            crate_file = root / "target" / "package" / f"{crate}-0.1.0.crate"
            crate_file.parent.mkdir(parents=True, exist_ok=True)
            with tarfile.open(crate_file, "w:gz") as archive:
                archive.add(root / "crates" / crate, arcname=f"{crate}-0.1.0")
            return 0

        check_publish.run_proof(
            root, runner=fake_runner, simulate=True, threshold_bytes=0
        )
        self.assertFalse((root / ".publish-vendor").exists())
        self.assertFalse((root / ".cargo").exists())

    def test_threshold_refusal_stops_before_first_package(self) -> None:
        root = make_workspace(RUNTIME_ONLY_MEMBERS, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in RUNTIME_ONLY_MEMBERS
        })
        ran: list[list[str]] = []

        def fake_runner(command: list[str], **_kwargs: object) -> int:
            ran.append(command)
            return 0

        summary = check_publish.run_proof(
            root, runner=fake_runner, simulate=False, threshold_bytes=10**12
        )
        self.assertEqual(summary, (0, 1))
        self.assertEqual(ran, [])

    def test_simulated_vendor_failure_counts_and_restores(self) -> None:
        root = make_workspace(RUNTIME_ONLY_MEMBERS, {
            name: manifests
            for name, manifests in FIXTURE_MANIFESTS.items()
            if name in RUNTIME_ONLY_MEMBERS
        })

        def failing_vendor(command: list[str], **_kwargs: object) -> int:
            return 9 if command[:2] == ["cargo", "vendor"] else 0

        summary = check_publish.run_proof(
            root, runner=failing_vendor, simulate=True, threshold_bytes=0
        )
        self.assertEqual(summary, (0, 1))
        self.assertFalse((root / ".cargo").exists())


class FixtureSelfCheck(unittest.TestCase):
    """The fixture manifests stay valid TOML — guards the suite itself."""

    def test_fixture_manifests_parse(self) -> None:
        for name, sections in FIXTURE_MANIFESTS.items():
            for section, deps in sections.items():
                parsed = tomllib.loads(f"[{section}]\n{deps}")
                self.assertIn(section, parsed, name)


if __name__ == "__main__":
    unittest.main()
