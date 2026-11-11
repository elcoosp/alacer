//! The wire format: the documented WS protocol's JSON vocabulary
//! (`docs/inspector-protocol.md`). These DTOs are the inspector's product —
//! the stable, versioned surface devtools clients code against; the core
//! vocabulary (`alacer_core::inspect`) never leaks onto the wire.

use alacer_core::history::EditAction;
use alacer_core::inspect::{
    EdgeActivation, EditRecord, FocusRecord, FrameSummary, InputKind, SignalWrite,
};
use alacer_core::passes::FocusRefusal;
use serde::{Deserialize, Serialize};

/// The protocol version this server speaks. Clients gate on it.
pub const PROTOCOL_VERSION: u32 = 1;

/// The server's greeting, sent on every accepted connection.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Hello {
    /// Always `"hello"`.
    pub kind: &'static str,
    /// The protocol version (1).
    pub version: u32,
    /// The serving app's name (the facade sets it).
    pub app: String,
}

/// The client's control messages (all optional; the server streams
/// regardless).
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Clear the server-side history (the client keeps its own buffer).
    Clear,
    /// The client is going away; the server closes the socket.
    Bye,
}

/// One protocol event. The JSON tag vocabulary is `snake_case`; the shape
/// mirrors the core `InspectEvent` with identities as plain
/// integers.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WireEvent {
    /// A reactive edge activated — "why did this render".
    Edge {
        /// The binding (signal→node edge) that activated.
        binding: u64,
        /// The signal whose write triggered the rerun.
        signal: u64,
        /// The widget node the masks landed on.
        node: u64,
        /// The masks queued, as bit names ("layout+paint").
        masks: String,
    },
    /// A provenance-tagged signal write — "who wrote this signal".
    Write {
        /// The written signal.
        signal: u64,
        /// The write's sequence number.
        sequence: u64,
        /// The declared origin.
        origin: String,
    },
    /// A routed input event.
    Input {
        /// The input kind (`pointer_down`, `key`, …).
        input: String,
        /// The target widget node, when routed.
        target: Option<u64>,
    },
    /// A focus-move door answered — the walk's landing or its named
    /// refusal ("why did focus change — or why didn't it", ADR-0034).
    /// Since M30 the event names its rider too (ADR-0036): the
    /// keyboard's Tab fallback, the ring's arrows, or the app's own
    /// doors — the additive field the protocol's versioning rule admits
    /// at version 1.
    Focus {
        /// The movement the caller asked for (`next`, `prev`, `left`,
        /// `right`, `up`, `down`).
        movement: String,
        /// Who rode the walk (M30, ADR-0036): `keyboard` (the Tab
        /// fallback), `ring` (the directional keys), or `consumer`
        /// (the app's own door calls).
        rider: String,
        /// The subtree root that bounded the walk (the scoped door's
        /// trap), `null` for the unscoped funnel.
        scope: Option<u64>,
        /// Whether the walk landed (`true`) or refused (`false`).
        moved: bool,
        /// The node that held focus before the move, `null` for the
        /// entry.
        from: Option<u64>,
        /// The node the walk rests on now; `null` on a refusal.
        to: Option<u64>,
        /// The tree-order move crossed a list end (directional moves
        /// never wrap).
        wrapped: bool,
        /// The refusal species (`no_candidates`, `dead_scope`,
        /// `no_geometry`, `nothing_in_direction`, `already_focused`),
        /// `null` on a landing.
        refusal: Option<String>,
        /// The refusal's node (the dead scope, the geometry-less focus,
        /// the searched-from node at the edge, the stall's node); `null`
        /// on a landing.
        node: Option<u64>,
        /// The empty direction at the edge (`nothing_in_direction`
        /// only), else `null`.
        direction: Option<String>,
    },
    /// A journal action answered — the edit trail (ADR-0037): the
    /// applied step, the refused chord, or the app's reset, the depths
    /// beside it. Additive at version 1 (the rider field's rule):
    /// clients that ignore unknown kinds are unaffected.
    Edit {
        /// The action the journal answered (`record`, `undo`, `redo`,
        /// `reset`).
        action: String,
        /// Whether a `record` folded into the journal's tip (the
        /// pure-append coalescing rule); `false` for every other action.
        coalesced: bool,
        /// Whether the chord was refused (an exhausted undo or redo
        /// branch — the depths name the journal it did not move).
        refused: bool,
        /// The undo branch's depth when the fact streamed.
        undo_depth: usize,
        /// The redo branch's depth when the fact streamed.
        redo_depth: usize,
    },
    /// A completed frame — the ledger + the mask heatmap.
    Frame {
        /// Monotonic frame number.
        number: u64,
        /// Whether the layout pass ran.
        layout_ran: bool,
        /// Whether the paint pass ran.
        paint_ran: bool,
        /// Whether the a11y pass ran.
        a11y_ran: bool,
        /// Whether focus moved.
        focus_changed: bool,
        /// The damage verdict ("none", "partial", "full").
        damage: String,
        /// The damage region count.
        regions: usize,
        /// A11y delta sizes.
        a11y_updated: usize,
        /// A11y delta removals.
        a11y_removed: usize,
        /// Total tick time in nanoseconds (0 when timings are off).
        total_nanos: u64,
        /// The pass ledger: `(pass, ran, nanos)` triples.
        passes: Vec<WirePass>,
        /// The mask heatmap entries (bounded; see `dirty_truncated`).
        dirty: Vec<WireDirtyNode>,
        /// Dirty nodes dropped from the heatmap.
        dirty_truncated: usize,
    },
}

/// One pass ledger entry on the wire.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct WirePass {
    /// The pass name ("layout", "paint", …).
    pub name: &'static str,
    /// Whether it ran.
    pub ran: bool,
    /// Wall time in nanoseconds.
    pub nanos: u64,
}

/// One mask-heatmap entry on the wire.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WireDirtyNode {
    /// The node id.
    pub node: u64,
    /// The masks, as bit names ("layout+paint").
    pub masks: String,
    /// The widget's role.
    pub role: &'static str,
    /// The widget's resolved label.
    pub label: String,
    /// The bounds, `[x, y, w, h]`, when laid out.
    pub bounds: Option<[f32; 4]>,
}

/// The damage verdict's wire name.
#[must_use]
pub fn damage_name(damage: &alacer_core::Damage) -> &'static str {
    if damage.is_full() {
        "full"
    } else if damage.is_empty() {
        "none"
    } else {
        "partial"
    }
}

/// The `Change` mask's wire name: bit names joined with `+`, lowest bit
/// first, stable for clients to parse.
#[must_use]
pub fn masks_name(mask: alacer_core::Change) -> String {
    let mut names: Vec<&'static str> = Vec::new();
    if mask.contains(alacer_core::Change::STRUCTURE) {
        names.push("structure");
    }
    if mask.contains(alacer_core::Change::LAYOUT) {
        names.push("layout");
    }
    if mask.contains(alacer_core::Change::PAINT) {
        names.push("paint");
    }
    if mask.contains(alacer_core::Change::A11Y) {
        names.push("a11y");
    }
    if mask.contains(alacer_core::Change::FOCUS) {
        names.push("focus");
    }
    names.join("+")
}

/// The input kind's wire name.
#[must_use]
pub const fn input_name(kind: InputKind) -> &'static str {
    match kind {
        InputKind::PointerDown => "pointer_down",
        InputKind::PointerUp => "pointer_up",
        InputKind::PointerMove => "pointer_move",
        InputKind::PointerLeave => "pointer_leave",
        InputKind::Wheel => "wheel",
        InputKind::Key => "key",
        InputKind::Ime => "ime",
    }
}

/// The role's wire name (the CI walker's diagnostics vocabulary).
#[must_use]
pub const fn role_name(role: alacer_core::Role) -> &'static str {
    role.name()
}

/// Converts a core [`FrameSummary`] into the wire frame.
#[must_use]
pub fn wire_frame(summary: &FrameSummary) -> WireEvent {
    let regions = summary.damage.regions(summary.viewport).len();
    WireEvent::Frame {
        number: summary.number,
        layout_ran: summary.layout_ran,
        paint_ran: summary.paint_ran,
        a11y_ran: summary.a11y_ran,
        focus_changed: summary.focus_changed,
        damage: damage_name(&summary.damage).to_string(),
        regions,
        a11y_updated: summary.a11y_updated,
        a11y_removed: summary.a11y_removed,
        total_nanos: summary.total_nanos,
        passes: summary
            .passes
            .iter()
            .map(|pass| WirePass {
                name: pass.kind.name(),
                ran: pass.ran,
                nanos: pass.nanos,
            })
            .collect(),
        dirty: summary
            .dirty
            .iter()
            .map(|node| WireDirtyNode {
                node: node.node.raw(),
                masks: masks_name(node.mask),
                role: role_name(node.role),
                label: node.label.clone(),
                bounds: node
                    .bounds
                    .map(|b| [b.origin.x, b.origin.y, b.size.width, b.size.height]),
            })
            .collect(),
        dirty_truncated: summary.dirty_truncated,
    }
}

/// Converts a core [`EdgeActivation`] into the wire event.
#[must_use]
pub fn wire_edge(activation: &EdgeActivation) -> WireEvent {
    WireEvent::Edge {
        binding: activation.binding,
        signal: activation.signal,
        node: activation.node.raw(),
        masks: masks_name(activation.masks),
    }
}

/// Converts a core [`SignalWrite`] into the wire event.
#[must_use]
pub fn wire_write(write: &SignalWrite) -> WireEvent {
    WireEvent::Write {
        signal: write.signal,
        sequence: write.sequence,
        origin: write.origin.to_string(),
    }
}

/// The refusal species' wire name (`snake_case`, the protocol's tag
/// vocabulary).
#[must_use]
pub fn refusal_name(refusal: &FocusRefusal) -> &'static str {
    match refusal {
        FocusRefusal::NoCandidates => "no_candidates",
        FocusRefusal::DeadScope { .. } => "dead_scope",
        FocusRefusal::NoGeometry { .. } => "no_geometry",
        FocusRefusal::NothingInDirection { .. } => "nothing_in_direction",
        FocusRefusal::AlreadyFocused { .. } => "already_focused",
    }
}

/// The rider's wire name
/// ([`alacer_core::inspect::FocusRider::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn rider_name(rider: alacer_core::inspect::FocusRider) -> &'static str {
    rider.name()
}

/// The edit action's wire name
/// ([`alacer_core::history::EditAction::name`], the protocol's
/// `snake_case` tag vocabulary).
#[must_use]
pub const fn edit_action_name(action: EditAction) -> &'static str {
    action.name()
}

/// Converts a core [`EditRecord`] into the wire event: the journal's own
/// fact, flat on the wire (ADR-0037).
#[must_use]
pub fn wire_edit(record: &EditRecord) -> WireEvent {
    WireEvent::Edit {
        action: edit_action_name(record.action).to_string(),
        coalesced: record.coalesced,
        refused: record.refused,
        undo_depth: record.undo_depth,
        redo_depth: record.redo_depth,
    }
}

/// Converts a core [`FocusRecord`] into the wire event: the outcome's
/// landing or refusal resolved into the documented flat shape, the rider
/// named beside the movement (M30, ADR-0036).
#[must_use]
pub fn wire_focus(record: &FocusRecord) -> WireEvent {
    let (moved, from, to, wrapped, refusal, node, direction) = match record.outcome {
        alacer_core::passes::FocusMoveOutcome::Moved(landing) => (
            true,
            landing.from.map(alacer_core::NodeId::raw),
            Some(landing.to.raw()),
            landing.wrapped,
            None,
            None,
            None,
        ),
        alacer_core::passes::FocusMoveOutcome::Refused(refusal) => {
            let node = match refusal {
                FocusRefusal::NoCandidates => None,
                FocusRefusal::DeadScope { scope } => Some(scope.raw()),
                FocusRefusal::NoGeometry { node }
                | FocusRefusal::NothingInDirection { node, .. }
                | FocusRefusal::AlreadyFocused { node } => Some(node.raw()),
            };
            let direction = match refusal {
                FocusRefusal::NothingInDirection { direction, .. } => {
                    Some(direction.name().to_string())
                }
                _ => None,
            };
            (
                false,
                None,
                None,
                false,
                Some(refusal_name(&refusal).to_string()),
                node,
                direction,
            )
        }
    };
    WireEvent::Focus {
        movement: record.movement.name().to_string(),
        rider: rider_name(record.rider).to_string(),
        scope: record.scope.map(alacer_core::NodeId::raw),
        moved,
        from,
        to,
        wrapped,
        refusal,
        node,
        direction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_serializes_with_the_kind_tag() {
        let hello = Hello {
            kind: "hello",
            version: PROTOCOL_VERSION,
            app: "demo".to_string(),
        };
        let json = serde_json::to_string(&hello).expect("hello json");
        assert_eq!(
            json, r#"{"kind":"hello","version":1,"app":"demo"}"#,
            "the documented greeting shape"
        );
    }

    #[test]
    fn client_messages_deserialize_by_tag() {
        let clear: ClientMessage = serde_json::from_str(r#"{"kind":"clear"}"#).expect("clear");
        assert_eq!(clear, ClientMessage::Clear);
        let bye: ClientMessage = serde_json::from_str(r#"{"kind":"bye"}"#).expect("bye");
        assert_eq!(bye, ClientMessage::Bye);
    }

    #[test]
    fn edge_events_carry_the_mask_vocabulary() {
        let event = wire_edge(&EdgeActivation {
            binding: 1,
            signal: 2,
            node: alacer_core::arena::NodeId::allocate(),
            masks: alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        });
        let WireEvent::Edge { masks, .. } = event else {
            panic!("edge event");
        };
        assert_eq!(masks, "layout+paint");
    }

    #[test]
    fn edit_events_carry_the_journal_vocabulary() {
        let event = wire_edit(&EditRecord {
            action: EditAction::Record,
            coalesced: true,
            refused: false,
            undo_depth: 1,
            redo_depth: 0,
        });
        let json = serde_json::to_string(&event).expect("edit json");
        assert_eq!(
            json,
            r#"{"kind":"edit","action":"record","coalesced":true,"refused":false,"undo_depth":1,"redo_depth":0}"#,
            "the documented edit shape (the focus event's flat rule)"
        );
        let WireEvent::Edit {
            action,
            coalesced,
            refused,
            undo_depth,
            redo_depth,
        } = event
        else {
            panic!("edit event");
        };
        assert_eq!(action, "record");
        assert!(coalesced);
        assert!(!refused);
        assert_eq!((undo_depth, redo_depth), (1, 0));
    }

    #[test]
    fn a_refused_chord_carries_the_refusal_flag() {
        let event = wire_edit(&EditRecord {
            action: EditAction::Undo,
            coalesced: false,
            refused: true,
            undo_depth: 0,
            redo_depth: 0,
        });
        let WireEvent::Edit {
            action, refused, ..
        } = event
        else {
            panic!("edit event");
        };
        assert_eq!(action, "undo", "the refusal names its action");
        assert!(refused, "the refusal is the fact the wire carries");
    }

    #[test]
    fn frames_carry_the_damage_verdict_and_regions() {
        use alacer_core::geom::Size;
        use alacer_core::passes::PassKind;
        let summary = FrameSummary {
            number: 7,
            layout_ran: true,
            paint_ran: true,
            a11y_ran: false,
            focus_changed: false,
            damage: alacer_core::Damage::FULL,
            viewport: Size::new(100.0, 80.0),
            a11y_updated: 0,
            a11y_removed: 0,
            total_nanos: 1_000,
            passes: vec![alacer_core::inspect::PassTiming {
                kind: PassKind::Layout,
                ran: true,
                nanos: 400,
            }],
            dirty: Vec::new(),
            dirty_truncated: 0,
        };
        let WireEvent::Frame {
            damage, regions, ..
        } = wire_frame(&summary)
        else {
            panic!("frame event");
        };
        assert_eq!(damage, "full");
        assert_eq!(regions, 1, "full damage is the whole viewport");
    }
}

#[cfg(test)]
mod client_tests {
    /// M30 (ADR-0036, this cut): the embedded standalone client renders
    /// the trail's rider beside the movement — the field the wire
    /// carries is a field the client shows, the attribution a reader of
    /// the log pane answers "who moved focus" with. The client is a
    /// single embedded file; the pin is the file's own fragments, the
    /// same surface route the exit suites ride.
    #[test]
    fn the_client_names_the_rider_beside_the_movement() {
        let page = include_str!("inspector.html");
        for fragment in ["event.rider", "The client names the rider (M30, ADR-0036)"] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the rider: {fragment}"
            );
        }
    }

    /// M31 (ADR-0037, this cut): the embedded standalone client renders
    /// the journal's echo — the edit event the wire now carries lands in
    /// the log pane beside the focus trail, the action and the depths
    /// named, the refusal rendered the way the walk's refusals are. The
    /// pin rides the file's own fragments (the M30 route).
    #[test]
    fn the_client_renders_the_journals_echo() {
        let page = include_str!("inspector.html");
        for fragment in [
            "case \"edit\":",
            "The client renders the journal's echo (M31, ADR-0037)",
        ] {
            assert!(
                page.contains(fragment),
                "the embedded client renders the edit event: {fragment}"
            );
        }
    }
}
