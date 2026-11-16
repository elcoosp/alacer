//! The allocation budget (M84, ADR-0088) driven from the consumer side:
//! the ledger the A7 budget's checker rides — a static UI built through
//! the headless pipeline, a warm tick priced by the armed ledger and
//! answering **zero heap allocations**, the observer's bounded record
//! priced beside it, the raster's owned pixmap named apart, and eight
//! warm ticks leaving the heap exactly where they found it.
//!
//! Run with `cargo run -p alacer --example alloc_budget`. The example
//! is headless — the real pass pipeline with the software raster, no
//! window (A4) — and it doubles as a runnable budget check: any beat
//! that allocates beyond its priced record panics with the delta that
//! broke it. Install the hook at your own binary's top level (the
//! framework never does) and the same three lines price your app's
//! warm frame.

use alacer_testing::TestHarness;
use alacer_testing::alloc::{Alloc, AllocLedger};
use alacer_widgets::{BoxedWidget as _, Button, Column, Label, TextInput};

// The adopted hook (A2): this binary's allocator is the tracking one.
// The framework never installs it — a test, a bench, or this example
// does, at its own top level, and dhat's counters only move while a
// ledger is armed.
#[global_allocator]
static LEDGER: Alloc = Alloc;

fn main() {
    let ledger = AllocLedger::arm();

    // The static UI the budget prices — the m84 suite's own subject:
    // mounted, styled, laid out, painted, published once, never touched
    // again. The cold beat's cost is the flywheel's retained work (A7):
    // paid once, never per frame.
    let mut harness = TestHarness::new(800.0, 600.0);
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _title = harness.add_child(root, Label::new("The allocation budget").boxed());
    let _action = harness.add_child(root, Button::new("The budget's button").boxed());
    let _field = harness.add_child(root, TextInput::new("The budget's field").boxed());
    for row in 0..6 {
        let _ = harness.add_child(
            root,
            Label::new(format!("Budget row {row}, mounted once")).boxed(),
        );
    }
    harness.settle();
    let (cold, cold_delta) = ledger.measure(|| harness.tick());
    let _ = cold;
    println!("cold build   : {cold_delta} — the one-time work the pipeline keeps");

    // The budget's answer (§A7): the warm tick allocates nothing. Two
    // warm-up beats first — the pass ledger's buffer rides back into the
    // pipeline, its capacity one-time work like everything else.
    let _ = harness.tick();
    let _ = harness.tick();
    let (warm, warm_delta) = ledger.measure(|| harness.tick());
    assert!(
        warm_delta.is_clean() && warm.is_idle(),
        "the warm tick allocated {warm_delta:?} — the A7 budget is broken"
    );
    println!("warm tick    : {warm_delta} — the budget's answer, to the byte");

    // The mechanism, named: two warm ticks carry the *same* pass ledger
    // — one Arc clone apart, never a rebuilt Vec.
    let first = harness.tick();
    let second = harness.tick();
    assert!(
        std::sync::Arc::ptr_eq(&first.passes, &second.passes),
        "the warm tick rebuilt its pass ledger"
    );
    println!("pass ledger  : shared by handle — both warm ticks carry one allocation");

    // Eight warm ticks leave the heap where they found it: the budget
    // holds every beat, not one.
    let before = ledger.stats();
    for _ in 0..8 {
        let _ = harness.tick();
    }
    let after = ledger.stats();
    assert_eq!(
        (before.live_allocs, before.live_bytes),
        (after.live_allocs, after.live_bytes),
        "eight warm ticks moved the heap"
    );
    println!(
        "eight ticks  : heap steady at {} live bytes — no ledger grows",
        after.live_bytes
    );

    // The observer's priced record: with the trail installed and its
    // ring at its cap, the tick's own frame record is transient churn —
    // allocated, evicted, balanced to the byte (the eviction counted,
    // never a silent loss).
    let trail = harness.trail_with_cap(8);
    for _ in 0..8 {
        let _ = harness.tick();
    }
    let (_, observed_delta) = ledger.measure(|| harness.tick());
    assert!(
        observed_delta.is_balanced() && observed_delta.net_bytes() == 0,
        "the observed warm tick's record is priced differently now: {observed_delta:?}"
    );
    println!(
        "observed tick: {observed_delta} — the ring's own record, paid back by eviction ({} dropped)",
        trail.dropped()
    );

    // The raster's honest scope: the headless harness materializes an
    // owned pixmap per call — the buffer the windowed backends hold and
    // reuse is, here, the caller's. The replay is the budget's subject
    // and answered clean above; the beat drops its pixmap inside the
    // window, and what it leaves behind is the caches' one-time warmup,
    // never a per-frame cost.
    let (size, raster_delta) = ledger.measure(|| {
        let frame = harness.pixels();
        let size = (frame.width(), frame.height());
        drop(frame);
        size
    });
    println!(
        "raster       : {} bytes allocated for the {}x{} frame (net {} — the caches' one-time warmup)",
        raster_delta.allocated_bytes,
        size.0,
        size.1,
        raster_delta.net_bytes()
    );

    println!(
        "\npeak heap    : {} bytes across the whole run",
        ledger.stats().peak_bytes
    );
}
