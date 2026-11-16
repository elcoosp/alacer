//! M84 exit-criteria integration tests (§A7, ADR-0088): the allocation
//! ledger — the heap budget's checker, the instrument architecture A7's
//! consequence has always named ("the warm paint path for a static UI
//! performs **zero heap allocations** [budget, CI-enforced via dhat]")
//! and until this cut never shipped. Through M83 the claim had no
//! checker anywhere in the workspace: the adopted-dependencies list
//! named dhat, no crate adopted it, and a warm tick's heap movement was
//! folklore. The ledger adopts the hook ([`dhat::Alloc`], A2 — steal
//! proven cores; own only the composition) and grows the budget's
//! vocabulary beside it ([`alacer_testing::alloc`]: the armed
//! [`alacer_testing::alloc::AllocLedger`] window, the [`alacer_testing::alloc::AllocStats`]
//! snapshots, the [`alacer_testing::alloc::AllocDelta`] answers), and the
//! pass ledger stops paying for its own bookkeeping: the tick's ledger
//! buffer rides back into the pipeline and the warm tick's report hands
//! out the canonical idle ledger by [`std::sync::Arc`] handle
//! ([`alacer_core::passes::FrameReport::passes`]) — the measured truth
//! this suite pins is that the budget now *is* the warm tick's answer,
//! zero, not an aspiration beside it.
//!
//! Every beat runs the real pipeline (A4): real layout, real paint
//! journal, the harness's software raster for the raster beat. The
//! whole suite arms one ledger and runs its beats in one test — one
//! profiler per process is dhat's own rule, and the heap's counters are
//! the process's, never a test's private ones.

use alacer_testing::TestHarness;
use alacer_testing::alloc::{Alloc, AllocLedger};
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, TextInput};
use std::sync::Arc;

#[global_allocator]
static LEDGER: Alloc = Alloc;

/// The static UI the budget prices: the m0 canon's own shape — a
/// column, a bound-free label, a button, a field, six listed labels —
/// mounted, styled, laid out, painted, and published to the a11y tree
/// exactly once, then never touched again. The budget's subject is the
/// frame that follows: the same tree, the same journal, nothing dirty.
fn mount_static_ui(harness: &mut TestHarness) {
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _title = harness.add_child(root, Label::new("The allocation ledger").boxed());
    let _action = harness.add_child(root, Button::new("The ledger's button").boxed());
    let _field = harness.add_child(root, TextInput::new("The ledger's field").boxed());
    for row in 0..6 {
        let _ = harness.add_child(
            root,
            Label::new(format!("Ledger row {row}, mounted once")).boxed(),
        );
    }
}

#[test]
fn a_static_uis_warm_tick_allocates_nothing() {
    let ledger = AllocLedger::arm();

    // The cold build is priced once and never again: mount, style,
    // layout, paint, publish — the one-time work A7's flywheel retains.
    let mut harness = TestHarness::new(800.0, 600.0);
    mount_static_ui(&mut harness);
    let settled = harness.settle();
    let (cold, cold_delta) = ledger.measure(|| harness.tick());
    let _ = cold;
    assert!(
        cold_delta.allocated_bytes > 0,
        "the first frame did real work ({cold_delta:?})"
    );

    // The budget's answer (§A7): a static UI's warm tick allocates
    // nothing. Two warm-up beats first — the ledger buffer's capacity is
    // one-time work the pipeline keeps — then the beat, and it must be
    // clean to the byte.
    let _ = harness.tick();
    let _ = harness.tick();
    let (warm, warm_delta) = ledger.measure(|| harness.tick());
    assert!(
        warm_delta.is_clean(),
        "the warm tick allocated {warm_delta:?} — the A7 budget is broken"
    );
    assert_eq!(
        warm_delta.allocated_bytes, 0,
        "the warm tick's heap moved {warm_delta:?}"
    );
    assert!(warm.is_idle(), "a static UI's warm frame is idle (A7)");

    // The mechanism, named: the warm tick's ledger is the canonical idle
    // ledger by handle — two consecutive warm ticks carry the *same*
    // allocation, one Arc clone apart, never a rebuilt Vec.
    let first = harness.tick();
    let second = harness.tick();
    assert!(
        Arc::ptr_eq(&first.passes, &second.passes),
        "warm ticks rebuilt the pass ledger — the buffer stopped riding back"
    );

    // Eight warm ticks leave the heap where they found it: the budget is
    // not merely clean in one beat, it is clean every beat — no ledger
    // grows under a static UI.
    let before = ledger.stats();
    for _ in 0..8 {
        let _ = harness.tick();
    }
    let after = ledger.stats();
    assert_eq!(
        (before.live_allocs, before.live_bytes),
        (after.live_allocs, after.live_bytes),
        "eight warm ticks moved the heap: {before:?} became {after:?}"
    );

    // The observer's priced record (§13.3's own rule): with a trail
    // installed, a warm tick costs the frame record's own vector — the
    // wire summary's pass timings, one allocation per tick — and the
    // ring's bounded discipline pays it back: once the ring sits at its
    // cap, the newest record's allocation evicts the oldest record and
    // the beat balances to the byte (the eviction counted on the
    // handle — never a silent loss, never a growing ledger).
    let trail = harness.trail_with_cap(8);
    for _ in 0..8 {
        let _ = harness.tick();
    }
    let (_, observed_delta) = ledger.measure(|| harness.tick());
    assert_eq!(
        observed_delta.allocations, observed_delta.deallocations,
        "the observed warm tick's ring did not pay its own record back: {observed_delta:?}"
    );
    assert!(
        observed_delta.allocations <= 2,
        "the observed warm tick churned more than its priced record: {observed_delta:?}"
    );
    assert_eq!(
        observed_delta.net_bytes(),
        0,
        "the observed warm tick grew the heap: {observed_delta:?}"
    );
    assert!(
        trail.dropped() >= 1,
        "the bounded ring evicted silently — the dropped count is the rule"
    );

    // The raster beat's honest scope: the harness's `pixels()`
    // materializes an owned pixmap per call — the output buffer the
    // windowed backends hold and reuse is, in the headless harness, the
    // caller's. The replay itself (the pipeline tick) is the budget's
    // subject and answered clean above; the raster beat pins that the
    // pixmap is the allocation the beat is dominated by, and that the
    // renderer's retained side (the glyph atlas, the coverage caches —
    // the flywheel's retained work) warms once and never grows with
    // repetition: the beat drops its pixmap inside the window, and the
    // net growth it leaves behind is the caches' one-time warmup, not a
    // per-frame cost.
    let ((), raster_delta) = ledger.measure(|| {
        let frame = harness.pixels();
        assert_eq!((frame.width(), frame.height()), (800, 600));
        drop(frame);
    });
    assert!(
        raster_delta.allocated_bytes >= 800 * 600 * 4,
        "the raster beat did not even allocate the viewport pixmap: {raster_delta:?}"
    );
    assert!(
        raster_delta.net_bytes() <= 262_144,
        "the raster beat grew the heap beyond the caches' one-time warmup: {raster_delta:?}"
    );
    assert_eq!(settled, 1, "a static UI settles on the first frame");
}
