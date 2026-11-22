//! The M4 hot-reload demo (§18 canon, hot edition): one signal, one bound
//! label, one button — plus a "simulate patch" button that swaps the view
//! generation live. The counter keeps counting across the patch: state
//! lives in the flux, code is the only thing a patch replaces (§13.2,
//! ADR-7).
//!
//! Run with `cargo run -p alacer --example hot_reload`. Under the
//! `hot-reload` feature the view generation is wrapped in `subsecond::call`,
//! so the very same example accepts real jump-table patches from the dioxus
//! CLI: `dx serve --hotpatch` — edit the view functions and watch the window
//! re-render sub-second, state intact, no restart.
//!
//! Run with `--headless` for the CI-runnable proof of the same contract:
//! no window, no patcher — the example mounts, interacts, swaps the `rsx!`
//! generation closure itself, and asserts the reconciler's answer
//! (`created: 0, removed: 0, patched: >= 1`) while the counter survives.
//! `cargo test -p alacer --test hot_reload_headless` executes the same run
//! as a test.
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer::prelude::*;
use alacer_core::FluxRwSignal;
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::Cell;
use std::rc::Rc;

/// What a hot patch replaces: the view function. `0` is the shipped UI; `1`
/// stands in for "the code the patcher swapped in" — a different layout and
/// a new widget. Under `dx serve --hotpatch` the jump table performs this
/// swap at the binary level instead; the pipeline path is identical.
type Generation = u32;

fn counter_view(
    count: FluxRwSignal<i32>,
    binding: &RenderBinding<String>,
    generation: Generation,
    simulate_patch: Rc<dyn Fn()>,
) -> ViewNode {
    let binding = binding.clone();
    let increment = move || {
        // One write batch per click: the ADR-6 contract at the seam. The
        // same signal serves every generation — state outlives patches.
        alacer_core::flux::Flux::batch_of(|| {
            let current = count.try_get_untracked().unwrap_or_default();
            count.set(current + 1);
        });
    };
    match generation {
        0 => rsx! {
            column { gap: 12.0, padding: 16.0,
                label { binding: binding }
                button { text: "Increment", on_click: increment }
                button { text: "Simulate patch", on_click: move || simulate_patch() }
            }
        },
        _ => rsx! {
            column { gap: 8.0, padding: 24.0,
                label { binding: binding }
                label { text: "patched layout — same signal, same count" }
                button { text: "Still incrementing", on_click: increment }
            }
        },
    }
}

fn main() {
    // The CI-runnable proof: no window, no patcher — the example asserts
    // the §13.2 contract itself and exits nonzero on any failure.
    if std::env::args().any(|arg| arg == "--headless") {
        headless_proof();
        return;
    }

    let mut pipeline = pipeline_default();
    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());

    // The generation slot and the patch channel: the two handles the
    // "simulate patch" button drives (and the subsecond handler fires).
    let generation = Rc::new(Cell::new(0_u32));
    let channel = PatchChannel::new();

    // The view function reads the slot; `mount_hot` wraps it in
    // `subsecond::call` under the `hot-reload` feature.
    let gen_reader = Rc::clone(&generation);
    let gen_writer = Rc::clone(&generation);
    let gen_binding = binding.clone();
    let patch_fire = channel.clone();
    let simulate_patch: Rc<dyn Fn()> = Rc::new(move || {
        // The patch arrives: the swapped generation stands in for the new
        // code a binary patch would install, the channel wakes the loop,
        // and the hook re-renders scoped. The counter never notices.
        gen_writer.set(1);
        patch_fire.fire();
    });
    let generate = move || {
        counter_view(
            count,
            &gen_binding,
            gen_reader.get(),
            Rc::clone(&simulate_patch),
        )
    };

    let mut hot = alacer::hot::mount_hot(&mut pipeline, generate);

    // The flux bridge drives the bound label across generations.
    let label_node = bound_count_label(&pipeline);
    pipeline.flux_mut().bind_value(
        count,
        label_node,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    // The patch hook: the loop answers one patch event with one scoped
    // regenerate. A real `dx serve --hotpatch` binary patch takes the same
    // path — register the handler, fire the channel, one regenerate.
    let mut app = AlacerApp::new(
        pipeline,
        "Alacer hot reload (M4)",
        (360.0, 240.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        hot.regenerate(pipeline);
    }));
    alacer::hot::register_patch_handler(&channel);

    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// The headless proof (§13.2, the example's own CI run): mount the shipped
/// generation, interact with it, swap the `rsx!` generation closure (the
/// code a binary patch installs — a prop-only text-constant edit, the exact
/// patch shape the CI hot-patch suite pins), assert the reconciler's
/// answer — nothing created, nothing removed, the swapped element patched
/// in place — and assert the reactive state survived it. Every check
/// exits nonzero on failure; success prints the report and exits 0.
#[expect(
    clippy::too_many_lines,
    reason = "the proof is one run through the whole contract; splitting it would hide the beats"
)]
fn headless_proof() {
    let mut pipeline = pipeline_default();
    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());

    // The shipped generation, mounted through the same `mount_hot` door
    // the window rides (without the patcher, `subsecond::call` is its
    // documented degenerate mode: a direct call).
    let gen_binding = binding.clone();
    let mut hot = alacer::hot::mount_hot(&mut pipeline, move || {
        headless_view(count, &gen_binding, HEADLESS_BEFORE)
    });
    let label_node = bound_count_label(&pipeline);
    pipeline.flux_mut().bind_value(
        count,
        label_node,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );
    settle(&mut pipeline);

    // Interact: three clicks through the same write batch the button
    // rides — reactive state the patch must never disturb.
    for _ in 0..3 {
        alacer_core::flux::Flux::batch_of(|| {
            let current = count.try_get_untracked().unwrap_or_default();
            count.set(current + 1);
        });
    }
    settle(&mut pipeline);
    let count_before = count.try_get_untracked().unwrap_or_default();
    require(
        count_before == 3,
        "the clicks did not write the signal before the patch",
    );
    require(
        bound_label_text(&pipeline, label_node) == "Clicked 3 times",
        "the bound label lost the count before the patch",
    );

    // The patch arrives: the swapped `rsx!` generation closure stands in
    // for the new code a binary patch installs — a prop-only edit (the
    // marker's text constant), the patch shape the CI suite pins. One
    // scoped re-render answers it.
    let patched_binding = binding.clone();
    hot.set_generation(move || headless_view(count, &patched_binding, HEADLESS_AFTER));
    let report = hot.regenerate(&mut pipeline);
    settle(&mut pipeline);

    // The reconciler's answer: a prop-only patch creates nothing, removes
    // nothing — it patches exactly the swapped element (and nothing else,
    // the keyed rows untouched).
    require(
        report.created == 0,
        &format!("the patch created {} nodes", report.created),
    );
    require(
        report.removed == 0,
        &format!("the patch removed {} nodes", report.removed),
    );
    require(
        report.patched >= 1,
        "the patch patched nothing — the swapped generation never rendered",
    );

    // The patched generation renders: the marker label carries the
    // patched-in constant, and the state the patch must never disturb is
    // intact — the signal keeps its value, the bound label keeps driving.
    require(
        marker_renders(&pipeline, HEADLESS_AFTER),
        "the patched generation's marker never rendered",
    );
    require(
        count.try_get_untracked().unwrap_or_default() == count_before,
        "the patch lost the counter's value — state rode the patch",
    );
    require(
        bound_label_text(&pipeline, label_node) == "Clicked 3 times",
        "the patch lost the bound label's reactive value",
    );

    println!("hot-reload headless proof: {report:?} — count {count_before} state intact");
}

/// The proof's two generations differ in exactly one prop: the marker
/// label's text constant (the CI hot-patch job performs the same edit on
/// this example's source and hands the two binaries to the suite).
const HEADLESS_BEFORE: &str = "hot-reload demo (shipped generation)";
const HEADLESS_AFTER: &str = "hot-reload demo (patched generation)";

/// The proof's view: the marker label over the bound counter and the
/// increment button — the same tree both generations, keyed, so the
/// generation swap is the prop-only patch shape.
fn headless_view(
    count: FluxRwSignal<i32>,
    binding: &RenderBinding<String>,
    marker: &str,
) -> ViewNode {
    let binding = binding.clone();
    rsx! {
        column { gap: 12.0, padding: 16.0,
            label { key: 1_u64, text: marker }
            label { key: 2_u64, binding: binding }
            button { key: 3_u64, text: "Increment", on_click: move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            } }
        }
    }
}

/// The headless frame budget: at most this many ticks before the pipeline
/// is taken as it stands (the a11y demos' settle shape).
const HEADLESS_SETTLE_TICKS: usize = 16;

/// The proof's viewport: the same window shape the demo opens live.
const HEADLESS_VIEWPORT: (f32, f32) = (360.0, 240.0);

/// Drives headless ticks until the pipeline reports idle (the first tick
/// builds the full tree, the next reports none left).
fn settle(pipeline: &mut alacer_core::passes::PassPipeline) {
    for _ in 0..HEADLESS_SETTLE_TICKS {
        let (width, height) = HEADLESS_VIEWPORT;
        let report = pipeline
            .tick(alacer_core::geom::Size::new(width, height))
            .expect("the hot-reload demo lays out headlessly");
        if report.is_idle() {
            break;
        }
    }
}

/// The arena node of the bound count label: the first Label-role node
/// whose resolved name starts with "Clicked" (the marker never does) —
/// the CI suite's own finder, on the same canon shape.
fn bound_count_label(pipeline: &alacer_core::passes::PassPipeline) -> alacer_core::NodeId {
    let arena = pipeline.arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none()
            && slot.widget().role() == alacer_core::Role::Label
            && slot.widget().label().resolve().starts_with("Clicked")
        {
            found = Some(id);
        }
    });
    found.expect("the view mounted a bound count label")
}

/// The bound label's resolved text — the reactive value a patch must
/// keep binding across generations.
fn bound_label_text(
    pipeline: &alacer_core::passes::PassPipeline,
    node: alacer_core::NodeId,
) -> String {
    pipeline.arena().widget(node).label().resolve()
}

/// Whether the marker label renders with the given text — the patched
/// generation's own truth, read from the arena.
fn marker_renders(pipeline: &alacer_core::passes::PassPipeline, marker: &str) -> bool {
    let mut found = false;
    pipeline.arena().for_each_pre_order(&mut |_, slot| {
        if !found
            && slot.widget().role() == alacer_core::Role::Label
            && slot.widget().label().resolve() == marker
        {
            found = true;
        }
    });
    found
}

/// The proof's one failure shape: the message names what failed, the
/// exit code tells CI — never a silent pass.
fn require(condition: bool, message: &str) {
    if !condition {
        eprintln!("alacer: the headless hot-reload proof failed: {message}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
