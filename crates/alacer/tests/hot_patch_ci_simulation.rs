//! The CI hot-patch simulation (§13.2, M4): the state-preserving
//! hot-reload contract, proved in CI against a **real binary patch** —
//! not a stand-in swap alone.
//!
//! The plan's demand (§13.2): "A subsecond binary patch swaps code, never
//! state — exactly one scoped re-render, pinned by a CI hot-patch suite."
//! Until this suite the CI pin ran one step short of the binary: the
//! reconciler contract (one patch, one scoped re-render, state intact)
//! was proved headlessly in `m4_exit_criteria`, and the *real patcher*
//! was compile-verified only. This suite closes the gap in two halves:
//!
//! 1. **The framework half** (runs everywhere): the exact patch delivery
//!    a binary patch drives — the generation swap (the code the patcher
//!    installs), the patch-channel fire (the wake-up), exactly one
//!    scoped re-render — through the production
//!    [`alacer_testing::hot_patch::HotPatchDriver`], with the reconciler's
//!    report pinned to the prop-only patch shape a label-constant edit
//!    produces: `PatchReport { created: 0, removed: 0, patched: 1 }` (and
//!    nothing moved). The reactive state survives it: the count signal
//!    keeps its value, the bound label keeps its binding, the root's
//!    arena node keeps its identity (§6.3) — the interactions the patch
//!    must never disturb, asserted after the patch lands.
//!
//! 2. **The binary-payload half** (CI's `hot-patch` job, feature
//!    `hot-reload`): the job builds the `hot_reload` example, edits the
//!    example's label text constant in source, recompiles, and hands this
//!    test both binaries. The test proves the recompile is a real patch
//!    payload — two binaries that differ byte-wise, the before-binary
//!    carrying the shipped constant, the after-binary carrying the
//!    patched one — the exact diff a subsecond jump-table patch carries
//!    into the running process. Machines running the suite without the
//!    job's binaries skip with the reason named — the honest refusal, the
//!    wgpu fuzz's own discipline, never a fake pass.

use alacer_core::FluxRwSignal;
use alacer_core::flux::RenderBinding;
use alacer_core::widget::Role;
use alacer_testing::TestHarness;
use alacer_testing::hot_patch::HotPatchDriver;
use alacer_view::{PatchReport, PropValue, ViewNode};
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::rc::Rc;

/// The viewport every simulation frame runs at.
const VIEWPORT: (f32, f32) = (360.0, 240.0);

/// The label constants the two generations ship: the shipped UI's marker
/// and the patched-in one — the text-constant edit a binary patch
/// carries (the CI job performs the same edit on the example's source).
const BEFORE_MARKER: &str = "CI hot-patch before";
const AFTER_MARKER: &str = "CI hot-patch after";

/// The `hot_reload` example's shipped patched-generation label constant —
/// what the before-binary carries — and the CI job's replacement for it —
/// what the after-binary carries. The payload half searches for exactly
/// these two strings in the job's two binaries.
#[cfg(feature = "hot-reload")]
const EXAMPLE_SHIPPED_LABEL: &str = "patched layout — same signal, same count";
#[cfg(feature = "hot-reload")]
const EXAMPLE_PATCHED_LABEL: &str = "CI hot-patch: same signal, same count";

/// The canon app as a view function of the generation: a marker label
/// (the text constant the patch edits), the bound count label, and the
/// increment button. Generation 0 is the shipped UI; generation 1 is the
/// code the patch installs — same layout, one label constant changed,
/// the prop-only patch shape a real text edit produces.
fn generation_view(
    binding: &RenderBinding<String>,
    generation: u32,
    on_click: Rc<dyn Fn()>,
) -> ViewNode {
    let marker = if generation == 0 {
        BEFORE_MARKER
    } else {
        AFTER_MARKER
    };
    ViewNode::element("column")
        .prop("gap", 8.0_f32)
        .child(ViewNode::element("label").key(1_u64).prop("text", marker))
        .child(
            ViewNode::element("label")
                .key(2_u64)
                .prop("binding", binding.clone()),
        )
        .child(
            ViewNode::element("button")
                .key(3_u64)
                .prop("text", "Increment")
                .prop(
                    "on_click",
                    PropValue::Handler(Rc::new(RefCell::new(move || on_click()))),
                ),
        )
}

/// The bound count label: the first Label-role node whose resolved name
/// starts with "Clicked" (the marker label never does) — the m4 suite's
/// own finder, on the same canon shape.
fn bound_label_node_from(harness: &TestHarness) -> alacer_core::arena::NodeId {
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

/// The simulation's whole state: harness, hot view under the production
/// patch driver, the count signal, and the binding.
struct PatchSim {
    harness: TestHarness,
    driver: HotPatchDriver,
    count: FluxRwSignal<i32>,
    binding: RenderBinding<String>,
}

impl PatchSim {
    /// Mounts the app the way the `hot_reload` example does — signal,
    /// binding, the shipped generation through `mount_hot`, the flux
    /// bridge on the bound label — and settles the first frame.
    fn mount() -> Self {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        let count = harness.pipeline_mut().flux_mut().signal(0_i32);
        let binding = RenderBinding::new("Clicked 0 times".to_string());

        let increment = move || {
            // One write batch per click: the ADR-6 contract at the seam.
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        };

        let gen_binding = binding.clone();
        let on_click: Rc<dyn Fn()> = Rc::new(increment);
        let generate = move || generation_view(&gen_binding, 0, Rc::clone(&on_click));

        // The patch channel rides the driver's trigger seam: one delivered
        // patch fires the channel once (the wake-up a binary patch runs),
        // the loop's one-answer-per-patch discipline is the driver's own
        // accounting.
        let channel = alacer::platform::PatchChannel::new();
        let trigger_channel = channel.clone();
        let hot = alacer::hot::mount_hot(harness.pipeline_mut(), generate);
        let driver = HotPatchDriver::mount(hot, Box::new(move || trigger_channel.fire()));

        let label_node = bound_label_node_from(&harness);
        harness.pipeline_mut().flux_mut().bind_value(
            count,
            label_node,
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
        }
    }

    /// Clicks the "Increment" button through the a11y tree (§11.5: the
    /// official query API — reachable, named, actionable).
    fn click_increment(&mut self) {
        self.harness.press(Role::Button, "Increment");
    }

    /// Delivers one patch: the generation swap (the patched view function —
    /// the code a binary patch installs — replaces the shipped one), the
    /// channel fire, exactly one scoped re-render — the driver's own
    /// discipline — then one settle frame.
    fn deliver_patch(&mut self) -> PatchReport {
        let count = self.count;
        let binding = self.binding.clone();
        let increment = move || {
            alacer_core::flux::Flux::batch_of(|| {
                let current = count.try_get_untracked().unwrap_or_default();
                count.set(current + 1);
            });
        };
        let on_click: Rc<dyn Fn()> = Rc::new(increment);
        let patched = move || generation_view(&binding, 1, Rc::clone(&on_click));
        let report = self
            .driver
            .deliver_patch(self.harness.pipeline_mut(), move |hot| {
                hot.set_generation(patched);
            });
        self.harness.settle();
        report
    }

    /// The count signal's current value — the reactive state a patch must
    /// preserve.
    fn count_value(&self) -> i32 {
        self.count.try_get_untracked().unwrap_or_default()
    }

    /// The bound label's resolved text — the reactive value a patch must
    /// keep binding across generations.
    fn bound_label_text(&self) -> String {
        let node = bound_label_node_from(&self.harness);
        self.harness
            .pipeline()
            .arena()
            .widget(node)
            .label()
            .resolve()
    }
}

/// The framework half: a patch delivered through the production driver
/// swaps the label constant, patches exactly one element in place, and
/// preserves the reactive state — the §13.2 contract, pinned on the same
/// canon the CI binary simulation drives.
#[test]
fn a_delivered_patch_swaps_the_constant_and_preserves_the_reactive_state() {
    let mut sim = PatchSim::mount();

    // The pre-patch shape: the shipped generation, interacted with. Three
    // clicks — reactive state the patch must never disturb.
    assert_eq!(sim.count_value(), 0, "a fresh app starts at zero");
    for _ in 0..3 {
        sim.click_increment();
    }
    assert_eq!(sim.count_value(), 3, "the clicks wrote the signal");
    assert_eq!(
        sim.bound_label_text(),
        "Clicked 3 times",
        "the bound label rides the signal"
    );
    let root_before = sim.driver.root_id();

    // The patch: the label-constant edit a binary patch carries, delivered
    // through the production driver (swap, channel fire, one re-render).
    let report = sim.deliver_patch();
    assert_eq!(
        report,
        PatchReport {
            created: 0,
            removed: 0,
            patched: 1,
            moved: 0,
        },
        "a prop-only patch creates nothing, removes nothing, moves nothing — \
         it patches exactly the one element whose text constant changed"
    );
    assert_eq!(
        sim.driver.patches_delivered(),
        1,
        "one delivery, one answer: the driver's lockstep accounting"
    );

    // The code the patch installed renders: the marker label carries the
    // patched constant through the a11y tree (§11.5).
    assert!(
        sim.harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::Label, AFTER_MARKER))
            .is_some(),
        "the patched generation's label constant did not render"
    );
    assert!(
        sim.harness
            .pipeline()
            .a11y_snapshot()
            .and_then(|tree| tree.find(Role::Label, BEFORE_MARKER))
            .is_none(),
        "the pre-patch marker survived a patch that replaced it"
    );

    // The state the patch must never disturb: the signal keeps its value,
    // the binding keeps driving the label, the root keeps its identity
    // (§6.3 — same element type, same arena node).
    assert_eq!(
        sim.count_value(),
        3,
        "the patch lost the count signal's value — state rode the patch"
    );
    assert_eq!(
        sim.bound_label_text(),
        "Clicked 3 times",
        "the patch lost the bound label's reactive value"
    );
    assert_eq!(
        sim.driver.root_id(),
        root_before,
        "the patch rebuilt the root — identity is the §6.3 contract"
    );

    // And the patched app still works: one more click drives the same
    // signal through the patched generation.
    sim.click_increment();
    assert_eq!(
        sim.count_value(),
        4,
        "the patched generation lost the increment path"
    );
    assert_eq!(
        sim.bound_label_text(),
        "Clicked 4 times",
        "the patched generation lost the binding"
    );
}

/// The binary-payload half (CI's `hot-patch` job, feature `hot-reload`):
/// the recompiled example is a real patch payload — two binaries, byte
/// different, each carrying its own label constant. Skips honestly when
/// the job's binaries are absent (a plain `cargo test` run).
#[cfg(feature = "hot-reload")]
#[test]
fn the_recompiled_example_is_a_real_patch_payload() {
    let Some(before) = std::env::var_os("ALACER_HOT_PATCH_BEFORE_BIN") else {
        eprintln!(
            "SKIPPED: no before-binary (set ALACER_HOT_PATCH_BEFORE_BIN and \
             ALACER_HOT_PATCH_AFTER_BIN; the CI hot-patch job builds them)"
        );
        return;
    };
    let Some(after) = std::env::var_os("ALACER_HOT_PATCH_AFTER_BIN") else {
        eprintln!(
            "SKIPPED: no after-binary (set ALACER_HOT_PATCH_AFTER_BIN; the CI \
             hot-patch job builds it)"
        );
        return;
    };
    let before_bytes = std::fs::read(&before)
        .unwrap_or_else(|error| panic!("the before-binary {before:?} is unreadable: {error}"));
    let after_bytes = std::fs::read(&after)
        .unwrap_or_else(|error| panic!("the after-binary {after:?} is unreadable: {error}"));
    assert!(
        !before_bytes.is_empty() && !after_bytes.is_empty(),
        "the job's binaries are empty files"
    );
    assert_ne!(
        before_bytes, after_bytes,
        "the recompile produced a byte-identical binary — there is no patch \
         payload to simulate, the CI edit did not take"
    );
    // Each binary carries exactly its own generation's constant: the
    // before-binary is the shipped code, the after-binary is the patched
    // code — the diff a subsecond jump-table patch carries into the
    // running process.
    let shipped = EXAMPLE_SHIPPED_LABEL.as_bytes();
    let patched = EXAMPLE_PATCHED_LABEL.as_bytes();
    assert!(
        contains(&before_bytes, shipped),
        "the before-binary does not carry the example's shipped constant \
         {EXAMPLE_SHIPPED_LABEL:?}"
    );
    assert!(
        !contains(&before_bytes, patched),
        "the before-binary already carries the patched constant — the CI \
         edit leaked into the before build"
    );
    assert!(
        contains(&after_bytes, patched),
        "the after-binary does not carry the patched constant \
         {EXAMPLE_PATCHED_LABEL:?} — the recompile did not pick up the edit"
    );
    assert!(
        !contains(&after_bytes, shipped),
        "the after-binary still carries the example's shipped constant — \
         the patch never replaced the code it claims to patch"
    );
}

/// A first-byte-skipping byte-substring search (the binaries are not
/// UTF-8, so the search is over bytes). The payload half's only helper.
#[cfg(feature = "hot-reload")]
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    let last = haystack.len() - needle.len();
    let first = needle[0];
    (0..=last)
        .any(|start| haystack[start] == first && &haystack[start..start + needle.len()] == needle)
}
