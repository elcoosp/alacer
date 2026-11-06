//! The hot-reload generation contract (§13.2, ADR-7).
//!
//! A [`HotView`] is a [`ViewRoot`] plus a swappable *view-function
//! generation*: the one piece of state a hot patch replaces. The contract:
//!
//! 1. **A patch swaps code, never state.** Signals live in the flux's
//!    reactive owner, dynamic values in [`alacer_core::flux::RenderBinding`] slots, retained
//!    widget state (text, focus, scroll offsets) in the arena keyed by the
//!    id-path identity — all outside the patched closure. [`HotView::regenerate`]
//!    re-runs the generation and reconciles; nothing else is touched.
//! 2. **The re-render is scoped (§13.2).** The reconciler diffs the fresh
//!    view against the mounted tree and applies the difference in place:
//!    prop changes patch in place, structural changes mount/remove exactly
//!    the differing subtrees, keyed lists route through peel→map→LIS. The
//!    touched nodes enter the frame as `Change` masks — the same currency
//!    every consumer reads.
//! 3. **One patch ⇒ one layout+paint cycle.** The reconciler queues masks;
//!    the next tick consumes them once (ADR-6's contract, unchanged).
//!
//! The facade's `hot-reload` feature wraps [`HotView`] with the real
//! `subsecond` machinery: `subsecond::call` around the generation (the
//! jump-table detour point when a binary patch lands) and a
//! `subsecond::register_handler` trigger answered with exactly one
//! regenerate.

use crate::mount::{PatchReport, ViewRoot};
use crate::registry::WidgetRegistry;
use crate::view::ViewNode;
use alacer_core::arena::NodeId;
use alacer_core::passes::PassPipeline;

/// A view-function generation: the swappable half of a [`HotView`].
///
/// Plain `FnMut` so generations can capture signals and bindings freely.
/// The closure must not create long-lived reactive state (the state-owner
/// contract, §13.2 item 1): signals and bindings belong to the app's flux,
/// created once, outside every generation.
pub type ViewGeneration = Box<dyn FnMut() -> ViewNode>;

/// A mounted view tree whose generating function can be swapped at runtime —
/// the state-preserving hot-reload runtime (§13.2, ADR-7).
pub struct HotView {
    generate: ViewGeneration,
    root: ViewRoot,
}

impl HotView {
    /// Mounts generation zero.
    ///
    /// # Panics
    ///
    /// Panics when the generation names an unregistered element.
    #[must_use]
    pub fn mount(
        pipeline: &mut PassPipeline,
        registry: WidgetRegistry,
        mut generate: impl FnMut() -> ViewNode + 'static,
    ) -> Self {
        let view = generate();
        Self {
            generate: Box::new(generate),
            root: ViewRoot::mount(&view, pipeline, registry),
        }
    }

    /// Swaps the view-function generation (the hot-patch seam).
    ///
    /// Nothing renders yet: the next [`Self::regenerate`] diffs the new
    /// generation against the mounted tree and patches the difference.
    pub fn set_generation(&mut self, generate: impl FnMut() -> ViewNode + 'static) {
        self.generate = Box::new(generate);
    }

    /// Re-runs the current generation and applies the difference in place —
    /// the scoped re-render (§13.2): patched nodes queue masks, untouched
    /// widgets keep their retained state and journal layers.
    ///
    /// # Panics
    ///
    /// Panics when the new generation names an unregistered element, when a
    /// keyed sibling list contains duplicate keys, or when the root element's
    /// type changed (the [`ViewRoot`] contract: mount a new [`HotView`]).
    pub fn regenerate(&mut self, pipeline: &mut PassPipeline) -> PatchReport {
        let view = (self.generate)();
        self.root.update(&view, pipeline)
    }

    /// The mounted root's arena node — stable across patches for the root's
    /// element type (the identity contract §6.3: the id-path never moves).
    #[must_use]
    pub const fn root_id(&self) -> NodeId {
        self.root.root_id()
    }

    /// The registry (extend it between generations to honor new elements).
    #[must_use]
    pub const fn registry(&self) -> &WidgetRegistry {
        self.root.registry()
    }
}

#[cfg(test)]
mod tests {
    use super::{HotView, ViewGeneration};
    use crate::registry::WidgetRegistry;
    use crate::view::ViewNode;
    use alacer_core::arena::NodeId;
    use alacer_core::geom::Size;
    use alacer_core::passes::{LayoutEngine, LayoutError, PassPipeline, SceneJournal};
    use alacer_core::widget::{LabelSource, Role, Widget};
    use rustc_hash::FxHashMap;

    /// A text-carrying probe widget (the foundation names no widget crates).
    struct Probe {
        text: String,
    }

    impl Widget for Probe {
        fn role(&self) -> Role {
            Role::Label
        }

        fn label(&self) -> LabelSource {
            LabelSource::text(self.text.clone())
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    fn probe_create(props: &crate::registry::ElementProps<'_>) -> Probe {
        Probe {
            text: props.text("text").unwrap_or_default().to_string(),
        }
    }

    fn probe_patch(probe: &mut Probe, props: &crate::registry::ElementProps<'_>) -> bool {
        match props.text("text") {
            Some(text) => {
                probe.text = text.to_string();
                true
            }
            None => false,
        }
    }

    fn registry() -> WidgetRegistry {
        let mut registry = WidgetRegistry::new();
        registry.register("probe", probe_create, probe_patch);
        registry
    }

    /// A no-op layout engine (these tests drive the arena, never geometry).
    struct NullLayout;

    impl LayoutEngine for NullLayout {
        fn sync_and_layout(
            &mut self,
            _arena: &mut alacer_core::Arena,
            _root: NodeId,
            _dirty: &FxHashMap<NodeId, alacer_core::mask::Change>,
            _viewport: Size,
        ) -> Result<(), LayoutError> {
            Ok(())
        }

        fn measure_calls(&self) -> u64 {
            0
        }

        fn reset_measure_calls(&mut self) {}
    }

    /// A no-op journal (same shape as the mount tests' fake).
    #[derive(Default)]
    struct NullJournal;

    impl SceneJournal for NullJournal {
        fn commit_layer(&mut self, _node: NodeId, _primitives: Vec<alacer_core::scene::Primitive>) {
        }

        fn drop_layer(&mut self, _node: NodeId) {}

        fn layer(&self, _node: NodeId) -> Option<Vec<alacer_core::scene::Primitive>> {
            None
        }

        fn clear(&mut self) {}

        fn snapshot(&self, _order: &[NodeId]) -> alacer_core::scene::SceneSnapshot {
            alacer_core::scene::SceneSnapshot {
                primitives: Vec::new(),
            }
        }

        fn retain(&mut self, _order: &[NodeId]) {}
    }

    #[derive(Default)]
    struct NullA11y;

    impl alacer_core::passes::A11ySink for NullA11y {
        fn apply(
            &mut self,
            _tree: std::sync::Arc<alacer_core::passes::TreeSnapshot>,
            _delta: &alacer_core::passes::A11yDelta,
        ) {
        }

        fn resolved_tree(&self) -> Option<&alacer_core::passes::TreeSnapshot> {
            None
        }
    }

    fn pipeline() -> PassPipeline {
        PassPipeline::new(
            alacer_core::Flux::new(),
            Box::new(NullLayout),
            Box::new(NullJournal),
            Box::new(NullA11y),
        )
    }

    fn generation(text: &str) -> ViewGeneration {
        let text = text.to_string();
        Box::new(move || ViewNode::element("probe").prop("text", text.clone()))
    }

    #[test]
    fn patch_swaps_the_rendered_value_in_place() {
        let mut pipeline = pipeline();
        let mut hot = HotView::mount(&mut pipeline, registry(), generation("v1"));
        let root = hot.root_id();
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");

        hot.set_generation(generation("v2"));
        let report = hot.regenerate(&mut pipeline);
        assert_eq!(report.patched, 1, "the prop patched in place");
        assert_eq!(report.created + report.removed, 0, "nothing rebuilt");
        assert_eq!(hot.root_id(), root, "node identity stable across patches");
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");
        assert_eq!(
            pipeline.arena().widget(root).label().resolve(),
            "v2",
            "the new generation renders"
        );
    }

    #[test]
    fn regenerate_without_a_patch_is_a_no_op() {
        let mut pipeline = pipeline();
        let mut hot = HotView::mount(&mut pipeline, registry(), generation("same"));
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");
        let report = hot.regenerate(&mut pipeline);
        assert_eq!(report.patched, 0, "identical generation patches nothing");
    }

    #[test]
    fn structural_generations_mount_and_remove_exactly_the_difference() {
        fn generation_of(children: usize) -> ViewGeneration {
            Box::new(move || {
                let mut node = ViewNode::element("probe").prop("text", "root");
                for i in 0..children {
                    node = node.child(ViewNode::element("probe").prop("text", i.to_string()));
                }
                node
            })
        }
        let mut pipeline = pipeline();
        let mut hot = HotView::mount(&mut pipeline, registry(), generation_of(3));
        let root = hot.root_id();
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");

        hot.set_generation(generation_of(1));
        let report = hot.regenerate(&mut pipeline);
        assert_eq!(report.created, 0);
        assert_eq!(report.removed, 2, "exactly the differing subtrees removed");
        assert_eq!(hot.root_id(), root);

        hot.set_generation(generation_of(5));
        let report = hot.regenerate(&mut pipeline);
        assert_eq!(report.created, 4);
        assert_eq!(report.removed, 0);
        assert_eq!(hot.root_id(), root, "still the same root");
    }

    #[test]
    fn state_captured_outside_the_generation_survives_patches() {
        // The state-owner contract (§13.2 item 1): the generation captures a
        // shared slot; swapping generations must not disturb it. A patch
        // replaces *code*; the captured cell keeps its mutations.
        let slot = std::rc::Rc::new(std::cell::RefCell::new(7_i32));
        let captured = std::rc::Rc::clone(&slot);
        let mut pipeline = pipeline();
        let mut hot = HotView::mount(&mut pipeline, registry(), move || {
            ViewNode::element("probe").prop("text", captured.borrow().to_string())
        });
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");

        *slot.borrow_mut() = 42;
        hot.set_generation(move || {
            ViewNode::element("probe").prop("text", slot.borrow().to_string())
        });
        hot.regenerate(&mut pipeline);
        pipeline.tick(Size::new(200.0, 100.0)).expect("tick");
        assert_eq!(
            pipeline.arena().widget(hot.root_id()).label().resolve(),
            "42",
            "state outside the generation survives the patch"
        );
    }
}
