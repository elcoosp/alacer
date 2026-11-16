//! The input-frame allocation budget (M87, ADR-0091) driven from the
//! consumer side: the budget at the tier the warm tick does not cover —
//! ADR-0088's own pre-registered re-open rule cashed. The instrument
//! measured first: an input frame is *not* the warm path — a
//! pointer-move frame churns a small, balanced burst (the dispatch
//! buffers' work, allocated and freed inside the same tick), the
//! dispatch queue's one-time block lands on the first tracked move
//! frame and is held ever after, and the warm tick beside it stays
//! clean. The ledger names each beat; any beat that moves past its
//! priced shape panics with the delta that broke it.
//!
//! Run with `cargo run -p alacer --example input_budget`. The example
//! is headless — the real pass pipeline with the software raster, no
//! window (A4) — and it doubles as a runnable budget check: install
//! the hook at your own binary's top level (the framework never does)
//! and the same beats price your app's input frames.

use alacer_core::geom::Point;
use alacer_testing::TestHarness;
use alacer_testing::alloc::{Alloc, AllocLedger};
use alacer_widgets::{BoxedWidget as _, Button, Column, Label};

// The adopted hook (A2): this binary's allocator is the tracking one.
// The framework never installs it — a test, a bench, or this example
// does, at its own top level, and dhat's counters only move while a
// ledger is armed.
#[global_allocator]
static LEDGER: Alloc = Alloc;

fn main() {
    let ledger = AllocLedger::arm();

    // The static UI the budget prices — mounted once, settled: the
    // warm frame's subject with an input surface beside it (the
    // button the pointer moves over).
    let mut harness = TestHarness::new(400.0, 300.0);
    let root = harness.mount(Column::new().gap(8.0).padding(12.0).boxed());
    let _title = harness.add_child(root, Label::new("The input budget").boxed());
    let _action = harness.add_child(root, Button::new("The budget's button").boxed());
    harness.settle();

    // The first tracked move frame: the dispatch queue's one-time
    // block lands here (the instrument measured +192 bytes) — named,
    // never wished away. Every frame after it holds the capacity.
    let ((), first) = ledger.measure(|| {
        harness.pointer_move(Point::new(64.0, 64.0));
        let _ = harness.tick();
    });
    println!("first move frame : {first}");
    assert!(
        first.net_bytes() > 0,
        "the first tracked move frame paid no one-time block — the measured truth moved: {first:?}"
    );

    // The steady-state input frame: the balanced burst the budget
    // names — blocks allocated and freed inside the same tick, the
    // net at zero to the byte. A frame that leaks is a broken budget,
    // never a fast-but-leaky one.
    let ((), steady) = ledger.measure(|| {
        harness.pointer_move(Point::new(68.0, 64.0));
        let _ = harness.tick();
    });
    println!("steady move frame: {steady}");
    assert!(
        steady.is_balanced() && steady.net_bytes() == 0,
        "the input frame retained memory: {steady:?}"
    );

    // Twenty more moves: the steady state holds across the run —
    // the churn priced, the net still zero.
    let ((), twenty) = ledger.measure(|| {
        for i in 0_u8..20 {
            harness.pointer_move(Point::new(72.0 + f32::from(i), 64.0));
            let _ = harness.tick();
        }
    });
    println!("twenty move frames: {twenty}");
    assert_eq!(
        twenty.net_bytes(),
        0,
        "the input frame's net moved off zero over the run: {twenty:?}"
    );

    // The warm idle tick beside it: the m84 budget, re-agreed in the
    // same window the input frame rides.
    let (_, idle) = ledger.measure(|| harness.tick());
    println!("warm idle frame  : {idle}");
    assert!(
        idle.is_clean(),
        "the warm tick allocated {idle:?} — the A7 budget is broken"
    );
}
