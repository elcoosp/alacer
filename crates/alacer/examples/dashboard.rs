//! The operations dashboard — the launch demo (§14.2's W3 story, told by
//! the compose tier): a live CPU/memory chart, a filter field narrowing a
//! service list, and a theme toggle, all riding the doors the platform
//! already ships. The example exists to prove three things on one screen:
//!
//! 1. **The compose tier animates** (the milestone's capability): the
//!    chart's bars fade by age through [`Compose`], the scan cursor is a
//!    record-time *translation* (a quad pushed at the origin, shifted by
//!    the compose state — never a layout move), and the LIVE badge pulses
//!    through `Label::set_compose` — the fade and the shift move with no
//!    re-measure, no style churn, no color rewrite.
//! 2. **Incremental layout + a11y stay honest under load**: the filter
//!    field's edits land through the mutate seam (`set_text` beside a
//!    `LAYOUT | PAINT | A11Y` request), the service rows re-text in
//!    place, and the tree says the new names the tick after.
//! 3. **The theme seam swaps live**: the button writes one flag, the
//!    patch hook answers through `PassPipeline::set_theme`, and the
//!    broadcast repaints every un-overridden role.
//!
//! The frame's driver is the patch channel, exactly as the hot-reload
//! runtime rides it (§13.2): a ticker thread advances the shared samples
//! and fires the channel from off-thread — one `Patch` event, one hook
//! call, one layout+paint cycle. The chart's cursor honors the OS's
//! reduced-motion fact (`ctx.reduced_motion()`) — motion itself is the
//! question the fact answers.
//!
//! ```sh
//! cargo run -p alacer --example dashboard            # the recordable window
//! cargo run -p alacer --example dashboard -- --headless
//! ```
//!
//! The `--headless` beat runs the *same* hook and the *same* pipeline for
//! 120 deterministic frames — no sleeps, no window (A4) — and asserts the
//! beats: the badge's fade moved, the chart recorded bars, the filter
//! narrowed the tree's names, the theme landed. CI runs the beat on every
//! pass; the screen recording needs only a display.
//!
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::Change;
use alacer_core::FluxRwSignal;
use alacer_core::NodeId;
use alacer_core::color::Color;
use alacer_core::geom::{Point, Rect, Size, Vector};
use alacer_core::num::{u32_to_f32, u64_to_f32};
use alacer_core::passes::PassPipeline;
use alacer_core::scene::{Compose, PaintCtx, Quad};
use alacer_core::theme::{Palette, ThemeSeam};
use alacer_core::widget::Widget as _;
use alacer_widgets::{BoxedWidget as _, Button, Canvas, Column, Label, Row, TextInput};
use reactive_graph::traits::Set as _;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// The chart's history: one slot per bar pair, oldest left, newest right.
const SLOTS: usize = 24;

/// The service rows the filter narrows (the visible slots).
const ROWS: usize = 6;

/// The services the demo pretends to run — the filter's catalog (a
/// `static` so the matched names are `'static` and outlive the read).
static SERVICES: [&str; 9] = [
    "net-link",
    "net-mesh",
    "auth-gate",
    "sensor-grid",
    "log-drain",
    "cache-pool",
    "queue-runner",
    "db-primary",
    "db-replica",
];

/// The dashboard's viewport (the recording's frame).
const VIEWPORT: Size = Size::new(480.0, 540.0);

/// The headless beat's frame count (deterministic — no sleeps, A7).
const BEAT_FRAMES: u64 = 120;

/// The scan cursor's travel: one sweep every N frames.
const CURSOR_PERIOD: u64 = 180;

/// The midnight palette (the theme toggle's dark fact — every role
/// distinct from the canon's, so the swap visibly repaints).
struct MidnightPalette;

impl ThemeSeam for MidnightPalette {
    fn palette(&self) -> Palette {
        Palette {
            surface: Color::rgb(12, 14, 20),
            panel: Color::rgb(22, 26, 36),
            accent: Color::rgb(240, 180, 41),
            accent_pressed: Color::rgb(214, 156, 22),
            selection: Color::rgb(240, 180, 41).with_alpha(72),
            text: Color::rgb(230, 233, 240),
            text_muted: Color::rgb(148, 156, 172),
            border: Color::rgb(58, 64, 82),
            on_accent: Color::rgb(18, 16, 8),
        }
    }

    fn name(&self) -> &'static str {
        "midnight"
    }
}

/// The canon palette under the toggle's other half — the light fact, the
/// palette the canon constants resolve (ADR-0079's own baseline).
struct LightPalette;

impl ThemeSeam for LightPalette {
    fn palette(&self) -> Palette {
        Palette::CANON
    }

    fn name(&self) -> &'static str {
        "canon"
    }
}

/// The state the ticker thread advances and every door reads: the ring of
/// samples, the filter text, the theme flag. One writer (the thread), many
/// readers (the painter, the hook, the bindings) — the `Arc<Mutex>` shape
/// the patch channel's own contract assumes (§13.2).
struct Dashboard {
    frame: u64,
    cpu: Vec<f32>,
    mem: Vec<f32>,
    filter: String,
    dark: bool,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            frame: 0,
            cpu: vec![0.4; SLOTS],
            mem: vec![0.6; SLOTS],
            filter: String::new(),
            dark: false,
        }
    }

    /// One tick of synthetic telemetry: a wave with deterministic jitter —
    /// no RNG, the headless beat replays it exactly.
    fn advance(&mut self) {
        self.frame += 1;
        let t = u64_to_f32(self.frame);
        let jitter = u64_to_f32(self.frame % 7) * 0.012;
        let cpu = (0.44 + 0.16 * (t * 0.11).sin() + jitter).clamp(0.05, 1.0);
        let mem = (0.58 + 0.10 * (t * 0.043).sin()).clamp(0.05, 1.0);
        self.cpu.rotate_left(1);
        self.mem.rotate_left(1);
        let last = SLOTS - 1;
        self.cpu[last] = cpu;
        self.mem[last] = mem;
    }

    /// The LIVE badge's pulse: a smooth 0.45–0.85 sweep over ~5 s of
    /// frames — a steady pulse is the badge's whole name; the
    /// reduced-motion door is the chart cursor's, not the badge's.
    fn pulse(&self) -> f32 {
        0.45 + 0.4 * (0.5 + 0.5 * (u64_to_f32(self.frame) * 0.09).sin())
    }

    /// The services the filter keeps, in catalog order.
    fn matches(&self) -> Vec<&'static str> {
        SERVICES
            .iter()
            .copied()
            .filter(|name| name.contains(self.filter.as_str()))
            .collect()
    }
}

/// The widget handles the patch hook drives (`NodeId` is `Copy`; the
/// signal handle clones).
struct Handles {
    badge: NodeId,
    rows: Vec<NodeId>,
    frame_signal: FluxRwSignal<u64>,
}

/// Builds the dashboard's tree and wires every door: the painter, the
/// stats binding, the field's edit handler, the button's click. Returns
/// the handles the hook drives.
fn build(
    pipeline: &mut PassPipeline,
    shared: &Arc<Mutex<Dashboard>>,
    channel: &PatchChannel,
) -> Handles {
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    // The header: the title beside the pulsing LIVE badge — the badge is
    // the compose tier's face (the hook drives its fade per frame).
    let header = pipeline.add_child(root, Row::new().gap(12.0).boxed());
    let _ = pipeline.add_child(header, Label::new("ALACER · operations").boxed());
    let badge = pipeline.add_child(
        header,
        Label::new("● LIVE")
            .color(Color::rgb(31, 122, 77))
            .opacity(0.8)
            .boxed(),
    );

    // The chart: one canvas, one painter — the painter reads the shared
    // samples at paint time, fades each bar by its age through
    // [`Compose`], and rides the scan cursor as a record-time translation.
    let _chart = pipeline.add_child(
        root,
        Canvas::new("CPU and memory history")
            .size(448.0, 150.0)
            .painter(chart_painter(Arc::clone(shared)))
            .boxed(),
    );

    // The stats line: the binding the flux bridge drives — one signal
    // write per frame ⇒ one repaint, the text re-derived from the shared
    // samples (the auto_complete demo's shape).
    let stats_binding = alacer_core::flux::RenderBinding::new(String::new());
    let frame_signal = pipeline.flux().signal(0_u64);
    let shared_for_stats = Arc::clone(shared);
    let stats = pipeline.add_child(root, Label::bound(stats_binding.clone()).boxed());
    pipeline.flux_mut().bind_value(
        frame_signal,
        stats,
        &stats_binding,
        Change::PAINT,
        move |_frame: &u64| {
            let telemetry = shared_for_stats.lock().expect("dashboard state");
            format!(
                "CPU {:>2.0}% · MEM {:>2.0}% · frame {}",
                telemetry.cpu[SLOTS - 1] * 100.0,
                telemetry.mem[SLOTS - 1] * 100.0,
                telemetry.frame,
            )
        },
    );

    // The filter: the field's edits land through the mutate seam — the
    // handler writes the shared filter and fires the channel; the hook
    // re-texts the rows and requests the masks (layout repaints, the tree
    // re-speaks).
    let shared_for_field = Arc::clone(shared);
    let field_channel = channel.clone();
    let _ = pipeline.add_child(
        root,
        TextInput::new("Filter services")
            .placeholder("Type to narrow the services…")
            .on_changed(move |text| {
                shared_for_field.lock().expect("dashboard state").filter = text.to_string();
                field_channel.fire();
            })
            .boxed(),
    );

    // The service rows: fixed slots, re-texted in place — the incremental
    // path a real list would ride (no rebuild, no remount).
    let mut rows = Vec::with_capacity(ROWS);
    for _ in 0..ROWS {
        let node = pipeline.add_child(root, Label::new("").boxed());
        rows.push(node);
    }

    // The theme toggle: the button writes the flag, the hook answers
    // through `set_theme` — the broadcast repaints every un-overridden
    // role the tick after (the theme seam's own contract).
    let shared_for_button = Arc::clone(shared);
    let button_channel = channel.clone();
    let _ = pipeline.add_child(
        root,
        Button::new("Toggle dark theme")
            .on_click(move || {
                let mut guard = shared_for_button.lock().expect("dashboard state");
                guard.dark = !guard.dark;
                drop(guard);
                button_channel.fire();
            })
            .boxed(),
    );

    Handles {
        badge,
        rows,
        frame_signal,
    }
}

/// The chart's painter: reads the shared samples at paint time, fades each
/// bar by its age through [`Compose`] (the oldest slot paints at 0.35, the
/// newest at full strength — the fade is the compose state the fold
/// applies at record time, never a per-bar color rewrite), and rides the
/// scan cursor as a record-time translation (one quad pushed at the
/// origin, shifted by the compose state — never a layout move). Under
/// reduced motion the cursor parks at the half: the fact answers when
/// motion itself is the question.
fn chart_painter(
    shared: Arc<Mutex<Dashboard>>,
) -> impl for<'ctx> Fn(&mut PaintCtx<'ctx>) {
    move |ctx: &mut PaintCtx<'_>| {
        let state = shared.lock().expect("dashboard state");
        let size = ctx.size();
        let slots = u32_to_f32(u32::try_from(SLOTS).unwrap_or(1));
        let slot_w = size.width / slots;
        for (index, slot) in state.cpu.iter().enumerate() {
            let age = f32::from(u16::try_from(index).unwrap_or(u16::MAX)) / slots;
            ctx.set_compose(Compose::faded(0.35 + 0.65 * age));
            let x = u32_to_f32(u32::try_from(index).unwrap_or(0)) * slot_w;
            let cpu_h = slot * (size.height - 8.0);
            ctx.push_quad(
                Quad::filled(
                    Rect::new(
                        Point::new(x + 1.0, size.height - cpu_h),
                        Size::new(slot_w * 0.42, cpu_h),
                    ),
                    ctx.palette().accent,
                ),
            );
            let mem_h = state.mem.get(index).copied().unwrap_or(0.0) * (size.height - 8.0);
            ctx.push_quad(
                Quad::filled(
                    Rect::new(
                        Point::new(x + 1.0 + slot_w * 0.46, size.height - mem_h),
                        Size::new(slot_w * 0.42, mem_h),
                    ),
                    ctx.palette().text_muted,
                ),
            );
        }
        let sweep =
            (u64_to_f32(state.frame % CURSOR_PERIOD) / u64_to_f32(CURSOR_PERIOD)).clamp(0.0, 1.0);
        let cursor_x = if ctx.reduced_motion() {
            size.width * 0.5
        } else {
            sweep * size.width
        };
        ctx.set_compose(Compose::shifted(Vector::new(cursor_x, 0.0)));
        ctx.push_quad(
            Quad::filled(
                Rect::new(Point::new(0.0, 0.0), Size::new(2.0, size.height)),
                ctx.palette().accent_pressed,
            ),
        );
    }
}

/// The patch hook: one call per patch event, the whole frame's reactive
/// side — the badge's compose fade, the stats signal, the filter rows, the
/// theme. Exactly one hook call per event (§13.2).
fn make_hook(
    handles: &Handles,
    shared: &Arc<Mutex<Dashboard>>,
    dark_landed: &Rc<Cell<bool>>,
) -> impl FnMut(&mut PassPipeline) + use<> {
    let hook_badge = handles.badge;
    let hook_rows = handles.rows.clone();
    let hook_signal = handles.frame_signal;
    let hook_shared = Arc::clone(shared);
    let hook_dark = Rc::clone(dark_landed);
    let mut applied_dark = false;
    // The rows' shown texts — the hook's own memory of what the tree
    // already says: a write landing where the row already is queues
    // nothing (the idle-frame rule, at the row tier).
    let mut shown: Vec<String> = vec![String::new(); ROWS];
    move |pipeline: &mut PassPipeline| {
        let state = hook_shared.lock().expect("dashboard state");
        // The badge: the compose fade rides the mutate seam — the write
        // landing where the pulse already is requests nothing (the
        // idle-frame rule at the compose tier).
        let pulse = state.pulse();
        {
            let mut guard = pipeline.arena_mut().mutate(hook_badge);
            let moved = guard
                .widget_mut()
                .as_any_mut()
                .downcast_mut::<Label>()
                .is_some_and(|label| label.set_compose(Compose::faded(pulse)));
            if moved {
                guard.request(Change::PAINT);
            }
        }
        // The stats line: one signal write, the bridge does the rest.
        let frame = state.frame;
        let dark = state.dark;
        drop(state);
        hook_signal.set(frame);
        // The filter rows: re-text the slots the filter keeps, clear the
        // rest — the masks pay layout, paint, and the tree's new names,
        // and only when a slot's text actually moved.
        let matches = hook_shared.lock().expect("dashboard state").matches();
        for (slot, node) in hook_rows.iter().enumerate() {
            let text = matches.get(slot).copied().unwrap_or("");
            if shown.get(slot).map(String::as_str) == Some(text) {
                continue;
            }
            let mut guard = pipeline.arena_mut().mutate(*node);
            let written = guard
                .widget_mut()
                .as_any_mut()
                .downcast_mut::<Label>()
                .is_some_and(|label| {
                    label.set_text(text);
                    true
                });
            if written && let Some(slot_shown) = shown.get_mut(slot) {
                *slot_shown = text.to_string();
                guard.request(Change::LAYOUT.union(Change::PAINT).union(Change::A11Y));
            }
        }
        // The theme: one swap per toggle, `false` answered honestly when
        // the palette already is (the idle-frame property at the tier).
        if dark != applied_dark {
            let theme: Box<dyn ThemeSeam> = if dark {
                Box::new(MidnightPalette)
            } else {
                Box::new(LightPalette)
            };
            if pipeline.set_theme(theme) {
                applied_dark = dark;
                hook_dark.set(true);
            }
        }
    }
}

fn main() {
    let headless = std::env::args().any(|arg| arg == "--headless");
    let channel = PatchChannel::new();
    let shared = Arc::new(Mutex::new(Dashboard::new()));
    let dark_landed = Rc::new(Cell::new(false));
    let mut pipeline = pipeline_default();

    let handles = build(&mut pipeline, &shared, &channel);
    let mut hook = make_hook(&handles, &shared, &dark_landed);

    if headless {
        run_headless(&mut pipeline, &shared, &dark_landed, &mut hook, &handles);
        return;
    }

    // The recordable window: a ticker thread advances the samples and
    // fires the channel from off-thread — the patch channel's own
    // documented use (the watcher thread's shape).
    let ticker_shared = Arc::clone(&shared);
    let ticker_channel = channel.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(33));
            ticker_shared.lock().expect("dashboard state").advance();
            ticker_channel.fire();
        }
    });

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer operations",
        (VIEWPORT.width, VIEWPORT.height),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(hook));
    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// The headless beat: the same hook, the same pipeline, 120 deterministic
/// frames — then the assertions, printed for the CI log.
fn run_headless(
    pipeline: &mut PassPipeline,
    shared: &Arc<Mutex<Dashboard>>,
    dark_landed: &Rc<Cell<bool>>,
    hook: &mut impl FnMut(&mut PassPipeline),
    handles: &Handles,
) {
    // Frames one by one: advance the samples, run the hook, settle the
    // pipeline (tick until idle — the demo shape A7 prices).
    for _ in 0..BEAT_FRAMES {
        shared.lock().expect("dashboard state").advance();
        hook(pipeline);
        settle(pipeline);
    }

    // Beat one: the chart recorded the bars — the compose tier painted
    // them (two per slot, the cursor beside).
    let report = pipeline.tick(VIEWPORT).expect("the settled frame");
    let quads = report
        .scene
        .primitives
        .iter()
        .filter(|p| matches!(p, alacer_core::scene::Primitive::Quad(_)))
        .count();
    assert!(
        quads >= 2 * SLOTS,
        "the chart painted its bars, got {quads} quads"
    );

    // Beat two: the badge's fade moved — the compose state the hook wrote
    // rides the label; at frame 120 the pulse sits well below the
    // builder's 0.8 (the sweep is deterministic, no RNG).
    let pulse = {
        let mut guard = pipeline.arena_mut().mutate(handles.badge);
        let Some(label) = guard.widget_mut().as_any_mut().downcast_mut::<Label>() else {
            panic!("the badge is a label");
        };
        let compose = label.compose();
        (0.0..0.8).contains(&compose.opacity)
    };
    assert!(pulse, "the LIVE badge's fade moved off the builder's own");

    // Beat three: the filter narrowed the tree — a kept name speaks, a
    // dropped one does not.
    shared.lock().expect("dashboard state").filter = "db".to_string();
    hook(pipeline);
    settle(pipeline);
    let snapshot = pipeline.a11y_snapshot().cloned();
    assert!(
        snapshot
            .as_ref()
            .and_then(|t| t.find(alacer_core::Role::Label, "db-primary"))
            .is_some(),
        "the kept service speaks by name"
    );
    assert!(
        snapshot
            .as_ref()
            .and_then(|t| t.find(alacer_core::Role::Label, "auth-gate"))
            .is_none(),
        "the dropped service is off the tree"
    );

    // Beat four: the theme landed — the broadcast answered true.
    shared.lock().expect("dashboard state").dark = true;
    hook(pipeline);
    settle(pipeline);
    assert!(
        dark_landed.get(),
        "the midnight palette landed through the theme seam"
    );

    println!("dashboard headless beat: {BEAT_FRAMES} frames settled");
    println!("  chart  : {quads} quads recorded (bars + cursor)");
    println!("  badge  : the compose fade moved off the builder's own");
    println!("  filter : 'db' keeps db-primary/db-replica, drops the rest");
    println!("  theme  : midnight landed (set_theme answered true)");
}

/// Ticks until no pass has work (the demo shape, A7).
fn settle(pipeline: &mut PassPipeline) {
    for _ in 0..16 {
        let Ok(report) = pipeline.tick(VIEWPORT) else {
            return;
        };
        if report.is_idle() {
            return;
        }
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
