# Migrating from egui to Alacer

> Scope note (honesty rule): this guide maps egui **0.31-era** concepts
> (`eframe`, immediate-mode repaints, `egui_kittest`) onto Alacer **0.16.0**.
> egui moves fast; treat exact signatures as "check your version", the
> *concepts* as the durable part.

Alacer and egui make opposite bets: egui re-runs your whole UI closure every
frame and rebuilds widgets as it goes; Alacer keeps widgets alive in a
retained arena and asks one question per frame — *what changed?* — answered
by a per-node `Change` mask (`LAYOUT | PAINT | A11Y | FOCUS | STRUCTURE`)
that every subsystem reads. The migration is mostly translating
"re-run everything, diff nothing" into "write a signal, queue a mask, pay
for the delta".

## Mental model

| egui | Alacer |
|---|---|
| Immediate mode: `update` re-runs each frame; widgets are re-created | Retained mode: widgets live in an arena; views reconcile by key |
| State in your `App` struct, read by closure every frame | State in flux signals; effects queue masks, never mutate widgets |
| Cost scales with everything drawn | Cost scales with what changed (the one invalidation currency, A1) |
| Repaint driven by `ctx.request_repaint()` | One write batch ⇒ at most one layout+paint cycle (ADR-6, test-enforced) |

The practical consequence: in egui you never think about invalidation, and
you always pay full-frame costs. In Alacer you state *what* changed once and
the pipeline routes it — incremental Taffy relayout, damage-bounded paint,
dirty-node-only AccessKit updates.

## App shell

egui (eframe):

```rust
fn main() -> eframe::Result {
    eframe::run_native(
        "App",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(MyApp { count: 0 }))),
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Increment").clicked() {
                self.count += 1;
            }
            ui.label(format!("Clicked {} times", self.count));
        });
    }
}
```

Alacer:

```rust
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};

fn main() {
    let mut pipeline = alacer::pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());
    let label = pipeline.add_child(root, Label::bound(binding.clone()).boxed());
    let _ = pipeline.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
            .boxed(),
    );
    pipeline.flux_mut().bind_value(
        count,
        label,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    let _ = alacer::run_window(pipeline, "App", (360.0, 200.0), Default::default());
}
```

Or write the tree as a view function with `rsx!` (`alacer::rsx!`, mounted
with `alacer::mount_view`) — the closest thing to "the closure" you had in
egui, except it *reconciles* instead of rebuilding. See
[`counter_view`](../crates/alacer/examples/counter_view.rs).

## Widget mapping

| egui | Alacer | Notes |
|---|---|---|
| `ui.label(text)` | `Label::new(text)` / `Label::bound(binding)` | bound labels re-format from a signal; `.font_size`, `.color` |
| `ui.button("…")` | `Button::new("…").on_click(|| …)` | press/keyboard focus handled for you; `.colors(rest, pressed)` |
| `ui.text_edit_singleline(&mut s)` | `TextInput::new(name).on_changed(\|v\| …)` | IME contract built in (UTF-16 caret, preedit); secure mode: `.secure()` |
| `ui.vertical` / `ui.horizontal` | `Column::new()` / `Row::new()` | `.gap`, `.padding`, `.grow`, `.align`, `.justify` |
| `ui.add_space(x)` | `Spacer::new(x)` | fixed filler along the main axis |
| `egui::ScrollArea::vertical()` | `List::new(height)` | translation-only scrolling: zero measure calls per tick (§8.2); `set_content_height` via the handle |
| `egui::Frame` / panels | `Panel::new(height, color)` | solid decoration |
| `egui::Visuals` / `Style` | `alacer::widgets::theme` constants | `SURFACE`, `PANEL`, `ACCENT`, `TEXT_MUTED`, … |

## Reactivity

egui's model is "no reactivity — just re-run". Alacer's is a reactive graph
with a strict write path:

- `pipeline.flux().signal(value)` creates a signal (a handle; `Copy`).
- Handlers write inside one `Flux::batch_of(|| …)` — N writes coalesce into
  one effect rerun, one frame. This is the falsifiable scheduling contract
  (ADR-6) your egui instincts never needed.
- `bind_value(signal, node, &binding, masks, format)` connects a signal to a
  widget slot. The `masks` argument is you declaring what the change means
  (`LAYOUT | PAINT`, or just `PAINT`) — the currency every pass reads.
- Effects never mutate widgets directly (axiom A1). If you find yourself
  wanting `&mut Label` from a handler, that's an egui habit: reach for a
  signal + binding instead, or queue a mutation through the arena's
  `queue_mutation`.

The `Causality` inspector (`alacer-inspect`) answers "who wrote this signal,
why did this render" — the questions immediate mode cannot ask.

## Layout

egui lays out in a single immediate pass with implicit constraints; you hint
with `Layout`, wraps, and `Sense`. Alacer uses Taffy (flexbox) through a
persistent tree: containers carry a `Style` (gap, padding, grow, size,
alignment), leaf widgets expose a `measure_spec`. Two habits change:

1. You *declare* the style on the widget (`.gap(12.0)`, `.grow(1.0)`) rather
   than arranging calls inside a closure.
2. Layout is incremental. Relayout is bounded by the closest common ancestor
   of the dirty set (the root candidate, ADR-0008) — big trees do not
   relayout wholesale because one label changed. Scrolling lists translate
   without re-measuring.

## Input and events

egui exposes `Response` objects (`clicked()`, `changed()`, `hovered()`) and
raw input via `ctx.input(...)`. Alacer routes events through the pass
contract: pointer events go capture→bubble (§10.1), key and IME events route
to the *focused* widget with Tab cycling, and the pipeline derives the
per-frame `ImeRequest` (purpose + caret anchor) for the windowing system.
Widget handlers (`on_click`, `on_changed`) cover the common cases; custom
widgets implement `Widget::on_pointer_event` / `on_key_event` /
`on_ime_event` and return `EventResponse`.

## Accessibility

egui integrates AccessKit lazily — widgets register when read, and
`egui_kittest` is the recommended harness. Alacer makes accessibility a
*type-level* contract: `role()` and `label()` are required `Widget` methods
with no default — a widget that cannot be named does not compile (ADR-5).
Updates are dirty-node-only deltas, and the official test API *is* the a11y
API: you query by role and name, press through the AccessKit default action.
Porting an egui app, expect to *delete* a11y fixes you hand-patched.

## Testing

egui: `egui_kittest` snapshots + harness clicks. Alacer: the headless
`TestHarness` runs the real pipeline with the software rasterizer — no
windowing, no mocks (A4) — plus 38 blessed PNG snapshots (37 styled scenarios) of the canonical
scenarios. Porting notes:

```rust
let mut harness = TestHarness::new(320.0, 240.0);
let root = harness.mount(Column::new().gap(12.0).boxed());
harness.add_child(root, Button::new("Increment").on_click(move || { /* … */ }).boxed());
harness.tick();

assert!(harness.query(Role::Button).named("Increment").exists());
harness.press(Role::Button, "Increment");   // a11y-driven query + press
harness.snapshot("my_scenario");            // blessed PNG, pixel-diffed
harness.assert_all_named();                 // the CI walker: no unnamed widgets
```

Kittest's snapshot-review workflow (`KITTEST_BLESS=1`) maps to
`ALACER_BLESS=1 cargo test -p alacer-testing`.

## Interop limits

Be honest about what does not translate 1:1:

- **No immediate escape hatch.** There is no `ui` closure to draw into;
  painting is the scene journal + damage ladder. Custom widgets implement
  the `Widget` trait and paint primitives (`PaintCtx`), not per-frame draws.
- **No per-pixel egui painting (`epaint`) equivalent.** Primitives are the
  scene vocabulary (quads, glyphs); arbitrary vector paths are a roadmap
  item, not a 0.12 feature.
- **Docking / multi-viewport.** egui's `egui_dock`, `egui_flex`, and
  multi-viewport support have no Alacer equivalent yet; one pipeline, one
  window.
- **State persistence.** egui apps persist via `egui::Storage`; Alacer state
  lives in your signals — persistence is your code, exactly as in egui
  custom apps.
