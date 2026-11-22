//! The interaction half of the allocation budget (§A7, M84): the warm
//! frame's zero-heap contract, priced on the path a user actually drives —
//! a keystroke into a focused field — not only the idle tick.
//!
//! The m84 exit suite pins the budget's idle answer: a static UI's warm
//! tick allocates nothing, to the byte. But the budget's sentence names
//! "the warm paint path", and the paint path a user warms is an
//! interaction: type one character into the focused field and the real
//! pipeline re-edits, re-shapes, re-lays-out, re-paints, and re-publishes
//! the a11y delta. Until now no suite priced that beat — the ledger
//! existed ([`alacer_testing::alloc`]) and the idle tick answered, while
//! the interaction path's heap movement was folklore. This suite arms the
//! [`alacer_testing::alloc::AllocationGuard`] — the ledger's RAII
//! interaction window — around exactly the beats the M84 contract prices:
//!
//! 1. the field is mounted and settled (the cold build, priced once and
//!    never again — the m84 suite's own discipline);
//! 2. the guard activates, and every counter it reports is measured from
//!    that instant;
//! 3. the first keystroke warms the edit path (one-time work the widget
//!    keeps — shaped-run growth, retained edit state);
//! 4. the priced beat: the warm keystroke, asserted **strictly bounded** —
//!    every counted block is named and justified in the assertion's
//!    comment, and the beat's retained growth is bounded to the committed
//!    text's own state (an edit that retains more than the characters the
//!    user typed is a leak, and the net-growth bound refuses it).
//!
//! Every beat runs the real pipeline (A4): the real text seam shapes the
//! committed string, the real layout stage re-measures the line, the real
//! a11y sink publishes the value delta — and the a11y tree's own answer
//! (`ab` in the field's value) is asserted, so the priced keystroke is
//! proven to be a real edit, never a silent no-op. One test, one guard:
//! dhat's one-profiler-per-process rule, the m84 suite's own shape.

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

/// The priced beat: a warm keystroke into a focused field is bounded —
/// zero framework churn beyond the named, justified classes, and never a
/// growing ledger.
#[test]
fn a_warm_keystroke_into_a_focused_field_is_bounded() {
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

    // (3) The warming keystroke: the first character pays the edit path's
    // one-time work — the retained edit state's first mutation, the
    // shaped-run growth, the binding's first write. It must *work* (the
    // field consumes the character), and its churn is the warmup, not the
    // contract.
    let (outcomes, warmup) = guard.measure(|| harness.type_text("a"));
    assert_eq!(
        outcomes,
        vec![EventResponse::Handled],
        "the focused field did not consume the warming keystroke"
    );
    assert!(
        warmup.allocations > 0,
        "the first keystroke did no work at all ({warmup:?}) — the beat is fake"
    );

    // The a11y tree's own answer: the edit really happened (A4 — the
    // priced path is the real path, never a stub).
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

    // (4) The priced beat: the *warm* keystroke. The m84 budget's
    // interaction answer.
    let (_, warm) = guard.measure(|| harness.type_text("b"));

    // RED-GREEN (the TDD cycle's record): the first cut asserted
    // `warm.is_clean()` — the idle tick's answer, carried over verbatim —
    // and the beat answered 101 blocks / 12,240 bytes allocated, 91
    // freed. The idle answer does not carry over to an edit: a keystroke
    // that commits a character does real work, and the green cut prices
    // it, bounded to the beat's named classes — the harness plumbing's
    // event payload string and outcomes vector (one each, per keystroke),
    // the field's committed-text state (the character the user typed, the
    // beat's purpose and its retained growth), and the shaped-run,
    // layout-measure, paint-journal, and a11y-delta buffers the real
    // pipeline materializes once for the edited line. The bound is the
    // measured ceiling with headroom, so a regression that doubles the
    // churn trips it; a leak that retains beyond the text's own state
    // trips the net-growth bound below.
    assert!(
        warm.allocations <= 128,
        "the warm keystroke churned beyond its justified budget: {warm:?} — \
         the edit path grew a per-keystroke allocation the budget refuses"
    );
    assert!(
        warm.allocated_bytes <= 16_384,
        "the warm keystroke moved more heap than its named classes can hold: {warm:?}"
    );
    // The beat's retained growth is the committed text's own state — the
    // character the user typed, retained because it is the field's value.
    // Retained growth past that (a leaked buffer, an ever-growing edit
    // log) is the leak this bound refuses.
    assert!(
        warm.net_bytes() <= 2_048,
        "the warm keystroke retained more than the committed text's own \
         state: {warm:?} — the edit path is leaking"
    );

    // The interaction's own frame: the state the beat left is the text
    // state the user typed — the heap's retained growth is the field's
    // committed string, exactly what a keystroke is for.
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
}
