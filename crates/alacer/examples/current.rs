//! The current tier as a consumer's vocabulary (M114, ADR-0118): the
//! set's current item said by the a11y tree at last — AccessKit's
//! `AriaCurrent` property (ARIA's `aria-current`) beside the name and
//! the position the M6 slot already carried, the word every platform AT
//! turns into "current page" while it walks the nav. Four buttons
//! beside one status line: the nav's page, the wizard's step, the
//! calendar's date, the plain button between them — the honest absence
//! the tier keeps honest (the ordinary button, never a guessed flavor).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the current word names a currency, it walls nothing
//! (the items stay put, the plain button beside them answers its press
//! as ever), and the app's own door (`Button::set_current` through the
//! mutate seam) moves the word live, the `PAINT | A11Y` request paying
//! the re-announcement — the write that lands where the word already is
//! queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example current
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{CurrentKind, Role};
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 360.0;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(alacer_core::geom::Size::new(
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
        )) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The currencies the tree says for the items, read back through the
/// snapshot in walk order — the same answer the sink maps onto
/// AccessKit's `AriaCurrent` property: the kind beside the word.
/// Buttons are nameless here in the read-back (the names ride the
/// labels), so the line walks the snapshot's items by position.
fn item_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let items: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| n.role == Role::Button)
                .map(|n| match n.current {
                    Some(kind) => format!("item ({})", kind_word(kind)),
                    None => "item (plain)".to_owned(),
                })
                .collect()
        })
        .unwrap_or_default();
    items.join(" · ")
}

/// The ARIA flavor the kind spells — the word the AT reads beside the
/// name (`aria-current="page"` and friends).
fn kind_word(kind: CurrentKind) -> &'static str {
    match kind {
        CurrentKind::True => "current",
        CurrentKind::Page => "current page",
        CurrentKind::Step => "current step",
        CurrentKind::Location => "current location",
        CurrentKind::Date => "current date",
        CurrentKind::Time => "current time",
    }
}

/// The app's current door: the mutate seam the app's code rides — the
/// word moved, `PAINT | A11Y` requested on the item, the
/// re-announcement paid for. `None` clears the word.
fn set_item_current(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    current: Option<CurrentKind>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_current(current);
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new("The current tier: the set's item, said by the tree (ADR-0118)").boxed(),
    );

    // The items: the currency rides the tree from the first frame —
    // the nav's page, the wizard's step, the calendar's date — beside
    // the plain button the honest absence between them.
    let nav = pipeline.add_child(root, Button::new("Home").current(CurrentKind::Page).boxed());
    let _wizard = pipeline.add_child(
        root,
        Button::new("Step 2 of 4")
            .current(CurrentKind::Step)
            .boxed(),
    );
    let _calendar = pipeline.add_child(
        root,
        Button::new("Oct 14").current(CurrentKind::Date).boxed(),
    );
    let plain = pipeline.add_child(root, Button::new("Send").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = item_words(snapshot.as_ref());
    let status_signal = pipeline.flux_mut().signal(line.clone());
    let status_binding = RenderBinding::new(line);
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // Beat one: the word is a word, not a wall — the plain button
    // beside the worded items answers its press exactly as it always
    // did.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::Button, "Send"))
            .map(|node| node.id)
            .expect("the plain button rides the snapshot"),
        alacer_core::widget::A11yAction::DefaultPress,
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::current",
        consumed,
        "beat 1: the AT's press lands unchanged beside the worded items — the word names a currency, it walls nothing"
    );

    // Beat two: the live door — the app's currency moved, the word the
    // tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_item_current(&mut pipeline, plain, Some(CurrentKind::Time));
    set_item_current(&mut pipeline, nav, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the items' words",
        item_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::current",
        items = "Send, Home",
        word = "current",
        "beat 2: the app's door moved the words the tree says — the currency follows the app's own"
    );
    set_item_current(&mut pipeline, plain, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        item_words(snapshot.as_ref()),
    ));

    // The window: live play — the worded items say the currency beside
    // the plain button, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: the items say the word — every door answers as it always did").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer current",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
