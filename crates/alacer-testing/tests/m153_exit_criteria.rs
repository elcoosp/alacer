//! M153 exit-criteria integration tests (§13.3, ADR-0157): the cursor
//! trail — the hover's resolved changes as facts on the inspect trail.
//!
//! Through M152 the cursor seam named the hover's shape and the platform
//! applied it, and the only observer of the churn was the platform's
//! own debug line: "why did the cursor move to a pointer?" was a
//! re-implement-it-yourself question — resolve the door twice and diff
//! by hand, the work every other tier's trail removed (the clipboard
//! trail's shape, ADR-0045). ADR-0157 closes the gap where the answer
//! resolves: the [`alacer_core::passes::PassPipeline::cursor_at`] door
//! notes one [`alacer_core::inspect::CursorRecord`] when the resolved
//! style moves — the style beside the source it rode (the widget whose
//! hook named it, the node and role beside it — the lengths rule: the
//! label never rides) — and the tick's start drain streams it as
//! [`alacer_core::inspect::InspectEvent::Cursor`], the clipboard
//! drain's own shape, one frame after the hover moved.
//!
//! The flood rule holds at the cursor tier: a style that did not move
//! is not a fact — the door coalesces against the last noted answer,
//! one record per change, never per frame. The idle contract holds:
//! while nobody listens the note is one branch (the disabled trail's
//! whole cost), and the drain empties the queue whether an observer
//! listens or not — a fact streams once, an absent reader just never
//! sees it.

use alacer_core::cursor::CursorStyle;
use alacer_core::geom::Point;
use alacer_core::inspect::InspectEvent;
use alacer_core::style::Size2;
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_testing::TestHarness;
use alacer_widgets::{BoxedWidget as _, Column, Link};

/// The center of a mounted widget's laid-out bounds — the hover point
/// the tests resolve with (the M152 suite's same helper shape).
fn center(harness: &TestHarness, node: alacer_core::NodeId) -> Point {
    let bounds = harness.bounds(node);
    Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    )
}

/// A styled leaf far from the column's face — the second word the
/// change tests resolve between (the void answers the default, the
/// styled leaf the grab, so one alternating pair drives every note).
struct StyledBox;

impl Widget for StyledBox {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Canvas
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("Styled")
    }

    fn style(&self) -> alacer_core::style::Style {
        alacer_core::style::Style {
            size: Size2 {
                width: Some(120.0),
                height: Some(48.0),
            },
            ..alacer_core::style::Style::default()
        }
    }

    fn cursor_style(&self) -> CursorStyle {
        CursorStyle::Grab
    }
}

/// The cursor facts a drained window carries, oldest first — the filter
/// the tier's readers share (the settings bench's own shape): the
/// reader's window empties per beat (the drain owns it), so each
/// assertion reads exactly the facts its own frame streamed.
fn cursor_facts(window: &[InspectEvent]) -> Vec<(CursorStyle, &'static str)> {
    window
        .iter()
        .filter_map(|event| match event {
            InspectEvent::Cursor(record) => Some((record.style, record.source.name())),
            _ => None,
        })
        .collect()
}

/// A resolved change streams one record at the tick's start drain: the
/// hover moving from the void onto the styled leaf is one fact — the
/// grab style beside the widget source that named it — and the same
/// hover resolving again is no fact at all (a style that did not move
/// is not a fact, the flood rule at the cursor tier).
#[test]
fn m153_a_resolved_change_streams_one_record_and_a_still_style_streams_none() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(12.0).padding(16.0).boxed());
    let styled = harness.add_child(root, StyledBox.boxed());
    harness.settle();
    let trail = harness.trail();

    let over = center(&harness, styled);
    let void = Point::new(-40.0, -40.0);

    // First resolution: the pointer rests on the styled leaf — one
    // fact, the grab style beside the widget that named it.
    let _ = harness.pipeline().cursor_at(over);
    harness.tick();
    let facts = cursor_facts(&trail.drain());
    assert_eq!(
        facts,
        vec![(CursorStyle::Grab, "widget")],
        "one resolved change, one record: the style beside the source that named it"
    );

    // The same hover again: no change, no fact — the drain reads empty.
    let _ = harness.pipeline().cursor_at(over);
    harness.tick();
    assert!(
        cursor_facts(&trail.drain()).is_empty(),
        "a style that did not move is not a fact"
    );

    // The hover leaving for the void (a point nothing accepts —
    // outside the viewport): the default stands — one fact, the honest
    // absence named as the default's own source.
    let _ = harness.pipeline().cursor_at(void);
    harness.tick();
    let facts = cursor_facts(&trail.drain());
    assert_eq!(
        facts,
        vec![(CursorStyle::Default, "default")],
        "the void's answer is the default fact, the absence named as a source"
    );
}

/// The trail streams through a real widget's word: the hover moving
/// onto a link names the pointer beside the link's own node and role —
/// the source the record carries is the hit the route would click.
#[test]
fn m153_the_record_names_the_widget_word_it_rode() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(12.0).padding(16.0).boxed());
    let link = harness.add_child(root, Link::new("Documentation").boxed());
    harness.settle();
    let trail = harness.trail();

    let _ = harness.pipeline().cursor_at(center(&harness, link));
    harness.tick();
    let facts = cursor_facts(&trail.drain());
    assert_eq!(
        facts.first().map(|(style, _)| *style),
        Some(CursorStyle::Pointer),
        "the link's word is the pointer — the M152 vocabulary"
    );
    assert_eq!(
        facts.first().map(|(_, source)| *source),
        Some("widget"),
        "the source names the widget tier — the hook that answered"
    );
}

/// The disabled trail costs nothing and streams nothing: with no
/// observer installed the resolved changes queue no facts — the idle
/// contract's one branch per resolution — and the trail a *later*
/// observer reads starts where that observer came in: the disabled
/// period's changes are not facts waiting to stream.
#[test]
fn m153_the_disabled_period_queues_nothing_the_later_reader_starts_where_it_came_in() {
    let mut harness = TestHarness::new(400.0, 200.0);
    let root = harness.mount(Column::new().gap(12.0).padding(16.0).boxed());
    let link = harness.add_child(root, Link::new("Documentation").boxed());
    harness.settle();

    // The disabled period: two resolved changes, no observer — the
    // notes are one branch each and queue nothing.
    let _ = harness.pipeline().cursor_at(center(&harness, link));
    let _ = harness.pipeline().cursor_at(Point::new(380.0, 190.0));
    let _ = harness.pipeline().cursor_at(center(&harness, link));

    // The reader comes in now: the trail starts empty — the disabled
    // period's churn was never queued (a fact queues where it resolves,
    // and none of these resolved under a listener).
    let trail = harness.trail();
    assert!(
        cursor_facts(&trail.look()).is_empty(),
        "the disabled period's changes are not facts waiting to stream"
    );

    // And the live trail streams from here: the next real change is
    // the one fact the reader sees (the pointer still rests where the
    // last disabled resolution left it — the styled link — so the
    // outside void, a point nothing accepts, is the change).
    let _ = harness.pipeline().cursor_at(Point::new(-40.0, -40.0));
    harness.tick();
    let facts = cursor_facts(&trail.drain());
    assert_eq!(
        facts,
        vec![(CursorStyle::Default, "default")],
        "the live trail streams the change that resolved under it"
    );
}

// The gate's vocabulary re-agrees with the document (ADR-0157's own
// decision sentence: "the tier joining the subscribe gate's vocabulary
// and the stats answer's rows (the bounds sentence's own list — the two
// cannot drift apart)"). The M153 cut grew the `cursors` ring, the wire
// kind, and the stats answer's row — and the protocol document's
// subscribe-gate paragraph, the ask's own closed vocabulary, must name
// the tier too: one list the two doors read, the two unable to drift
// apart. The panel's half rides the served client (the html client's
// own beat in `alacer-inspect`); this one pins the document's.
#[test]
fn m153_the_gate_vocabulary_re_agrees_with_the_document() {
    let protocol = read_workspace_file("docs/inspector-protocol.md");

    // The stats answer's row list names the ring (the capture's half —
    // the vocabulary's order, `cursors` before `frames` last).
    assert!(
        protocol.contains("`cursors`, `frames`)"),
        "the stats answer's row list names the cursors ring"
    );

    // The subscribe ask's paragraph names the tier beside the others —
    // the gate's half of the one-list rule, the ask's own order.
    assert!(
        protocol.contains("`settings`, `cursors`, `frames`)"),
        "the subscribe gate's documented vocabulary names the cursors tier — the two lists cannot drift apart"
    );

    // The paragraph's own count phrase re-agrees with the vocabulary.
    assert!(
        protocol.contains("the twenty-three trail rings"),
        "the subscribe gate's count phrase re-agrees with the twenty-three trail rings"
    );
}

/// The m-suites' workspace-file read door (the m41/m57 idiom): the
/// document pins ride the repo's own files, read from the workspace
/// root the manifest path names.
fn read_workspace_file(path: &str) -> String {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let workspace = std::path::Path::new(&manifest)
        .ancestors()
        .nth(2)
        .expect("the suite rides two directories under the workspace root");
    std::fs::read_to_string(workspace.join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}
