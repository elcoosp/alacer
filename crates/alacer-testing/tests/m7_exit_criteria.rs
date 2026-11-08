//! The M7 exit criteria, as executable spec (the two remaining v1
//! definition-of-done items — architecture §1, §14.1/§14.2, §18; ADR-0013):
//!
//! - **The rig is declared** — `benchmarks/rig.json` carries the host, the
//!   toolchain, the protocol (5-run medians), and the fingerprint the
//!   results are collected against;
//! - **The competitor corpus** — {egui, masonry} × {W1, W2, W3} as
//!   standalone, pinned, no-strawman harness projects;
//! - **The runner** — `scripts/run_comparison.py --validate` checks the
//!   committed artifacts end to end;
//! - **The beat** — same-host medians per workload, every measurement
//!   labeled with its scope, the verdict computed from the committed file;
//! - **The publication** — the contract gate and the dashboard section;
//! - **The pins do not move** — the M5/M6 corpus stays at 48 workspace
//!   cases; M7 adds competitor data outside the corpus, never inside it.

use std::path::PathBuf;
use std::process::Command;

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

fn parse_workspace_json(path: &str) -> serde_json::Value {
    let text = read_workspace_file(path);
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path} is not valid JSON: {error}"))
}

/// The beat workloads (§18's kill-criterion trio) and their Alacer-side
/// published baseline ids (the M6 corpus cases the comparison reads).
const BEAT_WORKLOADS: [(&str, &str); 3] = [
    ("w1_churn_10k", "w1/incremental_churn_10pct"),
    ("w2_text_jitter_1k", "w2/jitter_incremental_1k"),
    ("w3_animation_1k", "w3/animation_frame_1k"),
];

const COMPETITORS: [&str; 2] = ["egui", "masonry"];

// ---------------------------------------------------------------------------
// The rig, declared once (ADR-0013 §1)
// ---------------------------------------------------------------------------

#[test]
fn the_rig_is_declared_and_the_results_reference_it() {
    let rig = parse_workspace_json("benchmarks/rig.json");
    assert_eq!(rig["schema"], 1, "the rig declaration is versioned");
    for field in [
        "os",
        "arch",
        "cpu_model",
        "cpu_count",
        "fingerprint",
        "toolchain",
        "backend_mode",
        "vsync",
    ] {
        let value = rig["host"][field]
            .as_str()
            .filter(|value| !value.is_empty())
            .or_else(|| rig[field].as_str())
            .unwrap_or_else(|| panic!("the rig declaration carries a non-empty `{field}`"));
        assert!(!value.trim().is_empty(), "`{field}` is not blank");
    }
    let fingerprint = rig["host"]["fingerprint"]
        .as_str()
        .expect("host fingerprint");
    assert_eq!(
        fingerprint.len(),
        16,
        "the fingerprint is the sha256-truncated host basis, got {fingerprint}"
    );

    // The protocol is part of the declaration (§14.1: 5-run medians).
    assert_eq!(
        rig["protocol"]["runs"], 5,
        "the 5-run median protocol is declared"
    );
    assert!(
        rig["protocol"]["estimator"]
            .as_str()
            .is_some_and(|estimator| { estimator.contains("median") }),
        "the estimator is a median"
    );

    // The results were collected against *this* rig.
    let results = parse_workspace_json("benchmarks/comparisons/results.json");
    assert_eq!(
        results["rig_fingerprint"], rig["host"]["fingerprint"],
        "the results' rig fingerprint matches the declared rig"
    );
}

// ---------------------------------------------------------------------------
// The competitor corpus (ADR-0013 §2)
// ---------------------------------------------------------------------------

#[test]
fn the_competitor_corpus_covers_the_beat_workloads() {
    for framework in COMPETITORS {
        let project = format!("benchmarks/competitors/{framework}_workloads");
        let manifest_text = read_workspace_file(&format!("{project}/Cargo.toml"));

        // Standalone: not a workspace member, its own package.
        assert!(
            manifest_text.contains("[workspace]"),
            "{project} must declare an empty [workspace] (standalone, own lockfile)"
        );

        // The competitor dependency is pinned.
        assert!(
            manifest_text.contains(&format!("{framework} = \"")),
            "{project} depends on {framework}"
        );

        // One bench per beat workload, registered and present.
        for (workload, _) in BEAT_WORKLOADS {
            // The stem drops the workload suffix both harnesses share.
            let stem = workload
                .rsplit_once('_')
                .map_or(workload, |(stem, _)| stem)
                .replace("_jitter", "")
                .replace("_10k", "");
            let stem = match (framework, workload) {
                _ if stem == "w1_churn" || stem == "w2_text" || stem == "w3_animation" => stem,
                _ => panic!("unmapped bench stem for ({framework}, {workload})"),
            };
            assert!(
                manifest_text.contains(&format!("name = \"{stem}\"")),
                "{project} registers the {stem} bench"
            );
            let bench_path = format!("{project}/benches/{stem}.rs");
            let bench = read_workspace_file(&bench_path);
            assert!(
                bench.contains("criterion_group") || bench.contains("Criterion"),
                "{bench_path} is a criterion bench"
            );
        }

        // The no-strawman provenance table: the README names the framework
        // APIs used and the virtualization answer it deliberately does not
        // take (ADR-0013 §2).
        let readme = read_workspace_file(&format!("{project}/README.md"));
        for topic in ["virtualization", "idiomatic", "scope"] {
            assert!(
                readme.to_lowercase().contains(topic),
                "{project}/README.md must document `{topic}` (no-strawman rule)"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The runner (ADR-0013 §4)
// ---------------------------------------------------------------------------

#[test]
fn the_runner_validates_the_committed_artifacts() {
    let output = Command::new("python3")
        .arg(
            workspace_root()
                .join("scripts/run_comparison.py")
                .as_os_str(),
        )
        .arg("--validate")
        .current_dir(workspace_root())
        .output()
        .expect("python3 is available (the CI and the DAG check already require it)");
    assert!(
        output.status.success(),
        "run_comparison.py --validate failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK"),
        "the runner reports the validated corpus, got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// The beat (ADR-0013 §3, §4)
// ---------------------------------------------------------------------------

#[test]
fn the_beat_is_computable_from_the_committed_results() {
    let results = parse_workspace_json("benchmarks/comparisons/results.json");
    let workloads = results["workloads"]
        .as_array()
        .expect("the results carry the workload array");

    assert_eq!(
        workloads.len(),
        BEAT_WORKLOADS.len(),
        "one result row per beat workload"
    );

    for (workload_id, alacer_baseline) in BEAT_WORKLOADS {
        let row = workloads
            .iter()
            .find(|row| row["id"] == workload_id)
            .unwrap_or_else(|| panic!("the results carry the {workload_id} row"));
        assert_eq!(
            row["alacer"]["baseline_id"], alacer_baseline,
            "the Alacer side is the published M6 baseline case"
        );
        let alacer_median = row["alacer"]["median_ns"]
            .as_f64()
            .expect("alacer median is a number");
        assert!(
            alacer_median > 0.0,
            "the {workload_id} alacer median is measured"
        );

        // Every competitor measurement is labeled (§14.1: everything
        // labeled) and carries a scope note (the asymmetry rule, §3).
        let competitors = row["competitors"]
            .as_array()
            .expect("competitor measurements");
        assert_eq!(competitors.len(), COMPETITORS.len());
        for competitor in competitors {
            assert!(
                competitor["median_ns"]
                    .as_f64()
                    .is_some_and(|median| median > 0.0),
                "the {} {workload_id} median is measured",
                competitor["framework"]
            );
            assert_eq!(
                competitor["runs"], 5,
                "the 5-run protocol held for {} {workload_id}",
                competitor["framework"]
            );
            assert!(
                competitor["scope"]
                    .as_str()
                    .is_some_and(|scope| !scope.trim().is_empty()),
                "every competitor row documents its measured scope"
            );
            assert!(
                competitor["version"]
                    .as_str()
                    .is_some_and(|v| !v.is_empty()),
                "the competitor version is recorded"
            );
        }
    }

    // The verdict: same-host, won-set ⊆ workload set.
    assert_eq!(results["same_host"], true, "the beat is same-host (§1)");
    let beat = &results["beat"];
    let won = beat["workloads_won"]
        .as_array()
        .expect("the beat verdict carries the won workloads");
    assert!(
        !won.is_empty(),
        "the committed comparison wins at least one workload — the §1 gate"
    );
    for won_id in won {
        assert!(
            BEAT_WORKLOADS.iter().any(|(id, _)| won_id == id),
            "the won set only names beat workloads, got {won_id}"
        );
    }
}

// ---------------------------------------------------------------------------
// The publication: contract gate + dashboard section (ADR-0013 §4)
// ---------------------------------------------------------------------------

#[test]
fn the_contracts_and_dashboard_publish_the_comparison() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let gate = contracts["test_enforced_gates"]
        .as_array()
        .expect("gates array")
        .iter()
        .find(|gate| gate["id"] == "comparison_corpus_published")
        .expect("the comparison corpus gate is published in contracts.json");
    assert_eq!(
        gate["enforced_by"], "alacer-testing/tests/m7_exit_criteria.rs",
        "the gate names its enforcement site"
    );

    // The dashboard renders the comparison section from the committed file.
    let dashboard = read_workspace_file("benchmarks/dashboard/index.html");
    for marker in [
        "Cross-framework comparison",
        "w1_churn_10k",
        "w3_animation_1k",
    ] {
        assert!(
            dashboard.contains(marker),
            "the dashboard does not publish `{marker}` (regenerate with perf_dashboard.py)"
        );
    }
}

// ---------------------------------------------------------------------------
// The pins do not move (ADR-0013 consequences)
// ---------------------------------------------------------------------------

#[test]
fn the_m5_and_m6_corpus_pins_do_not_move() {
    let contracts = parse_workspace_json("benchmarks/contracts.json");
    let baselines = parse_workspace_json("benchmarks/baselines/baselines.json");

    let bench_count = baselines["benches"]
        .as_object()
        .expect("the baseline carries a benches object")
        .len();
    assert_eq!(
        bench_count, 48,
        "the workspace corpus stays at 48 — M7/M8/M9/M10 add data by the coordinated corpus rule (ADR-0013/ADR-0014/ADR-0015/ADR-0016)"
    );
    assert_eq!(
        contracts["regressions"]
            .as_array()
            .expect("regressions array")
            .len(),
        48,
        "no absolute regression gates on competitor medians (ADR-0011's same-host rule)"
    );

    // All earlier suites still pin 48 — the suites cannot drift.
    for suite in ["m5_exit_criteria.rs", "m6_exit_criteria.rs"] {
        let source = read_workspace_file(&format!("crates/alacer-testing/tests/{suite}"));
        assert!(
            source.contains("const EXPECTED_BENCH_COUNT: usize = 48;"),
            "{suite} must keep the 48-case corpus pin"
        );
    }
}

// ---------------------------------------------------------------------------
// The ADR and the release (ADR-0013 §5)
// ---------------------------------------------------------------------------

#[test]
fn the_adr_documents_the_milestone() {
    let adr = read_workspace_file("docs/adr/ADR-0013-m7-scope.md");
    for piece in [
        "benchmarks/rig.json",
        "scripts/run_comparison.py",
        "egui_workloads",
        "masonry_workloads",
        "w1_churn_10k",
        "comparison_corpus_published",
    ] {
        assert!(adr.contains(piece), "ADR-0013 must name `{piece}`");
    }
    for framework in COMPETITORS {
        assert!(
            adr.contains(framework),
            "ADR-0013 names the {framework} harness"
        );
    }
}

#[test]
fn the_release_state_is_coherent() {
    // Version-agnostic since M10: the release cut bumps the workspace as a
    // branch commit under the tag, so a literal pin would go red the moment
    // the bump landed. The manifest stays the single source of truth and the
    // test asserts coherence around whatever it says.
    let manifest = read_workspace_file("Cargo.toml");
    let version = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.split('\"').next())
        .filter(|version| version.contains('.'))
        .unwrap_or_else(|| panic!("the manifest pins a workspace version"));
    assert!(
        manifest.contains(&format!("\"{version}\"")),
        "the workspace version is well-formed: {version}"
    );

    let changelog = read_workspace_file("CHANGELOG.md");
    assert!(
        changelog.contains("## [Unreleased]") || changelog.contains(&format!("## [{version}]")),
        "the CHANGELOG stages the pending cut or documents {version}"
    );

    let readme = read_workspace_file("README.md");
    assert!(
        readme.contains("M0 through M10") || readme.contains("M0–M10"),
        "the README status reflects M0–M10 (M10, ADR-0016, ships under [Unreleased])"
    );
    assert!(
        readme.contains(version),
        "the README badge matches the workspace version ({version})"
    );
}
