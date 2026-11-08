//! The M5 exit criteria, as executable spec (architecture §18, ADR-0011):
//!
//! - **The examples gallery** — the gallery example exercises the core
//!   widget vocabulary and the catalog (`docs/examples.md`) lists every
//!   shipped example with its contract;
//! - **The migration guides** — the egui and Iced notes exist, each with the
//!   full required section set, so a port covers the whole surface, not the
//!   happy path;
//! - **Criterion baselines published as contracts** — the exit criterion
//!   itself: every workspace benchmark is a published baseline
//!   (`benchmarks/baselines/baselines.json`, environment recorded), every
//!   baseline is gated by a regression contract and every contract references
//!   a published baseline (a bijection — no rotting data, no unmeasured
//!   benches), the budget contracts hold on the published data (the W1 ≥ 2×
//!   gate, measured), and the generated dashboard publishes every contract.
//!
//! No numbers are hard-coded here beyond the architecture's own budget: the
//! measured medians live in the published baseline, produced by
//! `scripts/collect_baseline.py` from real `cargo bench` runs.

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The corpus pin, mirroring the M1 suite's 30-scenario pin: 47 criterion
/// cases across the seven benching crates (38 M5 cases + the M6 workloads:
/// W2 ×2, W3 ×1, W5 ×2 — ADR-0012 + the M8 input-storm pair — ADR-0014 + the
/// M9 a11y-ratio pair — ADR-0015). Growing the corpus is the coordinated
/// bench + contracts.json + baseline-regeneration rule, reviewed here and by
/// the M6/M7/M8/M9 suites (which assert this pin agrees with their own).
const EXPECTED_BENCH_COUNT: usize = 47;

/// The one criterion-expressible architecture budget in M5's contract set:
/// the W1 gate (§18 M2, ADR-0008).
const W1_GATE: f64 = 2.0;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .expect("workspace root resolves")
}

fn read_workspace_file(path: &str) -> String {
    let path = workspace_root().join(path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("required file {} missing: {error}", path.display()))
}

fn parse_workspace_json(path: &str) -> Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

fn baseline_ids(baselines: &Value) -> BTreeSet<String> {
    baselines["benches"]
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

fn regression_baseline_ids(contracts: &Value) -> BTreeSet<String> {
    contracts["regressions"]
        .as_array()
        .map(|gates| {
            gates
                .iter()
                .filter_map(|gate| gate["baseline"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// The published baseline
// ---------------------------------------------------------------------------

#[test]
fn published_baselines_carry_the_full_schema_and_environment() {
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    assert_eq!(
        baselines["schema"].as_i64(),
        Some(1),
        "the baseline schema version must be 1"
    );

    let benches = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object");
    assert_eq!(
        benches.len(),
        EXPECTED_BENCH_COUNT,
        "the published corpus is pinned (regenerate with scripts/collect_baseline.py)"
    );
    for (id, entry) in benches {
        let median = entry["median_ns"]
            .as_f64()
            .unwrap_or_else(|| panic!("{id}: median_ns missing"));
        assert!(median > 0.0, "{id}: median must be a positive measurement");
        let mean = entry["mean_ns"]
            .as_f64()
            .unwrap_or_else(|| panic!("{id}: mean_ns missing"));
        assert!(mean > 0.0, "{id}: mean must be a positive measurement");
        let mad = entry["median_abs_dev_ns"].as_f64().unwrap_or_default();
        assert!(
            mad >= 0.0,
            "{id}: median absolute deviation is non-negative"
        );
        let samples = entry["samples"]
            .as_u64()
            .unwrap_or_else(|| panic!("{id}: samples missing"));
        assert!(
            samples >= 10,
            "{id}: {samples} samples is not a measurement, it is a rumble"
        );
    }

    // The environment: what was measured, where, on what — the honesty
    // header that makes the numbers citable rather than mysterious.
    let commit = baselines["git"]["commit"]
        .as_str()
        .expect("baseline records its git commit");
    assert_eq!(commit.len(), 40, "git.commit is a full sha");
    assert!(
        baselines["git"]["dirty"].is_boolean(),
        "baseline records whether the tree was dirty"
    );
    let toolchain = baselines["toolchain"].as_str().expect("toolchain recorded");
    assert!(
        toolchain.starts_with("rustc"),
        "toolchain is a rustc version string"
    );
    let fingerprint = baselines["host"]["fingerprint"]
        .as_str()
        .expect("host fingerprint recorded")
        .to_owned();
    assert_eq!(
        fingerprint.len(),
        16,
        "the host fingerprint is sha256-truncated to 16 hex"
    );
    assert!(
        baselines["host"]["cpu_model"]
            .as_str()
            .is_some_and(|model| !model.is_empty()),
        "cpu model recorded"
    );
    assert!(
        baselines["generated_at"].as_str().is_some(),
        "generation timestamp recorded"
    );

    // The generated_at timestamp parses as RFC 3339 (the script emits one).
    let generated_at = baselines["generated_at"].as_str().expect("generated_at");
    let date_part = generated_at
        .split(['T', 't'])
        .next()
        .expect("timestamp has a date part");
    assert_eq!(
        date_part.split('-').count(),
        3,
        "generated_at is an ISO-8601 timestamp, got {generated_at}"
    );
}

#[test]
fn every_published_baseline_has_a_regression_contract_and_vice_versa() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");

    let published = baseline_ids(&baselines);
    let contracted = regression_baseline_ids(&contracts);
    assert!(!published.is_empty(), "no baselines published");
    assert!(!contracted.is_empty(), "no regression contracts");

    let ungated: Vec<_> = published.difference(&contracted).collect();
    assert!(
        ungated.is_empty(),
        "published baselines without a regression gate (add the gate to contracts.json): {ungated:?}"
    );
    let unmeasured: Vec<_> = contracted.difference(&published).collect();
    assert!(
        unmeasured.is_empty(),
        "regression gates without a published baseline (re-run scripts/collect_baseline.py): {unmeasured:?}"
    );
    assert_eq!(
        contracted.len(),
        EXPECTED_BENCH_COUNT,
        "the contracted corpus is pinned"
    );

    // Contract hygiene: tolerances exist and are sane, ids unique.
    let tolerance_default = contracts["tolerance_default"]
        .as_f64()
        .expect("tolerance_default");
    assert!(
        tolerance_default > 0.0 && tolerance_default < 1.0,
        "the default tolerance is a fractional regression allowance in (0, 1)"
    );
    let mut gate_ids = BTreeSet::new();
    for gate in contracts["regressions"]
        .as_array()
        .expect("regressions array")
    {
        let id = gate["id"].as_str().expect("gate id");
        assert!(
            gate_ids.insert(id.to_owned()),
            "duplicate regression id {id}"
        );
        assert_eq!(
            gate["scope"].as_str(),
            Some("same-host"),
            "{id}: gates are same-host scoped"
        );
    }

    // The budget set: every budget cites the architecture and is a median
    // ratio over published benches.
    let budgets = contracts["budgets"].as_array().expect("budgets array");
    assert!(!budgets.is_empty(), "at least one budget contract");
    for budget in budgets {
        for field in ["id", "numerator", "denominator", "cite"] {
            assert!(
                budget[field].as_str().is_some_and(|v| !v.is_empty()),
                "budget {} missing {field}",
                budget["id"]
            );
        }
        assert!(
            published.contains(budget["numerator"].as_str().expect("numerator")),
            "budget {} numerator is not a published baseline",
            budget["id"]
        );
        assert!(
            published.contains(budget["denominator"].as_str().expect("denominator")),
            "budget {} denominator is not a published baseline",
            budget["id"]
        );
    }

    // The test-enforced gates are published too: what criterion cannot
    // express, the suite names with its enforcement site.
    for gate in contracts["test_enforced_gates"]
        .as_array()
        .expect("gates array")
    {
        assert!(
            gate["gate"].as_str().is_some_and(|v| !v.is_empty()),
            "gate text present"
        );
        assert!(
            gate["enforced_by"].as_str().is_some_and(|v| !v.is_empty()),
            "each test-enforced gate names its enforcement site"
        );
    }
}

#[test]
fn budget_contracts_hold_on_the_published_data() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let benches = baselines["benches"].as_object().expect("benches object");

    let median_of = |id: &str| -> f64 {
        benches[id]["median_ns"]
            .as_f64()
            .unwrap_or_else(|| panic!("{id} not in the published baseline"))
    };

    for budget in contracts["budgets"].as_array().expect("budgets array") {
        let id = budget["id"].as_str().expect("budget id");
        let numerator = budget["numerator"].as_str().expect("numerator");
        let denominator = budget["denominator"].as_str().expect("denominator");
        let ratio = median_of(numerator) / median_of(denominator);
        let limit = budget["budget"].as_f64().expect("budget limit");
        match budget["op"].as_str().expect("budget op") {
            ">=" => assert!(
                ratio >= limit,
                "budget {id}: measured ratio {ratio:.2}× violates >= {limit}×"
            ),
            "<=" => assert!(
                ratio <= limit,
                "budget {id}: measured ratio {ratio:.2}× violates <= {limit}×"
            ),
            other => panic!("budget {id}: unknown op {other}"),
        }
    }

    // The architecture's own gate value is pinned: W1 ≥ 2× (§18 M2, ADR-0008).
    let w1 = contracts["budgets"]
        .as_array()
        .expect("budgets array")
        .iter()
        .find(|budget| budget["id"].as_str() == Some("w1_incremental_speedup"))
        .expect("the W1 budget contract is published");
    assert_eq!(
        w1["budget"].as_f64(),
        Some(W1_GATE),
        "the W1 gate is the architecture's 2×"
    );
    assert_eq!(
        w1["op"].as_str(),
        Some(">="),
        "the W1 gate is a lower bound"
    );
}

// ---------------------------------------------------------------------------
// The dashboard
// ---------------------------------------------------------------------------

#[test]
fn the_perf_dashboard_publishes_every_contract() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");
    let dashboard = read_workspace_file("benchmarks/dashboard/index.html");

    for id in baseline_ids(&baselines) {
        assert!(
            dashboard.contains(&id),
            "the dashboard does not publish baseline {id} (regenerate with scripts/perf_dashboard.py)"
        );
    }
    for budget in contracts["budgets"].as_array().expect("budgets array") {
        let id = budget["id"].as_str().expect("budget id");
        assert!(
            dashboard.contains(id),
            "the dashboard does not publish budget {id}"
        );
    }
    for gate in contracts["test_enforced_gates"]
        .as_array()
        .expect("gates array")
    {
        let enforced_by = gate["enforced_by"].as_str().expect("enforced_by");
        assert!(
            dashboard.contains(enforced_by),
            "the dashboard does not publish the test-enforced gate's enforcement site"
        );
    }
    assert!(
        dashboard.contains("PASS"),
        "the dashboard renders contract statuses"
    );
    assert!(
        dashboard.contains("baselines as contracts"),
        "the dashboard names the regime it publishes"
    );
}

// ---------------------------------------------------------------------------
// The examples gallery and its catalog
// ---------------------------------------------------------------------------

#[test]
fn the_examples_catalog_covers_every_example() {
    let examples_dir = workspace_root().join("crates/alacer/examples");
    let mut stems = Vec::new();
    for entry in std::fs::read_dir(&examples_dir).expect("examples directory exists") {
        let entry = entry.expect("example entry readable");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            stems.push(
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .expect("utf-8 example name")
                    .to_owned(),
            );
        }
    }
    assert!(!stems.is_empty(), "examples exist");
    let catalog = read_workspace_file("docs/examples.md");
    for stem in stems {
        assert!(
            catalog.contains(&stem),
            "docs/examples.md does not list the `{stem}` example"
        );
    }
    // The catalog documents the gallery and the bless workflow.
    assert!(catalog.contains("gallery"), "the gallery is cataloged");
    assert!(
        catalog.contains("ALACER_BLESS=1"),
        "the catalog points at the snapshot bless workflow"
    );
}

#[test]
fn the_gallery_exercises_the_core_widget_vocabulary() {
    let gallery = read_workspace_file("crates/alacer/examples/gallery.rs");
    for widget in [
        "Label",
        "Button",
        "TextInput",
        "List",
        "Panel",
        "Row",
        "Column",
        "Spacer",
    ] {
        assert!(
            gallery.contains(widget),
            "the gallery must exercise the core widget `{widget}`"
        );
    }
    // The scheduling contract is demonstrated, not bypassed.
    assert!(
        gallery.contains("Flux::batch_of"),
        "the gallery shows the one-batch write seam (ADR-6)"
    );
    assert!(
        gallery.contains("RenderBinding"),
        "the gallery shows the dynamic-value binding seam"
    );
    assert!(
        gallery.contains("theme::"),
        "the gallery uses the shared theme vocabulary"
    );
}

// ---------------------------------------------------------------------------
// The migration guides
// ---------------------------------------------------------------------------

/// The section set both guides must carry: a port that skips one of these
/// is a port that will surprise its author.
const REQUIRED_SECTIONS: [&str; 9] = [
    "Mental model",
    "App shell",
    "Widget mapping",
    "Reactivity",
    "Layout",
    "Input and events",
    "Accessibility",
    "Testing",
    "Interop limits",
];

#[test]
fn the_egui_and_iced_migration_guides_are_complete() {
    for (guide, markers) in [
        (
            "docs/migration/from-egui.md",
            vec![
                "eframe",
                "immediate mode",
                "egui_kittest",
                "request_repaint",
            ],
        ),
        (
            "docs/migration/from-iced.md",
            vec!["Message", "Element", "Task", "Subscription", "update"],
        ),
    ] {
        let text = read_workspace_file(guide);
        for section in REQUIRED_SECTIONS {
            assert!(
                text.contains(&format!("## {section}")),
                "{guide} is missing the required `## {section}` section"
            );
        }
        for marker in markers {
            assert!(
                text.to_ascii_lowercase()
                    .contains(&marker.to_ascii_lowercase()),
                "{guide} never mentions `{marker}` — the guide must name the source framework's actual concepts"
            );
        }
        // The guides point at the runnable canon.
        assert!(
            text.contains("examples/"),
            "{guide} should link the examples that demonstrate the mapped APIs"
        );
    }
    // Both guides carry the honesty header about version drift.
    for guide in ["docs/migration/from-egui.md", "docs/migration/from-iced.md"] {
        let text = read_workspace_file(guide);
        assert!(
            text.contains("honesty rule"),
            "{guide} carries the version-scope honesty note"
        );
    }
}
