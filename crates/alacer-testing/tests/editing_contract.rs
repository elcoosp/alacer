//! The full editing contract, integration-proved as one session (§10.3,
//! ADR-0017/0019/0020): the M11 selection vocabulary, the M13 word tier,
//! and the M14 clipboard tier are three faces of one editing story, and
//! the per-milestone exit criteria prove the faces apart. This suite
//! proves the faces *together* — the compositions no single milestone's
//! suite asserts: the whole user story in one session, the UAX #29 runs
//! of `don't` through the pipeline, a refused clipboard write never
//! deleting, the whole-chord inertness sweep under one preedit, and the
//! paste rule riding the word tier's hops over what it sanitized.
//!
//! Every test runs the real pipeline (A4) with the real text seam — the
//! exact route the window's keys take — and the harness's in-memory
//! clipboard: a headless CI never races the operator's actual OS
//! clipboard (ADR-0020).

use alacer_core::clipboard::{ClipboardSeam, install};
use alacer_core::events::{ImeEvent, Key, KeyEvent, KeyState, Modifiers};
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, TextInput};
use std::rc::Rc;

/// An input mounted through the real pipeline and filled by typing
/// `content` — the fill borrows keyboard focus, so every chord under
/// test routes to the field the way the window delivers them (the M14
/// suite's same mount, focused before the first key).
fn mount_input(harness: &mut TestHarness, content: &str) {
    let root = harness.mount(Column::new().boxed());
    let node = harness.add_child(root, TextInput::new("Search").boxed());
    harness.settle();
    // Focus resolves in pass 3 of a tick: settle before typing, or the
    // keys route to no focused widget (the M11 suite's same rule).
    harness.pipeline_mut().request_focus(node);
    harness.settle();
    if !content.is_empty() {
        harness.type_text(content);
    }
}

/// The input's committed value, read through the a11y tree (§11.1 item 5).
fn input_value(harness: &TestHarness) -> String {
    harness
        .query(Role::TextInput)
        .named("Search")
        .first()
        .value
        .clone()
        .expect("an editable input carries its value")
}

/// The input's reported selection: `(anchor, caret)` in UTF-16 units,
/// the M11 contract pair the a11y delta carries.
fn selection(harness: &TestHarness) -> (usize, usize) {
    harness
        .selection(Role::TextInput, "Search")
        .expect("a focused, non-composing input reports its selection")
}

/// A pressed key with these modifiers (the suite's one chord builder —
/// every chord is the same shape, only the modifier flags move).
fn chord(key: Key, modifiers: Modifiers) -> KeyEvent {
    KeyEvent {
        key,
        state: KeyState::Pressed,
        modifiers,
        text: None,
    }
}

fn ctrl(key: Key) -> KeyEvent {
    chord(
        key,
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    )
}

fn shift(key: Key) -> KeyEvent {
    chord(
        key,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    )
}

fn alt(key: Key) -> KeyEvent {
    chord(
        key,
        Modifiers {
            alt: true,
            ..Modifiers::default()
        },
    )
}

fn shift_alt(key: Key) -> KeyEvent {
    chord(
        key,
        Modifiers {
            shift: true,
            alt: true,
            ..Modifiers::default()
        },
    )
}

/// The whole editing story in one session: selection → clipboard → word
/// hops → extension → cut → sanitized paste → select-all copy. The
/// per-milestone suites prove each tier apart; this proves the field
/// carries a user through all three tiers in one sitting without the
/// vocabulary ever slipping (the same anchor+caret pair, the same
/// refuse-don't-munge rules, the same journal funnel under every edit).
#[test]
fn the_full_editing_story_runs_as_one_session() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "don't stop me");

    // Select all and copy: the clipboard carries the whole value, the
    // value keeps itself (a copy edits nothing).
    harness.key(&ctrl(Key::Char('a')));
    assert_eq!(
        selection(&harness),
        (0, 13),
        "Ctrl+A selects the whole value"
    );
    harness.key(&ctrl(Key::Char('c')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("don't stop me"),
        "the copy writes the selection's exact span"
    );

    // A fresh caret gesture first: a plain End collapses the live
    // selection at the value's end — a plain chord over a live
    // selection never hops (the M13 anchor rule the arrows follow).
    harness.key(&chord(Key::End, Modifiers::default()));
    assert_eq!(
        selection(&harness),
        (13, 13),
        "a plain End collapses the selection"
    );

    // Word hops ride the UAX #29 runs: end → me's start → stop's start →
    // don't's start (the apostrophe never splits the run).
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (11, 11), "me's start");
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (6, 6), "stop's start");
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(
        selection(&harness),
        (0, 0),
        "don't's start, apostrophe inside"
    );

    // Shift+Alt+Right extends from the anchor (the M11 semantics the word
    // tier rides): the caret walks the run's end, the anchor stays.
    harness.key(&shift_alt(Key::ArrowRight));
    assert_eq!(selection(&harness), (0, 5), "the extension covers don't");

    // The cut moves the *selection* to the seam: the clipboard now
    // carries "don't", the value carries the rest, the caret collapses
    // at the cut's start.
    harness.key(&ctrl(Key::Char('x')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("don't"),
        "the cut writes the moved span"
    );
    assert_eq!(input_value(&harness), " stop me", "the selection went");
    assert_eq!(
        selection(&harness),
        (0, 0),
        "the caret collapses at the cut"
    );

    // Paste: the sanitizer folds the payload's line break into one space,
    // the insert lands at the caret, and the journal holds the step.
    harness.set_clipboard("never\ndone");
    harness.key(&ctrl(Key::Char('v')));
    assert_eq!(
        input_value(&harness),
        "never done stop me",
        "the pasted line break is one space"
    );
    assert_eq!(
        selection(&harness),
        (10, 10),
        "the caret parks after the insert"
    );

    // And the session closes the loop: select-all once more, copy, and
    // the clipboard carries the composed value whole.
    harness.key(&ctrl(Key::Char('a')));
    assert_eq!(
        selection(&harness),
        (0, 18),
        "the composed value selects whole"
    );
    harness.key(&ctrl(Key::Char('c')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("never done stop me"),
        "the composed value copies whole"
    );
}

/// The word tier walks the UAX #29 runs of `don't stop` — the apostrophe
/// is a `MidLetter` inside the word run, so a backward hop crosses it whole
/// and a word chord deletes exactly the span the caret would hop. The
/// pipeline drives the real segmentation tables (the M13 seam rule: no
/// honest default without them).
#[test]
fn word_hops_walk_the_uax29_runs_of_dont_stop() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "don't stop");

    // Backward hops from the value's end: stop's start, then don't's
    // start — one run, apostrophe and all — then the clamp.
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (6, 6));
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (0, 0), "don't hops as one UAX #29 run");
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (0, 0), "the run's start clamps");

    // Ctrl+Backspace at stop's start deletes exactly the hop's span: the
    // word with the separator following it, never half a run.
    harness.key(&alt(Key::ArrowRight));
    harness.key(&alt(Key::ArrowRight));
    assert_eq!(selection(&harness), (6, 6));
    harness.key(&ctrl(Key::Backspace));
    assert_eq!(input_value(&harness), "stop", "the hop's span went whole");
    assert_eq!(
        selection(&harness),
        (0, 0),
        "the caret parks where the span was"
    );

    // The forward chord mirrors it over an extended selection: End, then
    // Shift+Home extends from the anchor, and a plain word chord
    // collapses a live selection to its near edge first (the M13 rule) —
    // then Ctrl+Delete consumes the selection the collapse left.
    harness.key(&chord(Key::End, Modifiers::default()));
    harness.key(&shift(Key::Home));
    assert_eq!(
        selection(&harness),
        (0, 4),
        "Shift+Home extends from the anchor"
    );
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (0, 0), "a plain chord collapses first");
    harness.key(&ctrl(Key::Delete));
    assert_eq!(input_value(&harness), "", "the selection went whole");
}

/// A clipboard engine that refuses every write: the honest no-op tier.
/// A cut whose write was refused never deletes — the selection survives
/// a clipboard that failed it (ADR-0020's refuse-don't-munge rule at the
/// destructive edit), and both refusals cost an idle frame.
#[test]
fn a_refused_write_never_deletes_the_selection() {
    /// A seam whose write door always refuses — the sandboxed-session
    /// story, answered honestly through the `bool` door alone.
    struct WriteRefuser;

    impl ClipboardSeam for WriteRefuser {
        fn read_text(&self) -> Option<String> {
            None
        }

        fn write_text(&self, _text: &str) -> bool {
            false
        }
    }

    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "held value");

    // The refuser takes over the thread's clipboard seam: every chord's
    // round trip from here routes to it (the assembly-site discipline —
    // install replaces, and this thread's story ends with this test).
    install(Rc::new(WriteRefuser) as Rc<dyn ClipboardSeam>);

    harness.key(&ctrl(Key::Char('a')));
    assert_eq!(selection(&harness), (0, 10), "the selection built fine");
    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('x')));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the field consumes its own chord: the refusal is the answer"
    );
    assert_eq!(
        harness.settle(),
        0,
        "a refused cut queues no masks: the next frame is idle"
    );
    assert_eq!(
        input_value(&harness),
        "held value",
        "a refused cut never deletes the selection"
    );
    assert_eq!(
        selection(&harness),
        (0, 10),
        "the selection survives the clipboard that failed it"
    );

    // The same honesty at the copy chord: consumed, idle, nothing moved.
    let response = harness.pipeline_mut().dispatch_key(&ctrl(Key::Char('c')));
    assert!(
        matches!(response, alacer_core::EventResponse::Handled),
        "the copy is consumed with its refusal"
    );
    assert_eq!(harness.settle(), 0, "a refused copy is an idle frame too");
    assert_eq!(input_value(&harness), "held value", "a copy edits nothing");
}

/// Every editing chord the three milestones own — the M11 selection
/// chords, the M13 word chords, the M14 clipboard chords, the M15
/// journal chords — is inert while a preedit composes: ignored, no
/// mutation, no masks, one idle frame for the whole sweep. The IME owns
/// the canvas until commit (the rule each per-milestone suite pins for
/// its own chords; this sweep pins it for all of them at once).
#[test]
fn every_editing_chord_is_inert_while_a_preedit_composes() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "don't stop");
    harness.set_clipboard("the untouched payload");

    // The composition opens: the field's key door closes.
    harness.ime(&ImeEvent::Preedit {
        text: "ん".to_string(),
        cursor_utf16: Some(1),
    });

    let sweep = [
        shift(Key::ArrowLeft),
        shift(Key::ArrowRight),
        shift(Key::Home),
        shift(Key::End),
        ctrl(Key::Char('a')),
        alt(Key::ArrowLeft),
        alt(Key::ArrowRight),
        shift_alt(Key::ArrowRight),
        ctrl(Key::Backspace),
        ctrl(Key::Delete),
        ctrl(Key::Char('c')),
        ctrl(Key::Char('x')),
        ctrl(Key::Char('v')),
        ctrl(Key::Char('z')),
        ctrl(Key::Char('y')),
    ];
    for event in &sweep {
        let response = harness.pipeline_mut().dispatch_key(event);
        assert!(
            matches!(response, alacer_core::EventResponse::Ignored),
            "a composing field ignores {event:?}"
        );
    }
    assert_eq!(
        harness.settle(),
        0,
        "the whole inert sweep queues no masks: one idle frame"
    );
    assert_eq!(
        input_value(&harness),
        "don't stop",
        "the sweep never touched the value"
    );
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("the untouched payload"),
        "the sweep never touched the seam"
    );
    assert_eq!(
        harness.selection(Role::TextInput, "Search"),
        None,
        "the report drops while composing — a stale range would lie to the AT"
    );

    // The composition ends and the field's door reopens on the same
    // vocabulary: the commit inserts, and the chords answer again.
    harness.ime(&ImeEvent::Commit {
        text: " ok".to_string(),
    });
    assert_eq!(input_value(&harness), "don't stop ok", "the commit landed");
    harness.key(&ctrl(Key::Char('a')));
    assert_eq!(
        selection(&harness),
        (0, 13),
        "the chords answer the moment the composition ends"
    );
    harness.key(&ctrl(Key::Char('c')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("don't stop ok"),
        "the reopened door round-trips the seam"
    );
}

/// Ctrl+A selects all and the next typed character replaces the whole
/// value — the M11 edit rule at the select-all tier, proved through the
/// pipeline: one selection-consuming insert, one journal step, the caret
/// parked after the replacement.
#[test]
fn select_all_then_typing_replaces_the_whole_value() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "held");
    harness.key(&ctrl(Key::Char('a')));
    assert_eq!(selection(&harness), (0, 4));

    harness.key(&chord(Key::Char('n'), Modifiers::default()));
    harness.key(&chord(Key::Char('e'), Modifiers::default()));
    harness.key(&chord(Key::Char('w'), Modifiers::default()));
    assert_eq!(
        input_value(&harness),
        "new",
        "the first typed character consumed the whole selection"
    );
    assert_eq!(selection(&harness), (3, 3), "the caret parks after the run");
    assert_eq!(
        harness.clipboard_text(),
        None,
        "typing rides the keyboard, never the seam"
    );
}

/// The paste rule and the word tier compose: the sanitizer folds the
/// payload's breaks into spaces, and the word hops that follow walk the
/// runs the sanitized text actually carries — the field never hops what
/// the clipboard claimed, only what it pasted.
#[test]
fn the_paste_rule_and_the_word_tier_compose() {
    let mut harness = TestHarness::new(400.0, 200.0);
    mount_input(&mut harness, "");
    harness.set_clipboard("to\ndo\r\nlist");
    harness.key(&ctrl(Key::Char('v')));
    assert_eq!(
        input_value(&harness),
        "to do list",
        "CRLF and LF both pasted as one space each"
    );
    assert_eq!(selection(&harness), (10, 10));

    // The hops ride the pasted runs: list's start, do's start, to's start.
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (6, 6), "list's start");
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (3, 3), "do's start");
    harness.key(&alt(Key::ArrowLeft));
    assert_eq!(selection(&harness), (0, 0), "to's start");

    // And the sanitized value copies back out without a single break —
    // what the field holds is what the seam carries.
    harness.key(&ctrl(Key::Char('a')));
    harness.key(&ctrl(Key::Char('c')));
    assert_eq!(
        harness.clipboard_text().as_deref(),
        Some("to do list"),
        "the sanitized value round-trips whole"
    );
}
