//! The hot-reload runtime (§13.2, ADR-7): the facade's wiring between the
//! view generation contract ([`alacer_view::HotView`]) and `subsecond`'s
//! jump-table patching.
//!
//! The three integration points, exactly as the architecture specifies:
//!
//! 1. **State lives outside the patched code.** Signals live in the flux's
//!    reactive owner, dynamic values in `RenderBinding` slots, retained
//!    widget state in the arena. The view function is the only thing a
//!    patch replaces.
//! 2. **The generation is wrapped in `subsecond::call`** — the detour point
//!    where a jump-table patch swaps in the new code (feature `hot-reload`).
//! 3. **A patch re-renders scoped, once.** The patch handler fires the
//!    [`alacer_platform::PatchChannel`]; the loop answers with exactly one
//!    [`alacer_view::HotView::regenerate`] (the reconciler patches in place
//!    and queues masks — one layout+paint cycle, the ADR-6 contract
//!    unchanged).
//!
//! ## Running against the real patcher
//!
//! ```sh
//! cargo run -p alacer --features hot-reload --example hot_reload
//! # then, in another terminal, serve the same example with the dioxus CLI:
//! dx serve --hotpatch
//! ```
//!
//! Without a patch loaded, `subsecond::call` is subsecond's documented
//! degenerate mode: a direct call. The `hot_reload` example also
//! demonstrates the contract without the patcher — its "simulate patch"
//! control swaps the generation and fires the channel, the same path a real
//! patch takes.
//!
//! The CI hot-patch suite (`alacer-testing/tests/m4_exit_criteria.rs`) pins
//! the framework's half of the contract headlessly.

use crate::platform::PatchChannel;
use crate::widgets::default_registry;
use alacer_core::passes::PassPipeline;
use alacer_view::HotView;
use alacer_view::ViewNode;

/// The subsecond integration (§13.2, ADR-7) — compiled only under the
/// `hot-reload` feature: the patch handler registration and the
/// `subsecond::call` wrapper.
#[cfg(feature = "hot-reload")]
pub mod subsecond_wiring {
    use crate::platform::PatchChannel;
    use alacer_view::ViewNode;
    use std::sync::Arc;

    /// Registers `channel.fire()` as the subsecond patch handler.
    ///
    /// A binary patch runs this handler on the patcher's thread; it wakes
    /// the loop, and the loop answers with exactly one scoped re-render —
    /// "on patch, `RenderEffect`s re-dirty instead of rebuilding the world"
    /// (§13.2: the reconciler queues masks; the next tick is one frame).
    pub fn register(channel: PatchChannel) {
        subsecond::register_handler(Arc::new(move || channel.fire()));
    }

    /// Wraps a generation in `subsecond::call` — the detour point a
    /// jump-table patch swaps. Without a patch loaded this is subsecond's
    /// documented degenerate mode: a direct call.
    pub fn hot_call(mut generate: impl FnMut() -> ViewNode + 'static) -> impl FnMut() -> ViewNode {
        move || subsecond::call(&mut generate)
    }
}

/// The one-import assembly for apps that own their loop setup: mounts the
/// hot view over the default element registry.
///
/// Under the `hot-reload` feature the generation is wrapped in
/// `subsecond::call` — the detour point a binary patch swaps. Pair with
/// [`register_patch_handler`] and
/// [`AlacerApp::with_patch_channel`](crate::platform::AlacerApp::with_patch_channel) to
/// deliver patches.
///
/// # Panics
///
/// Panics when the view names an unregistered element.
#[must_use]
pub fn mount_hot(
    pipeline: &mut PassPipeline,
    generate: impl FnMut() -> ViewNode + 'static,
) -> HotView {
    #[cfg(feature = "hot-reload")]
    let generate = subsecond_wiring::hot_call(generate);
    HotView::mount(pipeline, default_registry(), generate)
}

/// Registers the subsecond patch handler on `channel`: a binary patch runs
/// it on the patcher's thread; it wakes the loop, and the loop answers with
/// exactly one scoped re-render — "on patch, `RenderEffect`s re-dirty
/// instead of rebuilding the world" (§13.2 item 3: the reconciler queues
/// masks; the next tick is one frame).
///
/// Without the `hot-reload` feature this is a no-op — the simulate-patch
/// path fires the channel directly and needs no patcher.
pub fn register_patch_handler(channel: &PatchChannel) {
    #[cfg(feature = "hot-reload")]
    subsecond_wiring::register(channel.clone());
    #[cfg(not(feature = "hot-reload"))]
    let _ = channel;
}
