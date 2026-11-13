//! The record/replay format driven from the consumer side (§13.4,
//! ADR-0012): a bug report that is also a regression test. A tiny
//! settings form — one text input, one checkbox — runs a real session
//! through the harness: the steps are recorded *as they execute*
//! (`alacer_testing::recording::Session` — what you did is what gets
//! saved), saved as a versioned `.alc` file, loaded back, and replayed
//! on a fresh harness through the real input methods — the replayed
//! recording *is* the regression test, panicking the moment a semantic
//! step no longer describes reality.
//!
//! The format's semantic discipline is the point: a `press` names a
//! role and an accessible name, never a pixel, so the recording
//! survives layout drift a coordinate click would not.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example recording
//! ```
//!
//! The demo prints the session's steps, the saved `.alc` JSON, and the
//! replay's verdict — the counters proving the replayed steps landed in
//! the fresh harness exactly as they landed in the recorded one. The
//! file lands in the system temp directory (`alacer-recording-demo.alc`)
//! and is rewritten every run.

use alacer::widgets::{BoxedWidget as _, Checkbox, Column, TextInput};
use alacer_core::Role;
use alacer_core::events::Key;
use alacer_testing::TestHarness;
use alacer_testing::recording::{Recording, Session, Step};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The viewport the session records (the replay harness must match it —
/// the determinism guard refuses a mismatched viewport loudly).
const VIEWPORT: (f32, f32) = (360.0, 240.0);

/// The settings form, mounted fresh: the text input first (Tab's first
/// focusable, the canon todo's shape), the checkbox under it. The
/// handlers tally the writes so the demo can prove the replay landed.
fn mount_form(harness: &mut TestHarness) -> (Rc<Cell<usize>>, Rc<RefCell<Option<String>>>) {
    let toggles = Rc::new(Cell::new(0));
    let submitted = Rc::new(RefCell::new(None));

    let root = harness.mount(Column::new().gap(12.0).boxed());
    let submit_tally = Rc::clone(&submitted);
    let input = TextInput::new("Title")
        .on_submit(move |text| *submit_tally.borrow_mut() = Some(text.to_string()));
    let _ = harness.add_child(root, input.boxed());
    let toggle_tally = Rc::clone(&toggles);
    let box_ =
        Checkbox::new("Ship it").on_toggle(move |_| toggle_tally.set(toggle_tally.get() + 1));
    let _ = harness.add_child(root, box_.boxed());
    harness.settle();
    (toggles, submitted)
}

/// Records the session live: every step executes on the harness as it
/// is appended — what you did is what gets saved.
fn record_session(harness: &mut TestHarness) -> Recording {
    let mut session = Session::begin(harness);
    session.press_key(Key::Tab); // first focusable: the input
    session.type_text("hello");
    session.press_key(Key::Enter);
    session.press(Role::CheckBox, "Ship it");
    let recording = session.finish();
    for (index, step) in recording.steps().iter().enumerate() {
        let Step::Press { role, name } = step else {
            continue;
        };
        println!("step {index}: {role} named {name:?} (semantic — role + name, never a pixel)");
    }
    println!("steps recorded: {}", recording.steps().len());
    recording
}

fn main() {
    // The recorded session: the real form, the real input methods.
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let (recorded_toggles, recorded_submit) = mount_form(&mut harness);
    let recording = record_session(&mut harness);
    println!(
        "recorded session landed: toggles {}, submitted {:?}",
        recorded_toggles.get(),
        recorded_submit.borrow(),
    );

    // Save as `.alc` (versioned JSON) and print it — the bug report is
    // the file.
    let path = std::env::temp_dir().join("alacer-recording-demo.alc");
    recording
        .save(&path)
        .unwrap_or_else(|error| panic!("save {}: {error}", path.display()));
    let saved = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    println!("--- {} ---\n{saved}---", path.display());

    // Load it back anywhere and replay on a fresh harness: the same
    // steps through the same real input methods — the regression test
    // a bug report always wanted to be. A step that no longer resolves
    // (a renamed control, a removed role) panics here, loudly.
    let loaded = Recording::load(&path).expect("the recording the demo just saved loads");
    let mut fresh = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let (replay_toggles, replay_submit) = mount_form(&mut fresh);
    loaded.replay(&mut fresh);
    println!(
        "replayed session landed: toggles {}, submitted {:?}",
        replay_toggles.get(),
        replay_submit.borrow(),
    );

    // The verdict: what the recording did, the replay did.
    assert_eq!(
        recorded_toggles.get(),
        replay_toggles.get(),
        "the replayed checkbox toggled like the recorded one"
    );
    assert_eq!(
        *recorded_submit.borrow(),
        *replay_submit.borrow(),
        "the replayed input submitted like the recorded one"
    );
    println!(
        "the replay agrees with the recording — the bug report is the regression test (ADR-0012)"
    );
}
