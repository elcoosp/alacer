# Migrating from Iced to Alacer

> Scope note (honesty rule): this guide maps Iced **0.13/0.14-era** concepts
> (`Application`/`Task`, `Element`, `Subscription`) onto Alacer **0.13.0**. Iced
> renamed its shell traits across releases; treat exact signatures as "check
> your version", the *concepts* as the durable part.

Iced is the closest relative in the audited family: retained widget trees,
Elm-style message passing, a real layout engine. Alacer keeps the retained
tree and the message discipline but replaces the *update loop* with a single
invalidation currency: instead of routing `Message`s into `update` and
re-running `view` to diff, you write signals and the per-node `Change` mask
(`LAYOUT | PAINT | A11Y | FOCUS | STRUCTURE`) routes exactly the subsystems
that need to run. The migration is mostly deleting plumbing.

## Mental model

| Iced | Alacer |
|---|---|
| Elm loop: `update(msg) -> Task` + `view() -> Element` re-diffed | Signals + bindings: write once, masks route the passes |
| `Message` enum you grow forever | No app-level enum; handlers are closures over signal handles |
| `view(&self)` returns a fresh `Element` tree each call | `rsx!` views reconcile in place (patch-in-place, keyed LIS) |
| Cost = full view diff per message | Cost = what the mask says changed (A1) |
| `Task<Message>` for async/commands | `Flux` effects + the platform loop; no command type |

## App shell

Iced:

```rust
#[derive(Default)]
enum Counter { #[default] Loading, Ready { count: i32 } }

#[derive(Debug, Clone)]
enum Message { Increment, Reset }

impl Application for Counter {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Task<Message>) {
        (Self::Ready { count: 0 }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                if let Self::Ready { count } = self { *count += 1; }
                Task::none()
            }
            Message::Reset => {
                if let Self::Ready { count } = self { *count = 0; }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Self::Loading => text("…").into(),
            Self::Ready { count } => column![
                text(format!("Clicked {count} times")).size(20),
                button("Increment").on_press(Message::Increment),
                button("Reset").on_press(Message::Reset),
            ]
            .spacing(12)
            .padding(16)
            .into(),
        }
    }
}
```

Alacer — the same program with the two `match` arms collapsed into two
closures:

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

    let increment = count;
    let reset = count;
    let _ = pipeline.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                alacer_core::flux::Flux::batch_of(|| {
                    let current = increment.try_get_untracked().unwrap_or_default();
                    increment.set(current + 1);
                });
            })
            .boxed(),
    );
    let _ = pipeline.add_child(
        root,
        Button::new("Reset")
            .on_click(move || alacer_core::flux::Flux::batch_of(|| reset.set(0_i32)))
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

The `Message` enum's job (naming *why* state changed) is taken over by the
mask + the Causality inspector's provenance writes when you need the history.
The runnable canon for everything mapped here lives in
[`crates/alacer/examples/`](../crates/alacer/examples/) — start with
[`counter`](../crates/alacer/examples/counter.rs) (the shell above),
[`gallery`](../crates/alacer/examples/gallery.rs) (the widget table), and
[`text_input`](../crates/alacer/examples/text_input.rs) (IME + focus).

## Widget mapping

| Iced | Alacer | Notes |
|---|---|---|
| `text("…")` | `Label::new("…")` / `Label::bound(binding)` | `.font_size`, `.color`; bound labels re-format from signals |
| `button("…").on_press(m)` | `Button::new("…").on_click(closure)` | no message routing; the closure writes signals |
| `text_input(placeholder, &value).on_input(m)` | `TextInput::new(name).on_changed(\|v\| …)` | IME contract built in (UTF-16 caret, preedit, commit); `.secure()` |
| `column![…].spacing(x).padding(x)` | `Column::new().gap(x).padding(x)` | also `.grow`, `.align`, `.justify` |
| `row![…].spacing(x)` | `Row::new().gap(x)` | |
| `scrollable(content)` | `List::new(height)` | translation-only scrolling, zero measure per tick (§8.2); content height via the `ListHandle` |
| `container(…)` / `Space::new(w, h)` | `Panel::new(height, color)` / `Spacer::new(size)` | |
| `theme::Palette` / custom `Theme` | `alacer::widgets::theme` constants | `SURFACE`, `PANEL`, `ACCENT`, `TEXT_MUTED`, `BORDER`, … |

## Reactivity

Iced's `update` is where logic lives; state is a struct you mutate and then
`view` re-derives. In Alacer, state lives in flux signals; *effects queue
masks* and never mutate widgets (axiom A1):

- `pipeline.flux().signal(value)` — a `Copy` handle you close over.
- All writes for one interaction go inside `Flux::batch_of(|| …)`: N
  synchronous writes coalesce into exactly one effect rerun, one frame —
  Iced's "one message = one update + full re-diff" becomes "one batch = one
  cycle", test-enforced (ADR-6).
- `bind_value(signal, node, &binding, masks, format)` is the declarative
  `view` fragment: it keeps a widget slot in sync with a signal. The masks
  say which passes care (`PAINT`-only for a color swap, `LAYOUT | PAINT`
  when size follows).
- Async work (Iced `Task`/`Subscription`): write the completion into a
  signal from your executor thread — the flux bridge and the patch channel
  wake the loop; the next tick coalesces everything that arrived.

## Layout

Iced uses its own flex-like layouter with `Length::Fill`/`Shrink`; Alacer
runs Taffy through a persistent tree. Concept mapping:

- `Length::Fill` → `.grow(1.0)` (and `Length::Fixed(px)` → `width`/`height`
  style setters).
- `spacing` → `gap`; container `padding` → style `padding`.
- Alignment (`AlignX`/`AlignY`) → `CrossAxisAlignment`/`MainAxisAlignment`
  via `.align`/`.justify`.
- Iced recomputes layout through its tree per render; Alacer's relayout is
  bounded by the dirty set's closest common ancestor (ADR-0008) — the W1
  churn workload measures ≥ 2× (published: ~14×) versus full relayout, and
  the baseline is a contract (`benchmarks/`).

## Input and events

Iced routes keyboard input through `Subscription`s (`keyboard::on_key_press`)
and widget-level `on_press`. Alacer routes events through the pass contract:
pointer events capture→bubble (§10.1); key/IME events go to the *focused*
widget, Tab cycles, and the pipeline derives the `ImeRequest` (purpose +
caret anchor) so the platform IME works without per-widget plumbing.
`TextInput` is the reference implementation of that contract (M3).

## Accessibility

Iced gates accessibility behind the `a11y` feature and a
`text_input::Id`-based focus model; labelers are opt-in. Alacer makes a11y
non-optional at the type level: `role()` and `label()` are required `Widget`
trait methods (ADR-5) — an unnameable widget does not compile — and AccessKit
node ids are the same id-path hashes as hit-testing. Updates are dirty-node
deltas (p99 < 0.5 ms @ 1k nodes, test-enforced). Porting: expect to *remove*
feature-flag plumbing and get the CI walker (`assert_all_named`) for free.

## Testing

Iced 0.14 ships a simulator-style `iced_test` harness around `view` output.
Alacer's `alacer-testing` runs the *whole pipeline* headlessly — the real
pass system, software raster, a11y tree — with a11y queries as the official
API (ADR-10):

```rust
let mut harness = TestHarness::new(320.0, 240.0);
let root = harness.mount(Column::new().gap(12.0).boxed());
harness.add_child(root, Button::new("Increment").boxed());
harness.tick();

assert!(harness.query(Role::Button).named("Increment").exists());
harness.press(Role::Button, "Increment");
harness.snapshot("counter_after_click");   // blessed PNG, pixel-diffed
```

Bless with `ALACER_BLESS=1`; the corpus lives in
`crates/alacer-testing/blessed/` (36 styled scenarios — 38 blessed PNGs including the M0 pair).

## Interop limits

- **No `Element`/custom widget-compiler layer.** Iced's `Element<Message,
  Theme, Renderer>` generics do not exist; custom widgets implement the
  `Widget` trait (paint via `PaintCtx`, required `role`/`label`).
- **No `Task` type.** Async effects write signals; there is no command
  cancellation vocabulary yet — wrap your own join handles.
- **Themes are constants, not a theme system.** `iced::Theme`'s palette
  switching has no equivalent in 0.12; restyle by writing style properties.
- **One window, no multi-viewport.** Iced's `window::open` multi-window
  support has no Alacer counterpart yet.
