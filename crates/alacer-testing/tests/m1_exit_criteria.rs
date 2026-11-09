//! The M1 exit criteria, as executable spec (architecture §18):
//!
//! - **32 blessed snapshots green** — the corpus below, one test, every
//!   scenario rendered through the real pipeline + software raster (A4);
//! - **a11y queries pass** — the query builder, press-through-a11y, walks,
//!   and plural counts (ADR-10: a11y-driven queries are the only API);
//! - the scheduling contract still holds under view reconciliation.
//!
//! Blessing is deliberate and reviewable: `ALACER_BLESS=1 cargo test` regenerates
//! the corpus (ADR-5 snapshot hygiene).

use alacer_core::Color;
use alacer_core::events::{EventCtx, PointerEvent};
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::PassPipeline;
use alacer_core::scene::{PaintCtx, Quad};
use alacer_core::style::{MeasureSpec, Size2, Style};
use alacer_core::widget::{A11yAction, EventResponse, LabelSource, Role, Widget};
use alacer_macro::rsx;
use alacer_testing::TestHarness;
use alacer_view::{ViewNode, ViewRoot};
use alacer_widgets::BoxedWidget as _;
use reactive_graph::traits::{Get, Set};

/// The harness-layer mount: the default M0 element registry (the facade's
/// `mount_view` re-exports exactly this composition, ADR-9).
fn mount(view: &ViewNode, pipeline: &mut PassPipeline) -> ViewRoot {
    ViewRoot::mount(view, pipeline, alacer_widgets::default_registry())
}

const VIEWPORT: (f32, f32) = (240.0, 320.0);

// ---------------------------------------------------------------------------
// Scenario building blocks (§5.2's vocabulary, via rsx!)
// ---------------------------------------------------------------------------

fn counter_view(count: alacer_core::FluxRwSignal<i32>) -> ViewNode {
    rsx! {
        column { gap: 8.0, padding: 12.0,
            label { text: "Clicked {count.get()} times" }
            button { text: "Increment", on_click: move || count.set(count.get() + 1) }
        }
    }
}

fn items_view(title: &str, items: &[u64]) -> ViewNode {
    let items = items.to_vec();
    rsx! {
        column { gap: 4.0,
            label { key: 0_u64, text: title }
            for item in items.iter() {
                label { key: *item * 10 + 1, text: "Item {item}" }
            }
        }
    }
}

fn click_center(harness: &mut TestHarness, role: Role, name: &str) {
    let bounds = harness.query(role).named(name).bounds();
    harness.click(Point::new(
        bounds.origin.x + bounds.size.width / 2.0,
        bounds.origin.y + bounds.size.height / 2.0,
    ));
}

// ---------------------------------------------------------------------------
// The 32 scenarios — each mounts, optionally interacts, and leaves the frame
// the snapshot captures. One function per scenario keeps the corpus readable
// and reviewable (ADR-5: baselines are contracts).
// ---------------------------------------------------------------------------

fn s01_hello_label(h: &mut TestHarness) {
    let view = rsx! { column { padding: 20.0, label { text: "Hello, Alacer" } } };
    let _root = mount(&view, h.pipeline_mut());
}

fn s02_styled_label(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 16.0,
            label { text: "Large and tinted", font_size: 28.0, color: alacer_widgets::theme::ACCENT }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s03_long_wrapped_label(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 12.0,
            label { text: "The quick brown fox jumps over the lazy dog and keeps going well past the viewport width" }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s04_button_default(h: &mut TestHarness) {
    let view = rsx! { column { padding: 24.0, button { text: "Save" } } };
    let _root = mount(&view, h.pipeline_mut());
}

fn s05_button_pressed(h: &mut TestHarness) {
    let view = rsx! { column { padding: 24.0, button { text: "Hold" } } };
    let _root = mount(&view, h.pipeline_mut());
    h.settle();
    // Press down and leave it down: the pressed fill is the snapshot.
    let bounds = h.query(Role::Button).named("Hold").bounds();
    h.pipeline_mut()
        .dispatch_pointer(&alacer_core::PointerEvent::Down {
            position: Point::new(
                bounds.origin.x + bounds.size.width / 2.0,
                bounds.origin.y + bounds.size.height / 2.0,
            ),
            button: alacer_core::MouseButton::Left,
            click_count: 1,
        });
}

fn s06_button_custom_colors(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 24.0,
            button { text: "Danger", color: alacer_core::Color::rgb(170, 40, 40) }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s07_column_gaps(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 12.0, gap: 12.0,
            label { text: "First" }
            label { text: "Second" }
            label { text: "Third" }
            label { text: "Fourth" }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s08_column_padding(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 40.0, gap: 4.0,
            label { text: "Deep inset" }
            label { text: "All sides" }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s09_row_mixed(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 16.0, gap: 8.0,
            row { gap: 8.0,
                label { text: "Left" }
                button { text: "Middle" }
                spacer { size: 12.0 }
                label { text: "Right" }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s10_nested_containers(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 12.0, gap: 8.0,
            row { gap: 6.0,
                column { gap: 4.0,
                    label { text: "Top left" }
                    label { text: "Bottom left" }
                }
                column { gap: 4.0,
                    label { text: "Top right" }
                    label { text: "Bottom right" }
                }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s11_panel_card(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 16.0, gap: 8.0,
            panel { height: 48.0, color: alacer_widgets::theme::PANEL }
            label { text: "A card over a panel" }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s12_spacer_stack(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 16.0, gap: 0.0,
            label { text: "Above" }
            spacer { size: 32.0 }
            label { text: "Middle" }
            spacer { size: 64.0 }
            label { text: "Below" }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s13_list_top(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 8.0,
            list { height: 220.0,
                for i in 0_u64..10 {
                    panel { key: i, height: 32.0, color: alacer_widgets::theme::PANEL }
                }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s14_list_scrolled_120(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 8.0,
            list { height: 220.0,
                for i in 0_u64..10 {
                    panel { key: i, height: 32.0, color: alacer_widgets::theme::PANEL }
                }
            }
        }
    };
    let root = mount(&view, h.pipeline_mut());
    h.settle();
    // Scroll through the facade's list handle lookup: the only list is the
    // first descendant of the root column.
    let list_node = first_descendant_of_kind(h, root.root_id());
    scroll_list(h, list_node, 120.0);
}

fn s15_list_scrolled_400(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 8.0,
            list { height: 220.0,
                for i in 0_u64..24 {
                    panel { key: i, height: 32.0, color: alacer_widgets::theme::PANEL }
                }
            }
        }
    };
    let root = mount(&view, h.pipeline_mut());
    h.settle();
    let list_node = first_descendant_of_kind(h, root.root_id());
    scroll_list(h, list_node, 400.0);
}

fn s16_list_few_items(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 8.0,
            list { height: 220.0,
                for i in 0_u64..2 {
                    panel { key: i, height: 48.0, color: alacer_widgets::theme::ACCENT_PRESSED }
                }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s17_theme_surfaces(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 10.0, gap: 6.0,
            panel { height: 36.0, color: alacer_widgets::theme::SURFACE }
            panel { height: 36.0, color: alacer_widgets::theme::PANEL }
            panel { height: 36.0, color: alacer_widgets::theme::ACCENT }
            panel { height: 36.0, color: alacer_widgets::theme::ACCENT_PRESSED }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s18_counter_after_clicks(h: &mut TestHarness) {
    let signal = h.pipeline_mut().flux_mut().signal(0_i32);
    let view = counter_view(signal);
    let mut root = mount(&view, h.pipeline_mut());
    h.settle();
    for _ in 0..3 {
        click_center(h, Role::Button, "Increment");
        let next = counter_view(signal);
        let _report = root.update(&next, h.pipeline_mut());
        h.settle();
    }
}

fn s19_counter_pressed_state(h: &mut TestHarness) {
    let signal = h.pipeline_mut().flux_mut().signal(0_i32);
    let view = counter_view(signal);
    let mut root = mount(&view, h.pipeline_mut());
    h.settle();
    let bounds = h.query(Role::Button).named("Increment").bounds();
    h.pipeline_mut()
        .dispatch_pointer(&alacer_core::PointerEvent::Down {
            position: Point::new(
                bounds.origin.x + bounds.size.width / 2.0,
                bounds.origin.y + bounds.size.height / 2.0,
            ),
            button: alacer_core::MouseButton::Left,
            click_count: 1,
        });
    h.settle();
    let next = counter_view(signal);
    let _report = root.update(&next, h.pipeline_mut());
    h.settle();
}

fn s20_big_font_stack(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 12.0, gap: 6.0,
            label { text: "Twelve", font_size: 12.0 }
            label { text: "Sixteen", font_size: 16.0 }
            label { text: "Twenty two", font_size: 22.0 }
            label { text: "Thirty two", font_size: 32.0 }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s21_muted_text_colors(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 12.0, gap: 6.0,
            label { text: "Primary", color: alacer_widgets::theme::TEXT }
            label { text: "Muted", color: alacer_widgets::theme::TEXT_MUTED }
            label { text: "Accent", color: alacer_widgets::theme::ACCENT }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s22_wide_button(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 24.0,
            button { text: "A very long button label indeed" }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s23_two_buttons_row(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 20.0, gap: 10.0,
            row { gap: 10.0,
                button { text: "Cancel" }
                button { text: "OK" }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s24_quiet_button(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 24.0,
            button { text: "Quiet", color: alacer_widgets::theme::PANEL, text_color: alacer_widgets::theme::TEXT_MUTED }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s25_deep_nesting(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 8.0, gap: 4.0,
            column { padding: 8.0, gap: 4.0,
                column { padding: 8.0, gap: 4.0,
                    column { padding: 8.0, gap: 4.0,
                        column { padding: 8.0, gap: 4.0,
                            label { text: "Five levels deep" }
                        }
                    }
                }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s26_many_labels(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 8.0, gap: 2.0,
            for i in 0_u64..20 {
                label { key: i, text: "Row {i}" }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s27_scroll_then_click(h: &mut TestHarness) {
    let signal = h.pipeline_mut().flux_mut().signal(0_i32);
    let view = rsx! {
        column { padding: 8.0, gap: 8.0,
            button { text: "Refresh", on_click: move || signal.set(signal.get() + 1) }
            label { text: "Refreshed {signal.get()} times" }
        }
    };
    let mut root = mount(&view, h.pipeline_mut());
    h.settle();
    click_center(h, Role::Button, "Refresh");
    let next = rsx! {
        column { padding: 8.0, gap: 8.0,
            button { text: "Refresh", on_click: move || signal.set(signal.get() + 1) }
            label { text: "Refreshed {signal.get()} times" }
        }
    };
    let _report = root.update(&next, h.pipeline_mut());
    h.settle();
}

fn s28_conditional_label(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 20.0,
            if false {
                button { text: "Never mounted" }
            } else {
                label { text: "The quiet branch" }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s29_conditional_button(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 20.0,
            if true {
                button { text: "The loud branch" }
            } else {
                label { text: "Never mounted" }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

fn s30_mixed_grid(h: &mut TestHarness) {
    let view = rsx! {
        column { padding: 12.0, gap: 8.0,
            row { gap: 8.0,
                panel { height: 40.0, color: alacer_widgets::theme::PANEL }
                panel { height: 40.0, color: alacer_widgets::theme::ACCENT }
            }
            row { gap: 8.0,
                label { text: "Name:" }
                spacer { size: 8.0 }
                button { text: "Submit" }
            }
        }
    };
    let _root = mount(&view, h.pipeline_mut());
}

/// The scenario's field: the M15 pair (text, journal) with the one edit
/// funnel the contract requires — apply and journal in the same place
/// (ADR-0021), exactly what the `undo_redo` example drives.
struct JournalField {
    text: String,
    history: alacer_core::history::EditHistory,
}

impl JournalField {
    /// The funnel: one definition — append at the end and journal it.
    fn type_hi(&mut self) {
        let at = alacer_core::utf16::len_utf16(&self.text);
        let from = alacer_core::utf16::byte_offset(&self.text, at);
        self.text.replace_range(from..from, "hi ");
        let caret = at + alacer_core::utf16::len_utf16("hi ");
        self.history.record(
            at,
            String::new(),
            "hi ".to_string(),
            (at, at),
            (caret, caret),
        );
    }

    /// The chord's return: the step's honesty is the boolean.
    fn undo(&mut self) -> bool {
        self.history.undo(&mut self.text).is_some()
    }

    /// The introspection reads the status line carries.
    fn status(&self) -> String {
        format!(
            "undo depth: {} · redo: {}",
            self.history.undo_depth(),
            if self.history.can_redo() { "yes" } else { "no" }
        )
    }

    /// The field line: the buffer, or the placeholder while it is empty.
    fn display(&self) -> String {
        if self.text.is_empty() {
            "the empty field".to_string()
        } else {
            format!("field: {:?}", self.text)
        }
    }
}

fn journal_view(
    field: &std::rc::Rc<std::cell::RefCell<JournalField>>,
    text: alacer_core::FluxRwSignal<String>,
    status: alacer_core::FluxRwSignal<String>,
) -> ViewNode {
    let field_type = std::rc::Rc::clone(field);
    let field_undo = std::rc::Rc::clone(field);
    rsx! {
        column { padding: 12.0, gap: 8.0,
            label { text: "The journal (ADR-0021)" }
            label { text: "{text.get()}" }
            label { text: "{status.get()}" }
            row { gap: 8.0,
                button { text: "Type hi", on_click: move || {
                    let mut field = field_type.borrow_mut();
                    field.type_hi();
                    let display = field.display();
                    let line = field.status();
                    drop(field);
                    text.set(display);
                    status.set(line);
                } }
                button { text: "Undo", on_click: move || {
                    // The honest refusal: an exhausted journal is consumed
                    // with no masks — no signal writes, an idle frame.
                    let stepped = field_undo.borrow_mut().undo();
                    if stepped {
                        let field = field_undo.borrow();
                        let display = field.display();
                        let line = field.status();
                        drop(field);
                        text.set(display);
                        status.set(line);
                    }
                } }
            }
        }
    }
}

fn s31_journal_undo_redo(h: &mut TestHarness) {
    // The M16 demonstration (ADR-0022): the M15 journal (ADR-0021) driven
    // by routed clicks — two typed appends coalesce into one undo step,
    // the undo takes the whole run back, and the status line reads the
    // depth and redo branch live.
    let field = std::rc::Rc::new(std::cell::RefCell::new(JournalField {
        text: String::new(),
        history: alacer_core::history::EditHistory::new(),
    }));
    let text = h
        .pipeline_mut()
        .flux_mut()
        .signal("the empty field".to_string());
    let status = h
        .pipeline_mut()
        .flux_mut()
        .signal("undo depth: 0 · redo: no".to_string());
    let view = journal_view(&field, text, status);
    let mut root = mount(&view, h.pipeline_mut());
    h.settle();
    for _ in 0..2 {
        click_center(h, Role::Button, "Type hi");
        let next = journal_view(&field, text, status);
        let _report = root.update(&next, h.pipeline_mut());
        h.settle();
    }
    click_center(h, Role::Button, "Undo");
    let next = journal_view(&field, text, status);
    let _report = root.update(&next, h.pipeline_mut());
    h.settle();
}

// ---------------------------------------------------------------------------
// The M17 storm probe: the coalescer's consumer (ADR-0023)
// ---------------------------------------------------------------------------

/// The consumer-side log the probe records into and the routes read —
/// shared `Rc<RefCell>` state, exactly what a custom widget and its
/// handlers share outside the pipeline (the m8 probe's shape, ADR-0014).
#[derive(Default)]
struct StormLog {
    routed: Vec<Point>,
}

/// The probe widget: a fixed panel that records every pointer event the
/// pipeline routes to it. `role()` and `label()` are required — the
/// a11y-by-construction contract applies to a demo's widget as much as to
/// a shipped one (ADR-5).
struct StormPanel {
    size: f32,
    log: std::rc::Rc<std::cell::RefCell<StormLog>>,
}

impl Widget for StormPanel {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn role(&self) -> Role {
        Role::Container
    }

    fn label(&self) -> LabelSource {
        LabelSource::text("storm panel")
    }

    fn style(&self) -> Style {
        Style {
            size: Size2::uniform(self.size),
            ..Style::default()
        }
    }

    fn measure_spec(&self) -> Option<MeasureSpec> {
        Some(MeasureSpec::Fixed(Size::new(self.size, self.size)))
    }

    fn paint(&self, ctx: &mut PaintCtx<'_>) {
        ctx.push_quad(Quad {
            rect: Rect::new(Point::zero(), ctx.size()),
            color: Color::rgb(61, 90, 128),
            corner_radius: 4.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }

    fn on_pointer_event(&mut self, _ctx: &mut EventCtx, event: &PointerEvent) -> EventResponse {
        if let PointerEvent::Move { position } = event {
            self.log.borrow_mut().routed.push(*position);
        }
        EventResponse::Handled
    }
}

fn s32_input_storm(h: &mut TestHarness) {
    // The M17 demonstration (ADR-0023): the M8 coalescer (ADR-0014)
    // driven through the pipeline's own queue — fifty queued moves route
    // as one event (the platform pushes, the tick drains), the probe's
    // log is the consumer's evidence, and the status line reads it back.
    let log = std::rc::Rc::new(std::cell::RefCell::new(StormLog::default()));
    let status = h.pipeline_mut().flux_mut().signal(String::new());
    let root = h
        .pipeline_mut()
        .mount_root(alacer_widgets::Column::new().gap(8.0).padding(12.0).boxed());
    let _title = h.pipeline_mut().add_child(
        root,
        alacer_widgets::Label::new("The input storm (ADR-0014)").boxed(),
    );
    let _panel = h.pipeline_mut().add_child(
        root,
        Box::new(StormPanel {
            size: 200.0,
            log: std::rc::Rc::clone(&log),
        }) as Box<dyn Widget>,
    );
    let binding = alacer_core::flux::RenderBinding::new("queued 0 moves".to_string());
    let status_label = h
        .pipeline_mut()
        .add_child(root, alacer_widgets::Label::bound(binding.clone()).boxed());
    h.pipeline_mut().flux_mut().bind_value(
        status,
        status_label,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |line: &String| line.clone(),
    );
    h.settle();

    // The storm: fifty moves into the queue — the seam the platform layer
    // pushes and never drains — then one tick routes the batch.
    let bounds = h.query(Role::Container).named("storm panel").bounds();
    for i in 0..50 {
        h.queue_pointer(PointerEvent::Move {
            position: Point::new(
                bounds.origin.x + alacer_core::num::usize_to_f32(i) * 3.0,
                bounds.origin.y + 10.0,
            ),
        });
    }
    let _report = h.tick();
    let routed = log.borrow().routed.len();
    assert_eq!(
        routed, 1,
        "fifty queued moves coalesce into one routed event"
    );

    // The mirror route: handlers write signals — the status reads the log.
    alacer_core::flux::Flux::batch_of(|| {
        reactive_graph::traits::Set::set(
            &status,
            format!("queued 50 moves · panel routed {routed}"),
        );
    });
    h.settle();
}

/// The corpus: (name, run). Order is the snapshot prefix.
/// One scenario: mount, interact, leave the frame to snapshot.
type Scenario = (&'static str, fn(&mut TestHarness));

const SCENARIOS: &[Scenario] = &[
    ("hello_label", s01_hello_label),
    ("styled_label", s02_styled_label),
    ("long_wrapped_label", s03_long_wrapped_label),
    ("button_default", s04_button_default),
    ("button_pressed", s05_button_pressed),
    ("button_custom_colors", s06_button_custom_colors),
    ("column_gaps", s07_column_gaps),
    ("column_padding", s08_column_padding),
    ("row_mixed", s09_row_mixed),
    ("nested_containers", s10_nested_containers),
    ("panel_card", s11_panel_card),
    ("spacer_stack", s12_spacer_stack),
    ("list_top", s13_list_top),
    ("list_scrolled_120", s14_list_scrolled_120),
    ("list_scrolled_400", s15_list_scrolled_400),
    ("list_few_items", s16_list_few_items),
    ("theme_surfaces", s17_theme_surfaces),
    ("counter_after_clicks", s18_counter_after_clicks),
    ("counter_pressed_state", s19_counter_pressed_state),
    ("big_font_stack", s20_big_font_stack),
    ("muted_text_colors", s21_muted_text_colors),
    ("wide_button", s22_wide_button),
    ("two_buttons_row", s23_two_buttons_row),
    ("quiet_button", s24_quiet_button),
    ("deep_nesting", s25_deep_nesting),
    ("many_labels", s26_many_labels),
    ("scroll_then_click", s27_scroll_then_click),
    ("conditional_label", s28_conditional_label),
    ("conditional_button", s29_conditional_button),
    ("mixed_grid", s30_mixed_grid),
    ("journal_undo_redo", s31_journal_undo_redo),
    ("input_storm", s32_input_storm),
];

// ---------------------------------------------------------------------------
// The exit criteria
// ---------------------------------------------------------------------------

#[test]
fn m1_corpus_is_thirty_two_scenarios() {
    assert_eq!(
        SCENARIOS.len(),
        32,
        "the M1 exit criterion is 32 blessed snapshots (M17 grew it, ADR-0023)"
    );
}

#[test]
fn m1_thirty_two_blessed_snapshots_are_green() {
    for (index, (name, run)) in SCENARIOS.iter().enumerate() {
        let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
        run(&mut harness);
        harness.settle();
        // The a11y CI walker runs on every scenario (§11.1 as code).
        harness.assert_all_named();
        harness.snapshot(&format!("m1_{index:02}_{name}"));
    }
}

#[test]
fn m1_a11y_queries_pass() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let signal = harness.pipeline_mut().flux_mut().signal(0_i32);
    let view = counter_view(signal);
    let mut root = mount(&view, harness.pipeline_mut());
    harness.settle();

    // The singular query (M0's API) and the builder agree.
    let _ = harness.expect_a11y(Role::Button, "Increment");
    assert!(harness.query(Role::Button).named("Increment").exists());
    assert_eq!(harness.query(Role::Button).named("Increment").count(), 1);
    assert_eq!(harness.query(Role::Label).named("Item").count(), 0);
    assert_eq!(harness.query(Role::Button).named("Nonexistent").count(), 0);

    // Walk: the whole tree contains exactly the two mounted widgets + column.
    let tree = harness.pipeline().a11y_snapshot().expect("a11y tree");
    assert_eq!(tree.walk(None).len(), 3, "column + label + button");
    let button = tree.find(Role::Button, "Increment").expect("button");
    assert_eq!(
        tree.walk(Some(button.id)).len(),
        1,
        "leaves walk to themselves"
    );

    // Press through the a11y tree (the actionable half of §11.5).
    harness.press(Role::Button, "Increment");
    let next = counter_view(signal);
    let _report = root.update(&next, harness.pipeline_mut());
    harness.settle();
    let _ = harness.expect_a11y(Role::Label, "Clicked 1 times");
}

#[test]
fn m1_a11y_action_on_an_untargeted_widget_reports_not_consumed() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let view = rsx! { column { padding: 8.0, label { text: "static" } } };
    let _root = mount(&view, harness.pipeline_mut());
    harness.settle();
    let label_id = harness
        .pipeline()
        .a11y_snapshot()
        .and_then(|tree| tree.find(Role::Label, "static"))
        .expect("label")
        .id;
    let consumed = harness
        .pipeline_mut()
        .dispatch_a11y_action(label_id, A11yAction::DefaultPress);
    assert!(!consumed, "a label is not actionable");
}

#[test]
fn m1_keyed_list_reconcile_keeps_the_scheduling_contract() {
    let mut harness = TestHarness::new(VIEWPORT.0, VIEWPORT.1);
    let view = items_view("Batch", &[1, 2, 3, 4]);
    let mut root = mount(&view, harness.pipeline_mut());
    harness.settle();
    let layout_runs = harness.pipeline().counters().layout_runs;

    // One update ⇒ at most one layout+paint cycle (ADR-6 kill-switch).
    let reversed = items_view("Batch", &[4, 3, 2, 1]);
    let _report = root.update(&reversed, harness.pipeline_mut());
    harness.settle();
    assert_eq!(
        harness.pipeline().counters().layout_runs,
        layout_runs + 1,
        "one write batch ⇒ one layout run"
    );
    let _ = harness.expect_a11y(Role::Label, "Item 4");
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The first descendant of `root` whose widget is a `List` (scroll setup in
/// scenarios; the a11y walker's role query is the public path, the arena walk
/// here is test-only plumbing for the shared `ListHandle`).
fn first_descendant_of_kind(
    harness: &TestHarness,
    root: alacer_core::arena::NodeId,
) -> alacer_core::arena::NodeId {
    let arena = harness.pipeline().arena();
    let mut found = None;
    arena.for_each_pre_order(&mut |id, slot| {
        if found.is_none() && slot.widget().role() == Role::ScrollArea {
            found = Some(id);
        }
    });
    let _ = root;
    found.expect("a list is mounted")
}

fn scroll_list(harness: &mut TestHarness, node: alacer_core::arena::NodeId, offset: f32) {
    {
        let mut guard = harness.pipeline_mut().arena_mut().mutate(node);
        let handle = guard
            .widget_mut()
            .as_any_mut()
            .downcast_mut::<alacer_widgets::List>()
            .expect("the node is a List")
            .handle();
        handle.scroll_to(offset);
        drop(handle);
        // A programmatic scroll is a mutation: the invalidation currency
        // applies — request the repaint + a11y-bounds refresh (§8.2), the
        // same masks the wheel path requests through `EventCtx`.
        guard.request(alacer_core::Change::PAINT.union(alacer_core::Change::A11Y));
    }
    harness.settle();
}
