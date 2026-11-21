//! The hot-patch driver (§13.2, ADR-7): the CI hot-patch suite's reusable
//! half of the state-preserving hot-reload contract.
//!
//! The plan's demand (§13.2): "A subsecond binary patch swaps code, never
//! state — exactly one scoped re-render, pinned by a CI hot-patch suite."
//! The platform's `PatchChannel` carries the wake-up (a binary patch's
//! handler fires it; before a loop attaches, `fire` is the documented
//! silent drop — nothing is running to wake). The *contract* the suite
//! proves lives one step downstream, and this module is its production
//! home: one delivered patch event is answered by **exactly one**
//! [`alacer_view::HotView::regenerate`] — the scoped re-render — and
//! nothing else. The generation swap (`HotView::set_generation`) is the
//! jump table's detour: it replaces the view-function closure, the one
//! piece of state a patch replaces; every signal, binding, and retained
//! widget state lives outside it.
//!
//! The driver keeps the patch accounting honest: deliveries and answers
//! move in lockstep (one `deliver_patch` = one trigger fire = one
//! regenerate), so a suite asserting the report's shape
//! (`created: 0, removed: 0, patched: 1, moved: 0` for a prop-only patch)
//! is also asserting the one-patch-one-frame discipline end to end.

use alacer_core::arena::NodeId;
use alacer_core::passes::PassPipeline;
use alacer_view::{HotView, PatchReport};

/// The patch delivery trigger: the CI hot-patch suite's stand-in for the
/// platform's patch-channel fire (§13.2). The facade's
/// `register_patch_handler` wiring fires the real `PatchChannel` when a
/// binary patch lands; headless CI runs the same contract through this
/// seam — one fire per delivered patch, answered by exactly one scoped
/// re-render.
pub type PatchTrigger = Box<dyn Fn()>;

/// A mounted [`HotView`] paired with its patch trigger and the loop's
/// one-answer-per-patch discipline (§13.2) — the harness the CI
/// hot-patch suite drives every generation swap through.
pub struct HotPatchDriver {
    hot: HotView,
    trigger: PatchTrigger,
    delivered: u64,
}

impl HotPatchDriver {
    /// Pairs a mounted [`HotView`] with the patch trigger (the channel
    /// fire's seam) and starts the accounting at zero.
    #[must_use]
    pub fn mount(hot: HotView, trigger: PatchTrigger) -> Self {
        Self {
            hot,
            trigger,
            delivered: 0,
        }
    }

    /// Delivers one patch: the jump table swaps the generation (`swap` —
    /// the code, never the state), the trigger fires (the channel's
    /// wake-up), and the loop answers with exactly one scoped re-render.
    /// Returns the reconciler's [`PatchReport`].
    pub fn deliver_patch(
        &mut self,
        pipeline: &mut PassPipeline,
        swap: impl FnOnce(&mut HotView),
    ) -> PatchReport {
        swap(&mut self.hot);
        (self.trigger)();
        let report = self.hot.regenerate(pipeline);
        self.delivered += 1;
        report
    }

    /// How many patches this driver has delivered — and answered. The two
    /// counts are one: every delivery ends in exactly one regenerate.
    #[must_use]
    pub const fn patches_delivered(&self) -> u64 {
        self.delivered
    }

    /// The mounted root's arena node — stable across patches for the
    /// root's element type (the identity contract, §6.3).
    #[must_use]
    pub const fn root_id(&self) -> NodeId {
        self.hot.root_id()
    }

    /// The paired view, for suites that need the raw patch seam
    /// (`set_generation` rides [`Self::deliver_patch`]'s `swap`).
    #[must_use]
    pub const fn hot(&self) -> &HotView {
        &self.hot
    }
}
