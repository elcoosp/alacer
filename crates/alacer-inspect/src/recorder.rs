//! The causal recorder (§13.3, ADR-0010): the
//! [`alacer_core::InspectObserver`] implementation the pipeline forwards
//! into — bounded ring buffers of frames, edges, writes, and inputs (every
//! buffer has an eviction policy and a test that proves it, §14.3), plus
//! the wire-shaped broadcast queue the server's sessions drain.

use crate::server::push_broadcast;
use crate::wire::WireEvent;
use alacer_core::inspect::{EdgeActivation, InputKind, InspectEvent, InspectObserver, SignalWrite};
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
        if let Some(wire_event) = crate::server::wire_event_of(&event) {
            push_broadcast(&mut recording.broadcast, wire_event);
        }
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
