//! The M18 announcement seam driven from the consumer side (§11.6,
//! ADR-0024): the app's voice — state changes that own no widget — ships
//! as the reserved live-region node in the a11y snapshot, the same delta
//! the widget tree rides. The buttons are the speech acts: *Save it*
//! announces politely, *Delete it* interrupts assertively, *Speak
//! nothing* refuses honestly (an empty announcement speaks nothing, no
//! masks, an idle frame). The status line mirrors what the region carries,
//! read back from the app's own state — the same mirror the input storm
//! and the journal demo use.
//!
//! The routes are the handler-reachable door: a click handler cannot hold
//! the pipeline, so `alacer_core::announce::announce` queues the speech
//! act (the clipboard-seam shape, ADR-0020; the input queue's shape,
//! ADR-0014) and the next tick drains it into the region — one frame, the
//! same currency as the handler's signal writes. The observability rides
//! the workspace's idiom: `tracing` spans at the `alacer::announce`
//! target, the politeness as a field.
//!
//! With a screen reader attached the region's changes are spoken (the
//! AccessKit update carries `aria-live` and the ARIA `status`/`alert`
//! pairing); without one, the status line is the demonstration — the
//! region's delta still rides the sink every frame.
//!
//! Run with `cargo run -p alacer --example announcements`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path.

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Row};
use alacer_core::announce::Politeness;
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::Set as _;
use std::cell::RefCell;
use std::rc::Rc;

/// The app's voice: the last speech act and how it answered — exactly what
/// a custom app owns outside the pipeline (the m15 `Journal` shape).
struct Announcer {
    last: Option<(String, Politeness)>,
    refused: bool,
}

impl Announcer {
    /// The status line: what the region carries right now, or the honest
    /// refusal — the reads live in the app's state, the window mirrors them.
    fn status(&self) -> String {
        if self.refused {
            return "refused: an empty announcement speaks nothing".to_string();
        }
        match &self.last {
            Some((text, politeness)) => {
                format!("announced {text:?} · {polite}", polite = politeness.name())
            }
            None => "the region is silent".to_string(),
        }
    }
}

/// One speech route: the ambient queue accepts (or refuses), the app's
/// state records what it answered, the batch mirrors it into the window —
/// handlers write signals, nothing mutates widgets (axiom A1). The tick
/// drains the queue into the reserved region one frame later.
fn speak(
    announcer: &Rc<RefCell<Announcer>>,
    route: &'static str,
    text: &str,
    politeness: Politeness,
    status_line: &alacer_core::FluxRwSignal<String>,
) {
    let _route =
        tracing::trace_span!(target: "alacer::announce", "announce", route = route).entered();
    let accepted = alacer_core::announce::announce(text, politeness);
    {
        let mut app = announcer.borrow_mut();
        if accepted {
            app.last = Some((text.to_string(), politeness));
            app.refused = false;
        } else {
            app.refused = true;
        }
    }
    tracing::trace!(
        target: "alacer::announce",
        politeness = politeness.name(),
        accepted,
        "{route} routed"
    );
    let line = announcer.borrow().status();
    alacer_core::flux::Flux::batch_of(|| status_line.set(line));
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// One control's payload: the boxed route the button drives (every route
/// closure is its own type, so the row carries trait objects).
type RouteHandler = Box<dyn FnMut()>;

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let announcer = Rc::new(RefCell::new(Announcer {
        last: None,
        refused: false,
    }));
    // The read the status line mirrors: the speech routes write this
    // signal, the binding maps it to the label — the only bridge (A1).
    let status_signal = pipeline.flux_mut().signal(announcer.borrow().status());

    let _ = pipeline.add_child(root, Label::new("The announcements (ADR-0024)").boxed());
    let status_binding = RenderBinding::new(announcer.borrow().status());
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |line: &String| line.clone(),
    );

    let actions = pipeline.add_child(root, Row::new().gap(8.0).boxed());
    let save_click = {
        let announcer = Rc::clone(&announcer);
        move || {
            speak(
                &announcer,
                "save",
                "Saved",
                Politeness::Polite,
                &status_signal,
            );
        }
    };
    let delete_click = {
        let announcer = Rc::clone(&announcer);
        move || {
            speak(
                &announcer,
                "delete",
                "Deleted",
                Politeness::Assertive,
                &status_signal,
            );
        }
    };
    let refuse_click = {
        let announcer = Rc::clone(&announcer);
        move || {
            speak(&announcer, "refuse", "", Politeness::Polite, &status_signal);
        }
    };
    for (name, handler) in [
        ("Save it", Box::new(save_click) as RouteHandler),
        ("Delete it", Box::new(delete_click) as RouteHandler),
        ("Speak nothing", Box::new(refuse_click) as RouteHandler),
    ] {
        let _ = pipeline.add_child(actions, Button::new(name).on_click(handler).boxed());
    }

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer announcements",
        (420.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
