//! The causal recorder (§13.3, ADR-0010): the
//! [`alacer_core::InspectObserver`] implementation the pipeline forwards
//! into — bounded ring buffers of frames, edges, writes, and inputs (every
//! buffer has an eviction policy and a test that proves it, §14.3), plus
//! the wire-shaped broadcast queue the server's sessions drain.

use crate::server::push_broadcast;
use crate::wire::WireEvent;
use alacer_core::inspect::{
    AnnounceRecord, EdgeActivation, EditRecord, FocusRecord, InputKind, InspectEvent,
    InspectObserver, KeyRecord, SignalWrite,
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// The broadcast queue's cap — mirrored from the server (a slow client must
/// never back-pressure the app's observer thread).
pub const BROADCAST_RING_CAP: usize = 4_096;

/// The frame ring's cap (oldest dropped first).
pub const FRAME_RING_CAP: usize = 256;
/// The edge ring's cap (oldest dropped first).
pub const EDGE_RING_CAP: usize = 1_024;
/// The write ring's cap (oldest dropped first).
pub const WRITE_RING_CAP: usize = 1_024;
/// The input ring's cap (oldest dropped first).
pub const INPUT_RING_CAP: usize = 512;
/// The focus-trail ring's cap (oldest dropped first, ADR-0034).
pub const FOCUS_RING_CAP: usize = 512;
/// The edit-trail ring's cap (oldest dropped first, ADR-0037) — the
/// journal's own bound ([`alacer_core::history::EDIT_TRAIL_CAP`]) at the
/// recorder side, the same order the input and focus rings ride.
pub const EDIT_RING_CAP: usize = 512;
/// The announcement-trail ring's cap (oldest dropped first, ADR-0039) —
/// the speech acts' facts at the recorder side, the same order the
/// input, focus, and edit rings ride.
pub const ANNOUNCE_RING_CAP: usize = 512;
/// The shortcut-trail ring's cap (oldest dropped first, ADR-0042) —
/// the keymap consults' facts at the recorder side, the same order the
/// input, focus, edit, and announce rings ride.
pub const KEY_RING_CAP: usize = 512;

/// One recorded frame (the wire conversion lives in [`crate::wire`]).
#[derive(Debug, Clone)]
pub struct RecordedFrame {
    /// The frame summary the pipeline produced.
    pub summary: alacer_core::inspect::FrameSummary,
}

/// The shared recording state (the recorder and its live handles share it).
#[derive(Debug, Default)]
struct Recording {
    app: String,
    frames: VecDeque<RecordedFrame>,
    edges: VecDeque<EdgeActivation>,
    writes: VecDeque<SignalWrite>,
    inputs: VecDeque<(InputKind, Option<alacer_core::arena::NodeId>)>,
    focus: VecDeque<FocusRecord>,
    edits: VecDeque<EditRecord>,
    announces: VecDeque<AnnounceRecord>,
    keys: VecDeque<KeyRecord>,
    /// The most recent frame, for "what does the inspector see right now".
    latest: Option<alacer_core::inspect::FrameSummary>,
    /// The wire-shaped events awaiting the server's session drain (push, not
    /// poll: the observer thread serializes as it records).
    broadcast: VecDeque<WireEvent>,
}

/// The causal recorder: implements the pipeline's observer sink over bounded
/// rings, and hands live handles to the server (the producer/consumer split
/// is one `Arc<Mutex<Recording>>`).
#[derive(Debug, Clone)]
pub struct CausalRecorder {
    recording: Arc<Mutex<Recording>>,
    frames_seen: Arc<AtomicU64>,
}

impl CausalRecorder {
    /// A recorder labeled with the app's name (the greeting's `app` field).
    #[must_use]
    pub fn new(app: impl Into<String>) -> Self {
        Self {
            recording: Arc::new(Mutex::new(Recording {
                app: app.into(),
                ..Recording::default()
            })),
            frames_seen: Arc::new(AtomicU64::new(0)),
        }
    }

    /// A thread-safe handle for [`alacer_core::PassPipeline::set_observer`]
    /// (the observer sink is `&mut`-driven; the handle forwards into the
    /// shared recording).
    #[must_use]
    pub fn clone_handle(&self) -> RecorderHandle {
        RecorderHandle {
            recording: Arc::clone(&self.recording),
            frames_seen: Arc::clone(&self.frames_seen),
        }
    }

    /// The recorded frames, oldest first (bounded by [`FRAME_RING_CAP`]).
    #[must_use]
    pub fn frames(&self) -> Vec<RecordedFrame> {
        self.lock().frames.iter().cloned().collect()
    }

    /// The recorded edge activations, oldest first.
    #[must_use]
    pub fn edges(&self) -> Vec<EdgeActivation> {
        self.lock().edges.iter().cloned().collect()
    }

    /// The recorded provenance writes, oldest first.
    #[must_use]
    pub fn writes(&self) -> Vec<SignalWrite> {
        self.lock().writes.iter().cloned().collect()
    }

    /// The recorded routed inputs, oldest first.
    #[must_use]
    pub fn inputs(&self) -> Vec<(InputKind, Option<alacer_core::arena::NodeId>)> {
        self.lock().inputs.iter().copied().collect()
    }

    /// The recorded focus trail, oldest first (ADR-0034): every
    /// focus-move door ride the pipeline answered, landings and
    /// refusals alike, bounded by [`FOCUS_RING_CAP`].
    #[must_use]
    pub fn focus(&self) -> Vec<FocusRecord> {
        self.lock().focus.iter().copied().collect()
    }

    /// The recorded edit trail, oldest first (ADR-0037): every journal
    /// action the pipeline's drain forwarded — applied steps, refused
    /// chords, the app's resets — bounded by [`EDIT_RING_CAP`].
    #[must_use]
    pub fn edits(&self) -> Vec<EditRecord> {
        self.lock().edits.iter().copied().collect()
    }

    /// The recorded announcement trail, oldest first (ADR-0039): every
    /// speech act the pipeline answered — delivered acts, superseded
    /// acts, the named refusals — bounded by [`ANNOUNCE_RING_CAP`].
    #[must_use]
    pub fn announces(&self) -> Vec<AnnounceRecord> {
        self.lock().announces.iter().copied().collect()
    }

    /// The recorded shortcut trail, oldest first (ADR-0042): every
    /// keymap consult the funnel ran — fired chords, the tables'
    /// fall-throughs, the uninstalled windows' honest answers — bounded
    /// by [`KEY_RING_CAP`].
    #[must_use]
    pub fn keys(&self) -> Vec<KeyRecord> {
        self.lock().keys.iter().copied().collect()
    }

    /// The most recent frame, when any frame was observed.
    #[must_use]
    pub fn latest_frame(&self) -> Option<alacer_core::inspect::FrameSummary> {
        self.lock().latest.clone()
    }

    /// Total frames observed (never evicted — the honest count).
    #[must_use]
    pub fn frames_seen(&self) -> u64 {
        self.frames_seen.load(Ordering::Relaxed)
    }

    /// Clears every ring (the client's `clear` control message).
    pub fn clear(&self) {
        if let Ok(mut recording) = self.recording.lock() {
            recording.frames.clear();
            recording.edges.clear();
            recording.writes.clear();
            recording.inputs.clear();
            recording.focus.clear();
            recording.edits.clear();
            recording.announces.clear();
            recording.keys.clear();
        }
    }

    /// Drains the broadcast queue (the session's push source), oldest first.
    #[must_use]
    pub fn take_broadcast(&self) -> Vec<WireEvent> {
        self.recording
            .lock()
            .map(|mut recording| recording.broadcast.drain(..).collect())
            .unwrap_or_default()
    }

    /// The app name (the greeting's `app` field).
    #[must_use]
    pub fn app(&self) -> String {
        self.lock().app.clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Recording> {
        self.recording
            .lock()
            .expect("causal recorder lock poisoned")
    }
}

/// The `&mut` observer handle installed on the pipeline.
#[derive(Debug, Clone)]
pub struct RecorderHandle {
    recording: Arc<Mutex<Recording>>,
    frames_seen: Arc<AtomicU64>,
}

/// Pushes into a bounded ring: evicts the oldest when at capacity (§14.3).
fn push_bounded<T>(ring: &mut VecDeque<T>, cap: usize, value: T) {
    if ring.len() >= cap {
        ring.pop_front();
    }
    ring.push_back(value);
}

impl InspectObserver for RecorderHandle {
    fn observe(&mut self, event: InspectEvent) {
        let Ok(mut recording) = self.recording.lock() else {
            return;
        };
        // The wire serialization happens on the observing thread: the
        // session drains ready-to-send payloads (push, not poll).
        let wire_event = crate::server::wire_event_of(&event);
        push_broadcast(&mut recording.broadcast, wire_event);
        match event {
            InspectEvent::Edge(activation) => {
                push_bounded(&mut recording.edges, EDGE_RING_CAP, activation);
            }
            InspectEvent::Write(write) => {
                push_bounded(&mut recording.writes, WRITE_RING_CAP, write);
            }
            InspectEvent::Input(input) => {
                push_bounded(
                    &mut recording.inputs,
                    INPUT_RING_CAP,
                    (input.kind, input.target),
                );
            }
            InspectEvent::Focus(record) => {
                push_bounded(&mut recording.focus, FOCUS_RING_CAP, record);
            }
            InspectEvent::Edit(record) => {
                push_bounded(&mut recording.edits, EDIT_RING_CAP, record);
            }
            InspectEvent::Announce(record) => {
                push_bounded(&mut recording.announces, ANNOUNCE_RING_CAP, record);
            }
            InspectEvent::Key(record) => {
                push_bounded(&mut recording.keys, KEY_RING_CAP, record);
            }
            InspectEvent::Frame(summary) => {
                self.frames_seen.fetch_add(1, Ordering::Relaxed);
                recording.latest = Some(summary.clone());
                push_bounded(
                    &mut recording.frames,
                    FRAME_RING_CAP,
                    RecordedFrame { summary },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CausalRecorder;
    use alacer_core::inspect::{EdgeActivation, InputKind, InspectEvent, InspectObserver};

    #[test]
    fn rings_evict_the_oldest_and_keep_honest_counts() {
        let recorder = CausalRecorder::new("rings");
        let mut handle = recorder.clone_handle();
        for i in 0..(super::EDGE_RING_CAP + 50) {
            handle.observe(InspectEvent::Edge(EdgeActivation {
                binding: u64::try_from(i).expect("binding id"),
                signal: 1,
                node: alacer_core::arena::NodeId::allocate(),
                masks: alacer_core::Change::PAINT,
            }));
        }
        assert_eq!(
            recorder.edges().len(),
            super::EDGE_RING_CAP,
            "the ring is bounded"
        );
        assert_eq!(
            recorder.edges()[0].binding,
            u64::try_from(50).expect("first survivor"),
            "the oldest entries were evicted"
        );
    }

    #[test]
    fn broadcast_drains_in_record_order_and_is_bounded() {
        let recorder = CausalRecorder::new("broadcast");
        let mut handle = recorder.clone_handle();
        for i in 0..(super::BROADCAST_RING_CAP + 10) {
            handle.observe(InspectEvent::Input(alacer_core::inspect::InputRecord {
                kind: InputKind::Key,
                target: None,
            }));
            let _ = i;
        }
        let drained = recorder.take_broadcast();
        assert_eq!(drained.len(), super::BROADCAST_RING_CAP, "bounded");
        assert!(recorder.take_broadcast().is_empty(), "drained empty");
    }

    #[test]
    fn frames_land_in_order_with_a_live_latest() {
        let recorder = CausalRecorder::new("frames");
        let mut handle = recorder.clone_handle();
        for number in 0..3_u64 {
            handle.observe(InspectEvent::Frame(dummy_summary(number)));
        }
        assert_eq!(recorder.frames_seen(), 3, "the honest count never evicts");
        let frames = recorder.frames();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].summary.number, 0);
        assert_eq!(
            recorder.latest_frame().expect("latest").number,
            2,
            "latest is the most recent frame"
        );
    }

    #[test]
    fn focus_records_ring_in_order_and_are_bounded() {
        use alacer_core::passes::{FocusMove, FocusMoveOutcome, FocusRefusal};
        let recorder = CausalRecorder::new("focus");
        let mut handle = recorder.clone_handle();
        let node = alacer_core::arena::NodeId::allocate();
        for i in 0..(super::FOCUS_RING_CAP + 10) {
            handle.observe(InspectEvent::Focus(alacer_core::inspect::FocusRecord {
                movement: FocusMove::Next,
                scope: None,
                rider: alacer_core::inspect::FocusRider::Consumer,
                outcome: FocusMoveOutcome::Moved(alacer_core::passes::FocusLanding {
                    from: None,
                    to: node,
                    wrapped: false,
                }),
            }));
            let _ = i;
        }
        let trail = recorder.focus();
        assert_eq!(trail.len(), super::FOCUS_RING_CAP, "the trail is bounded");
        assert_eq!(
            trail[0].movement,
            FocusMove::Next,
            "the survivors keep order"
        );

        // The refusal rides the same ring, the wire carrying its species.
        handle.observe(InspectEvent::Focus(alacer_core::inspect::FocusRecord {
            movement: FocusMove::Right,
            scope: None,
            rider: alacer_core::inspect::FocusRider::Consumer,
            outcome: FocusMoveOutcome::Refused(FocusRefusal::NothingInDirection {
                node,
                direction: FocusMove::Right,
            }),
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the refusal broadcast");
        let crate::wire::WireEvent::Focus {
            movement,
            moved,
            refusal,
            direction,
            ..
        } = last
        else {
            panic!("the focus broadcast");
        };
        assert_eq!(movement, "right");
        assert!(!*moved);
        assert_eq!(refusal.as_deref(), Some("nothing_in_direction"));
        assert_eq!(direction.as_deref(), Some("right"));
    }

    #[test]
    fn edit_records_ring_in_order_and_are_bounded() {
        use alacer_core::history::EditAction;
        let recorder = CausalRecorder::new("edits");
        let mut handle = recorder.clone_handle();
        for _ in 0..(super::EDIT_RING_CAP + 10) {
            handle.observe(InspectEvent::Edit(alacer_core::inspect::EditRecord {
                action: EditAction::Record,
                coalesced: true,
                refused: false,
                undo_depth: 1,
                redo_depth: 0,
            }));
        }
        let edits = recorder.edits();
        assert_eq!(edits.len(), super::EDIT_RING_CAP, "the ring is bounded");
        assert_eq!(
            edits[0].action,
            EditAction::Record,
            "the survivors keep order"
        );

        // The refused chord rides the same ring, the wire carrying the
        // refusal flag beside the action.
        handle.observe(InspectEvent::Edit(alacer_core::inspect::EditRecord {
            action: EditAction::Undo,
            coalesced: false,
            refused: true,
            undo_depth: 0,
            redo_depth: 0,
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the refusal broadcast");
        let crate::wire::WireEvent::Edit {
            action,
            refused,
            undo_depth,
            ..
        } = last
        else {
            panic!("the edit broadcast");
        };
        assert_eq!(action, "undo");
        assert!(*refused);
        assert_eq!(*undo_depth, 0);
    }

    #[test]
    fn announce_records_ring_in_order_and_are_bounded() {
        use alacer_core::announce::Politeness;
        use alacer_core::inspect::{AnnounceOutcome, AnnounceRefusal};
        let recorder = CausalRecorder::new("announces");
        let mut handle = recorder.clone_handle();
        for _ in 0..(super::ANNOUNCE_RING_CAP + 10) {
            handle.observe(InspectEvent::Announce(
                alacer_core::inspect::AnnounceRecord {
                    politeness: Politeness::Polite,
                    outcome: AnnounceOutcome::Delivered,
                },
            ));
        }
        let trail = recorder.announces();
        assert_eq!(trail.len(), super::ANNOUNCE_RING_CAP, "the ring is bounded");
        assert_eq!(
            trail[0].outcome,
            AnnounceOutcome::Delivered,
            "the survivors keep order"
        );

        // The refusal rides the same ring, the wire carrying its species
        // beside the outcome.
        handle.observe(InspectEvent::Announce(
            alacer_core::inspect::AnnounceRecord {
                politeness: Politeness::Assertive,
                outcome: AnnounceOutcome::Refused(AnnounceRefusal::NoRoot),
            },
        ));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the refusal broadcast");
        let crate::wire::WireEvent::Announce {
            politeness,
            outcome,
            refusal,
        } = last
        else {
            panic!("the announce broadcast");
        };
        assert_eq!(politeness, "assertive");
        assert_eq!(outcome, "refused");
        assert_eq!(refusal.as_deref(), Some("no_root"));
    }

    #[test]
    fn key_records_ring_in_order_and_are_bounded() {
        use alacer_core::events::Key;
        use alacer_core::keymap::{Chord, KeymapConsult};
        let recorder = CausalRecorder::new("keys");
        let mut handle = recorder.clone_handle();
        for _ in 0..(super::KEY_RING_CAP + 10) {
            handle.observe(InspectEvent::Key(alacer_core::inspect::KeyRecord {
                chord: Chord::new(Key::Char('s')).control(),
                outcome: KeymapConsult::Fired,
            }));
        }
        let keys = recorder.keys();
        assert_eq!(keys.len(), super::KEY_RING_CAP, "the ring is bounded");
        assert_eq!(
            keys[0].outcome,
            KeymapConsult::Fired,
            "the survivors keep order"
        );

        // The miss rides the same ring, the wire carrying the chord's
        // flat shape beside the species.
        handle.observe(InspectEvent::Key(alacer_core::inspect::KeyRecord {
            chord: Chord::new(Key::Char('x')),
            outcome: KeymapConsult::FellThrough,
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the miss broadcast");
        let crate::wire::WireEvent::Key {
            key,
            character,
            modifiers,
            outcome,
        } = last
        else {
            panic!("the key broadcast");
        };
        assert_eq!(key, "char");
        assert_eq!(character.as_deref(), Some("x"));
        assert_eq!(modifiers, "", "no modifiers held — the exact shape");
        assert_eq!(outcome, "fell_through");

        // The chord's modifier shape rides the same flat event.
        handle.observe(InspectEvent::Key(alacer_core::inspect::KeyRecord {
            chord: Chord::new(Key::ArrowUp).control(),
            outcome: KeymapConsult::Fired,
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the chord broadcast");
        let crate::wire::WireEvent::Key {
            key,
            character,
            modifiers,
            outcome,
        } = last
        else {
            panic!("the chord broadcast");
        };
        assert_eq!(key, "arrow_up");
        assert!(character.is_none(), "a non-char key carries no character");
        assert_eq!(modifiers, "ctrl");
        assert_eq!(outcome, "fired");
    }

    #[test]
    fn inputs_record_kind_and_target() {
        let recorder = CausalRecorder::new("inputs");
        let mut handle = recorder.clone_handle();
        handle.observe(InspectEvent::Input(alacer_core::inspect::InputRecord {
            kind: InputKind::PointerDown,
            target: Some(alacer_core::arena::NodeId::allocate()),
        }));
        handle.observe(InspectEvent::Input(alacer_core::inspect::InputRecord {
            kind: InputKind::Key,
            target: None,
        }));
        let inputs = recorder.inputs();
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].0, InputKind::PointerDown);
        assert!(inputs[0].1.is_some());
        assert_eq!(inputs[1].0, InputKind::Key);
        assert!(inputs[1].1.is_none());
    }

    /// A minimal frame summary for ring tests.
    fn dummy_summary(number: u64) -> alacer_core::inspect::FrameSummary {
        alacer_core::inspect::FrameSummary {
            number,
            layout_ran: false,
            paint_ran: false,
            a11y_ran: false,
            focus_changed: false,
            damage: alacer_core::Damage::NONE,
            viewport: alacer_core::geom::Size::new(1.0, 1.0),
            a11y_updated: 0,
            a11y_removed: 0,
            total_nanos: 0,
            passes: Vec::new(),
            dirty: Vec::new(),
            dirty_truncated: 0,
        }
    }
}
