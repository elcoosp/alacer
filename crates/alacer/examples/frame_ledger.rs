//! The frame's ledger as a consumer's vocabulary (§9.1, ADR-0029): what
//! one write batch costs. A static label and a bound ticker ride one
//! column; the ticker's *Tick* write re-commits exactly one journal layer
//! — the ticker's — while the static label's clean layer replays
//! untouched, and the journal's own ledger (commits, drops, live layers)
//! is read back through `PassPipeline::journal_stats`, the pipeline's one
//! read door over the `SceneJournal::stats` seam.
//!
//! The demo walks both beats at startup, before the window opens: the
//! mount frame's ledger (every layer commits once), then one ticker write
//! batch's ledger (one re-commit). In the window, each *Tick* press is
//! the same one-batch write — and the ledger rides the
//! `alacer::frame_ledger` tracing target, so any subscriber sees the
//! cost per route:
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example frame_ledger
//! ```
//!
//! The workspace's idiom: `tracing` spans at the `alacer::frame_ledger`
//! target, one per route. `ALACER_SOFTWARE=1` forces the softbuffer path.
//!
//! Run with `cargo run -p alacer --example frame_ledger`.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::{Flux, RenderBinding};
use alacer_core::geom::Size;
use alacer_core::mask::Change;
use reactive_graph::traits::{Get as _, Set as _};

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The demo's viewport — the size the startup beats tick against (the
/// window re-ticks at its own size once it opens).
const VIEWPORT: Size = Size::new(420.0, 260.0);

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut alacer_core::passes::PassPipeline) {
    for _ in 0..16 {
        let report = pipeline.tick(VIEWPORT).expect("the demo frame lays out");
        if report.is_idle() {
            return;
        }
    }
}

/// Reads the journal's ledger and announces it on the demo's target —
/// the seam's one read, `PassPipeline::journal_stats`.
fn report_ledger(pipeline: &alacer_core::passes::PassPipeline, beat: &'static str) {
    let ledger = pipeline.journal_stats();
    tracing::info!(
        target: "alacer::frame_ledger",
        commits = ledger.commits,
        drops = ledger.drops,
        live_layers = ledger.live_layers,
        "{beat}"
    );
}

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let _ = pipeline.add_child(root, Label::new("The frame's ledger (ADR-0029)").boxed());

    // The static layer: written once at mount, its journal layer replays
    // untouched for the rest of the demo — the clean-replay half of the
    // contract the m23 exit suite pins.
    let _ = pipeline.add_child(root, Label::new("This layer never re-commits").boxed());

    // The ticker: one bound label, one signal — the write that costs.
    let ticks = pipeline.flux_mut().signal(0_u32);
    let ticker_binding = RenderBinding::new("Ticks: 0".to_string());
    let ticker_node = pipeline.add_child(root, Label::bound(ticker_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        ticks,
        ticker_node,
        &ticker_binding,
        Change::PAINT.union(Change::A11Y),
        |n: &u32| format!("Ticks: {n}"),
    );

    // The interactive write: one press, one batch — the same route the
    // startup beat drives, with the span naming it.
    let _ = pipeline.add_child(
        root,
        Box::new(Button::new("Tick").on_click(move || {
            let _route = tracing::trace_span!(
                target: "alacer::frame_ledger",
                "tick",
                route = "bridge"
            )
            .entered();
            Flux::batch_of(|| ticks.set(ticks.get() + 1));
        })),
    );

    let _ = pipeline.add_child(
        root,
        Label::new("Each Tick re-commits one layer — the ledger rides alacer::frame_ledger")
            .boxed(),
    );

    // Beat one: the mount frame. Every layer commits once — the tree's
    // full shape, as the journal accounts it.
    settle(&mut pipeline);
    report_ledger(
        &pipeline,
        "the mount frame's ledger: every layer committed once",
    );

    // Beat two: one write batch. The ticker's layer re-commits; the
    // static label's clean layer replays untouched.
    let before = pipeline.journal_stats();
    {
        let _route = tracing::trace_span!(
            target: "alacer::frame_ledger",
            "tick",
            route = "startup-demo"
        )
        .entered();
        Flux::batch_of(|| ticks.set(1));
    }
    settle(&mut pipeline);
    let after = pipeline.journal_stats();
    tracing::info!(
        target: "alacer::frame_ledger",
        commits_delta = after.commits - before.commits,
        drops_delta = after.drops - before.drops,
        "one ticker write: the ticker's layer re-committed, the static layer replayed"
    );
    report_ledger(&pipeline, "the one-write frame's ledger");

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer frame ledger",
        (420.0, 260.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
