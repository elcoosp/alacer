//! The routing-outcome doors as a consumer's vocabulary (M118,
//! ADR-0122): the harness's keyboard and IME doors answer what the
//! pipeline answered, as data — `Handled` the field (or the keymap, or
//! a framework fallback) took the key, `Ignored` it bubbled on. The
//! beat the example exists to show: the one routing question a test
//! could only smuggle out through effects before — the value that
//! moved, the trail that streamed — now has a direct, deterministic
//! answer, one verdict per dispatched key, the dispatch byte-for-byte
//! the dispatch the door always ran.
//!
//! Six beats, driven headless through the real pipeline: the stray key
//! the empty funnel falls through (`Ignored`), the Tab the cycling
//! takes (`Handled`), the characters a focused field lands
//! (`Handled`, each), the characters a plain button takes no interest
//! in (`Ignored`, each), the read-only wall that consumes its
//! refusals while landing nothing (M105, ADR-0109) — and the IME
//! route's verdict beside them.
//!
//! ```sh
//! cargo run -p alacer --example key_routing
//! ```

use alacer::widgets::{BoxedWidget as _, Button, Column, TextInput};
use alacer_core::widget::{EventResponse, Role};
use alacer_testing::TestHarness;

/// The viewport the beats settle into (the editing canon's shape).
const VIEWPORT: (f32, f32) = (400.0, 240.0);

/// The first node in the snapshot whose role and name match.
fn node(harness: &TestHarness, role: Role, name: &str) -> alacer_core::passes::TreeNode {
    harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(role, name).cloned())
        .unwrap_or_else(|| panic!("no {role} named {name:?} is reachable"))
}

fn main() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let root = harness.mount(Column::new().gap(8.0).boxed());
    let field = harness.add_child(root, TextInput::new("field").boxed());
    let save = harness.add_child(root, Button::new("Save").boxed());
    let wall = harness.add_child(root, TextInput::new("wall").read_only().boxed());
    harness.settle();

    // Beat one: the stray key with nothing focused — the funnel falls
    // through honestly, and the door says so.
    let stray = harness.press_key(alacer_core::events::Key::Char('x'));
    assert_eq!(stray, EventResponse::Ignored, "no widget holds the key");
    println!("beat 1: stray key with nothing focused -> Ignored");

    // Beat two: the framework's own fallback answers as the
    // framework's — the button ignored Tab, the cycling took it.
    let tab = harness.press_key(alacer_core::events::Key::Tab);
    assert_eq!(tab, EventResponse::Handled, "the cycling took the Tab");
    println!("beat 2: Tab with nothing focused -> Handled (the cycling took it)");

    // Beat three: the focused field lands the characters — one verdict
    // per dispatched key, the value the verdicts carried.
    harness.pipeline_mut().request_focus(field);
    harness.settle();
    let outcomes = harness.type_text("hey");
    assert_eq!(
        outcomes,
        vec![EventResponse::Handled; 3],
        "the field took every character"
    );
    let landed = node(&harness, Role::TextInput, "field");
    assert_eq!(
        landed.value.as_deref(),
        Some("hey"),
        "the value rode the verdicts"
    );
    println!(
        "beat 3: type_text(\"hey\") into the field -> [Handled; 3], value {:?}",
        landed.value.as_deref(),
    );

    // Beat four: the plain button takes no characters — every verdict
    // `Ignored`, the contrast the doors exist to pin.
    harness.pipeline_mut().request_focus(save);
    harness.settle();
    let outcomes = harness.type_text("ab");
    assert_eq!(
        outcomes,
        vec![EventResponse::Ignored, EventResponse::Ignored],
        "the button takes no characters"
    );
    println!("beat 4: type_text(\"ab\") on the focused button -> [Ignored; 2]");

    // Beat five: the read-only wall consumes its refusals and lands
    // nothing — the verdicts `Handled` (the field heard its own keys),
    // the value untouched (M105, ADR-0109).
    harness.pipeline_mut().request_focus(wall);
    harness.settle();
    let outcomes = harness.type_text("no");
    assert_eq!(
        outcomes,
        vec![EventResponse::Handled; 2],
        "the wall answered its refusals — consumed, never munged"
    );
    let walled = node(&harness, Role::TextInput, "wall");
    assert!(
        walled.value.as_deref().unwrap_or("").is_empty(),
        "the wall landed nothing"
    );
    println!("beat 5: type_text(\"no\") on the read-only wall -> [Handled; 2], value empty");

    // Beat six: the IME route answers as the key routes do — the
    // committed composition the field inserted rides one `Handled`.
    harness.pipeline_mut().request_focus(field);
    harness.settle();
    let commit = harness.ime(&alacer_core::events::ImeEvent::Commit {
        text: "你好".to_owned(),
    });
    assert_eq!(commit, EventResponse::Handled, "the commit landed");
    let committed = node(&harness, Role::TextInput, "field");
    assert_eq!(
        committed.value.as_deref(),
        Some("hey你好"),
        "the commit joined the typed run"
    );
    println!(
        "beat 6: IME commit into the field -> Handled, value {:?}",
        committed.value.as_deref(),
    );

    println!("every door answered what the pipeline answered — the routing is data (ADR-0122)");
}
