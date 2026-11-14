//! Alacer's headless testing harness (A4, ADR-10).
//!
//! Headless tests run the **real pass pipeline** with a **software
//! rasterizer** — no mock event loop, no mock renderer. GPU-less CI is the
//! default, not a downgrade. The harness also provides:
//!
//! - the a11y-driven query API — the *only* official query API (§11.5);
//! - the debug CI walker that fails on unnamed input widgets (§11.1);
//! - snapshot testing with `ALACER_BLESS=1` re-blessing (§13.4);
//! - record/replay: user interactions saved as `.alc` files and replayed
//!   in `cargo test` — bug reports that are also regression tests (§13.4,
//!   M6);
//! - scheduling-contract counters for the ADR-6 kill-switch.

pub mod recording;

pub use recording::{
    RECORDING_SCHEMA, Recording, Session, Step, key_from_name, key_name, role_from_name,
};

use alacer_a11y::AccessKitSink;
use alacer_core::arena::NodeId;
use alacer_core::events::{EventCtx, MouseButton, PointerEvent};
use alacer_core::flux::Flux;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::PassPipeline;
use alacer_core::trail::TrailLog;
use alacer_core::widget::{A11yAction, Role, Widget};
use alacer_layout::LayoutHost;
use alacer_render::{SceneJournalImpl, SoftwareRenderer};
use alacer_text::{ThreadLocalGlyphSource, ThreadLocalMeasurer, ThreadLocalTextSeam};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tiny_skia::Pixmap;

/// The harness: pipeline + viewport + counters, everything headless.
pub struct TestHarness {
    pipeline: PassPipeline,
    viewport: Size,
    renderer: SoftwareRenderer,
    glyph_source: ThreadLocalGlyphSource,
    clipboard: Rc<alacer_core::clipboard::MemoryClipboard>,
}

impl TestHarness {
    /// A harness with a `width × height` viewport.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        // The harness is an assembly site: widgets reach text through the
        // core seam and the clipboard through the clipboard seam, so
        // install this thread's implementations first (ADR-9). The
        // clipboard is the honest in-memory store: a headless CI never
        // races the operator's actual OS clipboard (ADR-0020).
        alacer_core::text_seam::install(Rc::new(ThreadLocalTextSeam));
        let clipboard = Rc::new(alacer_core::clipboard::MemoryClipboard::default());
        alacer_core::clipboard::install(
            Rc::clone(&clipboard) as Rc<dyn alacer_core::clipboard::ClipboardSeam>
        );
        let flux = Flux::new();
        let layout = LayoutHost::new(Arc::new(ThreadLocalMeasurer));
        let journal = SceneJournalImpl::new();
        let a11y = AccessKitSink::new();
        Self {
            pipeline: PassPipeline::new(flux, Box::new(layout), Box::new(journal), Box::new(a11y)),
            viewport: Size::new(width, height),
            renderer: SoftwareRenderer::new(),
            glyph_source: ThreadLocalGlyphSource,
            clipboard,
        }
    }

    /// The wrapped pipeline.
    #[must_use]
    pub const fn pipeline(&self) -> &PassPipeline {
        &self.pipeline
    }

    /// The wrapped pipeline, mutably.
    #[must_use]
    pub const fn pipeline_mut(&mut self) -> &mut PassPipeline {
        &mut self.pipeline
    }

    /// Installs the trail's provided reader (§13.3, ADR-0069): the
    /// [`TrailLog`] becomes the pipeline's observer and a reading handle
    /// is returned — the m-suite's collector shape, provided once beside
    /// the seam instead of hand-rolled per test. The ring is bounded
    /// (the A7 window discipline): a trail nobody drains evicts its
    /// oldest and counts the eviction, never a leak.
    #[must_use]
    pub fn trail(&mut self) -> TrailLog {
        let trail = TrailLog::new();
        self.pipeline.set_observer(Box::new(trail.clone()));
        trail
    }

    /// The trail door at a named cap — the reader's window priced in
    /// events (the ring evicts its oldest once the cap is exceeded, the
    /// eviction counted on the handle).
    #[must_use]
    pub fn trail_with_cap(&mut self, cap: usize) -> TrailLog {
        let trail = TrailLog::with_cap(cap);
        self.pipeline.set_observer(Box::new(trail.clone()));
        trail
    }

    /// Mounts the root widget.
    #[must_use]
    pub fn mount(&mut self, widget: Box<dyn Widget>) -> NodeId {
        self.pipeline.mount_root(widget)
    }

    /// Appends a child widget.
    #[must_use]
    pub fn add_child(&mut self, parent: NodeId, widget: Box<dyn Widget>) -> NodeId {
        self.pipeline.add_child(parent, widget)
    }

    /// Runs one frame.
    ///
    /// # Panics
    ///
    /// Panics if layout fails (a test bug, not a runtime condition).
    pub fn tick(&mut self) -> alacer_core::passes::FrameReport {
        self.pipeline
            .tick(self.viewport)
            .expect("harness layout must succeed")
    }

    /// Ticks until no pass has work (bounded; the idle contract, A7).
    ///
    /// # Panics
    ///
    /// Panics if the tree never settles within 16 frames.
    pub fn settle(&mut self) -> usize {
        for frame in 0..16 {
            let report = self.tick();
            if report.is_idle() {
                return frame;
            }
        }
        panic!("harness did not settle within 16 frames");
    }

    /// Renders the current scene to pixels.
    ///
    /// # Panics
    ///
    /// Panics if the renderer cannot allocate the viewport pixmap.
    #[must_use]
    pub fn pixels(&mut self) -> Pixmap {
        let scene = self.pipeline.tick(self.viewport).expect("layout").scene;
        self.renderer
            .render(
                &scene,
                u32_width(self.viewport.width),
                u32_height(self.viewport.height),
                alacer_widgets::theme::SURFACE,
                &mut self.glyph_source,
            )
            .expect("viewport pixmap")
    }

    /// Dispatches a click: press + release at `position`, then settle.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn click(&mut self, position: Point) {
        let mut ctx = EventCtx::new();
        let _ = &mut ctx;
        self.pipeline.dispatch_pointer(&PointerEvent::Down {
            position,
            button: alacer_core::events::MouseButton::Left,
            click_count: 1,
        });
        self.pipeline.dispatch_pointer(&PointerEvent::Up {
            position,
            button: alacer_core::events::MouseButton::Left,
        });
        self.settle();
    }

    /// Queues a raw pointer event into the pipeline's frame-batched input
    /// queue (§10.2, ADR-0014): the storm coalesces and routes on the next
    /// tick — exactly as the platform loop delivers it. Settle separately.
    pub fn queue_pointer(&mut self, event: PointerEvent) {
        self.pipeline.queue_pointer(event);
    }

    /// Dispatches a pointer press at `position`, then settles (M12,
    /// ADR-0018: the first half of a drag gesture — the field places its
    /// caret at the press's hit point and asks for focus).
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn pointer_press(&mut self, position: Point) {
        self.pipeline.dispatch_pointer(&PointerEvent::Down {
            position,
            button: MouseButton::Left,
            click_count: 1,
        });
        self.settle();
    }

    /// Dispatches a pointer move to `position` while a press is held, then
    /// settles (M12: the drag's extension half — the selection follows the
    /// pointer from the anchor). Without a prior [`Self::pointer_press`]
    /// this is hover, and widgets that drag-select will ignore it.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn pointer_move(&mut self, position: Point) {
        self.pipeline
            .dispatch_pointer(&PointerEvent::Move { position });
        self.settle();
    }

    /// Dispatches a pointer release at `position`, then settles (M12: the
    /// gesture's end — the selection it built survives as a state).
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn pointer_release(&mut self, position: Point) {
        self.pipeline.dispatch_pointer(&PointerEvent::Up {
            position,
            button: MouseButton::Left,
        });
        self.settle();
    }

    /// Drives the full press-move-release gesture from `from` to `to`,
    /// then settles (M12: drag-select as one gesture, the way a user's
    /// hand delivers it and the way recordings replay it).
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn drag(&mut self, from: Point, to: Point) {
        self.pointer_press(from);
        self.pointer_move(to);
        self.pointer_release(to);
    }

    /// Drives a double-click at `position`, then settles (M13, ADR-0019:
    /// the platform's click chain reported as a second press — the field
    /// selects the word under the hit). Single clicks stay
    /// [`Self::click`] / [`Self::pointer_press`].
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn double_click(&mut self, position: Point) {
        self.pipeline.dispatch_pointer(&PointerEvent::Down {
            position,
            button: MouseButton::Left,
            click_count: 2,
        });
        self.pipeline.dispatch_pointer(&PointerEvent::Up {
            position,
            button: MouseButton::Left,
        });
        self.settle();
    }

    /// Dispatches a wheel event at `position`, then settles.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn wheel(&mut self, position: Point, delta: (f32, f32)) {
        self.pipeline.dispatch_pointer(&PointerEvent::Wheel {
            delta: alacer_core::geom::Vector::new(delta.0, delta.1),
            position,
        });
        self.settle();
    }

    /// Dispatches one key event to the focused widget, then settles (§10.3:
    /// keyboard input is focus-routed; Tab cycles when ignored).
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn key(&mut self, event: &alacer_core::events::KeyEvent) {
        let _ = self.pipeline.dispatch_key(event);
        self.settle();
    }

    /// Types `text` as character presses into the focused widget.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            let _ = self
                .pipeline
                .dispatch_key(&alacer_core::events::KeyEvent::char(ch));
        }
        self.settle();
    }

    /// Sets the clipboard's text payload (M14, ADR-0020: a test's paste
    /// side — the payload the next paste chord reads). The seam is the
    /// harness's in-memory store, never the operator's OS clipboard.
    pub fn set_clipboard(&self, text: impl Into<String>) {
        self.clipboard.set(text);
    }

    /// The clipboard's current text payload (M14, ADR-0020: a test's copy
    /// side — exactly what the last write delivered). `None` when the
    /// store is empty, the state a refused copy leaves behind.
    #[must_use]
    pub fn clipboard_text(&self) -> Option<String> {
        self.clipboard.text()
    }

    /// Presses a special key (no text, no modifiers), then settles.
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn press_key(&mut self, key: alacer_core::events::Key) {
        let _ = self
            .pipeline
            .dispatch_key(&alacer_core::events::KeyEvent::pressed(key));
        self.settle();
    }

    /// Dispatches one IME event to the focused widget, then settles (§10.3:
    /// the fcitx5/ibus path — preedit composes, commit inserts).
    ///
    /// # Panics
    ///
    /// Panics if layout fails.
    pub fn ime(&mut self, event: &alacer_core::events::ImeEvent) {
        let _ = self.pipeline.dispatch_ime(event);
        self.settle();
    }

    /// The IME request the pipeline derived from the focused widget (§10.3).
    #[must_use]
    pub fn ime_request(&self) -> Option<alacer_core::passes::ImeRequest> {
        self.pipeline.ime_request().cloned()
    }

    /// The a11y query: first node with `role` whose name contains `name`
    /// (§11.5: tests drive the a11y tree, never the widget tree).
    ///
    /// # Panics
    ///
    /// Panics naming the missing node when the query fails.
    #[must_use]
    pub fn expect_a11y(&self, role: Role, name: &str) -> &alacer_core::passes::TreeNode {
        self.pipeline
            .a11y_snapshot()
            .and_then(|tree| tree.find(role, name))
            .unwrap_or_else(|| {
                panic!("a11y query failed: no {role} named {name:?} is reachable");
            })
    }

    /// The a11y query builder (§11.5) — the ergonomic plural form:
    /// `harness.query(Role::Label).named("Item").count()`.
    #[must_use]
    pub fn query(&self, role: Role) -> A11yQuery<'_> {
        A11yQuery {
            harness: self,
            role,
            name: String::new(),
        }
    }

    /// Dispatches the default accessibility action ("press") to the first
    /// node with `role` named `name`, then settles — the actionable half of
    /// §11.5's contract (reachable, named, **actionable**).
    ///
    /// # Panics
    ///
    /// Panics when no such node is reachable: a test that cannot find its
    /// target is a test bug.
    pub fn press(&mut self, role: Role, name: &str) {
        let consumed = self.act(role, name, A11yAction::DefaultPress);
        assert!(
            consumed,
            "the {role} named {name:?} did not consume the default a11y action"
        );
    }

    /// Dispatches `action` to the first node with `role` named `name`, then
    /// settles (M10, ADR-0016: the full action tier is drivable the same way
    /// the platform drives it). Returns whether the target **consumed** the
    /// action — `false` is a valid, assertable outcome: the no-munging
    /// contract means a refused action fires nothing and queues no masks.
    ///
    /// # Panics
    ///
    /// Panics when no such node is reachable: a test that cannot find its
    /// target is a test bug.
    pub fn act(&mut self, role: Role, name: &str, action: A11yAction) -> bool {
        let node = self
            .pipeline
            .a11y_snapshot()
            .and_then(|tree| tree.find(role, name))
            .unwrap_or_else(|| {
                panic!("a11y action failed: no {role} named {name:?} is reachable");
            });
        let id = node.id;
        let consumed = self.pipeline.dispatch_a11y_action(id, action);
        self.settle();
        consumed
    }

    /// The selection the a11y tree carries for the first node with `role`
    /// named `name` (M11, ADR-0017): the widget's reported range as
    /// `(start, end)` UTF-16 offsets in ascending order. `None` when the
    /// widget exposes no selection (unfocused, composing, or a valueless
    /// role) — the AT sees exactly what the sink sees.
    #[must_use]
    pub fn selection(&self, role: Role, name: &str) -> Option<(usize, usize)> {
        self.pipeline
            .a11y_snapshot()
            .and_then(|tree| tree.find(role, name))
            .and_then(|node| node.selection)
            .map(|selection| selection.normalized())
    }

    /// Renders the current frame's pixels — same as [`Self::pixels`].
    ///
    /// # Panics
    ///
    /// Same as [`Self::pixels`].
    #[must_use]
    pub fn frame_pixels(&mut self) -> Pixmap {
        self.pixels()
    }

    /// Compares the current frame against the blessed snapshot `name`.
    ///
    /// With `ALACER_BLESS=1` the snapshot is regenerated instead. Comparison
    /// allows a small percentage of tolerance pixels (per-OS font
    /// differences must not rot the corpus, §13.4 snapshot hygiene).
    ///
    /// # Panics
    ///
    /// Panics when the render differs beyond tolerance and `ALACER_BLESS` is
    /// unset.
    pub fn snapshot(&mut self, name: &str) {
        let pixmap = self.pixels();
        let path = blessed_dir().join(format!("{name}.png"));
        if std::env::var_os("ALACER_BLESS").is_some_and(|v| v == "1") {
            let _ = std::fs::create_dir_all(blessed_dir());
            pixmap.save_png(&path).expect("bless write");
            return;
        }
        let bytes = std::fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "blessed snapshot {} missing ({e}); run with ALACER_BLESS=1",
                path.display()
            )
        });
        let blessed = Pixmap::decode_png(&bytes).expect("blessed snapshot decodes");
        let diff = pixel_diff_ratio(&pixmap, &blessed);
        assert!(
            diff <= 0.02,
            "snapshot {name} drifted: {diff:.4} of pixels differ (bless with ALACER_BLESS=1 if intended)"
        );
    }

    /// The CI walker (§11.1): every widget's name must resolve, and input-like
    /// controls (roles that `requires_name()`) must be *named*.
    ///
    /// # Panics
    ///
    /// Panics listing every violation when any widget is unnamed.
    pub fn assert_all_named(&self) {
        let tree = self
            .pipeline
            .a11y_snapshot()
            .expect("a11y tree built — tick first");
        let offenders: Vec<String> = tree
            .nodes
            .iter()
            .filter(|n| n.role.requires_name() && n.label.trim().is_empty())
            .map(|n| format!("unnamed {} (id {})", n.role, n.id))
            .collect();
        assert!(
            offenders.is_empty(),
            "a11y CI walker found unnamed widgets: {offenders:?}"
        );
    }

    /// Total measure calls the layout engine made (translation-only probes).
    #[must_use]
    pub fn measure_calls(&self) -> u64 {
        self.pipeline.measure_calls()
    }

    /// Resets the layout engine's measure counter.
    pub fn reset_measure_calls(&mut self) {
        self.pipeline.reset_measure_calls();
    }

    /// Current frame counters (scheduling-contract probes).
    #[must_use]
    pub fn counters(&self) -> alacer_core::passes::FrameCounters {
        self.pipeline.counters()
    }

    /// Bounds of a node (geometry probes).
    ///
    /// # Panics
    ///
    /// Panics if the node has no bounds yet (tick first).
    #[must_use]
    pub fn bounds(&self, node: NodeId) -> Rect {
        self.pipeline
            .arena()
            .bounds(node)
            .unwrap_or_else(|| panic!("{node} has no bounds — tick first"))
    }

    /// The viewport size.
    #[must_use]
    pub const fn viewport(&self) -> Size {
        self.viewport
    }
}

fn u32_width(w: f32) -> u32 {
    alacer_core::num::f32_to_u32(w)
}

fn u32_height(h: f32) -> u32 {
    alacer_core::num::f32_to_u32(h)
}

fn blessed_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("blessed")
}

/// Fraction of pixels whose channel deltas exceed 8/255.
#[must_use]
pub fn pixel_diff_ratio(a: &Pixmap, b: &Pixmap) -> f32 {
    if a.width() != b.width() || a.height() != b.height() {
        return 1.0;
    }
    let a_bytes = a.data();
    let b_bytes = b.data();
    let total = a_bytes.len() / 4;
    if total == 0 {
        return 0.0;
    }
    let (a_px, b_px) = (a_bytes.as_chunks::<4>().0, b_bytes.as_chunks::<4>().0);
    let differing = a_px
        .iter()
        .zip(b_px.iter())
        .filter(|(p, q)| p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > 8))
        .count();
    alacer_core::num::usize_to_f32(differing) / alacer_core::num::usize_to_f32(total)
}

/// The a11y query builder (§11.5): the official query API, in plural form.
/// `harness.query(Role::Label).named("Item").count()` — every read routes
/// through the AccessKit tree, never the widget tree.
pub struct A11yQuery<'h> {
    harness: &'h TestHarness,
    role: Role,
    name: String,
}

impl A11yQuery<'_> {
    /// Narrows the query to nodes whose name contains `name`.
    #[must_use]
    pub fn named(self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// The resolved snapshot (settle first — the query reads what pass 9
    /// last published).
    fn tree(&self) -> Option<&alacer_core::passes::TreeSnapshot> {
        self.harness.pipeline().a11y_snapshot()
    }

    /// Whether any node matches.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.tree()
            .is_some_and(|tree| tree.contains(self.role, &self.name))
    }

    /// How many nodes match.
    #[must_use]
    pub fn count(&self) -> usize {
        self.tree()
            .map_or(0, |tree| tree.find_all(self.role, &self.name).len())
    }

    /// Every matching node, in tree order.
    #[must_use]
    pub fn all(&self) -> Vec<&alacer_core::passes::TreeNode> {
        self.tree()
            .map(|tree| tree.find_all(self.role, &self.name))
            .unwrap_or_default()
    }

    /// The first match — panics naming the query when nothing matches.
    ///
    /// # Panics
    ///
    /// Panics when no node matches.
    #[must_use]
    pub fn first(&self) -> &alacer_core::passes::TreeNode {
        self.tree()
            .and_then(|tree| tree.find(self.role, &self.name))
            .unwrap_or_else(|| {
                panic!(
                    "a11y query failed: no {} named {:?} is reachable",
                    self.role, self.name
                )
            })
    }

    /// The first match's bounds (click targeting).
    ///
    /// # Panics
    ///
    /// Panics when nothing matches or the node has no bounds.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        self.first().bounds.expect("matched node has bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::{TestHarness, pixel_diff_ratio};

    #[test]
    fn pixel_diff_ratio_is_symmetric_and_exact_on_identity() {
        let mut harness = TestHarness::new(8.0, 8.0);
        let a = harness.pixels();
        let b = harness.pixels();
        let diff = pixel_diff_ratio(&a, &b);
        assert!(diff <= 0.0, "identical pixmaps diff to zero, got {diff}");
    }
}
