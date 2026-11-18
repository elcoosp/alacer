//! The role-description tier as a consumer's vocabulary (M116,
//! ADR-0120): the role a custom control is *read as*, said by the a11y
//! tree at last — AccessKit's `RoleDescription` property (ARIA's
//! `aria-roledescription`) beside the generic role, the word every
//! platform AT turns into "star rating" while it walks the form. Two
//! worded controls beside their plain twins and a status line: the
//! star-rating slider, the wizard-page button — the plain slider and
//! the plain button between them, the honest absence the tier keeps
//! honest (a control that declared nothing is read as the generic role
//! it already was, never a guessed specialization).
//!
//! The beat the example exists to show: the word prices the word and
//! nothing else — the role-description word names a read-as role, it
//! walls nothing (the controls stay put, the plain button beside them
//! answers its press as ever), and the app's own door
//! (`Button::set_role_description` through the mutate seam) moves the
//! word live, the `PAINT | A11Y` request paying the re-announcement —
//! the write that lands where the word already is queues nothing.
//!
//! ```sh
//! RUST_LOG=alacer=info cargo run -p alacer --example role_description
//! ```
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label, Slider};
use alacer_core::flux::RenderBinding;
use alacer_core::passes::PassPipeline;
use alacer_core::widget::Role;
use reactive_graph::traits::Set as _;

/// The viewport the demo's beats settle into (the editing canon's shape).
const VIEWPORT_WIDTH: f32 = 480.0;
const VIEWPORT_HEIGHT: f32 = 360.0;

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}

/// Ticks until no pass has work (the idle contract's demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(alacer_core::geom::Size::new(
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
        )) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// The read-as words the tree says for the custom controls, read back
/// through the snapshot in walk order — the same answer the sink maps
/// onto AccessKit's `RoleDescription` property: the specialization
/// beside the generic role, the plain controls the honest absence.
fn read_as_words(tree: Option<&alacer_core::passes::TreeSnapshot>) -> String {
    let words: Vec<String> = tree
        .map(|t| {
            t.nodes
                .iter()
                .filter(|n| matches!(n.role, Role::Button | Role::Slider))
                .map(|n| {
                    let generic = match n.role {
                        Role::Slider => "slider",
                        _ => "button",
                    };
                    match n.role_description.as_deref() {
                        Some(word) => format!("{generic} → {word}"),
                        None => format!("{generic} (generic)"),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    words.join(" · ")
}

/// The app's role-description door: the mutate seam the app's code
/// rides — the word moved, `PAINT | A11Y` requested on the control, the
/// re-announcement paid for. `None` clears the word.
fn set_control_role_description(
    pipeline: &mut PassPipeline,
    node: alacer_core::NodeId,
    role_description: Option<&str>,
) {
    let mut guard = pipeline.arena_mut().mutate(node);
    if let Some(button) = guard.widget_mut().as_any_mut().downcast_mut::<Button>() {
        button.set_role_description(role_description.map(str::to_owned));
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
}

fn main() {
    let mut pipeline = pipeline_default();

    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());
    let _ = pipeline.add_child(
        root,
        Label::new(
            "The role-description tier: the role a control is read as, said by the tree (ADR-0120)",
        )
        .boxed(),
    );

    // The custom controls: the read-as role rides the tree from the
    // first frame — the star-rating slider, the wizard-page button —
    // beside the plain twins the honest absence between them.
    let _rating = pipeline.add_child(
        root,
        Slider::new("Rating", 0.0, 5.0)
            .role_description("star rating")
            .boxed(),
    );
    let _ = pipeline.add_child(root, Slider::new("Volume", 0.0, 10.0).boxed());
    let wizard = pipeline.add_child(
        root,
        Button::new("Chapter 3")
            .role_description("wizard page")
            .boxed(),
    );
    let plain = pipeline.add_child(root, Button::new("Send").boxed());

    // The status line: the words the tree says, read back through the
    // snapshot (the consumer's side of the fact the AT reads) — the
    // snapshot read after the first settle, the pass-9 product the AT's
    // own tree rides.
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    let line = read_as_words(snapshot.as_ref());
    let status_signal = pipeline.flux_mut().signal(line.clone());
    let status_binding = RenderBinding::new(line);
    let status_label = pipeline.add_child(root, Label::bound(status_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        status_signal,
        status_label,
        &status_binding,
        alacer_core::Change::PAINT,
        |line: &String| line.clone(),
    );

    // Beat one: the word is a word, not a wall — the plain button
    // beside the worded controls answers its press exactly as it always
    // did, and the star rating's value doors answer as ever.
    let consumed = pipeline.dispatch_a11y_action(
        snapshot
            .as_ref()
            .and_then(|t| t.find(Role::Button, "Send"))
            .map(|node| node.id)
            .expect("the plain button rides the snapshot"),
        alacer_core::widget::A11yAction::DefaultPress,
    );
    settle(&mut pipeline);
    tracing::info!(
        target: "alacer::role_description",
        consumed,
        "beat 1: the AT's press lands unchanged beside the worded controls — the word names a read-as role, it walls nothing"
    );

    // Beat two: the live door — the app's read-as role moved, the word
    // the tree says following it (the re-announcement the mutate seam's
    // request paid for); then the honest absence restored.
    set_control_role_description(&mut pipeline, plain, Some("checklist step"));
    set_control_role_description(&mut pipeline, wizard, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the live door moved the buttons' words",
        read_as_words(snapshot.as_ref()),
    ));
    tracing::info!(
        target: "alacer::role_description",
        controls = "Send, Chapter 3",
        word = "role_description",
        "beat 2: the app's door moved the words the tree says — the specialization follows the app's own"
    );
    set_control_role_description(&mut pipeline, plain, None);
    settle(&mut pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    status_signal.set(format!(
        "{} — the door cleared the words",
        read_as_words(snapshot.as_ref()),
    ));

    // The window: live play — the star rating says the word beside the
    // plain slider, every door answers as it always did.
    let _ = pipeline.add_child(
        root,
        Label::new(
            "Live: the custom controls say the read-as role — every door answers as it always did",
        )
        .boxed(),
    );
    settle(&mut pipeline);

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer role-description",
        (VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}
