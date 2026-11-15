//! The activation ask answers on the causal log (§13.3, ADR-0075):
//! the most common ask in the action tier — the AT's press, the default
//! accessibility action the platform's action route aims at a control —
//! answered the dispatch funnel's bare `bool`, whose `false` collapsed
//! the two facts a trail reader wants apart: the availability tier's
//! wall and the doorless target's silence. Since M69 the ask streams
//! one `InspectEvent::Action` the tick after it resolves — the landing,
//! or the refusal species beside the aim, the species computed where
//! both facts live (the route holds the ask, the aimed id, the door's
//! answer, and the tree's one `enabled` truth, M22).
//!
//! The demo drives the real action tier
//! (`PassPipeline::dispatch_a11y_action`, the exact route the platform's
//! AccessKit adapter rides — never a second channel) against a button,
//! a button the availability tier holds down, and a heading with no
//! activation door at all: the beats press each in turn, and the *trail*
//! label mirrors what the log streamed. One fact per trail (the M39
//! rule at the activation tier): the value tier's step beside the beats
//! streams no action fact — the value trail's own record prices it.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example action_trail
//! ```
//!
//! Run with `cargo run -p alacer --example action_trail`. The trail
//! label mirrors what the log streamed — the press's row reads "the
//! AT's press landed (control N)", the wall's "refused: unavailable",
//! the heading's "refused: `no_door`". `ALACER_SOFTWARE=1` forces the
//! softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Slider};
use alacer_core::flux::RenderBinding;
use alacer_core::geom::Size;
use alacer_core::inspect::{ActionOutcome, InspectEvent, InspectObserver};
use alacer_core::passes::PassPipeline;
use alacer_core::widget::{A11yAction, Role};
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport the demo's beats settle into (the trail demos' shape).
const VIEWPORT: Size = Size::new(480.0, 340.0);

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// The fact as the trail's reader renders it: the ask beside the
/// outcome, the species beside a refusal, the control's id beside it
/// all — the log answering "did the press land, and where" from the row
/// alone. The record is `Copy`, so it passes by value (the pedantic
/// group's rule).
fn describe(record: alacer_core::ActionRecord) -> String {
    let ask = record.ask.name();
    let outcome = match record.outcome {
        ActionOutcome::Delivered => "the AT's press landed".to_string(),
        ActionOutcome::Refused(refusal) => format!("refused: {}", refusal.name()),
    };
    let who = record.target.map_or_else(
        || "control (unrouted)".to_string(),
        |id| format!("control {id}"),
    );
    format!("{ask}: {outcome} — {who}")
}

/// The causal trail the demo shares between the pipeline's observer and
/// the *trail* label's signal: the observer half writes the fact into a
/// bounded ring (the recorder's own discipline, §14.3) and mirrors it
/// into the signal — one write batch, the label re-renders the same
/// frame the drain rode (axiom A1: signals in, never widgets).
struct Trail {
    actions: Rc<RefCell<std::collections::VecDeque<alacer_core::ActionRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl Trail {
    /// The read half: every streamed fact so far, oldest first — the
    /// demo's log and its closing read-back.
    fn all(&self) -> Vec<alacer_core::ActionRecord> {
        self.actions.borrow().iter().copied().collect()
    }
}

/// The observer the pipeline owns: the write half of [`Trail`].
struct TrailWriter {
    actions: Rc<RefCell<std::collections::VecDeque<alacer_core::ActionRecord>>>,
    trail_line: alacer_core::FluxRwSignal<String>,
}

impl InspectObserver for TrailWriter {
    fn observe(&mut self, event: InspectEvent) {
        let InspectEvent::Action(record) = event else {
            return;
        };
        let line = describe(record);
        {
            let mut ring = self.actions.borrow_mut();
            if ring.len() >= 64 {
                ring.pop_front();
            }
            ring.push_back(record);
        }
        self.trail_line.set(line);
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(VIEWPORT) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// One beat: the span names the route, the real action tier answers,
/// the fact streams to the observer, and the response is logged — the
/// funnel's own route, no second channel. The aimed id rides the record
/// (the funnel's stamp, the M43 rule at the activation tier).
fn beat(pipeline: &mut PassPipeline, label: &'static str, target: (Role, &str)) {
    let _route =
        tracing::trace_span!(target: "alacer::action_trail", "beat", label = label).entered();
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let node = tree.find(target.0, target.1).unwrap_or_else(|| {
        panic!(
            "the AT can reach the {} named {}",
            target.0.name(),
            target.1
        )
    });
    let id = node.id;
    let consumed = pipeline.dispatch_a11y_action(id, A11yAction::DefaultPress);
    settle(pipeline);
    tracing::info!(
        target: "alacer::action_trail",
        consumed = consumed,
        routed_id = id,
        "{label}"
    );
}

/// The beats the demo walks before the window opens: the press that
/// lands, the press the availability wall holds, the press the heading
/// never had a door for — each a real AT activation through the real
/// dispatch against the control the tree names. The value tier's step
/// rides between them streaming no action fact: one fact per trail.
/// Since M70 the keyboard's Space activation rides beside them — the
/// rider named on the same log.
fn run_beats(
    pipeline: &mut PassPipeline,
    value_signal: alacer_core::FluxRwSignal<String>,
    button_node: alacer_core::NodeId,
) {
    // Beat one: the enabled button — the press lands, the handler
    // fires, the fact names the control it rode.
    beat(
        pipeline,
        "beat 1: press the enabled button — the landing names its control",
        (Role::Button, "Increment"),
    );

    // Beat two: the slider's own tier — the step the AT asks prices
    // itself on the value trail (ADR-0046), the action trail streaming
    // nothing for it (one fact per trail, never a double-priced ask).
    let tree = pipeline
        .a11y_snapshot()
        .expect("the a11y snapshot after the settle");
    let slider = tree
        .find(Role::Slider, "Volume")
        .expect("the AT can reach the slider");
    let consumed = pipeline.dispatch_a11y_action(slider.id, A11yAction::Increment);
    settle(pipeline);
    tracing::info!(
        target: "alacer::action_trail",
        consumed = consumed,
        "beat 2: Increment on the slider — the value trail's fact, no action fact beside it"
    );
    value_signal.set("the value: 51 — the slider's step landed".to_string());

    // Beat three: the held-down button — the availability tier's wall
    // (M22, ADR-0028) names itself at last: the refusal's species
    // beside the aim, the collapsed `false` split.
    beat(
        pipeline,
        "beat 3: press the disabled button — the wall names itself",
        (Role::Button, "Send"),
    );

    // Beat four: the heading — no activation door grew there, and the
    // silence is a named fact beside the aim.
    beat(
        pipeline,
        "beat 4: press the heading — the doorless silence names itself",
        (Role::Label, "The activation trail"),
    );

    // Beat five: the keyboard's activation (M70, ADR-0076) — the
    // focused button answering Space like a click, the GTK rule
    // landing the activation at the release. The pressed phase arms
    // nothing on the log (the flood rule); the release streams one
    // fact with `Keyboard` beside it, the key route's stamp naming the
    // focused control (M45). One log, two riders, tellable apart.
    pipeline.request_focus(button_node);
    settle(pipeline);
    let pressed = pipeline.dispatch_key(&alacer_core::events::KeyEvent {
        key: alacer_core::events::Key::Space,
        state: alacer_core::events::KeyState::Pressed,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::action_trail",
        handled = matches!(pressed, alacer_core::widget::EventResponse::Handled),
        "beat 5: Space press on the focused button — the pressed phase arms nothing on the log"
    );
    let released = pipeline.dispatch_key(&alacer_core::events::KeyEvent {
        key: alacer_core::events::Key::Space,
        state: alacer_core::events::KeyState::Released,
        modifiers: alacer_core::events::Modifiers::default(),
        text: None,
    });
    settle(pipeline);
    tracing::info!(
        target: "alacer::action_trail",
        handled = matches!(released, alacer_core::widget::EventResponse::Handled),
        "beat 5: Space release — the keyboard's activation names its rider"
    );
    value_signal.set("the value: the app's own — the keyboard's click fired".to_string());
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(root, Label::new("The activation trail").boxed());
    let button_node = pipeline.add_child(root, Button::new("Increment").on_click(|| {}).boxed());
    let _ = pipeline.add_child(
        root,
        Button::new("Send").enabled(false).on_click(|| {}).boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Slider::new("Volume", 0.0, 100.0)
            .value(50.0)
            .step(1.0)
            .boxed(),
    );

    let value_signal = pipeline
        .flux_mut()
        .signal("the value: (the tree carries what the app wrote)".to_string());
    let trail_signal = pipeline
        .flux_mut()
        .signal("the trail: (nothing streamed yet)".to_string());
    let value_binding =
        RenderBinding::new("the value: (the tree carries what the app wrote)".to_string());
    let value_label = pipeline.add_child(root, Label::bound(value_binding.clone()).boxed());
    let trail_binding = RenderBinding::new("the trail: (nothing streamed yet)".to_string());
    let trail_label = pipeline.add_child(root, Label::bound(trail_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        value_signal,
        value_label,
        &value_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );
    pipeline.flux_mut().bind_value(
        trail_signal,
        trail_label,
        &trail_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // The observer rides from here on: every activation ask's record
    // streams into the trail label the frame after the ask resolves.
    let trail = Trail {
        actions: Rc::new(RefCell::new(std::collections::VecDeque::new())),
        trail_line: trail_signal,
    };
    pipeline.set_observer(Box::new(TrailWriter {
        actions: Rc::clone(&trail.actions),
        trail_line: trail.trail_line,
    }));

    settle(&mut pipeline);
    run_beats(&mut pipeline, value_signal, button_node);

    // The window: live play — an AT driving the button streams the same
    // records the beats did; the keyboard's Space rides the widget's
    // own activation door beside it (the M10 parity), its own trail
    // story growing with the tier's rider rule.
    let _ = pipeline.add_child(
        root,
        Label::new("Live: press the button with an AT — the trail mirrors the log").boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer action trail",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window closed: the trail, read back — every fact the session
    // streamed, oldest first, the activation tier's history as data.
    // The landings streamed beside the walls; the controls named
    // themselves.
    let read_back = trail.all();
    tracing::info!(
        target: "alacer::action_trail",
        actions = read_back.len(),
        outcomes = ?read_back.iter().map(|record| record.outcome.name()).collect::<Vec<_>>(),
        "the activation asks streamed as they resolved — the causal log carries the action trail, the species beside the aim"
    );
}
