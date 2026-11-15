//! The a11y sink is a seam (ADR-0015): the pipeline's one
//! `TreeUpdate` materialization path is whatever the assembly site
//! installs — `pipeline_default` installs the provided
//! [`alacer_a11y::AccessKitSink`], but a consumer bridging Alacer's
//! accessibility stream into their own stack implements
//! [`alacer_core::passes::A11ySink`] and hands the pipeline *their*
//! door. This demo builds that consumer side: a thin bridge that
//! forwards every snapshot to the provided sink (the real adapter
//! stays) and prices what flowed — every produced `TreeUpdate` walked
//! (the [`alacer_a11y::walk_update`] visit order), the focused node
//! looked up in the update it moved in
//! ([`alacer_a11y::find_in_update`]), the ledger kept beside the
//! bridge.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example a11y_sink_custom
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (named at startup, the
//! `BackendPreference::software_env` door).

use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_a11y::{AccessKitSink, find_in_update, walk_update};
use alacer_core::geom::Size;
use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline, TreeSnapshot};
use alacer_layout::LayoutHost;
use alacer_text::ThreadLocalMeasurer;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// What the bridge prices: one row per `TreeUpdate` the pipeline
/// produced — the nodes the walk named, in visit order (the consumer's
/// own stack would consume exactly this).
struct Ledger {
    updates: Vec<(usize, usize)>,
    focus_seen: Vec<String>,
}

impl Ledger {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            updates: Vec::new(),
            focus_seen: Vec::new(),
        }))
    }
}

/// The consumer's door: forwards to the provided adapter (the real
/// `TreeUpdate` production, untouched) and prices the flow beside it —
/// the seam's composition contract, ADR-0015's one-slot rule.
struct BridgedSink {
    inner: AccessKitSink,
    ledger: Rc<RefCell<Ledger>>,
}

impl A11ySink for BridgedSink {
    fn apply(&mut self, snapshot: Arc<TreeSnapshot>, delta: &A11yDelta) {
        let focus = delta.focus;
        self.inner.apply(snapshot, delta);
        // The consumer's own side: every update the adapter produced,
        // walked in visit order — the node count is the row the ledger
        // keeps (a real bridge would feed its own stack here).
        for update in self.inner.drain_updates() {
            let nodes = walk_update(&update).len();
            let index = self.ledger.borrow().updates.len();
            self.ledger.borrow_mut().updates.push((index, nodes));
        }
        if let Some(node) = focus.and_then(|id| {
            self.inner
                .last_update()
                .and_then(|update| find_in_update(update, id))
        }) {
            let name = format!("{:?}", node.role());
            self.ledger.borrow_mut().focus_seen.push(name);
        }
    }

    fn resolved_tree(&self) -> Option<&TreeSnapshot> {
        self.inner.resolved_tree()
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let report = pipeline.tick(VIEWPORT).expect("the demo's frames lay out");
        if report.is_idle() {
            return;
        }
    }
}

fn main() {
    // The pipeline assembled like `pipeline_default`, the a11y slot
    // behind the demo's bridge (the injected seam is the consumer's
    // own composition site — the layout host, the journal, the text
    // seam, and the clipboard are the production pieces).
    alacer_core::text_seam::install(Rc::new(alacer_text::ThreadLocalTextSeam));
    let clipboard = Rc::new(alacer_platform::clipboard::ArboardClipboard::new());
    alacer_core::clipboard::install(clipboard);
    let ledger = Ledger::new();
    let mut pipeline = PassPipeline::new(
        alacer_core::Flux::new(),
        Box::new(LayoutHost::new(Arc::new(ThreadLocalMeasurer))),
        Box::new(alacer_render::SceneJournalImpl::new()),
        Box::new(BridgedSink {
            inner: AccessKitSink::new(),
            ledger: Rc::clone(&ledger),
        }),
    );

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(root, Label::new("The bridged a11y sink").boxed());
    let submit = pipeline.add_child(root, Button::new("Submit").boxed());
    let _ = pipeline.add_child(root, Button::new("Cancel").boxed());

    // Beat one: the first frame — the adapter's full-tree build, the
    // bridge pricing the nodes the walk named.
    settle(&mut pipeline);

    // Beat two: the keyboard's focus ride — the focus delta moves
    // platform focus, the bridge looks the node up in the update it
    // moved in (the consumer's own stack would announce it here).
    pipeline.request_focus(submit);
    settle(&mut pipeline);

    // Beat three: the idle frame — a clean subtree costs nothing, the
    // bridge forwards no update (§11.1 item 4).
    settle(&mut pipeline);

    {
        let ledger = ledger.borrow();
        let total: usize = ledger.updates.iter().map(|(_, nodes)| nodes).sum();
        tracing::info!(
            target: "alacer::a11y_sink_custom",
            updates = ledger.updates.len(),
            nodes = total,
            "the bridge priced the flow: every update walked, {} nodes named in visit order",
            total
        );
        for (index, (_, nodes)) in ledger.updates.iter().enumerate() {
            tracing::info!(
                target: "alacer::a11y_sink_custom",
                update = index,
                nodes,
                "update {index}: {nodes} nodes"
            );
        }
        for role in &ledger.focus_seen {
            tracing::info!(
                target: "alacer::a11y_sink_custom",
                role = %role,
                "the focused node's role, named from the update it moved in"
            );
        }
    }

    // The window: live play — every frame the pipeline produces rides
    // the same bridge (the AT connects through the provided adapter
    // underneath; the ledger keeps pricing what flowed).
    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer bridged a11y sink",
        (VIEWPORT.width, VIEWPORT.height),
        BackendPreference::Wgpu.software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    {
        let ledger = ledger.borrow();
        tracing::info!(
            target: "alacer::a11y_sink_custom",
            updates = ledger.updates.len(),
            "the window closed: {} updates crossed the bridge in this session",
            ledger.updates.len()
        );
    }
}
