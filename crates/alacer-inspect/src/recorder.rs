//! The causal recorder (§13.3, ADR-0010): the
//! [`alacer_core::InspectObserver`] implementation the pipeline forwards
//! into — bounded ring buffers of frames, edges, writes, and inputs (every
//! buffer has an eviction policy and a test that proves it, §14.3), plus
//! the wire-shaped broadcast queue the server's sessions drain.

use crate::server::push_broadcast;
use crate::wire::WireEvent;
use alacer_core::inspect::{
    AnnounceRecord, ClipboardRecord, EdgeActivation, EditRecord, FocusRecord, InputKind,
    InputRecord, InspectEvent, InspectObserver, KeyRecord, SignalWrite,
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// The broadcast queue's cap — the one constant the two halves share
/// (the server's drain imports it): pending wire events before the
/// oldest drop, so a slow client must never back-pressure the app's
/// observer thread.
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
/// The clipboard-trail ring's cap (oldest dropped first, ADR-0045) —
/// the round trips' facts at the recorder side, the same order the
/// input, focus, edit, announce, and key rings ride.
pub const CLIPBOARD_RING_CAP: usize = 512;
/// The value-trail ring's cap (oldest dropped first, ADR-0046) — the
/// value tier's facts at the recorder side, the same order the input,
/// focus, edit, announce, key, and clipboard rings ride.
pub const VALUE_RING_CAP: usize = 512;
/// The gesture-trail ring's cap (oldest dropped first, ADR-0052) — the
/// drag's per-phase facts at the recorder side, the same order the
/// input, focus, edit, announce, key, clipboard, and value rings ride.
pub const GESTURE_RING_CAP: usize = 512;
/// The scroll-trail ring's cap (oldest dropped first, ADR-0056) — the
/// scroll tier's facts at the recorder side, the same order the input,
/// focus, edit, announce, key, clipboard, value, and gesture rings ride.
pub const SCROLL_RING_CAP: usize = 512;
/// The text-write trail ring's cap (oldest dropped first, ADR-0061) —
/// the editing tier's facts at the recorder side, the same order the
/// input, focus, edit, announce, key, clipboard, value, gesture, and
/// scroll rings ride.
pub const TEXT_WRITE_RING_CAP: usize = 512;
/// The route-drop ring's cap (oldest dropped first, ADR-0063) — the
/// boundary's facts at the recorder side, the same order the input,
/// focus, edit, announce, key, clipboard, value, gesture, scroll, and
/// text-write rings ride.
pub const ROUTE_DROP_RING_CAP: usize = 512;
/// The key-edit trail ring's cap (oldest dropped first, ADR-0064) —
/// the typing door's facts at the recorder side, the same order the
/// input, focus, edit, announce, key, clipboard, value, gesture,
/// scroll, text-write, and route-drop rings ride.
pub const KEY_EDIT_RING_CAP: usize = 512;

/// The preedit ring's cap: composing-state facts carried before the
/// oldest evicts (the key-edit ring's own order, §14.3).
pub const PREEDIT_RING_CAP: usize = 512;

/// The a11y-echo ring's cap: adapter-echo facts carried before the
/// oldest evicts (the preedit ring's own order, §14.3 — a connect is
/// rare, the ring stays quiet between connects).
pub const A11Y_ECHO_RING_CAP: usize = 512;

/// The action ring's cap: activation-ask facts carried before the
/// oldest evicts (the a11y-echo ring's own order, §14.3).
pub const ACTION_RING_CAP: usize = 512;

/// The frame-refusal ring's cap: the frame tier's own facts — the
/// ladder stages' species, bounded and named like every other tier's
/// ring (ADR-0077).
pub const FRAME_REFUSAL_RING_CAP: usize = 512;

/// The view-patch ring's cap: the view tier's own facts — the
/// reconciler's four counts, bounded and named like every other
/// tier's ring (ADR-0078).
pub const VIEW_PATCH_RING_CAP: usize = 512;

/// The theme ring's cap: the theme tier's own facts — the swaps'
/// palette tags, bounded and named like every other tier's ring
/// (ADR-0079).
pub const THEME_RING_CAP: usize = 512;

/// The setting ring's cap: the setting tier's own facts — the OS
/// flips' dark-mode facts, bounded and named like every other tier's
/// ring (ADR-0080, the theme ring's shape).
pub const SETTING_RING_CAP: usize = 512;

/// The cursor ring's cap: the cursor tier's own facts — the hover's
/// resolved changes, bounded and named like every other tier's ring
/// (ADR-0157, the theme ring's shape).
pub const CURSOR_RING_CAP: usize = 512;

/// One recorded frame (the wire conversion lives in [`crate::wire`]).
#[derive(Debug, Clone)]
pub struct RecordedFrame {
    /// The frame summary the pipeline produced.
    pub summary: alacer_core::inspect::FrameSummary,
}

/// One bounded ring: the stamped queue, its cap, and the eviction
/// counter the cap causes (§14.3 — every buffer has an eviction policy
/// and the counter is the loss the policy names, never a silent drop).
/// The entries are `(fact_number, record)` pairs, oldest first — the
/// stamps are what the replay ask's cursor (M140) filters on, and the
/// counter is what the stats ask (M141) answers beside the occupancy.
#[derive(Debug)]
struct Ring<T> {
    queue: VecDeque<(u64, T)>,
    cap: usize,
    evicted: u64,
}

impl<T> Ring<T> {
    /// A ring held to `cap` (the `*_RING_CAP` order).
    fn new(cap: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            cap,
            evicted: 0,
        }
    }

    /// Pushes one stamped fact, evicting the oldest at the cap — the
    /// eviction counted, the loss named.
    fn push(&mut self, fact_number: u64, value: T) {
        if self.queue.len() >= self.cap {
            self.queue.pop_front();
            self.evicted = self.evicted.saturating_add(1);
        }
        self.queue.push_back((fact_number, value));
    }

    /// Empties the ring (the `clear` control's reset); the eviction
    /// counter is history, not contents — it keeps its count.
    fn clear(&mut self) {
        self.queue.clear();
    }

    /// The stamped entries, oldest first.
    fn iter(&self) -> std::collections::vec_deque::Iter<'_, (u64, T)> {
        self.queue.iter()
    }
}

/// The shared recording state (the recorder and its live handles share it).
///
/// Every ring's entries are stamped with the fact number the observation
/// carried (`(fact_number, record)` pairs, oldest first): the stamp is
/// what the replay ask's cursor (M140) filters on — "only the facts
/// recorded after this fact number" needs each entry's number, and the
/// number is assigned once, at the observation, in the same lock the
/// rings are written under.
#[derive(Debug)]
struct Recording {
    app: String,
    /// The total facts observed — every `observe` call counts one, the
    /// number the next fact carries (never evicted, never reset: a
    /// cleared recording answers an empty burst regardless, and the
    /// count's monotonicity is what makes a cursor stable across a
    /// clear-reconnect window). The reader door is
    /// [`CausalRecorder::facts`].
    facts: u64,
    frames: Ring<RecordedFrame>,
    edges: Ring<EdgeActivation>,
    writes: Ring<SignalWrite>,
    inputs: Ring<(InputKind, Option<alacer_core::arena::NodeId>)>,
    focus: Ring<FocusRecord>,
    edits: Ring<EditRecord>,
    announces: Ring<AnnounceRecord>,
    keys: Ring<KeyRecord>,
    clipboards: Ring<ClipboardRecord>,
    values: Ring<alacer_core::inspect::ValueRecord>,
    gestures: Ring<alacer_core::inspect::GestureRecord>,
    scrolls: Ring<alacer_core::inspect::ScrollRecord>,
    /// The recorded text writes, oldest first (the door's own facts —
    /// the whole-value write, the replacement, the selection ask, the
    /// closed door's refusal — bounded by [`TEXT_WRITE_RING_CAP`]).
    text_writes: Ring<alacer_core::inspect::TextWriteRecord>,
    /// The recorded route drops, oldest first (the boundary's own
    /// facts — the unknown action, the payloadless write, the
    /// off-target selection, the unknown target — bounded by
    /// [`ROUTE_DROP_RING_CAP`]).
    route_drops: Ring<alacer_core::inspect::RouteDropRecord>,
    /// The recorded key edits, oldest first (the typing door's own
    /// facts — the typed run folded to one record, the IME commit,
    /// the unit and word deletes — bounded by [`KEY_EDIT_RING_CAP`]).
    key_edits: Ring<alacer_core::inspect::KeyEditRecord>,
    /// The composing-state facts (ADR-0067: the composition's inline
    /// shape, the phases beside it — bounded by [`PREEDIT_RING_CAP`]).
    preedits: Ring<alacer_core::inspect::PreeditRecord>,
    /// The adapter-echo facts (ADR-0073: the tree the activation
    /// answered, named by its node count — bounded by
    /// [`A11Y_ECHO_RING_CAP`]).
    a11y_echoes: Ring<alacer_core::inspect::A11yEchoRecord>,
    /// The activation-ask facts (ADR-0075: the press the AT's action
    /// route aimed at a control, the landing or the named refusal
    /// beside the aim — bounded by [`ACTION_RING_CAP`]).
    actions: Ring<alacer_core::inspect::ActionRecord>,
    /// The frame-refusal facts (ADR-0077: the ladder stage's species,
    /// the failing node beside it when one is the fact — bounded by
    /// [`FRAME_REFUSAL_RING_CAP`]).
    frame_refusals: Ring<alacer_core::inspect::FrameRefusalRecord>,
    /// The view-patch facts (ADR-0078: the reconciler's four counts
    /// beside each other, the mount's build and every re-render's
    /// diff — bounded by [`VIEW_PATCH_RING_CAP`]).
    view_patches: Ring<alacer_core::inspect::ViewPatchRecord>,
    /// The theme facts (ADR-0079: the swaps' palette tags beside the
    /// door — bounded by [`THEME_RING_CAP`]).
    themes: Ring<alacer_core::inspect::ThemeRecord>,
    /// The setting facts (ADR-0080: the flips' dark-mode facts beside
    /// the door — bounded by [`SETTING_RING_CAP`]).
    settings: Ring<alacer_core::inspect::SettingRecord>,
    /// The cursor facts (ADR-0157: the hover's resolved changes beside
    /// the source that named them — bounded by [`CURSOR_RING_CAP`]).
    cursors: Ring<alacer_core::inspect::CursorRecord>,
    /// The most recent frame, for "what does the inspector see right now".
    latest: Option<alacer_core::inspect::FrameSummary>,
    /// The most recent resolved tree, for the dump ask (the observer's
    /// tree door retains it — state, never history: `clear` empties the
    /// rings and leaves the shape standing). The reader door is
    /// [`CausalRecorder::latest_tree`].
    latest_tree: Option<std::sync::Arc<alacer_core::TreeSnapshot>>,
    /// The wire-shaped events awaiting the server's session drain (push, not
    /// poll: the observer thread serializes as it records).
    broadcast: VecDeque<WireEvent>,
    /// The facts the broadcast queue's own bound has dropped — the
    /// stream's named loss (a slow session's window, priced on the
    /// stats answer beside the trail rings' rows).
    broadcast_evicted: u64,
}

impl Recording {
    /// A recording labeled with the app's name; every ring born at its
    /// documented cap (the `*_RING_CAP` order), the counts at zero.
    fn new(app: impl Into<String>) -> Self {
        Self {
            app: app.into(),
            facts: 0,
            frames: Ring::new(FRAME_RING_CAP),
            edges: Ring::new(EDGE_RING_CAP),
            writes: Ring::new(WRITE_RING_CAP),
            inputs: Ring::new(INPUT_RING_CAP),
            focus: Ring::new(FOCUS_RING_CAP),
            edits: Ring::new(EDIT_RING_CAP),
            announces: Ring::new(ANNOUNCE_RING_CAP),
            keys: Ring::new(KEY_RING_CAP),
            clipboards: Ring::new(CLIPBOARD_RING_CAP),
            values: Ring::new(VALUE_RING_CAP),
            gestures: Ring::new(GESTURE_RING_CAP),
            scrolls: Ring::new(SCROLL_RING_CAP),
            text_writes: Ring::new(TEXT_WRITE_RING_CAP),
            route_drops: Ring::new(ROUTE_DROP_RING_CAP),
            key_edits: Ring::new(KEY_EDIT_RING_CAP),
            preedits: Ring::new(PREEDIT_RING_CAP),
            a11y_echoes: Ring::new(A11Y_ECHO_RING_CAP),
            actions: Ring::new(ACTION_RING_CAP),
            frame_refusals: Ring::new(FRAME_REFUSAL_RING_CAP),
            view_patches: Ring::new(VIEW_PATCH_RING_CAP),
            themes: Ring::new(THEME_RING_CAP),
            settings: Ring::new(SETTING_RING_CAP),
            cursors: Ring::new(CURSOR_RING_CAP),
            latest: None,
            latest_tree: None,
            broadcast: VecDeque::new(),
            broadcast_evicted: 0,
        }
    }

    /// The stats answer's rows (M141): the stream's own bound first,
    /// the trail rings in the event vocabulary's order, `frames` last
    /// — one row per ring, the occupancy beside the cap beside the
    /// evictions the cap caused.
    fn stat_rows(&self) -> Vec<crate::wire::RingStat> {
        // One row per ring: the stream's own bound first, the trails in
        // the event vocabulary's order, `frames` last — the same list
        // the protocol document's bounds sentence names (the two cannot
        // drift apart; the stats answer's rows are that sentence on the
        // wire). The broadcast row carries its own counter; the trail
        // rows ride the generic reader.
        #[must_use]
        fn ring_row<T>(name: &'static str, ring: &Ring<T>, cap: usize) -> crate::wire::RingStat {
            crate::wire::RingStat {
                name,
                held: ring.queue.len(),
                cap,
                evicted: ring.evicted,
            }
        }
        vec![
            crate::wire::RingStat {
                name: "broadcast",
                held: self.broadcast.len(),
                cap: BROADCAST_RING_CAP,
                evicted: self.broadcast_evicted,
            },
            ring_row("edges", &self.edges, EDGE_RING_CAP),
            ring_row("writes", &self.writes, WRITE_RING_CAP),
            ring_row("inputs", &self.inputs, INPUT_RING_CAP),
            ring_row("focus", &self.focus, FOCUS_RING_CAP),
            ring_row("edits", &self.edits, EDIT_RING_CAP),
            ring_row("announces", &self.announces, ANNOUNCE_RING_CAP),
            ring_row("keys", &self.keys, KEY_RING_CAP),
            ring_row("clipboards", &self.clipboards, CLIPBOARD_RING_CAP),
            ring_row("values", &self.values, VALUE_RING_CAP),
            ring_row("gestures", &self.gestures, GESTURE_RING_CAP),
            ring_row("scrolls", &self.scrolls, SCROLL_RING_CAP),
            ring_row("text_writes", &self.text_writes, TEXT_WRITE_RING_CAP),
            ring_row("route_drops", &self.route_drops, ROUTE_DROP_RING_CAP),
            ring_row("key_edits", &self.key_edits, KEY_EDIT_RING_CAP),
            ring_row("preedits", &self.preedits, PREEDIT_RING_CAP),
            ring_row("a11y_echoes", &self.a11y_echoes, A11Y_ECHO_RING_CAP),
            ring_row("actions", &self.actions, ACTION_RING_CAP),
            ring_row(
                "frame_refusals",
                &self.frame_refusals,
                FRAME_REFUSAL_RING_CAP,
            ),
            ring_row("view_patches", &self.view_patches, VIEW_PATCH_RING_CAP),
            ring_row("themes", &self.themes, THEME_RING_CAP),
            ring_row("settings", &self.settings, SETTING_RING_CAP),
            ring_row("cursors", &self.cursors, CURSOR_RING_CAP),
            ring_row("frames", &self.frames, FRAME_RING_CAP),
        ]
    }
}

/// The causal recorder: implements the pipeline's observer sink over bounded
/// rings, and hands live handles to the server (the producer/consumer split
/// is one `Arc<Mutex<Recording>>`).
#[derive(Debug, Clone)]
pub struct CausalRecorder {
    recording: Arc<Mutex<Recording>>,
    frames_seen: Arc<AtomicU64>,
    /// The loss counter the handle shares: every fact the observer
    /// could not record (the poisoned-mutex drop), named for the
    /// reader — the loss never silent.
    dropped: Arc<AtomicU64>,
}

impl CausalRecorder {
    /// A recorder labeled with the app's name (the greeting's `app` field).
    #[must_use]
    pub fn new(app: impl Into<String>) -> Self {
        Self {
            recording: Arc::new(Mutex::new(Recording::new(app))),
            frames_seen: Arc::new(AtomicU64::new(0)),
            dropped: Arc::new(AtomicU64::new(0)),
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
            dropped: Arc::clone(&self.dropped),
        }
    }

    /// The facts the observer could not record (the poisoned-mutex
    /// drop): the loss the rings cannot carry, priced here — the
    /// named-eviction rule the trail reader rides, never a silent
    /// swallow.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// The recorded frames, oldest first (bounded by [`FRAME_RING_CAP`]).
    #[must_use]
    pub fn frames(&self) -> Vec<RecordedFrame> {
        self.try_lock()
            .map(|recording| {
                recording
                    .frames
                    .iter()
                    .map(|(_, frame)| frame.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded edge activations, oldest first.
    #[must_use]
    pub fn edges(&self) -> Vec<EdgeActivation> {
        self.try_lock()
            .map(|recording| {
                recording
                    .edges
                    .iter()
                    .map(|(_, activation)| activation.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded provenance writes, oldest first.
    #[must_use]
    pub fn writes(&self) -> Vec<SignalWrite> {
        self.try_lock()
            .map(|recording| {
                recording
                    .writes
                    .iter()
                    .map(|(_, write)| write.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded routed inputs, oldest first.
    #[must_use]
    pub fn inputs(&self) -> Vec<(InputKind, Option<alacer_core::arena::NodeId>)> {
        self.try_lock()
            .map(|recording| {
                recording
                    .inputs
                    .iter()
                    .map(|(_, (kind, target))| (*kind, *target))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded focus trail, oldest first (ADR-0034): every
    /// focus-move door ride the pipeline answered, landings and
    /// refusals alike, bounded by [`FOCUS_RING_CAP`].
    #[must_use]
    pub fn focus(&self) -> Vec<FocusRecord> {
        self.try_lock()
            .map(|recording| recording.focus.iter().map(|(_, record)| *record).collect())
            .unwrap_or_default()
    }

    /// The recorded edit trail, oldest first (ADR-0037): every journal
    /// action the pipeline's drain forwarded — applied steps, refused
    /// chords, the app's resets — bounded by [`EDIT_RING_CAP`].
    #[must_use]
    pub fn edits(&self) -> Vec<EditRecord> {
        self.try_lock()
            .map(|recording| recording.edits.iter().map(|(_, record)| *record).collect())
            .unwrap_or_default()
    }

    /// The recorded announcement trail, oldest first (ADR-0039): every
    /// speech act the pipeline answered — delivered acts, superseded
    /// acts, the named refusals — bounded by [`ANNOUNCE_RING_CAP`].
    #[must_use]
    pub fn announces(&self) -> Vec<AnnounceRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .announces
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded shortcut trail, oldest first (ADR-0042): every
    /// keymap consult the funnel ran — fired chords, the tables'
    /// fall-throughs, the uninstalled windows' honest answers — bounded
    /// by [`KEY_RING_CAP`].
    #[must_use]
    pub fn keys(&self) -> Vec<KeyRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .keys
                    .iter()
                    .map(|(_, record)| record.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded clipboard trail, oldest first (ADR-0045): every
    /// round trip the editing chords answered — delivered copies, cuts,
    /// pastes, and the named refusals — bounded by
    /// [`CLIPBOARD_RING_CAP`].
    #[must_use]
    pub fn clipboards(&self) -> Vec<ClipboardRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .clipboards
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded value trail, oldest first (ADR-0046): every
    /// value-tier write the AT asked — delivered writes and the named
    /// refusals — bounded by [`VALUE_RING_CAP`].
    #[must_use]
    pub fn values(&self) -> Vec<alacer_core::inspect::ValueRecord> {
        self.try_lock()
            .map(|recording| recording.values.iter().map(|(_, record)| *record).collect())
            .unwrap_or_default()
    }

    /// The recorded gesture trail, oldest first (ADR-0052): every
    /// drag phase the value tier's doors answered — the press that
    /// armed the drag, the release or leave that resolved it — bounded
    /// by [`GESTURE_RING_CAP`].
    #[must_use]
    pub fn gestures(&self) -> Vec<alacer_core::inspect::GestureRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .gestures
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded scroll trail, oldest first (ADR-0056): every scroll
    /// ask the scroll doors answered — the wheel's clamp, the AT's unit
    /// scrolls and value steps, the app's own programmatic scroll —
    /// bounded by [`SCROLL_RING_CAP`].
    #[must_use]
    pub fn scrolls(&self) -> Vec<alacer_core::inspect::ScrollRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .scrolls
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded text-write trail, oldest first (ADR-0061): every
    /// text write the input's AT door answered — the whole-value write,
    /// the paste-shaped replacement, the selection ask, and the closed
    /// door's refusal — bounded by [`TEXT_WRITE_RING_CAP`].
    #[must_use]
    pub fn text_writes(&self) -> Vec<alacer_core::inspect::TextWriteRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .text_writes
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded route-drop trail, oldest first (ADR-0063): every
    /// ask the boundary dropped before any widget's door — the unknown
    /// action, the payloadless write, the off-target selection, the
    /// unknown target — bounded by [`ROUTE_DROP_RING_CAP`].
    #[must_use]
    pub fn route_drops(&self) -> Vec<alacer_core::inspect::RouteDropRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .route_drops
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded key-edit trail, oldest first (ADR-0064): every edit
    /// the typing door landed — the typed run folded to one record, the
    /// IME's committed composition, the unit and word deletes — bounded
    /// by [`KEY_EDIT_RING_CAP`].
    #[must_use]
    pub fn key_edits(&self) -> Vec<alacer_core::inspect::KeyEditRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .key_edits
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The composing-state facts, oldest first (ADR-0067): every fact
    /// the composing door answered — the live session's growing shape,
    /// the cleared state beside it — bounded by [`PREEDIT_RING_CAP`].
    #[must_use]
    pub fn preedits(&self) -> Vec<alacer_core::inspect::PreeditRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .preedits
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The adapter-echo facts, oldest first (ADR-0073): every tree the
    /// activation route answered — the connect's full-tree echo, the
    /// reconnect's beside it — bounded by [`A11Y_ECHO_RING_CAP`].
    #[must_use]
    pub fn a11y_echoes(&self) -> Vec<alacer_core::inspect::A11yEchoRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .a11y_echoes
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded activation asks, oldest first (the funnel's own
    /// facts — the landed press, the wall's refusal, the doorless
    /// silence — bounded by [`ACTION_RING_CAP`]).
    #[must_use]
    pub fn actions(&self) -> Vec<alacer_core::inspect::ActionRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .actions
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded frame refusals, oldest first (ADR-0077: the ladder
    /// stage's species, the failing node beside it when one is the
    /// fact — bounded by [`FRAME_REFUSAL_RING_CAP`]).
    #[must_use]
    pub fn frame_refusals(&self) -> Vec<alacer_core::inspect::FrameRefusalRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .frame_refusals
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded view patches, oldest first (ADR-0078: the
    /// reconciler's four counts beside each other — bounded by
    /// [`VIEW_PATCH_RING_CAP`]).
    #[must_use]
    pub fn view_patches(&self) -> Vec<alacer_core::inspect::ViewPatchRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .view_patches
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded theme swaps, oldest first (ADR-0079: the
    /// palette's tag beside the door — bounded by
    /// [`THEME_RING_CAP`]).
    #[must_use]
    pub fn themes(&self) -> Vec<alacer_core::inspect::ThemeRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .themes
                    .iter()
                    .map(|(_, record)| record.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The recorded setting flips, oldest first (ADR-0080: the
    /// dark-mode fact beside the door — bounded by
    /// [`SETTING_RING_CAP`]).
    #[must_use]
    pub fn settings(&self) -> Vec<alacer_core::inspect::SettingRecord> {
        self.try_lock()
            .map(|recording| {
                recording
                    .settings
                    .iter()
                    .map(|(_, record)| *record)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The most recent frame, when any frame was observed.
    #[must_use]
    pub fn latest_frame(&self) -> Option<alacer_core::inspect::FrameSummary> {
        self.try_lock()
            .and_then(|recording| recording.latest.clone())
    }

    /// The most recent resolved tree — the dump ask's answer (the
    /// observer's tree door retained it). State, never history: a
    /// `clear` empties the rings and leaves the shape standing, and a
    /// poisoned mutex answers the honest absence the other readers
    /// answer (a shape that cannot be read names no shape).
    #[must_use]
    pub fn latest_tree(&self) -> Option<std::sync::Arc<alacer_core::TreeSnapshot>> {
        self.try_lock()
            .and_then(|recording| recording.latest_tree.clone())
    }

    /// Total frames observed (never evicted — the honest count).
    #[must_use]
    pub fn frames_seen(&self) -> u64 {
        self.frames_seen.load(Ordering::Relaxed)
    }

    /// Total facts observed — every [`InspectObserver::observe`] call
    /// counts one, and the count is the number the next fact carries.
    /// Never evicted and never reset: evictions drop ring contents,
    /// `clear` empties them, and the count keeps its monotonicity
    /// through both (the stability a replay cursor needs — the
    /// greeting carries it, M140). The server's greeting names it; a
    /// reconnecting client asks its replay with the number a previous
    /// greeting named.
    #[must_use]
    pub fn facts(&self) -> u64 {
        self.try_lock().map_or(0, |recording| recording.facts)
    }

    /// The occupancy answer (M141, the wire's `stats` control): the
    /// fact count beside one row per ring — the stream's own bound
    /// first, the trails in the event vocabulary's order, `frames`
    /// last — each row naming its occupancy, its cap, and the
    /// evictions the cap caused. The look reads without disturbing
    /// (the replay ask's own rule at the stats tier): no ring drains,
    /// the broadcast queue is untouched. A poisoned mutex answers the
    /// empty row list — the readers' honest absence (the caps are
    /// static knowledge, the counts answer nothing to read).
    #[must_use]
    pub fn stats(&self) -> WireEvent {
        let Some(recording) = self.try_lock() else {
            return WireEvent::Stats {
                facts: 0,
                rings: Vec::new(),
            };
        };
        WireEvent::Stats {
            facts: recording.facts,
            rings: recording.stat_rows(),
        }
    }

    /// Clears every ring (the client's `clear` control message) — every
    /// ring, not the sixteen the first door grew: the a11y-echo, action,
    /// frame-refusal, view-patch, theme, and setting rings the later
    /// tiers added answer the same reset, or a cleared panel would keep
    /// those trails and the door's own sentence would lie.
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
            recording.clipboards.clear();
            recording.values.clear();
            recording.gestures.clear();
            recording.scrolls.clear();
            recording.text_writes.clear();
            recording.route_drops.clear();
            recording.key_edits.clear();
            recording.preedits.clear();
            recording.a11y_echoes.clear();
            recording.actions.clear();
            recording.frame_refusals.clear();
            recording.view_patches.clear();
            recording.themes.clear();
            recording.settings.clear();
            recording.cursors.clear();
        }
    }

    /// The rings' history as one replay burst, oldest first per trail,
    /// tiers in the event vocabulary's order — the durable facts the
    /// broadcast queue never holds for a session that connects after a
    /// first panel's drain took them (the wire door the rings never
    /// had). The look reads without disturbing: no ring drains, the
    /// broadcast queue is untouched, and a session that saw facts live
    /// folds the burst's repeats client-side (the ask's own rule, the
    /// wire's `Replay` control). A poisoned mutex answers an empty
    /// burst — the readers' honest absence, the data that survives.
    #[must_use]
    pub fn replay(&self) -> Vec<WireEvent> {
        self.replay_from(None)
    }

    /// The replay burst above a cursor — only the facts recorded
    /// *after* fact number `after` answer (M140, the wire ask's
    /// optional `after` field): the history at or below the cursor
    /// stays silent, so a panel that already carried the first N facts
    /// asks with N and the rings answer with the delta, not the whole
    /// worth. The cursor a client asks with is the `facts` count a
    /// greeting named; a cursor above every recorded fact answers an
    /// empty burst. Everything else is [`CausalRecorder::replay`]'s
    /// contract, unchanged: oldest first per trail, the vocabulary's
    /// order, the look reading without disturbing.
    #[must_use]
    pub fn replay_after(&self, after: u64) -> Vec<WireEvent> {
        self.replay_from(Some(after))
    }

    /// The replay walk itself, cursor-optional — the one place the
    /// rings convert through their wire shapes. Every tier's walk is
    /// the same shape (walk the ring in order, keep the stamps above
    /// the cursor, convert each survivor through its wire shape), so
    /// the tiers are one generic walker beside one call apiece — the
    /// vocabulary's own order spelled out once, here.
    fn replay_from(&self, after: Option<u64>) -> Vec<WireEvent> {
        let Some(recording) = self.try_lock() else {
            return Vec::new();
        };
        // The cursor's own filter: a fact answers when no cursor rode
        // the ask (the M139 whole-worth shape) or when its stamp sits
        // strictly above the cursor (at-or-below stays silent).
        let above = |seq: u64| after.is_none_or(|cursor| seq > cursor);
        let mut burst = Vec::new();
        walk_ring(&mut burst, &recording.edges, &above, |r| {
            InspectEvent::Edge(r.clone())
        });
        walk_ring(&mut burst, &recording.writes, &above, |r| {
            InspectEvent::Write(r.clone())
        });
        walk_ring(&mut burst, &recording.inputs, &above, |r| {
            InspectEvent::Input(InputRecord {
                kind: r.0,
                target: r.1,
            })
        });
        walk_ring(&mut burst, &recording.focus, &above, |r| {
            InspectEvent::Focus(*r)
        });
        walk_ring(&mut burst, &recording.edits, &above, |r| {
            InspectEvent::Edit(*r)
        });
        walk_ring(&mut burst, &recording.announces, &above, |r| {
            InspectEvent::Announce(*r)
        });
        walk_ring(&mut burst, &recording.keys, &above, |r| {
            InspectEvent::Key(r.clone())
        });
        walk_ring(&mut burst, &recording.clipboards, &above, |r| {
            InspectEvent::Clipboard(*r)
        });
        walk_ring(&mut burst, &recording.values, &above, |r| {
            InspectEvent::Value(*r)
        });
        walk_ring(&mut burst, &recording.gestures, &above, |r| {
            InspectEvent::Gesture(*r)
        });
        walk_ring(&mut burst, &recording.scrolls, &above, |r| {
            InspectEvent::Scroll(*r)
        });
        walk_ring(&mut burst, &recording.text_writes, &above, |r| {
            InspectEvent::TextWrite(*r)
        });
        walk_ring(&mut burst, &recording.route_drops, &above, |r| {
            InspectEvent::RouteDrop(*r)
        });
        walk_ring(&mut burst, &recording.key_edits, &above, |r| {
            InspectEvent::KeyEdit(*r)
        });
        walk_ring(&mut burst, &recording.preedits, &above, |r| {
            InspectEvent::Preedit(*r)
        });
        walk_ring(&mut burst, &recording.a11y_echoes, &above, |r| {
            InspectEvent::A11yEcho(*r)
        });
        walk_ring(&mut burst, &recording.actions, &above, |r| {
            InspectEvent::Action(*r)
        });
        walk_ring(&mut burst, &recording.frame_refusals, &above, |r| {
            InspectEvent::FrameRefusal(*r)
        });
        walk_ring(&mut burst, &recording.view_patches, &above, |r| {
            InspectEvent::ViewPatch(*r)
        });
        walk_ring(&mut burst, &recording.themes, &above, |r| {
            InspectEvent::Theme(r.clone())
        });
        walk_ring(&mut burst, &recording.settings, &above, |r| {
            InspectEvent::Setting(*r)
        });
        walk_ring(&mut burst, &recording.cursors, &above, |r| {
            InspectEvent::Cursor(*r)
        });
        walk_ring(&mut burst, &recording.frames, &above, |r| {
            InspectEvent::Frame(r.summary.clone())
        });
        burst
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
        self.try_lock()
            .map(|recording| recording.app.clone())
            .unwrap_or_default()
    }

    /// The recording behind a graceful lock (the observer half's own
    /// degradation): a poisoned mutex — an observer that panicked
    /// mid-callback — answers `None`, the readers' empty honest
    /// absence, never a panic of their own (the data survives the
    /// poison; the rings drain whole beside it).
    fn try_lock(&self) -> Option<std::sync::MutexGuard<'_, Recording>> {
        self.recording.lock().ok()
    }
}

/// Walks one ring's stamped entries in order, keeping the stamps above
/// the cursor and converting each survivor through its wire shape (the
/// replay walk's one tier shape — the bursts' vocabulary order lives in
/// the call sites, this helper is the shape itself).
fn walk_ring<T>(
    burst: &mut Vec<WireEvent>,
    ring: &Ring<T>,
    above: &impl Fn(u64) -> bool,
    convert: impl Fn(&T) -> InspectEvent,
) {
    for (seq, record) in ring.iter() {
        if above(*seq) {
            burst.push(crate::server::wire_event_of(&convert(record)));
        }
    }
}

/// The `&mut` observer handle installed on the pipeline.
#[derive(Debug, Clone)]
pub struct RecorderHandle {
    recording: Arc<Mutex<Recording>>,
    frames_seen: Arc<AtomicU64>,
    /// The facts the handle could not record — a poisoned mutex (an
    /// observer that panicked mid-callback) is a named loss, never a
    /// silent one: the counter answers what the trail reader's own
    /// `TrailLog::dropped` answers on the read side (the loss named,
    /// the reader pricing the absence beside the data).
    dropped: Arc<AtomicU64>,
}

impl InspectObserver for RecorderHandle {
    fn observe(&mut self, event: InspectEvent) {
        let Ok(mut recording) = self.recording.lock() else {
            // The named loss (the trail reader's own rule — the
            // eviction named, never a silent drop): a poisoned mutex
            // is the one place a fact cannot ride a ring, and the
            // counter is where the loss stays readable.
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return;
        };
        // The fact's number, assigned once under the same lock the
        // rings write under — the stamp the replay ask's cursor (M140)
        // filters on, and the count the greeting carries. A poisoned
        // lock's loss is named on the dropped counter, never counted
        // here: a fact that never landed names no number.
        recording.facts = recording.facts.saturating_add(1);
        let fact_number = recording.facts;
        // The wire serialization happens on the observing thread: the
        // session drains ready-to-send payloads (push, not poll). The
        // guard's fields split once — the queue and its counter are
        // two disjoint borrows the push takes together.
        let wire_event = crate::server::wire_event_of(&event);
        let Recording {
            broadcast,
            broadcast_evicted,
            ..
        } = &mut *recording;
        push_broadcast(broadcast, broadcast_evicted, wire_event);
        match event {
            InspectEvent::Edge(activation) => {
                recording.edges.push(fact_number, activation);
            }
            InspectEvent::Write(write) => {
                recording.writes.push(fact_number, write);
            }
            InspectEvent::Input(input) => {
                recording
                    .inputs
                    .push(fact_number, (input.kind, input.target));
            }
            InspectEvent::Focus(record) => {
                recording.focus.push(fact_number, record);
            }
            InspectEvent::Edit(record) => {
                recording.edits.push(fact_number, record);
            }
            InspectEvent::Announce(record) => {
                recording.announces.push(fact_number, record);
            }
            InspectEvent::Key(record) => {
                recording.keys.push(fact_number, record);
            }
            InspectEvent::Clipboard(record) => {
                recording.clipboards.push(fact_number, record);
            }
            InspectEvent::Value(record) => {
                recording.values.push(fact_number, record);
            }
            InspectEvent::Gesture(record) => {
                recording.gestures.push(fact_number, record);
            }
            InspectEvent::Scroll(record) => {
                recording.scrolls.push(fact_number, record);
            }
            InspectEvent::TextWrite(record) => {
                recording.text_writes.push(fact_number, record);
            }
            InspectEvent::RouteDrop(record) => {
                recording.route_drops.push(fact_number, record);
            }
            InspectEvent::KeyEdit(record) => {
                recording.key_edits.push(fact_number, record);
            }
            InspectEvent::Preedit(record) => {
                recording.preedits.push(fact_number, record);
            }
            InspectEvent::A11yEcho(record) => {
                recording.a11y_echoes.push(fact_number, record);
            }
            InspectEvent::Action(record) => {
                recording.actions.push(fact_number, record);
            }
            InspectEvent::FrameRefusal(record) => {
                recording.frame_refusals.push(fact_number, record);
            }
            InspectEvent::ViewPatch(record) => {
                recording.view_patches.push(fact_number, record);
            }
            InspectEvent::Theme(record) => {
                recording.themes.push(fact_number, record);
            }
            InspectEvent::Setting(record) => {
                recording.settings.push(fact_number, record);
            }
            InspectEvent::Cursor(record) => {
                recording.cursors.push(fact_number, record);
            }
            InspectEvent::Frame(summary) => {
                self.frames_seen.fetch_add(1, Ordering::Relaxed);
                recording.latest = Some(summary.clone());
                recording
                    .frames
                    .push(fact_number, RecordedFrame { summary });
            }
        }
    }

    fn tree(&mut self, snapshot: std::sync::Arc<alacer_core::TreeSnapshot>) {
        // The dump seam's read side: the latest shape retained as state
        // — no fact number, no ring, no broadcast (the events are the
        // history; the tree is the answer to "what stands right now").
        // A poisoned mutex is the named loss the observe path prices on
        // the dropped counter; the door answers nothing to read.
        if let Ok(mut recording) = self.recording.lock() {
            recording.latest_tree = Some(snapshot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CausalRecorder;
    use crate::wire::{RingStat, WireEvent};
    use alacer_core::inspect::{EdgeActivation, InputKind, InspectEvent, InspectObserver};

    /// The poisoned lock names its loss: an observer that panicked
    /// mid-callback poisons the shared mutex, the handle's next fact
    /// cannot ride a ring — and the counter answers, the named loss
    /// the read side prices, never a silent swallow.
    #[test]
    fn the_poisoned_lock_names_its_loss() {
        let recorder = CausalRecorder::new("loss");
        let mut handle = recorder.clone_handle();
        assert_eq!(recorder.dropped(), 0, "a clean lock records no loss");

        // The poison: a panic while the lock is held (the observer
        // that died mid-callback), unwinding through the guard — the
        // joined handle carries the payload, the mutex stays poisoned.
        let recording = std::sync::Arc::clone(&handle.recording);
        let joiner = std::thread::spawn(move || {
            let _held = recording.lock().expect("clean lock before the poison");
            panic!("the observer panicked mid-callback");
        });
        let _poison = joiner.join();

        handle.observe(InspectEvent::Edge(EdgeActivation {
            binding: 1,
            signal: 1,
            node: alacer_core::arena::NodeId::allocate(),
            masks: alacer_core::Change::PAINT,
        }));
        assert_eq!(
            recorder.dropped(),
            1,
            "the poisoned lock's loss is named, not swallowed"
        );
    }

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

    /// A poisoned mutex — an observer that panicked mid-callback —
    /// degrades, never cascades: the readers answer their honest
    /// absence (empty rings, no latest frame, no broadcast), the same
    /// shape `observe` and `take_broadcast` already answered before the
    /// readers joined them (the graceful degradation the panicking
    /// helper pre-registered as debt).
    #[test]
    fn a_poisoned_mutex_degrades_instead_of_panicking() {
        let recorder = CausalRecorder::new("poisoned");
        let mut handle = recorder.clone_handle();
        handle.observe(InspectEvent::Input(alacer_core::inspect::InputRecord {
            kind: InputKind::Key,
            target: None,
        }));
        // Poison the mutex the way a panicking observer would: hold it
        // and unwind inside the thread.
        let poison = std::sync::Arc::clone(&recorder.recording);
        let _ = std::thread::spawn(move || {
            let _guard = poison.lock();
            panic!("the observer panicked mid-callback");
        })
        .join();
        // The data survived the poison; the readers answer empty
        // instead of panicking with it.
        assert!(recorder.frames().is_empty(), "the ring reads empty");
        assert!(recorder.inputs().is_empty(), "the ring reads empty");
        assert!(recorder.latest_frame().is_none(), "no latest to answer");
        assert!(recorder.take_broadcast().is_empty(), "no broadcast");
        assert_eq!(recorder.app(), "", "the greeting answers its absence");
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
                target: None,
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
            target: None,
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
                name: Some("save".to_string()),
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
        // flat shape beside the species — and no name, a miss naming
        // no binding (M37, ADR-0043).
        handle.observe(InspectEvent::Key(alacer_core::inspect::KeyRecord {
            chord: Chord::new(Key::Char('x')),
            outcome: KeymapConsult::FellThrough,
            name: None,
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the miss broadcast");
        let crate::wire::WireEvent::Key {
            key,
            character,
            modifiers,
            outcome,
            name,
        } = last
        else {
            panic!("the key broadcast");
        };
        assert_eq!(key, "char");
        assert_eq!(character.as_deref(), Some("x"));
        assert_eq!(modifiers, "", "no modifiers held — the exact shape");
        assert_eq!(outcome, "fell_through");
        assert_eq!(name, &None, "the miss carries no name on the wire");

        // The chord's modifier shape rides the same flat event — the
        // fired binding's prose name beside it (M37, ADR-0043).
        handle.observe(InspectEvent::Key(alacer_core::inspect::KeyRecord {
            chord: Chord::new(Key::ArrowUp).control(),
            outcome: KeymapConsult::Fired,
            name: Some("increment".to_string()),
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the chord broadcast");
        let crate::wire::WireEvent::Key {
            key,
            character,
            modifiers,
            outcome,
            name,
        } = last
        else {
            panic!("the chord broadcast");
        };
        assert_eq!(key, "arrow_up");
        assert!(character.is_none(), "a non-char key carries no character");
        assert_eq!(modifiers, "ctrl");
        assert_eq!(outcome, "fired");
        assert_eq!(
            name.as_deref(),
            Some("increment"),
            "the fired binding's name rides the flat event"
        );
    }

    #[test]
    fn clipboard_records_ring_in_order_and_are_bounded() {
        use alacer_core::inspect::{ClipboardAction, ClipboardOutcome, ClipboardRefusal};
        let recorder = CausalRecorder::new("clipboards");
        let mut handle = recorder.clone_handle();
        for _ in 0..(super::CLIPBOARD_RING_CAP + 10) {
            handle.observe(InspectEvent::Clipboard(
                alacer_core::inspect::ClipboardRecord {
                    action: ClipboardAction::Copy,
                    outcome: ClipboardOutcome::Delivered,
                    engine: None,
                    read: None,
                    chars: 11,
                },
            ));
        }
        let trail = recorder.clipboards();
        assert_eq!(
            trail.len(),
            super::CLIPBOARD_RING_CAP,
            "the ring is bounded"
        );
        assert_eq!(
            trail[0].action,
            ClipboardAction::Copy,
            "the survivors keep order"
        );

        // The refusal rides the same ring, the wire carrying its species
        // beside the outcome and the honest zero count.
        handle.observe(InspectEvent::Clipboard(
            alacer_core::inspect::ClipboardRecord {
                action: ClipboardAction::Paste,
                outcome: ClipboardOutcome::Refused(ClipboardRefusal::EmptyClipboard),
                engine: None,
                read: None,
                chars: 0,
            },
        ));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the refusal broadcast");
        let crate::wire::WireEvent::Clipboard {
            action,
            outcome,
            refusal,
            engine,
            read,
            chars,
        } = last
        else {
            panic!("the clipboard broadcast");
        };
        assert_eq!(action, "paste");
        assert_eq!(outcome, "refused");
        assert_eq!(refusal.as_deref(), Some("empty_clipboard"));
        assert_eq!(
            engine.as_deref(),
            None,
            "a read asked no write question — the engine fact rides null"
        );
        assert_eq!(
            read.as_deref(),
            None,
            "a paste with no observer-driven refusal rode the plain verdict"
        );
        assert_eq!(*chars, 0, "a refusal moved nothing");
    }

    #[test]
    fn gesture_records_ring_in_order_and_are_bounded() {
        use alacer_core::inspect::{GesturePhase, RangeBound};
        let recorder = CausalRecorder::new("gestures");
        let mut handle = recorder.clone_handle();
        for _ in 0..(super::GESTURE_RING_CAP + 10) {
            handle.observe(InspectEvent::Gesture(alacer_core::inspect::GestureRecord {
                phase: GesturePhase::Started,
                from: 20.0,
                to: None,
                target: None,
                bound: None,
            }));
        }
        let trail = recorder.gestures();
        assert_eq!(trail.len(), super::GESTURE_RING_CAP, "the ring is bounded");
        assert_eq!(
            trail[0].phase,
            GesturePhase::Started,
            "the survivors keep order"
        );

        // The landing rides the same ring, the wire carrying the phase
        // beside the found and landed values — and since M48 (ADR-0054)
        // the wall a landing pinned beside both.
        handle.observe(InspectEvent::Gesture(alacer_core::inspect::GestureRecord {
            phase: GesturePhase::Ended,
            from: 20.0,
            to: Some(42.0),
            target: Some(7),
            bound: Some(RangeBound::Max),
        }));
        let drained = recorder.take_broadcast();
        let last = drained.last().expect("the gesture broadcast");
        let crate::wire::WireEvent::Gesture {
            phase,
            from,
            to,
            target,
            bound,
        } = last
        else {
            panic!("the gesture broadcast");
        };
        assert_eq!(phase, "ended");
        assert!((from - 20.0).abs() < f32::EPSILON);
        assert_eq!(*to, Some(42.0));
        assert_eq!(*target, Some(7));
        assert_eq!(
            bound.as_deref(),
            Some("max"),
            "the pinned wall rides the broadcast beside the landing"
        );
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

    /// The stats answer names the occupancy the rings hold, the caps
    /// they are held to, and the evictions the caps caused (M141): the
    /// frame ring's 256-order cap filled twice over names its two
    /// evictions, the fact count carrying the whole story, and the
    /// rows riding the vocabulary's order with the stream's bound
    /// first.
    #[test]
    fn stats_reads_occupancy_caps_and_evictions() {
        let recorder = CausalRecorder::new("stats");
        let mut handle = recorder.clone_handle();
        for number in 0..258_u64 {
            handle.observe(InspectEvent::Frame(dummy_summary(number)));
        }
        let WireEvent::Stats { facts, rings } = recorder.stats() else {
            panic!("the stats answer");
        };
        assert_eq!(facts, 258, "the fact count is the whole story");
        assert_eq!(
            rings.first().map(|row| row.name),
            Some("broadcast"),
            "the stream's own bound rides first"
        );
        let frames = rings
            .iter()
            .find(|row: &&RingStat| row.name == "frames")
            .expect("the frame ring's row");
        assert_eq!(frames.held, super::FRAME_RING_CAP, "the ring is bounded");
        assert_eq!(frames.cap, super::FRAME_RING_CAP, "the cap is named");
        assert_eq!(frames.evicted, 2, "the two oldest frames evicted");
        let inputs = rings
            .iter()
            .find(|row: &&RingStat| row.name == "inputs")
            .expect("the input ring's row");
        assert_eq!(inputs.held, 0, "the quiet rings answer zero");
        assert_eq!(inputs.evicted, 0, "a quiet ring evicted nothing");
    }

    /// The `clear` reset and the stats answer agree: the trails read
    /// zero holds after a clear while the eviction counters keep their
    /// history (the loss named, never un-named by a reset), and the
    /// fact count keeps its monotonicity — the stability a replay
    /// cursor needs across a clear-reconnect window.
    #[test]
    fn a_cleared_recording_answers_zero_holds_and_keeps_its_count() {
        let recorder = CausalRecorder::new("stats-clear");
        let mut handle = recorder.clone_handle();
        handle.observe(InspectEvent::Input(alacer_core::inspect::InputRecord {
            kind: InputKind::Key,
            target: None,
        }));
        recorder.clear();
        let WireEvent::Stats { facts, rings } = recorder.stats() else {
            panic!("the stats answer");
        };
        assert_eq!(facts, 1, "the count never moves backward");
        let inputs = rings
            .iter()
            .find(|row: &&RingStat| row.name == "inputs")
            .expect("the input ring's row");
        assert_eq!(inputs.held, 0, "the cleared trail answers zero");
        assert_eq!(inputs.evicted, 0, "a clear evicts nothing — it empties");
    }

    /// A minimal frame summary for ring tests.
    fn dummy_summary(number: u64) -> alacer_core::inspect::FrameSummary {
        alacer_core::inspect::FrameSummary {
            number,
            layout_ran: false,
            paint_ran: false,
            a11y_ran: false,
            focus_changed: false,
            degradation_step: alacer_core::passes::DegradationStep::None,
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

    // ------------------------------------------------------------------
    // The tree dump's read side: the observer's tree door retains the
    // latest resolved snapshot — the whole shape the dump ask answers —
    // the `latest_frame` precedent grown a second retained answer. The
    // tree is *state*, never history: a `clear` empties the rings and
    // leaves the tree standing (the panel asking again reads the shape
    // the last frame resolved), and a poisoned lock answers the honest
    // absence the other readers answer.
    // ------------------------------------------------------------------

    /// Builds a minimal snapshot: one root, the named child count.
    fn dump_snapshot(root_id: u64) -> std::sync::Arc<alacer_core::TreeSnapshot> {
        std::sync::Arc::new(alacer_core::TreeSnapshot {
            nodes: vec![alacer_core::TreeNode {
                id: root_id,
                role: alacer_core::Role::Container,
                label: "root".to_string(),
                bounds: None,
                children: Vec::new(),
                value: None,
                toggled: None,
                table_slot: None,
                selection: None,
                live: None,
                enabled: true,
                description: None,
                placeholder: None,
                value_range: None,
                numeric_value: None,
                secure_entry: false,
                scroll_orientation: None,
                scroll_position: None,
                secure_revealed: false,
                read_only: false,
                required: false,
                invalid: None,
                selected: None,
                multiselectable: false,
                busy: false,
                expanded: None,
                auto_complete: None,
                sort_direction: None,
                level: None,
                current: None,
                has_popup: None,
                role_description: None,
                position_in_set: None,
                size_of_set: None,
                keyboard_shortcut: None,
                multiline: false,
                language: None,
                search_field: false,
            }],
            root: Some(root_id),
        })
    }

    #[test]
    fn the_latest_tree_answers_the_dump_reader() {
        let recorder = CausalRecorder::new("dump");
        let mut handle = recorder.clone_handle();
        assert!(
            recorder.latest_tree().is_none(),
            "no tree door yet — the honest absence"
        );
        handle.tree(dump_snapshot(1));
        let first = recorder.latest_tree().expect("the first tree");
        assert_eq!(first.root, Some(1), "the retained shape is the shape given");
        handle.tree(dump_snapshot(2));
        let second = recorder.latest_tree().expect("the latest tree");
        assert_eq!(
            second.root,
            Some(2),
            "the dump answers the latest resolution, never the first"
        );
    }

    #[test]
    fn the_tree_survives_clear() {
        let recorder = CausalRecorder::new("dump-clear");
        let mut handle = recorder.clone_handle();
        handle.tree(dump_snapshot(5));
        recorder.clear();
        let tree = recorder.latest_tree().expect("the tree stands");
        assert_eq!(
            tree.root,
            Some(5),
            "the rings are history, the tree is state — clear empties one, not the other"
        );
    }

    #[test]
    fn the_poisoned_lock_answers_no_tree() {
        let recorder = CausalRecorder::new("dump-poison");
        let mut handle = recorder.clone_handle();
        handle.tree(dump_snapshot(3));
        let recording = std::sync::Arc::clone(&handle.recording);
        let joiner = std::thread::spawn(move || {
            let _guard = recording.lock().expect("the first hold");
            panic!("the observer that died mid-callback");
        });
        let _ = joiner.join();
        assert!(
            recorder.latest_tree().is_none(),
            "the poisoned mutex names the absence, never a stale shape"
        );
    }
}
