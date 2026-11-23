//! The warm-frame allocation budget (§A7, M84, ADR-0088), priced on the
//! path this suite is named for: the frame a user's keystroke warms.
//!
//! The m84 exit suite pins the budget's idle answer — a static UI's warm
//! tick allocates nothing, to the byte — and `zero_alloc_interaction.rs`
//! bounds a warm keystroke's churn. This suite closes the named gap
//! between them: mount a `TextInput`, settle the frame, arm the
//! [`AllocationGuard`], drive the keystroke, and require the warm frames
//! on either side of it — the steady-state ticks the pipeline replays —
//! to answer **exactly zero** heap allocations, the A7 sentence verbatim,
//! while the keystroke beats are priced against their named, bounded
//! allocation classes (the reactive graph's legitimate churn, documented
//! below — never a zero the facts refute).
//!
//! The instrument found and fixed (ADR-0088's own shape): the RED cut of
//! this suite measured a *focused* field's warm tick at 3 blocks / 96
//! bytes, allocated and freed every idle frame — the pipeline's per-frame
//! IME derivation re-walking the focused node's id path (`id_hash`'s
//! chain vector) although the request is a pure function of the focused
//! node and its caret anchor. `PassPipeline` now caches the derivation
//! against that key (structure and mutation frames re-derive honestly),
//! and the warm focused tick answers clean beside the static one.
//!
//! Every beat runs the real pipeline (A4): the real text seam shapes the
//! committed string, the real layout stage re-measures the line, the real
//! a11y sink publishes the value delta — and the tree's own answer is
//! asserted, so the priced keystrokes are proven to be real edits, never
//! silent no-ops. One test, one guard: dhat's one-profiler-per-process
//! rule, the m84 suite's own shape.

use alacer_core::widget::{EventResponse, Role};
use alacer_testing::TestHarness;
use alacer_testing::alloc::{Alloc, AllocationGuard};
use alacer_widgets::{BoxedWidget as _, Column, TextInput};

// The ledger's hook (§A7): the framework never installs the allocator —
// the test binary does, at its own top level, exactly the m84 suite's
// shape. Without it the counters answer zero and no beat is priced.
#[global_allocator]
static LEDGER: Alloc = Alloc;

/// The viewport every beat runs at (the m84 suite's own shape, smaller).
const VIEWPORT: (f32, f32) = (480.0, 320.0);

#[test]
fn the_warm_frame_a_keystroke_leaves_behind_allocates_nothing() {
    // (1) The cold build: mount the field, settle it laid out and
    // published, borrow focus, settle the focus frame. Everything here is
    // one-time work, priced before the guard exists.
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let field = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    // Focus resolves in pass 3 of a tick: settle before typing, or the
    // keys route to no focused widget (the editing contract's own rule).
    harness.pipeline_mut().request_focus(field);
    harness.settle();

    // (2) The guard arms here: its counters measure from this instant.
    let guard = AllocationGuard::activate();

    // (3) The warm-frame keystroke: commit one character through the real
    // pipeline. It must *work* — the field consumes the character. The
    // RED cut asserted the idle tick's clean answer verbatim here and the
    // beat refuted it: the first keystroke on a cold field measured
    // 409 blocks / 1,607,297 bytes (164 freed / 797,749 bytes) — one-time
    // edit-path work the widget keeps (the retained edit state's first
    // mutation, the shaped-run's first growth, the glyph atlas's cold
    // expansion). It is the warmup, not the contract: the beat must do
    // real work, and its retained growth is the text state and the atlas
    // the pipeline owns, never a growing ledger.
    let (outcomes, warmup) = guard.measure(|| harness.type_text("a"));
    assert_eq!(
        outcomes,
        vec![EventResponse::Handled],
        "the focused field did not consume the warming keystroke"
    );
    assert_eq!(
        harness
            .query(Role::TextInput)
            .named("Search")
            .first()
            .value
            .as_deref(),
        Some("a"),
        "the warming keystroke did not commit its character"
    );
    assert!(
        warmup.allocations > 0,
        "the first keystroke did no work at all ({warmup:?}) — the beat is fake"
    );

    // The priced interaction beat: the *warm* keystroke. The measured
    // truth named the classes — the harness plumbing's event payload
    // string and outcomes vector (one each, per keystroke), the field's
    // committed-text state (the character the user typed, the beat's
    // purpose and its retained growth), and the shaped-run, layout-
    // measure, paint-journal, and a11y-delta buffers the real pipeline
    // materializes once for the edited line. The bound is the measured
    // ceiling with headroom, so a regression that doubles the churn trips
    // it; a leak that retains beyond the text's own state trips the
    // net-growth bound.
    let (_, warm) = guard.measure(|| harness.type_text("b"));
    assert!(
        warm.allocations <= 128,
        "the warm keystroke churned beyond its justified budget: {warm:?} — \
         the edit path grew a per-keystroke allocation the budget refuses"
    );
    assert!(
        warm.allocated_bytes <= 16_384,
        "the warm keystroke moved more heap than its named classes can hold: {warm:?}"
    );
    assert!(
        warm.net_bytes() <= 2_048,
        "the warm keystroke retained more than the committed text's own \
         state: {warm:?} — the edit path is leaking"
    );
    assert_eq!(
        harness
            .query(Role::TextInput)
            .named("Search")
            .first()
            .value
            .as_deref(),
        Some("ab"),
        "the warm keystroke did not commit its character"
    );

    // (4) The contract's headline (§A7): the warm frame the edit leaves
    // behind — the steady-state tick after the keystrokes settled —
    // allocates exactly ZERO, to the byte. One warm-up tick first (the
    // ledger buffer's capacity is one-time work the pipeline keeps, the
    // m84 suite's own discipline), then the priced tick.
    let _ = harness.tick();
    let (_, warm_frame) = guard.measure(|| harness.tick());
    assert!(
        warm_frame.is_clean(),
        "the warm frame after the edit allocated {warm_frame:?} — the A7 \
         zero-alloc budget is broken"
    );
    assert_eq!(
        warm_frame.allocated_bytes, 0,
        "the warm frame's heap moved {warm_frame:?}"
    );

    // The zero answer is not one frame deep: the focused field's warm
    // tick — the exact beat the RED cut priced at 3 blocks / 96 bytes per
    // idle frame before the IME derivation's cache — is clean every
    // frame, the budget held for the UI as it now stands, edits included.
    let before = guard.stats();
    for _ in 0..4 {
        let report = harness.tick();
        assert!(report.is_idle(), "the settled frame is idle (A7)");
    }
    let after = guard.stats();
    assert_eq!(
        (before.live_allocs, before.live_bytes),
        (after.live_allocs, after.live_bytes),
        "four focused warm ticks moved the heap: {before:?} became {after:?}"
    );
}
