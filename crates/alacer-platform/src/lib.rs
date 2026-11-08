//! Alacer's platform: stock winit behind a thin seam (ADR-1 — never forked),
//! presenting through wgpu or softbuffer.
//!
//! §10.2 coalescing keeps input storms at one frame of work; §14.5 keeps the
//! main thread owning the frame loop. Idle is `ControlFlow::Wait` — zero CPU
//! between interactions (A7).

pub mod keys;
pub mod softbuffer;
pub mod wgpu_backend;

pub use keys::{ime_cursor_utf16, map_ime_event, map_key, map_logical_key};
pub use softbuffer::{SoftbufferPresenter, pixmap_to_argb};
pub use wgpu_backend::{BackendError, WgpuRenderer};

use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use alacer_a11y::AccessKitSink;
use alacer_core::events::{ImeEvent, KeyEvent, PointerEvent};
use alacer_core::geom::{Point, Size};
use alacer_core::passes::{FrameReport, ImeRequest, PassPipeline};
use alacer_core::widget::EventResponse;
use alacer_text::{ThreadLocalGlyphSource, ThreadLocalTextSeam};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

/// User events flowing through the loop: AccessKit's messages plus our own
/// redraw nudge (the adapter's activation handler uses the proxy).
#[derive(Debug)]
pub enum PlatformEvent {
    /// An AccessKit platform message.
    AccessKit(AccessKitEvent),
    /// The tree changed; AccessKit wants the update applied.
    TreeChanged,
    /// A hot patch landed; the app's patch hook re-renders the view scoped
    /// (§13.2, ADR-0010). The hook owns the how; the loop owns the when —
    /// exactly one regenerate per patch event.
    Patch,
}

impl From<AccessKitEvent> for PlatformEvent {
    fn from(event: AccessKitEvent) -> Self {
        Self::AccessKit(event)
    }
}

/// Which present path to open at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendPreference {
    /// wgpu (ADR-2 primary).
    #[default]
    Wgpu,
    /// tiny-skia + softbuffer (headless/low-end).
    Softbuffer,
}

/// Platform failures.
#[derive(Debug)]
pub enum PlatformError {
    /// The winit loop failed to start or run.
    Loop(winit::error::EventLoopError),
    /// Window creation failed.
    Window(winit::error::OsError),
    /// The present backend failed at startup.
    Backend(BackendError),
    /// The softbuffer path failed at startup.
    Softbuffer(String),
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loop(e) => write!(f, "event loop: {e}"),
            Self::Window(e) => write!(f, "window: {e}"),
            Self::Backend(e) => write!(f, "backend: {e}"),
            Self::Softbuffer(e) => write!(f, "softbuffer: {e}"),
        }
    }
}

impl std::error::Error for PlatformError {}

enum Backend {
    Wgpu(Box<WgpuBackend>),
    Softbuffer(SoftbufferPresenter),
}

/// The wgpu present path, boxed to keep the enum small.
struct WgpuBackend {
    surface: wgpu::Surface<'static>,
    renderer: WgpuRenderer,
}

/// The running app: pipeline + window + backend, driven by winit.
pub struct AlacerApp {
    pipeline: PassPipeline,
    glyph_source: ThreadLocalGlyphSource,
    backend_preference: BackendPreference,
    title: String,
    start_size: (f32, f32),
    window: Option<Arc<Window>>,
    backend: Option<Backend>,
    adapter: Option<Adapter>,
    proxy: Option<EventLoopProxy<PlatformEvent>>,
    cursor: Point,
    needs_present: bool,
    /// Tracked keyboard modifiers (winit delivers them via
    /// `ModifiersChanged`, not on key events).
    modifiers: alacer_core::events::Modifiers,
    /// The last IME request applied to the window — the platform only acts
    /// on the delta frame over frame (§10.3).
    applied_ime: Option<ImeRequest>,
    /// The softbuffer path's persistent framebuffer: partial paint repaints
    /// damage regions onto this instead of re-rasterizing everything (§9.3).
    framebuffer: Option<tiny_skia::Pixmap>,
    /// The hot-reload patch channel: `Patch` user events wake the loop for
    /// the patch hook (§13.2). `None` in every default path.
    patch_channel: Option<PatchChannel>,
    /// The patch hook: re-renders the view scoped, exactly once per patch
    /// event. `None` in every default path.
    patch_hook: Option<PatchHook>,
}

/// The patch hook type: one scoped re-render per patch event (§13.2).
pub type PatchHook = Box<dyn FnMut(&mut PassPipeline)>;

/// A thread-safe wake-up channel for hot-reload patches (§13.2, ADR-0010).
///
/// The framework installs the loop's proxy into the channel when the app
/// starts; `fire` then wakes the loop from any thread — the subsecond patch
/// handler's thread, a watcher thread, or the app's own UI code. One `fire`
/// equals one `Patch` user event equals one hook call.
#[derive(Clone, Default)]
pub struct PatchChannel {
    proxy: Arc<std::sync::Mutex<Option<EventLoopProxy<PlatformEvent>>>>,
}

impl PatchChannel {
    /// A channel not yet attached to a loop (the app attaches it on start).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Wakes the loop with one patch event. Safe to call from any thread;
    /// silently dropped before the loop attaches (nothing is running to
    /// wake — the flag loss is harmless, the next fire works).
    pub fn fire(&self) {
        if let Ok(guard) = self.proxy.lock()
            && let Some(proxy) = guard.as_ref()
            && proxy.send_event(PlatformEvent::Patch).is_err()
        {
            // The loop is gone (window closed); a patch arriving then is a
            // race the process is already exiting from.
        }
    }

    fn attach(&self, proxy: EventLoopProxy<PlatformEvent>) {
        if let Ok(mut guard) = self.proxy.lock() {
            *guard = Some(proxy);
        }
    }
}

impl AlacerApp {
    /// Assembles the app around a fully-wired [`PassPipeline`] (root mounted,
    /// bindings installed by the caller before `run`).
    ///
    /// The platform layer is an assembly site: it installs this thread's
    /// text seam so widgets (core-only dependencies) can shape text (ADR-9).
    #[must_use]
    pub fn new(
        pipeline: PassPipeline,
        title: impl Into<String>,
        size: (f32, f32),
        backend: BackendPreference,
    ) -> Self {
        alacer_core::text_seam::install(std::rc::Rc::new(ThreadLocalTextSeam));
        Self {
            pipeline,
            glyph_source: ThreadLocalGlyphSource,
            backend_preference: backend,
            title: title.into(),
            start_size: size,
            window: None,
            backend: None,
            adapter: None,
            proxy: None,
            cursor: Point::zero(),
            needs_present: true,
            modifiers: alacer_core::events::Modifiers::default(),
            applied_ime: None,
            framebuffer: None,
            patch_channel: None,
            patch_hook: None,
        }
    }

    /// Attaches a patch channel: the loop's proxy is installed into it on
    /// start, so `PatchChannel::fire` wakes this app (§13.2, ADR-0010).
    #[must_use]
    pub fn with_patch_channel(mut self, channel: PatchChannel) -> Self {
        self.patch_channel = Some(channel);
        self
    }

    /// Installs the patch hook — the closure that answers one `Patch` event
    /// with one scoped re-render (`HotView::regenerate` in the facade's hot
    /// runtime). Exactly one hook call per event.
    pub fn set_patch_hook(&mut self, hook: PatchHook) {
        self.patch_hook = Some(hook);
    }

    /// Runs until the window closes.
    ///
    /// # Errors
    ///
    /// [`PlatformError`] for loop, window, or backend startup failures.
    pub fn run(mut self) -> Result<(), PlatformError> {
        let event_loop = EventLoop::with_user_event()
            .build()
            .map_err(PlatformError::Loop)?;
        event_loop.set_control_flow(ControlFlow::Wait);
        let proxy = event_loop.create_proxy();
        if let Some(channel) = self.patch_channel.take() {
            channel.attach(proxy.clone());
        }
        self.proxy = Some(proxy);
        event_loop.run_app(&mut self).map_err(PlatformError::Loop)
    }

    fn ensure_backend(&mut self, event_loop: &ActiveEventLoop) -> Result<(), PlatformError> {
        let Some(window) = self.window.clone() else {
            return Ok(());
        };
        if self.backend.is_some() {
            return Ok(());
        }
        match self.backend_preference {
            BackendPreference::Wgpu => {
                let surface = wgpu::Instance::new(instance_descriptor())
                    .create_surface(window.clone())
                    .map_err(|e| {
                        PlatformError::Backend(BackendError::SurfaceCreation(e.to_string()))
                    })?;
                let (renderer, _config) = WgpuRenderer::new(
                    &surface,
                    (size_px(self.start_size.0), size_px(self.start_size.1)),
                )
                .map_err(PlatformError::Backend)?;
                self.backend = Some(Backend::Wgpu(Box::new(WgpuBackend { surface, renderer })));
            }
            BackendPreference::Softbuffer => {
                let display: Arc<dyn raw_window_handle::HasDisplayHandle> =
                    Arc::new(window.clone());
                let window_handle: Arc<dyn raw_window_handle::HasWindowHandle> = Arc::new(window);
                let mut presenter = SoftbufferPresenter::new(display, window_handle)
                    .map_err(|e| PlatformError::Softbuffer(e.to_string()))?;
                presenter
                    .resize(size_px(self.start_size.0), size_px(self.start_size.1))
                    .map_err(|e| PlatformError::Softbuffer(e.to_string()))?;
                self.backend = Some(Backend::Softbuffer(presenter));
            }
        }
        let _ = event_loop;
        Ok(())
    }

    fn render_frame(&mut self) {
        let viewport = self.viewport_size();
        if self.backend.is_none() {
            return;
        }
        let report = match self.pipeline.tick(viewport) {
            Ok(report) => report,
            Err(error) => {
                tracing::error!("layout failed: {error}");
                return;
            }
        };
        // The a11y frame boundary (ADR-0015): the tick produced this frame's
        // delta — dirty nodes only — and the sink holds its `TreeUpdate`.
        // Drain now, while the pass that built it is still this frame.
        self.push_pending_a11y();
        // Apply this frame's IME request to the window (§10.3): attach or
        // detach the IME, move the candidate window to the caret. Only the
        // delta acts — platform calls are not free.
        self.apply_ime_request();
        let width = size_px(viewport.width).max(1);
        let height = size_px(viewport.height).max(1);
        self.present(&report, width, height);
        self.needs_present = false;
    }

    /// The damage ladder (§9.3): a NONE frame skips the present entirely —
    /// zero fill rate for zero change — a FULL frame re-rasterizes, a partial
    /// frame repaints only the damage regions.
    fn present(&mut self, report: &FrameReport, width: u32, height: u32) {
        let Some(backend) = &mut self.backend else {
            return;
        };
        let scene = &report.scene;
        let damage = &report.damage;
        match backend {
            Backend::Wgpu(backend) => {
                let result = if damage.is_empty() {
                    Ok(()) // nothing changed; the last presented frame stands
                } else if damage.is_full() {
                    backend
                        .renderer
                        .render(scene, &mut self.glyph_source, &backend.surface)
                } else {
                    backend.renderer.render_partial(
                        scene,
                        damage,
                        &mut self.glyph_source,
                        &backend.surface,
                    )
                };
                if let Err(error) = result {
                    tracing::error!("wgpu render failed: {error}");
                }
            }
            Backend::Softbuffer(presenter) => {
                // The persistent framebuffer (§9.3): partial frames repaint
                // damage regions onto the previous frame instead of
                // re-rasterizing the whole scene.
                if damage.is_empty() {
                    return;
                }
                let size_changed = self
                    .framebuffer
                    .as_ref()
                    .is_none_or(|frame| frame.width() != width || frame.height() != height);
                if size_changed {
                    self.framebuffer = None;
                }
                let Some(frame) = self.framebuffer.as_mut() else {
                    let mut renderer = alacer_render::SoftwareRenderer::new();
                    let Some(pixmap) = renderer.render(
                        scene,
                        width,
                        height,
                        alacer_widgets::theme::SURFACE,
                        &mut self.glyph_source,
                    ) else {
                        tracing::error!("framebuffer allocation failed");
                        return;
                    };
                    self.framebuffer = Some(pixmap);
                    let Some(frame) = self.framebuffer.as_mut() else {
                        return;
                    };
                    if let Err(error) = presenter.present(frame) {
                        tracing::error!("present failed: {error}");
                    }
                    return;
                };
                let mut renderer = alacer_render::SoftwareRenderer::new();
                if damage.is_full() {
                    let fresh = renderer.render(
                        scene,
                        width,
                        height,
                        alacer_widgets::theme::SURFACE,
                        &mut self.glyph_source,
                    );
                    if let Some(pixmap) = fresh {
                        *frame = pixmap;
                    }
                } else if renderer
                    .render_partial(
                        scene,
                        frame,
                        damage,
                        alacer_widgets::theme::SURFACE,
                        &mut self.glyph_source,
                    )
                    .is_none()
                {
                    tracing::error!("partial repaint failed");
                }
                if let Err(error) = presenter.present(frame) {
                    tracing::error!("present failed: {error}");
                }
            }
        }
    }

    /// Applies the pipeline's IME request delta to the window (§10.3):
    /// `set_ime_allowed` when the purpose set changes, `set_ime_cursor_area`
    /// when the caret moves. The fcitx5/ibus seam.
    fn apply_ime_request(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let request = self.pipeline.ime_request().cloned();
        if request == self.applied_ime {
            return;
        }
        match (&request, &self.applied_ime) {
            (Some(request), _) => {
                let enabled = request.purpose.enabled();
                let was_enabled = self
                    .applied_ime
                    .as_ref()
                    .is_some_and(|r| r.purpose.enabled());
                if enabled != was_enabled {
                    window.set_ime_allowed(enabled);
                }
                if let Some(caret) = request.caret {
                    window.set_ime_cursor_area(
                        winit::dpi::Position::Logical(winit::dpi::LogicalPosition::new(
                            f64::from(caret.origin.x),
                            f64::from(caret.origin.y),
                        )),
                        winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                            f64::from(caret.size.width),
                            f64::from(caret.size.height),
                        )),
                    );
                }
            }
            (None, Some(_)) => {
                window.set_ime_allowed(false);
            }
            (None, None) => {}
        }
        self.applied_ime = request;
    }

    /// Queues one raw pointer event into the pipeline's frame-batched input
    /// queue (§10.2, ADR-0014) and asks for the frame that routes it. The
    /// platform pushes and never drains — the tick owns the batch.
    fn queue_pointer(&mut self, event: PointerEvent) {
        self.pipeline.queue_pointer(event);
        self.needs_present = true;
    }

    /// Routes one mapped key event; handling dirties the frame.
    fn route_key(&mut self, event: &KeyEvent) {
        if self.pipeline.dispatch_key(event) == EventResponse::Handled {
            self.needs_present = true;
        }
    }

    /// Routes one mapped IME event; handling dirties the frame.
    fn route_ime(&mut self, event: &ImeEvent) {
        if self.pipeline.dispatch_ime(event) == EventResponse::Handled {
            self.needs_present = true;
        }
    }

    fn viewport_size(&self) -> Size {
        self.window
            .as_ref()
            .map_or(Size::new(self.start_size.0, self.start_size.1), |window| {
                let size = window.inner_size();
                Size::new(
                    f32::from(u16::try_from(size.width).unwrap_or(u16::MAX)),
                    f32::from(u16::try_from(size.height).unwrap_or(u16::MAX)),
                )
            })
    }

    /// The steady-state a11y frame boundary (ADR-0015): drain the updates
    /// the pipeline's own sink produced during the last tick and hand each
    /// to the adapter. The adapter forwards while an assistive technology is
    /// connected and drops otherwise — correctness is re-established at
    /// connect time by [`Self::push_initial_tree`], so a dropped delta is
    /// never a lost tree. The platform builds no `TreeUpdate` of its own.
    fn push_pending_a11y(&mut self) {
        let updates = drain_a11y_updates(&mut self.pipeline);
        if updates.is_empty() {
            return;
        }
        if let Some(adapter) = &mut self.adapter {
            for update in updates {
                adapter.update_if_active(|| update);
            }
        }
    }

    /// The adapter-activation response (ADR-0015): one full-tree
    /// `TreeUpdate` built by the sink — the same builder the deltas use,
    /// full vocabulary included — so an assistive technology that connects
    /// (or reconnects) sees the whole resolved tree, not a platform-side
    /// summary of it. Before the first tick there is no resolved tree; the
    /// first frame's init delta covers the gap.
    fn push_initial_tree(&mut self) {
        let update = a11y_sink_of(&mut self.pipeline).and_then(|sink| sink.full_tree_update());
        if let Some(adapter) = &mut self.adapter
            && let Some(update) = update
        {
            adapter.update_if_active(|| update);
        }
    }
}

/// The pipeline's installed sink, as the concrete AccessKit sink — the
/// downcast every a11y push goes through (ADR-0015). `None` when an
/// embedder installed a foreign sink: their updates are their business.
fn a11y_sink_of(pipeline: &mut PassPipeline) -> Option<&mut AccessKitSink> {
    pipeline
        .a11y_sink_mut()
        .as_any_mut()?
        .downcast_mut::<AccessKitSink>()
}

/// Drains the pipeline sink's pending updates (empty for a foreign sink).
fn drain_a11y_updates(pipeline: &mut PassPipeline) -> Vec<accesskit::TreeUpdate> {
    a11y_sink_of(pipeline)
        .map(AccessKitSink::drain_updates)
        .unwrap_or_default()
}

impl ApplicationHandler<PlatformEvent> for AlacerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.start_size.0,
                self.start_size.1,
            ))
            .with_visible(false);
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                tracing::error!("window creation failed: {error}");
                return;
            }
        };

        // ADR-5 + accesskit contract: the adapter must exist before the window
        // is shown for the first time.
        if let Some(proxy) = self.proxy.clone() {
            self.adapter = Some(Adapter::with_event_loop_proxy::<PlatformEvent>(
                event_loop, &window, proxy,
            ));
        }
        window.set_visible(true);
        self.window = Some(window);
        if let Err(error) = self.ensure_backend(event_loop) {
            tracing::error!("backend startup failed: {error}");
            return;
        }
        self.window.as_ref().expect("just stored").request_redraw();
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the winit event match is one flat dispatch table; splitting it would hide which events the loop answers"
    )]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.clone() else {
            return;
        };
        if window.id() != window_id {
            return;
        }
        let _ = event_loop;
        // AccessKit observes every platform event.
        if let Some(adapter) = &mut self.adapter {
            adapter.process_event(&window, &event);
        }

        match event {
            WindowEvent::Resized(size) => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                if let Some(Backend::Wgpu(backend)) = &mut self.backend {
                    backend.renderer.resize(width, height, &backend.surface);
                }
                if let Some(Backend::Softbuffer(presenter)) = &mut self.backend
                    && let Err(error) = presenter.resize(width, height)
                {
                    tracing::error!("softbuffer resize failed: {error}");
                }
                self.needs_present = true;
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Point::new(
                    alacer_core::num::i32_to_f32(f64_to_i32(position.x)),
                    alacer_core::num::i32_to_f32(f64_to_i32(position.y)),
                );
                self.queue_pointer(PointerEvent::Move {
                    position: self.cursor,
                });
            }
            WindowEvent::CursorLeft { .. } => {
                self.queue_pointer(PointerEvent::Leave);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let event = match (state, button) {
                    (ElementState::Pressed, MouseButton::Left) => PointerEvent::Down {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Left,
                    },
                    (ElementState::Released, MouseButton::Left) => PointerEvent::Up {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Left,
                    },
                    (ElementState::Pressed, MouseButton::Right) => PointerEvent::Down {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Right,
                    },
                    (ElementState::Released, MouseButton::Right) => PointerEvent::Up {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Right,
                    },
                    (ElementState::Pressed, MouseButton::Middle) => PointerEvent::Down {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Middle,
                    },
                    (ElementState::Released, MouseButton::Middle) => PointerEvent::Up {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Middle,
                    },
                    _ => PointerEvent::Move {
                        position: self.cursor,
                    },
                };
                self.queue_pointer(event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.queue_pointer(PointerEvent::Wheel {
                    delta: wheel_delta(delta),
                    position: self.cursor,
                });
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                self.modifiers.shift = state.shift_key();
                self.modifiers.control = state.control_key();
                self.modifiers.alt = state.alt_key();
                self.modifiers.meta = state.super_key();
            }
            // Keyboard routes to the focused widget (§10.3) — focus-routed,
            // never hit-tested. Editing acts on presses; releases pass too
            // (widgets decide).
            WindowEvent::KeyboardInput { event, .. } => {
                let mapped = keys::map_key(
                    &event.logical_key,
                    event.text.as_deref(),
                    event.state,
                    self.modifiers,
                );
                self.route_key(&mapped);
            }
            // IME composition (§10.3): preedit is composing input; commit is
            // final text. Enabled/Disabled carry no widget-level meaning —
            // the pipeline's ImeRequest owns the attachment decision.
            WindowEvent::Ime(ime) => {
                if let Some(mapped) = keys::map_ime_event(&ime) {
                    self.route_ime(&mapped);
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        // Frame-batched input (§10.2, ADR-0014): the pipeline queue owns the
        // batch now — the tick drains it, so one redraw covers the storm.
        if self.needs_present {
            window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: PlatformEvent) {
        match event {
            PlatformEvent::AccessKit(access) => match access.window_event {
                AccessKitWindowEvent::InitialTreeRequested => {
                    self.push_initial_tree();
                }
                AccessKitWindowEvent::ActionRequested(request) => {
                    let action = alacer_core::widget::A11yAction::DefaultPress;
                    let handled = self
                        .pipeline
                        .dispatch_a11y_action(request.target_node.0, action);
                    if handled {
                        self.needs_present = true;
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
                AccessKitWindowEvent::AccessibilityDeactivated => {}
            },
            PlatformEvent::TreeChanged => {
                // An embedder's nudge: flush the sink's pending updates now,
                // without waiting for the next frame boundary (ADR-0015).
                self.push_pending_a11y();
            }
            PlatformEvent::Patch => {
                // §13.2: the patch lands, the hook re-renders scoped — one
                // regenerate per patch event, then one layout+paint cycle.
                if let Some(hook) = self.patch_hook.as_mut() {
                    hook(&mut self.pipeline);
                }
                self.needs_present = true;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.needs_present
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = cause;
    }
}

fn size_px(v: f32) -> u32 {
    alacer_core::num::f32_to_u32(v)
}

/// Maps a winit wheel delta to the pipeline's pixel vector — both axes, no
/// dropped component (the pre-M8 mapping dropped `x`, ADR-0014).
fn wheel_delta(delta: MouseScrollDelta) -> alacer_core::geom::Vector {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => alacer_core::geom::Vector::new(x * 40.0, y * 40.0),
        MouseScrollDelta::PixelDelta(position) => alacer_core::geom::Vector::new(
            alacer_core::num::f64_to_f32(position.x),
            alacer_core::num::f64_to_f32(position.y),
        ),
    }
}

fn f64_to_i32(v: f64) -> i32 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "winit hands out f64 device pixels; the saturating clamp bounds the result"
    )]
    {
        v.round().clamp(-2_147_483_647.0, 2_147_483_647.0) as i32
    }
}

fn instance_descriptor() -> wgpu::InstanceDescriptor {
    wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    }
}

#[cfg(test)]
mod tests {
    use super::wheel_delta;
    use alacer_a11y::AccessKitSink;
    use alacer_core::Flux;
    use alacer_core::geom::Vector;
    use alacer_core::passes::{A11yDelta, A11ySink, PassPipeline};
    use winit::event::MouseScrollDelta;

    #[test]
    fn line_deltas_map_both_axes_with_the_line_height() {
        let delta = wheel_delta(MouseScrollDelta::LineDelta(2.0, -3.0));
        assert_eq!(
            delta,
            Vector::new(80.0, -120.0),
            "x rides along, y keeps the 40 px/line"
        );
    }

    #[test]
    fn pixel_deltas_map_both_axes_unscaled() {
        let delta = wheel_delta(MouseScrollDelta::PixelDelta(
            winit::dpi::PhysicalPosition::new(12.0, -9.0),
        ));
        assert_eq!(delta, Vector::new(12.0, -9.0));
    }

    // ------------------------------------------------------------------
    // M9 (ADR-0015): the windowed path reads the pipeline's own sink —
    // the platform builds no `TreeUpdate` of its own, ever. These tests
    // exercise the exact drain helper the loop calls at frame boundaries.
    // ------------------------------------------------------------------

    /// The windowed composition, headless: real layout, journal, and the
    /// `AccessKitSink` the facade's `pipeline_default` installs.
    fn headless_pipeline() -> PassPipeline {
        alacer_core::text_seam::install(std::rc::Rc::new(alacer_text::ThreadLocalTextSeam));
        PassPipeline::new(
            Flux::new(),
            Box::new(alacer_layout::LayoutHost::new(std::sync::Arc::new(
                alacer_text::ThreadLocalMeasurer,
            ))),
            Box::new(alacer_render::SceneJournalImpl::new()),
            Box::new(AccessKitSink::new()),
        )
    }

    #[test]
    fn the_pipeline_sink_downcasts_to_the_accesskit_sink() {
        let mut pipeline = headless_pipeline();
        assert!(
            super::a11y_sink_of(&mut pipeline).is_some(),
            "the facade composition exposes the AccessKit sink through the erased trait"
        );
    }

    #[test]
    fn a_foreign_sink_downcasts_to_none() {
        let mut pipeline = PassPipeline::new(
            Flux::new(),
            Box::new(alacer_layout::LayoutHost::new(std::sync::Arc::new(
                alacer_text::ThreadLocalMeasurer,
            ))),
            Box::new(alacer_render::SceneJournalImpl::new()),
            Box::new(NullA11y),
        );
        alacer_core::text_seam::install(std::rc::Rc::new(alacer_text::ThreadLocalTextSeam));
        assert!(
            super::a11y_sink_of(&mut pipeline).is_none(),
            "an embedder's custom sink is untouched — no forced AccessKit"
        );
        assert_eq!(
            super::drain_a11y_updates(&mut pipeline),
            Vec::new(),
            "an embedder's custom sink has nothing to drain"
        );
    }

    #[test]
    fn pending_a11y_updates_drain_exactly_once() {
        alacer_core::text_seam::install(std::rc::Rc::new(alacer_text::ThreadLocalTextSeam));
        let mut pipeline = headless_pipeline();
        let sink = super::a11y_sink_of(&mut pipeline).expect("AccessKit sink");
        let snapshot = std::sync::Arc::new(alacer_core::passes::TreeSnapshot {
            root: Some(1),
            nodes: vec![alacer_core::passes::TreeNode {
                id: 1,
                role: alacer_core::Role::Container,
                label: "app".into(),
                bounds: None,
                children: Vec::new(),
                value: None,
                toggled: None,
                table_slot: None,
            }],
        });
        sink.apply(
            std::sync::Arc::clone(&snapshot),
            &A11yDelta {
                initializes: true,
                ..A11yDelta::empty()
            },
        );

        let updates = super::drain_a11y_updates(&mut pipeline);
        assert_eq!(updates.len(), 1, "the init update drains once");
        assert!(updates[0].tree.is_some());
        assert_eq!(
            super::drain_a11y_updates(&mut pipeline),
            Vec::new(),
            "the second drain sees a quiet frame — no rebuild, no noise"
        );
    }

    /// An embedder sink: the downcast must refuse it, not panic.
    struct NullA11y;

    impl A11ySink for NullA11y {
        fn apply(
            &mut self,
            _snapshot: std::sync::Arc<alacer_core::passes::TreeSnapshot>,
            _delta: &A11yDelta,
        ) {
        }
    }
}
