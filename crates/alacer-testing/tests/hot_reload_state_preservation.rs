//! The §13.2 state-preservation hot-reload proof, end to end: "a subsecond
//! binary patch swaps code, never state — exactly one scoped re-render."
//!
//! The M4 exit-criteria suite pins the hot-patch contract at the journal
//! level (untouched subtrees do not enter the patch frame). This suite
//! proves the same contract at the tiers that suite does not reach,
//! through the production [`HotPatchDriver`]:
//!
//! 1. **The code swap is real.** Two *distinct* generation functions (not
//!    one closure parameterized by a cell) stand in for the shipped UI
//!    and the binary patch's swapped-in code: identical structure — the
//!    bound count label, the panel, the increment button — with one
//!    different static prop (the panel's fill color). The patch swaps the
//!    generator via `HotView::set_generation` (the jump table's detour),
//!    fires the patch trigger (the platform channel's wake-up seam —
//!    headless, `PatchChannel::fire` before a loop attaches is the
//!    documented silent drop), and the driver answers with exactly one
//!    [`HotView::regenerate`].
//! 2. **State never moves.** The counter signal keeps its value across
//!    the patch, the bound label keeps rendering it (the flux bridge
//!    survives — a click after two patches still counts), and the arena's
//!    pre-order node sequence is identical before and after: nothing was
//!    rebuilt, the reconciler's in-place mutation is the whole story. The
//!    report pins the minimal shape: `created: 0, removed: 0, patched: 1,
//!    moved: 0`.
//! 3. **The re-render is pixel-scoped.** The frame after the patch
//!    differs from the frame before it only inside the panel's bounds —
//!    the new static prop is on screen, and not one pixel outside the
//!    panel changed.

use alacer_core::arena::NodeId;
use alacer_core::color::Color;
use alacer_core::flux::RenderBinding;
use alacer_core::geom::{Point, Rect};
use alacer_core::num::u32_to_f32;
use alacer_core::widget::Role;
use alacer_testing::{HotPatchDriver, TestHarness};
use alacer_view::{HotView, PatchReport, PropValue, ViewNode};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The viewport every harness in this suite runs at.
const VIEWPORT: (f32, f32) = (320.0, 240.0);

/// The panel's fill in the shipped UI (generation A).
const GEN0_PANEL: Color = Color::rgb(64, 92, 148);

/// The panel's fill in the patched-in code (generation B) — a different
/// static prop, the only delta between the two generations.
const GEN1_PANEL: Color = Color::rgb(148, 64, 92);

/// The panel's fixed height (both generations).
const PANEL_HEIGHT: f32 = 48.0;

/// Generation A's view function: the shipped UI.
fn generation_a(
    binding: RenderBinding<String>,
    on_click: Rc<dyn Fn()>,
) -> impl FnMut() -> ViewNode {
    move || shipped_view(GEN0_PANEL, &binding, &on_click)
}

/// Generation B's view function: the patch's swapped-in code — same
/// structure, one different static prop. A separate closure object on
/// purpose: a hot patch replaces the closure itself, never
/// re-parameterizes one shared closure.
fn generation_b(
    binding: RenderBinding<String>,
    on_click: Rc<dyn Fn()>,
) -> impl FnMut() -> ViewNode {
    move || shipped_view(GEN1_PANEL, &binding, &on_click)
}

/// The shipped UI both generations render: a bound count label, the
/// panel, the increment button. Only `panel_color` differs between them.
/// The binding and the handler are captured from *outside* every
/// generation — the state-owner contract (§13.2 item 1): signals and
/// bindings belong to the app, created once, never inside patched code.
fn shipped_view(
    panel_color: Color,
    binding: &RenderBinding<String>,
    on_click: &Rc<dyn Fn()>,
) -> ViewNode {
    ViewNode::element("column")
        .prop("gap", 8.0_f32)
        .child(ViewNode::element("label").prop("binding", binding.clone()))
        .child(
            ViewNode::element("panel")
                .prop("height", PANEL_HEIGHT)
                .prop("color", panel_color),
        )
        .child(ViewNode::element("button").prop("text", "Increment").prop(
            "on_click",
            PropValue::Handler(Rc::new(RefCell::new({
                let on_click = Rc::clone(on_click);
                move || on_click()
            }))),
        ))
}

/// The mounted proof rig: harness + driver + the state the patch must
/// preserve.
struct PatchProof {
    harness: TestHarness,
    driver: HotPatchDriver,
    count: alacer_core::FluxRwSignal<i32>,
    binding: RenderBinding<String>,
    on_click: Rc<dyn Fn()>,
    fires: Rc<Cell<u32>>,
}

impl PatchProof {
    fn mount() -> Self {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let count = harness.pipeline_mut().flux_mut().signal(0_i32);
        let binding = RenderBinding::new("Clicked 0 times".to_string());

        // The increment handler: one write batch per click (ADR-6), the
        // same signal every generation serves — state outlives patches.
        let on_click: Rc<dyn Fn()> = {
            Rc::new(move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
        };

        let hot = HotView::mount(
            harness.pipeline_mut(),
            default_registry(),
            generation_a(binding.clone(), Rc::clone(&on_click)),
        );

        // The patch trigger: the channel fire's seam — one fire per
        // delivered patch, counted so the suite can pin the
        // one-fire-one-answer discipline.
        let fires = Rc::new(Cell::new(0_u32));
        let trigger_fires = Rc::clone(&fires);
        let driver = HotPatchDriver::mount(
            hot,
            Box::new(move || trigger_fires.set(trigger_fires.get() + 1)),
        );

        // The flux bridge drives the bound label across generations: one
        // batch ⇒ one label repaint, the M0 seam unchanged by patches.
        let label = bound_label_node(&harness);
        harness.pipeline_mut().flux_mut().bind_value(
            count,
            label,
            &binding,
            alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
            |n: &i32| format!("Clicked {n} times"),
        );

        harness.settle();
        Self {
            harness,
            driver,
            count,
            binding,
            on_click,
            fires,
        }
    }

    /// Delivers one patch through the driver: swaps the generation (the
    /// code), fires the trigger (the channel), answers with exactly one
    /// scoped re-render, settles one frame.
    fn deliver(&mut self, next: Generation) -> PatchReport {
        let (binding, on_click) = (self.binding.clone(), Rc::clone(&self.on_click));
        let generation: Box<dyn FnMut() -> ViewNode> = match next {
            Generation::A => Box::new(generation_a(binding, on_click)),
            Generation::B => Box::new(generation_b(binding, on_click)),
        };
        let report = self
            .driver
            .deliver_patch(self.harness.pipeline_mut(), |hot| {
                hot.set_generation(generation);
            });
        self.harness.settle();
        report
    }

    /// The count signal's current value — the state a patch must
    /// preserve.
    fn count_value(&self) -> i32 {
        self.count.try_get_untracked().unwrap_or_default()
    }

    /// The bound label's resolved a11y name — what a screen reader sees.
    fn label_text(&self) -> String {
        let node = bound_label_node(&self.harness);
        self.harness
            .pipeline()
            .arena()
            .widget(node)
            .label()
            .resolve()
    }

    /// The panel node (the one Decoration in the tree).
    fn panel(&self) -> NodeId {
        let mut found = None;
        self.harness
            .pipeline()
            .arena()
            .for_each_pre_order(&mut |id, slot| {
                if found.is_none() && slot.widget().role() == Role::Decoration {
                    found = Some(id);
                }
            });
        found.expect("the view mounted a panel")
    }

    /// The arena's pre-order node sequence — the identity fingerprint a
    /// patch must leave untouched (nothing rebuilt, nothing moved).
    fn node_ids(&self) -> Vec<NodeId> {
        let mut ids = Vec::new();
        self.harness
            .pipeline()
            .arena()
            .for_each_pre_order(&mut |id, _slot| ids.push(id));
        ids
    }

    /// The rendered frame through the harness's own pipeline + glyph
    /// source.
    fn frame(&mut self) -> tiny_skia::Pixmap {
        self.harness.pixels()
    }
}

/// Which generation is mounted — the two code objects the patches swap.
#[derive(Clone, Copy)]
enum Generation {
    /// The shipped UI.
    A,
    /// The patched-in code.
    B,
}

/// The bound count label: the first Label-role node whose resolved name
/// starts with "Clicked" (the panel's label never does).
fn bound_label_node(harness: &TestHarness) -> NodeId {
    let mut found = None;
    harness
        .pipeline()
        .arena()
        .for_each_pre_order(&mut |id, slot| {
            if found.is_none()
                && slot.widget().role() == Role::Label
                && slot.widget().label().resolve().starts_with("Clicked")
            {
                found = Some(id);
            }
        });
    found.expect("the view mounted a bound count label")
}

/// One pixel's RGBA from a pixmap (nearest pixel to `point`).
fn sample(pixmap: &tiny_skia::Pixmap, point: Point) -> [u8; 4] {
    let x = usize::try_from(alacer_core::num::f32_to_i32(point.x).max(0))
        .unwrap_or(0)
        .min(usize::try_from(pixmap.width().saturating_sub(1)).unwrap_or(0));
    let y = usize::try_from(alacer_core::num::f32_to_i32(point.y).max(0))
        .unwrap_or(0)
        .min(usize::try_from(pixmap.height().saturating_sub(1)).unwrap_or(0));
    let index = y * usize::try_from(pixmap.width()).unwrap_or(0) * 4 + x * 4;
    let bytes = pixmap.data();
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}

/// Counts pixels whose channel deltas exceed 8/255 (the
/// `pixel_diff_ratio` tolerance) whose pixel center lies inside `region`.
fn differing_inside(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap, region: &Rect) -> usize {
    differing_where(a, b, |x, y| {
        region.contains(Point::new(u32_to_f32(x), u32_to_f32(y)))
    })
}

/// Counts differing pixels outside `region`, grown by `margin` pixels on
/// every side (the AA slack the patched panel's own edges may touch).
fn differing_outside(
    a: &tiny_skia::Pixmap,
    b: &tiny_skia::Pixmap,
    region: &Rect,
    margin: f32,
) -> usize {
    differing_where(a, b, |x, y| {
        !region
            .inflate(margin, margin)
            .contains(Point::new(u32_to_f32(x), u32_to_f32(y)))
    })
}

/// The pixel-walking core of the two counters above.
fn differing_where(
    a: &tiny_skia::Pixmap,
    b: &tiny_skia::Pixmap,
    keep: impl Fn(u32, u32) -> bool,
) -> usize {
    let (width, height) = (a.width(), a.height());
    let stride = usize::try_from(width).unwrap_or(0) * 4;
    let a_bytes = a.data();
    let b_bytes = b.data();
    let mut differing = 0_usize;
    for y in 0..height {
        for x in 0..width {
            if !keep(x, y) {
                continue;
            }
            let index =
                usize::try_from(y).unwrap_or(0) * stride + usize::try_from(x).unwrap_or(0) * 4;
            let (p, q) = (&a_bytes[index..index + 4], &b_bytes[index..index + 4]);
            if p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > 8) {
                differing += 1;
            }
        }
    }
    differing
}

#[test]
fn a_patch_swaps_the_static_prop_and_never_the_state() {
    let mut proof = PatchProof::mount();

    // State accumulates before the patch: two clicks, the bound label
    // carries the count, the panel wears the shipped color.
    proof.harness.press(Role::Button, "Increment");
    proof.harness.press(Role::Button, "Increment");
    proof.harness.settle();
    assert_eq!(proof.count_value(), 2, "two clicks landed in the signal");
    assert_eq!(proof.label_text(), "Clicked 2 times");

    let ids_before = proof.node_ids();
    let panel = proof.panel();
    let panel_bounds = proof.harness.bounds(panel);
    let frame_before = proof.frame();
    let center = Point::new(
        panel_bounds.origin.x + panel_bounds.size.width * 0.5,
        panel_bounds.origin.y + panel_bounds.size.height * 0.5,
    );
    assert_eq!(
        sample(&frame_before, center),
        [GEN0_PANEL.r, GEN0_PANEL.g, GEN0_PANEL.b, GEN0_PANEL.a],
        "the panel wears the shipped color before the patch"
    );

    // The patch: generation B (the swapped-in code), delivered through
    // the driver — one trigger fire, one scoped re-render.
    let report = proof.deliver(Generation::B);
    assert_eq!(
        report,
        PatchReport {
            created: 0,
            removed: 0,
            patched: 1,
            moved: 0,
        },
        "exactly the one visual prop patched in place, nothing else"
    );
    assert_eq!(
        u32::try_from(proof.driver.patches_delivered()).unwrap_or(u32::MAX),
        proof.fires.get(),
        "one trigger fire per delivered patch"
    );

    // State never moved: the signal keeps its value, the label keeps
    // rendering it, and the tree's identity is untouched.
    assert_eq!(proof.count_value(), 2, "the signal survived the patch");
    assert_eq!(
        proof.label_text(),
        "Clicked 2 times",
        "the bound label still renders the preserved state"
    );
    assert_eq!(
        proof.node_ids(),
        ids_before,
        "the pre-order node sequence is identical: nothing rebuilt"
    );
    assert_eq!(
        proof.driver.root_id(),
        ids_before[0],
        "the root never moved"
    );

    // The new static prop is on screen — and the re-render is
    // pixel-scoped: nothing outside the panel's bounds changed.
    let frame_after = proof.frame();
    assert_eq!(
        sample(&frame_after, center),
        [GEN1_PANEL.r, GEN1_PANEL.g, GEN1_PANEL.b, GEN1_PANEL.a],
        "the panel wears the patched-in color after the patch"
    );
    assert!(
        differing_inside(&frame_before, &frame_after, &panel_bounds) > 0,
        "the panel's pixels really changed"
    );
    assert_eq!(
        differing_outside(&frame_before, &frame_after, &panel_bounds, 2.0),
        0,
        "not one pixel outside the panel changed: the re-render is scoped, \
         never a full rebuild"
    );
}

#[test]
fn patches_back_and_forth_keep_counting() {
    let mut proof = PatchProof::mount();
    proof.harness.press(Role::Button, "Increment");
    proof.harness.settle();

    let report = proof.deliver(Generation::B);
    assert_eq!(report.patched, 1, "A → B patches the one prop");
    let report = proof.deliver(Generation::A);
    assert_eq!(
        report,
        PatchReport {
            created: 0,
            removed: 0,
            patched: 1,
            moved: 0,
        },
        "B → A patches the same prop back, still nothing rebuilt"
    );
    assert_eq!(proof.count_value(), 1, "the count survived both patches");

    // The flux bridge survived both patches: the next click still drives
    // the bound label — the reactive seams a patch must never disturb.
    proof.harness.press(Role::Button, "Increment");
    proof.harness.settle();
    assert_eq!(proof.count_value(), 2);
    assert_eq!(proof.label_text(), "Clicked 2 times");
    assert_eq!(
        u32::try_from(proof.driver.patches_delivered()).unwrap_or(u32::MAX),
        proof.fires.get(),
        "two deliveries, two fires, two answers — the lockstep held"
    );
}
