//! Alacer's platform: stock winit behind a thin seam (ADR-1 — never forked),
//! presenting through wgpu or softbuffer.
//!
//! §10.2 coalescing keeps input storms at one frame of work; §14.5 keeps the
//! main thread owning the frame loop. Idle is `ControlFlow::Wait` — zero CPU
//! between interactions (A7).

pub mod coalesce;
pub mod softbuffer;
pub mod wgpu_backend;

pub use coalesce::EventCoalescer;
pub use softbuffer::{SoftbufferPresenter, pixmap_to_argb};
pub use wgpu_backend::{BackendError, WgpuRenderer};

use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use alacer_core::events::PointerEvent;
use alacer_core::geom::{Point, Size};
use alacer_core::passes::PassPipeline;
use alacer_text::ThreadLocalGlyphSource;
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
    coalescer: EventCoalescer,
    backend_preference: BackendPreference,
    title: String,
    start_size: (f32, f32),
    window: Option<Arc<Window>>,
    backend: Option<Backend>,
    adapter: Option<Adapter>,
    proxy: Option<EventLoopProxy<PlatformEvent>>,
    cursor: Point,
    needs_present: bool,
    tree_dirty: bool,
}

impl AlacerApp {
    /// Assembles the app around a fully-wired [`PassPipeline`] (root mounted,
    /// bindings installed by the caller before `run`).
    #[must_use]
    pub fn new(
        pipeline: PassPipeline,
        title: impl Into<String>,
        size: (f32, f32),
        backend: BackendPreference,
    ) -> Self {
        Self {
            pipeline,
            glyph_source: ThreadLocalGlyphSource,
            coalescer: EventCoalescer::new(),
            backend_preference: backend,
            title: title.into(),
            start_size: size,
            window: None,
            backend: None,
            adapter: None,
            proxy: None,
            cursor: Point::zero(),
            needs_present: true,
            tree_dirty: true,
        }
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
        self.proxy = Some(event_loop.create_proxy());
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
        let Some(backend) = &mut self.backend else {
            return;
        };
        let report = match self.pipeline.tick(viewport) {
            Ok(report) => report,
            Err(error) => {
                tracing::error!("layout failed: {error}");
                return;
            }
        };
        let scene = report.scene;
        match backend {
            Backend::Wgpu(backend) => {
                if let Err(error) =
                    backend
                        .renderer
                        .render(&scene, &mut self.glyph_source, &backend.surface)
                {
                    tracing::error!("wgpu render failed: {error}");
                }
            }
            Backend::Softbuffer(presenter) => {
                let mut renderer = alacer_render::SoftwareRenderer::new();
                if let Some(pixmap) = renderer.render(
                    &scene,
                    size_px(viewport.width).max(1),
                    size_px(viewport.height).max(1),
                    alacer_widgets::theme::SURFACE,
                    &mut self.glyph_source,
                ) && let Err(error) = presenter.present(&pixmap)
                {
                    tracing::error!("present failed: {error}");
                }
            }
        }
        self.needs_present = false;
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

    fn request_tree_update(&mut self) {
        if let Some(adapter) = &mut self.adapter {
            adapter.update_if_active(|| {
                let snapshot = self.pipeline.a11y_snapshot().cloned().unwrap_or_default();
                let focus = snapshot
                    .root
                    .map_or(accesskit::NodeId(0), accesskit::NodeId);
                let mut nodes = Vec::new();
                for node in &snapshot.nodes {
                    let mut access_node = accesskit::Node::new(alacer_a11y::map_role(node.role));
                    access_node.set_label(node.label.as_str());
                    if let Some(bounds) = node.bounds {
                        access_node.set_bounds(alacer_a11y::map_bounds(bounds));
                    }
                    access_node.set_children(
                        node.children
                            .iter()
                            .map(|c| accesskit::NodeId(*c))
                            .collect::<Vec<_>>(),
                    );
                    nodes.push((accesskit::NodeId(node.id), access_node));
                }
                let tree = snapshot
                    .root
                    .map(|root| accesskit::TreeInfo::new(accesskit::NodeId(root)));
                TreeUpdateBuilder::build(nodes, tree, focus)
            });
            self.tree_dirty = false;
        }
    }
}

/// Helper that materializes a [`accesskit::TreeUpdate`] from prepared nodes.
struct TreeUpdateBuilder;

impl TreeUpdateBuilder {
    fn build(
        nodes: Vec<(accesskit::NodeId, accesskit::Node)>,
        tree: Option<accesskit::TreeInfo>,
        focus: accesskit::NodeId,
    ) -> accesskit::TreeUpdate {
        accesskit::TreeUpdate {
            nodes,
            tree,
            tree_id: accesskit::TreeId::ROOT,
            focus,
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

        let mut handled = false;
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
                if self.tree_dirty {
                    self.request_tree_update();
                }
                self.render_frame();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Point::new(
                    alacer_core::num::i32_to_f32(f64_to_i32(position.x)),
                    alacer_core::num::i32_to_f32(f64_to_i32(position.y)),
                );
                self.coalescer.push(PointerEvent::Move {
                    position: self.cursor,
                });
            }
            WindowEvent::CursorLeft { .. } => {
                self.coalescer.push(PointerEvent::Leave);
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
                self.coalescer.push(event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let amount: f32 = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 40.0,
                    MouseScrollDelta::PixelDelta(position) => {
                        alacer_core::num::i32_to_f32(f64_to_i32(position.y))
                    }
                };
                self.coalescer.push(PointerEvent::Wheel {
                    delta: alacer_core::geom::Vector::new(0.0, amount),
                    position: self.cursor,
                });
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        // Drain coalesced input and route it; any handling dirties the frame.
        for pointer in self.coalescer.drain() {
            if self.pipeline.dispatch_pointer(&pointer)
                == alacer_core::widget::EventResponse::Handled
            {
                handled = true;
            }
        }
        if handled && !self.pipeline.flux().take_pending_masks().is_empty() {
            self.needs_present = true;
        }
        if self.pipeline.flux().take_pending_masks().is_empty() && handled {
            self.needs_present = true;
        }
        if self.needs_present {
            window.request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: PlatformEvent) {
        match event {
            PlatformEvent::AccessKit(access) => match access.window_event {
                AccessKitWindowEvent::InitialTreeRequested => {
                    self.tree_dirty = true;
                    self.request_tree_update();
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
                self.tree_dirty = true;
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
