//! The `subsecond` wiring, pinned headlessly (ADR-7, §13.2, ADR-0010):
//!
//! - under the `hot-reload` feature: `subsecond::call` around a generation
//!   is transparent when no jump table is loaded (subsecond's documented
//!   degenerate mode — a direct call), `register_patch_handler` accepts
//!   handler registrations, and `mount_hot` composes with the harness-free
//!   pipeline; firing a patch channel before the loop attaches is a no-op;
//! - without the feature: the same generation contract runs bare.
//!
//! The full hot-patch contract (state intact, one frame, scoped) lives in
//! `alacer-testing/tests/m4_exit_criteria.rs` — this file pins the facade
//! wiring.

use alacer::pipeline_default;
use alacer::platform::PatchChannel;
use alacer_view::ViewNode;

fn generation(text: &'static str) -> impl FnMut() -> ViewNode {
    move || ViewNode::element("label").prop("text", text)
}

#[test]
fn mount_hot_patches_a_generation_headlessly() {
    let mut pipeline = pipeline_default();
    let mut hot = alacer::hot::mount_hot(&mut pipeline, generation("before"));
    pipeline
        .tick(alacer_core::geom::Size::new(200.0, 100.0))
        .expect("first tick");

    hot.set_generation(generation("after"));
    let report = hot.regenerate(&mut pipeline);
    assert_eq!(report.patched, 1, "the label patched in place");
    pipeline
        .tick(alacer_core::geom::Size::new(200.0, 100.0))
        .expect("second tick");
    assert_eq!(
        pipeline.arena().widget(hot.root_id()).label().resolve(),
        "after",
        "the patched generation renders"
    );
}

#[test]
fn a_patch_channel_fire_before_the_loop_attaches_is_a_no_op() {
    // The channel is inert until `AlacerApp::run` attaches the proxy: firing
    // earlier loses the wake-up harmlessly (nothing is running to wake).
    let channel = PatchChannel::new();
    channel.fire();
    channel.fire();
}

#[cfg(feature = "hot-reload")]
mod with_subsecond {
    use super::{PatchChannel, generation, pipeline_default};
    use alacer_core::geom::Size;
    use alacer_view::PropValue;

    #[test]
    fn subsecond_call_is_a_transparent_passthrough_headless() {
        // No jump table loaded: `subsecond::call` is subsecond's documented
        // degenerate mode — a direct call. The wiring never gets in the way
        // of an unpatched binary.
        let value = subsecond::call(|| 40 + 2);
        assert_eq!(value, 42);

        // The exact shape `hot_call` gives every generation: a closure
        // invoked through `subsecond::call`.
        let mut generate = generation("hot");
        let node = subsecond::call(&mut generate);
        assert_eq!(
            node.props.first().map(|(_, prop)| match prop {
                PropValue::Text(text) => text.clone(),
                _ => String::new(),
            }),
            Some("hot".to_string()),
            "the wrapped generation still builds its view"
        );
    }

    #[test]
    fn register_patch_handler_accepts_registrations() {
        // The handler runs when a patch lands; registering one headlessly
        // must be side-effect-free. (The fire path is covered by the
        // channel test above; the loop attach path is the platform's.)
        let channel = PatchChannel::new();
        alacer::hot::register_patch_handler(&channel);
        channel.fire();
    }

    #[test]
    fn mount_hot_wraps_the_generation_in_subsecond_call() {
        // mount_hot under the feature wraps the generation; the view still
        // reconciles identically headless.
        let mut pipeline = pipeline_default();
        let hot = alacer::hot::mount_hot(&mut pipeline, generation("wrapped"));
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");
        assert_eq!(
            pipeline.arena().widget(hot.root_id()).label().resolve(),
            "wrapped",
            "the subsecond-wrapped generation renders through the call"
        );
    }
}
