//! Alacer's platform: stock winit behind a thin seam (ADR-1 — never forked),
//! presenting through wgpu or softbuffer.
//!
//! §10.2 coalescing keeps input storms at one frame of work; §14.5 keeps the
//! main thread owning the frame loop. Idle is `ControlFlow::Wait` — zero CPU
//! between interactions (A7).

pub mod clipboard;
pub mod keys;
// The backend features are true (M34, ADR-0040): each present path's
// module compiles only when its feature rides — a single-backend build
// (no wgpu, no softbuffer, or either alone) is a supported shape, not a
// latent breakage. With no backend compiled the platform still builds:
// the pipeline, the window plumbing, and the typed refusal at startup
// are the crate's whole surface then.
#[cfg(feature = "backend-softbuffer")]
pub mod softbuffer;
#[cfg(feature = "backend-wgpu")]
pub mod wgpu_backend;

pub use keys::{ime_cursor_utf16, map_ime_event, map_key, map_logical_key};
#[cfg(feature = "backend-softbuffer")]
pub use softbuffer::{SoftbufferPresenter, pixmap_to_argb};
#[cfg(feature = "backend-wgpu")]
pub use wgpu_backend::{BackendError, WgpuRenderer};

use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use alacer_a11y::AccessKitSink;
use alacer_core::events::{ImeEvent, KeyEvent, PointerEvent};
use alacer_core::geom::{Point, Size};
use alacer_core::passes::{FrameReport, ImeRequest, PassPipeline};
use alacer_core::widget::EventResponse;
#[cfg(any(feature = "backend-wgpu", feature = "backend-softbuffer"))]
use alacer_text::ThreadLocalGlyphSource;
use alacer_text::ThreadLocalTextSeam;
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

impl BackendPreference {
    /// The preference's name, the build-refusal's vocabulary (the
    /// protocol's `snake_case` tag style, [`FocusRider::name`]'s rule).
    ///
    /// [`FocusRider::name`]: alacer_core::inspect::FocusRider::name
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wgpu => "wgpu",
            Self::Softbuffer => "softbuffer",
        }
    }

    /// The env's own override: `ALACER_SOFTWARE=1` names the software
    /// path. The env the docs promised is now the platform's own — every
    /// consumer honors it, not only the examples' `choose_backend`
    /// helpers: a `Wgpu` preference the env overrides becomes
    /// `Softbuffer`, a `Softbuffer` preference is already the env's
    /// answer, and an env set to anything else is refused honestly — a
    /// `tracing::warn!` naming the value it saw and the one it accepts,
    /// never a guess the value did not make (the M11 rule at the
    /// platform seam).
    ///
    /// `env` is passed rather than read, so the parser stays pure —
    /// tests drive it without mutating process state, and the caller
    /// owns where the value comes from (`AlacerApp::new` reads
    /// `ALACER_SOFTWARE` itself).
    #[must_use]
    pub fn software_env(self, env: Option<&std::ffi::OsStr>) -> Self {
        match env {
            None => self,
            Some(value) if value == std::ffi::OsStr::new("1") => match self {
                Self::Wgpu => {
                    tracing::info!(
                        target: "alacer::platform",
                        backend = "softbuffer",
                        "ALACER_SOFTWARE=1 names the present path"
                    );
                    Self::Softbuffer
                }
                Self::Softbuffer => Self::Softbuffer,
            },
            Some(value) => {
                tracing::warn!(
                    target: "alacer::platform",
                    value = ?value,
                    "ALACER_SOFTWARE accepts only `1` — the env is ignored and the preference stands"
                );
                self
            }
        }
    }
}

/// Platform failures.
#[derive(Debug)]
pub enum PlatformError {
    /// The winit loop failed to start or run.
    Loop(winit::error::EventLoopError),
    /// Window creation failed.
    Window(winit::error::OsError),
    /// The present backend failed at startup.
    #[cfg(feature = "backend-wgpu")]
    Backend(BackendError),
    /// The softbuffer path failed at startup (M18, ADR-0024): the
    /// structured [`SoftBufferError`](::softbuffer::SoftBufferError) rides the chain — a
    /// consumer walks to the real cause instead of a formatted string.
    #[cfg(feature = "backend-softbuffer")]
    Softbuffer(::softbuffer::SoftBufferError),
    /// The requested present path is not part of this build (M34,
    /// ADR-0040): the backend features gate their own backends, so a
    /// preference the build compiled out refuses honestly at startup —
    /// a fact about the build, not a wrapped failure (the
    /// [`PlatformError::Loop`]-adjacent leaf rule, the chain staying
    /// walkable).
    BackendUnavailable {
        /// The preference that has no compiled backend (`wgpu`,
        /// `softbuffer`).
        requested: &'static str,
    },
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loop(e) => write!(f, "event loop: {e}"),
            Self::Window(e) => write!(f, "window: {e}"),
            #[cfg(feature = "backend-wgpu")]
            Self::Backend(e) => write!(f, "backend: {e}"),
            #[cfg(feature = "backend-softbuffer")]
            Self::Softbuffer(e) => write!(f, "softbuffer present path: {e}"),
            Self::BackendUnavailable { requested } => write!(
                f,
                "no present backend compiled: `{requested}` is gated off — enable backend-wgpu or backend-softbuffer"
            ),
        }
    }
}

impl std::error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The chain a consumer walks with anyhow or `source().unwrap()`:
        // every wrapped failure stays reachable; the honest leaves — a
        // fact about the host or the build, not a wrapped failure — name
        // themselves and carry no cause.
        match self {
            Self::Loop(e) => Some(e),
            Self::Window(e) => Some(e),
            #[cfg(feature = "backend-wgpu")]
            Self::Backend(e) => Some(e),
            #[cfg(feature = "backend-softbuffer")]
            Self::Softbuffer(e) => Some(e),
            Self::BackendUnavailable { .. } => None,
        }
    }
}

// The present paths compile with their features (M34, ADR-0040): the
// enum's variants are the compiled backends — a single-backend build
// carries one, a no-backend build carries none and refuses at the
// preference resolution instead.
enum Backend {
    #[cfg(feature = "backend-wgpu")]
    Wgpu(Box<WgpuBackend>),
    #[cfg(feature = "backend-softbuffer")]
    Softbuffer(SoftbufferPresenter),
}

/// The wgpu present path, boxed to keep the enum small.
#[cfg(feature = "backend-wgpu")]
struct WgpuBackend {
    surface: wgpu::Surface<'static>,
    renderer: WgpuRenderer,
}

/// The running app: pipeline + window + backend, driven by winit.
pub struct AlacerApp {
    pipeline: PassPipeline,
    backend_preference: BackendPreference,
    title: String,
    start_size: (f32, f32),
    window: Option<Arc<Window>>,
    backend: Option<Backend>,
    adapter: Option<Adapter>,
    proxy: Option<EventLoopProxy<PlatformEvent>>,
    cursor: Point,
    needs_present: bool,
    /// The frame's glyph source (the text seam's rasterizer side): the
    /// present paths' last rendering argument — compiled with the
    /// backends that consume it (M34, ADR-0040).
    #[cfg(any(feature = "backend-wgpu", feature = "backend-softbuffer"))]
    glyph_source: ThreadLocalGlyphSource,
    /// Tracked keyboard modifiers (winit delivers them via
    /// `ModifiersChanged`, not on key events).
    modifiers: alacer_core::events::Modifiers,
    /// The click chain behind every `Down`'s click count (ADR-0019): the
    /// platform owns the clock, the widget never does.
    clicks: ClickCounter,
    /// The last IME request applied to the window — the platform only acts
    /// on the delta frame over frame (§10.3).
    applied_ime: Option<ImeRequest>,
    /// The softbuffer path's persistent framebuffer: partial paint repaints
    /// damage regions onto this instead of re-rasterizing everything (§9.3).
    #[cfg(feature = "backend-softbuffer")]
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
        // The clipboard seam's engine rides here (ADR-0020): the platform
        // owns the arboard adapter dependency (ADR-9), widgets only ever
        // see the seam in core.
        let clipboard = std::rc::Rc::new(crate::clipboard::ArboardClipboard::new());
        alacer_core::clipboard::install(clipboard);
        Self {
            pipeline,
            // The env's own override (the docs' `ALACER_SOFTWARE=1`, now
            // the platform's own): the caller's preference stands unless
            // the env names the software path.
            backend_preference: backend
                .software_env(std::env::var_os("ALACER_SOFTWARE").as_deref()),
            title: title.into(),
            start_size: size,
            window: None,
            backend: None,
            adapter: None,
            proxy: None,
            cursor: Point::zero(),
            needs_present: true,
            #[cfg(any(feature = "backend-wgpu", feature = "backend-softbuffer"))]
            glyph_source: ThreadLocalGlyphSource,
            modifiers: alacer_core::events::Modifiers::default(),
            clicks: ClickCounter::default(),
            applied_ime: None,
            #[cfg(feature = "backend-softbuffer")]
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
        if self.backend.is_some() {
            return Ok(());
        }
        let outcome = match self.backend_preference {
            #[cfg(feature = "backend-wgpu")]
            BackendPreference::Wgpu => self.build_wgpu_backend(),
            #[cfg(feature = "backend-softbuffer")]
            BackendPreference::Softbuffer => self.build_softbuffer_backend(),
            // The build compiled this preference's backend out (M34,
            // ADR-0040): the refusal is typed and names the preference —
            // a fact about the build, never a panic, never a munged
            // fallback to a backend the consumer did not ask for.
            #[cfg(not(all(feature = "backend-wgpu", feature = "backend-softbuffer")))]
            other => Err(PlatformError::BackendUnavailable {
                requested: other.name(),
            }),
        };
        let _ = event_loop;
        outcome
    }

    /// Builds the wgpu present path (the window's surface + the renderer)
    /// — the preference's arm, compiled with the feature that owns it
    /// (M34, ADR-0040).
    #[cfg(feature = "backend-wgpu")]
    fn build_wgpu_backend(&mut self) -> Result<(), PlatformError> {
        let Some(window) = self.window.clone() else {
            return Ok(());
        };
        let surface = wgpu::Instance::new(instance_descriptor())
            .create_surface(window.clone())
            .map_err(BackendError::SurfaceCreation)
            .map_err(PlatformError::Backend)?;
        let (renderer, _config) = WgpuRenderer::new(
            &surface,
            (size_px(self.start_size.0), size_px(self.start_size.1)),
        )
        .map_err(PlatformError::Backend)?;
        self.backend = Some(Backend::Wgpu(Box::new(WgpuBackend { surface, renderer })));
        Ok(())
    }

    /// Builds the softbuffer present path (the presenter over the
    /// window's raw handles) — the preference's arm, compiled with the
    /// feature that owns it (M34, ADR-0040).
    #[cfg(feature = "backend-softbuffer")]
    fn build_softbuffer_backend(&mut self) -> Result<(), PlatformError> {
        let Some(window) = self.window.clone() else {
            return Ok(());
        };
        let display: Arc<dyn raw_window_handle::HasDisplayHandle> = Arc::new(window.clone());
        let window_handle: Arc<dyn raw_window_handle::HasWindowHandle> = Arc::new(window);
        let mut presenter =
            SoftbufferPresenter::new(display, window_handle).map_err(PlatformError::Softbuffer)?;
        presenter
            .resize(size_px(self.start_size.0), size_px(self.start_size.1))
            .map_err(PlatformError::Softbuffer)?;
        self.backend = Some(Backend::Softbuffer(presenter));
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
    /// frame repaints only the damage regions. Each compiled backend
    /// answers its own variant (M34, ADR-0040); a no-backend build never
    /// reaches the ladder — `ensure_backend` refused at startup.
    fn present(&mut self, report: &FrameReport, width: u32, height: u32) {
        #[cfg(feature = "backend-wgpu")]
        self.present_wgpu(report);
        #[cfg(feature = "backend-softbuffer")]
        self.present_softbuffer(report, width, height);
        #[cfg(not(feature = "backend-softbuffer"))]
        let _ = (width, height);
        #[cfg(not(feature = "backend-wgpu"))]
        let _ = report;
    }

    /// The wgpu ladder: a NONE frame skips, a FULL frame re-renders, a
    /// partial frame re-renders the damage regions on the GPU path.
    #[cfg(feature = "backend-wgpu")]
    fn present_wgpu(&mut self, report: &FrameReport) {
        let Some(Backend::Wgpu(backend)) = &mut self.backend else {
            return;
        };
        let scene = &report.scene;
        let damage = &report.damage;
        let result = if damage.is_empty() {
            Ok(()) // nothing changed; the last presented frame stands
        } else if damage.is_full() {
            backend
                .renderer
                .render(scene, &mut self.glyph_source, &backend.surface)
        } else {
            backend
                .renderer
                .render_partial(scene, damage, &mut self.glyph_source, &backend.surface)
        };
        if let Err(error) = result {
            tracing::error!("wgpu render failed: {error}");
        }
    }

    /// The softbuffer ladder: a NONE frame skips, a FULL frame
    /// re-rasterizes, a partial frame repaints only the damage regions
    /// onto the persistent framebuffer (§9.3).
    #[cfg(feature = "backend-softbuffer")]
    fn present_softbuffer(&mut self, report: &FrameReport, width: u32, height: u32) {
        let Some(Backend::Softbuffer(presenter)) = &mut self.backend else {
            return;
        };
        let scene = &report.scene;
        let damage = &report.damage;
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

/// Maps an AccessKit `ActionRequest` onto the core action vocabulary
/// (ADR-0016): one variant per AccessKit action the framework has a widget
/// contract for, the request's `ScrollUnit` data carried through. Actions
/// with no consumer in the vocabulary map to `None` — they are dropped at
/// the boundary (traced, once, with the action name) instead of being
/// munged into a press: a `Focus` request that clicked something would be
/// a correctness bug, not a fallback.
fn map_action_request(
    request: &accesskit::ActionRequest,
) -> Option<alacer_core::widget::A11yAction> {
    use accesskit::{Action, ActionData, ScrollUnit as PlatformUnit};
    use alacer_core::widget::{A11yAction as Core, ScrollUnit, TextSelection};

    // Carries the request's scroll unit through, defaulting to one item —
    // the granularity an AT means when it sends no unit at all.
    let unit = |data: Option<&ActionData>| -> ScrollUnit {
        match data {
            Some(ActionData::ScrollUnit(PlatformUnit::Page)) => ScrollUnit::Page,
            _ => ScrollUnit::Item,
        }
    };

    // The value tier (M11, ADR-0017): the payload rides the request, so a
    // mapping without it is a munge — a SetValue with no `Value` data is
    // dropped at the boundary, traced, exactly like an unknown action.
    let value = |data: Option<&ActionData>| -> Option<String> {
        match data {
            Some(ActionData::Value(text)) => Some(text.to_string()),
            other => {
                tracing::trace!(?other, "accesskit value action dropped: no payload");
                None
            }
        }
    };

    match request.action {
        Action::Click => Some(Core::DefaultPress),
        Action::Increment => Some(Core::Increment),
        Action::Decrement => Some(Core::Decrement),
        Action::ScrollUp => Some(Core::ScrollUp(unit(request.data.as_ref()))),
        Action::ScrollDown => Some(Core::ScrollDown(unit(request.data.as_ref()))),
        Action::SetValue => value(request.data.as_ref()).map(Core::SetValue),
        Action::ReplaceSelectedText => value(request.data.as_ref()).map(Core::ReplaceSelectedText),
        Action::SetTextSelection => match request.data.as_ref() {
            Some(ActionData::SetTextSelection(selection)) => {
                // The single-node selection contract (ADR-0017): Alacer's
                // editable nodes carry their own value, so the AT's
                // character indexes map straight through. A position aimed
                // at another node cannot be honored here and is dropped.
                if selection.anchor.node != request.target_node
                    || selection.focus.node != request.target_node
                {
                    tracing::trace!(
                        ?selection,
                        "accesskit SetTextSelection dropped: positions off target"
                    );
                    None
                } else {
                    Some(Core::SetTextSelection(TextSelection::new(
                        selection.anchor.character_index,
                        selection.focus.character_index,
                    )))
                }
            }
            other => {
                tracing::trace!(?other, "accesskit value action dropped: no payload");
                None
            }
        },
        other => {
            tracing::trace!(?other, "accesskit action dropped: no widget contract yet");
            None
        }
    }
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
                #[cfg(feature = "backend-wgpu")]
                if let Some(Backend::Wgpu(backend)) = &mut self.backend {
                    backend.renderer.resize(width, height, &backend.surface);
                }
                #[cfg(feature = "backend-softbuffer")]
                if let Some(Backend::Softbuffer(presenter)) = &mut self.backend
                    && let Err(error) = presenter.resize(width, height)
                {
                    tracing::error!("softbuffer resize failed: {error}");
                }
                #[cfg(not(any(feature = "backend-wgpu", feature = "backend-softbuffer")))]
                let _ = (width, height);
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
                        click_count: self.clicks.on_down(self.cursor),
                    },
                    (ElementState::Released, MouseButton::Left) => PointerEvent::Up {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Left,
                    },
                    (ElementState::Pressed, MouseButton::Right) => PointerEvent::Down {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Right,
                        click_count: self.clicks.on_down(self.cursor),
                    },
                    (ElementState::Released, MouseButton::Right) => PointerEvent::Up {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Right,
                    },
                    (ElementState::Pressed, MouseButton::Middle) => PointerEvent::Down {
                        position: self.cursor,
                        button: alacer_core::events::MouseButton::Middle,
                        click_count: self.clicks.on_down(self.cursor),
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
                    // M10 (ADR-0016): the request's own action is the
                    // contract — map it faithfully onto the core vocabulary
                    // and let the widget refuse what it does not implement.
                    // The pre-M10 blanket press (every request munged to
                    // `DefaultPress`) turned a braille display's Increment
                    // into a click; that class of munging is the bug this
                    // mapping deletes.
                    if let Some(action) = map_action_request(&request)
                        && self
                            .pipeline
                            .dispatch_a11y_action(request.target_node.0, action)
                    {
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

/// The double-click window (GTK's default press-chain interval): two
/// presses inside it and within [`DOUBLE_CLICK_SLOP`] form one chain.
const DOUBLE_CLICK_TIME: std::time::Duration = std::time::Duration::from_millis(400);

/// The press-chain slop: a second press farther than this many pixels
/// from the previous one starts a fresh chain (GTK's double-click
/// distance).
const DOUBLE_CLICK_SLOP: f32 = 4.0;

/// Tracks the platform's click chain: presses within
/// [`DOUBLE_CLICK_TIME`] of the previous one and within
/// [`DOUBLE_CLICK_SLOP`] of it count up (1, 2, 3 — the single click, the
/// double-click, the triple), a press outside either bound starts a fresh
/// chain at 1. The count rides every [`PointerEvent::Down`] so widgets see
/// which gesture the user's hand made (§10.3, ADR-0019) — the platform
/// owns the clock, the widget never does.
#[derive(Default)]
struct ClickCounter {
    /// The previous press: when it landed and where.
    last: Option<(std::time::Instant, Point)>,
    /// The chain's running count.
    count: u8,
}

impl ClickCounter {
    /// Records a press at `at` and returns its click count.
    fn on_down(&mut self, at: Point) -> u8 {
        let now = std::time::Instant::now();
        let chained = self.last.take().is_some_and(|(then, before)| {
            now.duration_since(then) <= DOUBLE_CLICK_TIME
                && (at - before).length() <= DOUBLE_CLICK_SLOP
        });
        self.count = if chained {
            self.count.saturating_add(1)
        } else {
            1
        };
        self.last = Some((now, at));
        self.count
    }
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

/// The wgpu instance descriptor (the GPU path's own; M34, ADR-0040 —
/// the descriptor compiles with the feature that uses it).
#[cfg(feature = "backend-wgpu")]
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
    use alacer_core::geom::{Point, Vector};
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
    // M13 (ADR-0019): the platform's click chain — consecutive presses
    // within the slop count up, a press outside it resets. The time bound
    // rides `Instant` and is not sleep-tested; the distance bound is.
    // ------------------------------------------------------------------

    #[test]
    fn a_press_chain_counts_up_within_the_slop() {
        let mut clicks = super::ClickCounter::default();
        assert_eq!(clicks.on_down(Point::new(10.0, 10.0)), 1, "a fresh click");
        assert_eq!(
            clicks.on_down(Point::new(12.0, 10.0)),
            2,
            "2 px away: the double-click"
        );
        assert_eq!(
            clicks.on_down(Point::new(13.0, 11.0)),
            3,
            "the chain keeps counting (the triple)"
        );
    }

    #[test]
    fn a_far_press_starts_a_fresh_chain() {
        let mut clicks = super::ClickCounter::default();
        assert_eq!(clicks.on_down(Point::new(10.0, 10.0)), 1);
        assert_eq!(
            clicks.on_down(Point::new(100.0, 10.0)),
            1,
            "outside the slop: a fresh click, not a chain step"
        );
        assert_eq!(clicks.on_down(Point::new(100.0, 10.0)), 2);
    }

    // ------------------------------------------------------------------
    // M16 (ADR-0022): the error boundary carries its chain. source()
    // walks the winit/wgpu errors the variants hold instead of dropping
    // them, and the messages that know an escape hatch name it.
    // ------------------------------------------------------------------

    #[cfg(feature = "backend-wgpu")]
    use super::BackendError;
    use super::PlatformError;

    #[test]
    fn platform_error_sources_the_underlying_failure() {
        let error = PlatformError::Loop(winit::error::EventLoopError::ExitFailure(3));
        let source =
            std::error::Error::source(&error).expect("the loop's own error rides the chain");
        assert!(
            source
                .downcast_ref::<winit::error::EventLoopError>()
                .is_some(),
            "the chain keeps the loop error's type"
        );
    }

    #[cfg(feature = "backend-wgpu")]
    #[test]
    fn an_error_without_an_inner_error_sources_none() {
        // NoAdapter is a fact about the host, not a wrapped failure: the
        // PlatformError→BackendError hop is real, and the chain ends there.
        let error = PlatformError::Backend(BackendError::NoAdapter);
        let backend = std::error::Error::source(&error).expect("the backend error rides the chain");
        assert!(
            std::error::Error::source(backend).is_none(),
            "NoAdapter is a leaf: the chain ends at the platform's own fact"
        );
    }

    #[cfg(feature = "backend-softbuffer")]
    #[test]
    fn the_softbuffer_path_sources_the_present_path_error() {
        // M18 (ADR-0024): the last String leaf grew its chain — the
        // structured `SoftBufferError` rides `source()`, and a consumer
        // walks from the platform error to the present path's cause.
        let error = PlatformError::Softbuffer(::softbuffer::SoftBufferError::Unimplemented);
        let source =
            std::error::Error::source(&error).expect("the present-path error rides the chain");
        assert!(
            source
                .downcast_ref::<::softbuffer::SoftBufferError>()
                .is_some(),
            "the chain keeps the softbuffer error's type"
        );
        assert!(
            error.to_string().contains("softbuffer present path"),
            "the stage is still named for a human: {error}"
        );
    }

    #[cfg(feature = "backend-wgpu")]
    #[test]
    fn a_failed_surface_creation_carries_the_wgpu_cause() {
        // M18 (ADR-0024): the surface-creation String flattened the
        // `CreateSurfaceError`; the structured error rides the variant now.
        // wgpu's type is opaque (its inner kind is private), so the contract
        // is pinned at the type level — the payload a consumer downcasts
        // through `source()` implements `Error + 'static`.
        fn require_chainable<E: std::error::Error + 'static>() {}
        require_chainable::<wgpu::CreateSurfaceError>();
        let error: BackendError = BackendError::NoAdapter;
        assert!(
            error.to_string().contains("ALACER_SOFTWARE=1"),
            "the leaf keeps its escape-hatch help text: {error}"
        );
    }

    #[cfg(feature = "backend-wgpu")]
    #[test]
    fn the_no_adapter_message_names_the_software_escape_hatch() {
        let error = PlatformError::Backend(BackendError::NoAdapter);
        let message = error.to_string();
        assert!(
            message.contains("ALACER_SOFTWARE"),
            "the dead-end message names the softbuffer path: {message}"
        );
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
                selection: None,
                live: None,
                enabled: true,
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

    // ------------------------------------------------------------------
    // M34 (ADR-0040): the backend features are true. The refusal the
    // build itself makes is typed at the platform boundary — a fact
    // about the build, never a panic, never a munged fallback — and it
    // names the preference it cannot honor, the leaf rule the chain's
    // doc comment carries (nothing wrapped rides it).
    // ------------------------------------------------------------------

    #[test]
    fn the_build_refusal_names_its_preference_and_is_its_own_leaf() {
        let error = PlatformError::BackendUnavailable { requested: "wgpu" };
        assert_eq!(
            error.to_string(),
            "no present backend compiled: `wgpu` is gated off — enable backend-wgpu or backend-softbuffer",
            "the refusal names the gate that answers it"
        );
        assert!(
            std::error::Error::source(&error).is_none(),
            "a build fact is its own leaf — nothing wrapped rides the chain"
        );
    }

    #[test]
    fn preferences_name_themselves_for_the_refusal() {
        assert_eq!(super::BackendPreference::Wgpu.name(), "wgpu");
        assert_eq!(super::BackendPreference::Softbuffer.name(), "softbuffer");
    }

    // ------------------------------------------------------------------
    // The env's own override: `ALACER_SOFTWARE=1` is the platform's —
    // the parser is pure (the env is passed, never read), so the tests
    // drive every shape without mutating process state.
    // ------------------------------------------------------------------

    #[test]
    fn the_software_env_names_the_present_path() {
        use std::ffi::OsStr;
        // No env: the preference stands.
        assert_eq!(
            super::BackendPreference::Wgpu.software_env(None),
            super::BackendPreference::Wgpu,
            "an unset env overrides nothing"
        );
        // The env's answer: a wgpu preference becomes the software path.
        assert_eq!(
            super::BackendPreference::Wgpu.software_env(Some(OsStr::new("1"))),
            super::BackendPreference::Softbuffer,
            "ALACER_SOFTWARE=1 names the software path"
        );
        // Idempotent: a softbuffer preference is already the env's answer.
        assert_eq!(
            super::BackendPreference::Softbuffer.software_env(Some(OsStr::new("1"))),
            super::BackendPreference::Softbuffer,
            "the env never un-names the software path"
        );
    }

    #[test]
    fn a_junk_software_env_is_refused_and_the_preference_stands() {
        use std::ffi::OsStr;
        for junk in ["0", "yes", "true", "software", ""] {
            assert_eq!(
                super::BackendPreference::Wgpu.software_env(Some(OsStr::new(junk))),
                super::BackendPreference::Wgpu,
                "the value {junk:?} is nobody's answer — the preference stands, the warn names it"
            );
        }
    }
}
