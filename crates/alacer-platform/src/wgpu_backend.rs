//! The wgpu backend (ADR-2 primary): one instanced pipeline over the scene
//! journal.
//!
//! Quads render as rounded-rect SDFs (GPUI's scene pattern: corners and
//! borders are instance properties, never polygons). Glyphs render from a
//! CPU-rasterized coverage atlas — the *same* bitmaps the software backend
//! blends (A8: one rasterization path, two honest backends).
//!
//! Compile-verified on CI; headless machines exercise the software path (A4).

use alacer_core::num::f32_to_i32;
use alacer_core::scene::{
    Glyph, GlyphBitmap, GlyphKey, GlyphSource, Image, ImageSampling, Primitive, Quad, SceneSnapshot,
};
use std::num::NonZeroU64;

/// WGSL: one pipeline for all three primitive kinds; `kind` selects the
/// fragment path (SDF rounded rect vs coverage-atlas sample vs premultiplied
/// RGBA image sample).
const SHADER: &str = r"
struct Uniforms {
    view: vec2<f32>,
    pad: vec2<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var s: sampler;
@group(0) @binding(2) var t: texture_2d<f32>;
@group(0) @binding(3) var s_near: sampler;

const CORNERS = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
);

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) geom: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
    @location(0) rect: vec4<f32>,   // x, y, w, h (device px)
    @location(1) geom: vec4<f32>,   // radius, border_width, kind, nearest
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,     // quads: (w, h, 0, 0) · glyphs: (u0, v0, u1, v1) · images: (u0, v0, u1, v1)
) -> VOut {
    let corner = CORNERS[vi];
    let p = rect.xy + corner * rect.zw;
    var out: VOut;
    out.pos = vec4<f32>(
        p.x / uniforms.view.x * 2.0 - 1.0,
        1.0 - p.y / uniforms.view.y * 2.0,
        0.0,
        1.0,
    );
    out.local = corner * rect.zw;
    out.geom = geom;
    out.color = color;
    // The texture kinds (glyph kind 1, image kind 2) mix their atlas window
    // per corner here: corner is a per-vertex constant, so the mix equals
    // the fragment-side `local / size` a 1:1 blit computes — generalized to
    // destinations that do not match the source size (the image path's
    // scaled blits). Quads keep their raw (w, h) SDF payload.
    if (geom.z > 0.5) {
        out.uv = vec4<f32>(mix(uv.xy, uv.zw, corner), vec2<f32>(0.0, 0.0));
    } else {
        out.uv = uv;
    }
    return out;
}

fn rect_sdf(local: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(local) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let kind = in.geom.z;
    if (kind > 1.5) {
        // Image: the atlas row is premultiplied RGBA — sample and ship.
        // Nearest rides geom.w; `textureSampleLevel` because instance-driven
        // branching is not uniform control flow.
        if (in.geom.w > 0.5) {
            return textureSampleLevel(t, s_near, in.uv.xy, 0.0);
        }
        return textureSampleLevel(t, s, in.uv.xy, 0.0);
    }
    if (kind > 0.5) {
        // Glyph: bilinear sample of the coverage atlas (the window mixed
        // per corner in the vertex stage).
        let coverage = textureSample(t, s, in.uv.xy).r;
        return vec4<f32>(in.color.rgb * coverage, in.color.a * coverage);
    }
    // Quad: SDF fill + border ring.
    let half_size = in.uv.xy * 0.5;
    let d = rect_sdf(in.local - half_size, half_size, in.geom.x);
    let fill = smoothstep(0.5, -0.5, d);
    let ring = smoothstep(0.5, -0.5, abs(d) - in.geom.y * 0.5) * step(0.01, in.geom.y);
    let alpha = max(fill * (1.0 - ring), ring);
    return vec4<f32>(in.color.rgb * alpha, in.color.a * alpha);
}
";

/// Instance data, 64 bytes: one quad or one glyph.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    rect: [f32; 4],
    geom: [f32; 4],
    color: [f32; 4],
    uv: [f32; 4],
}

/// Backend failures, surfaced with the failing stage named.
#[derive(Debug)]
pub enum BackendError {
    /// No adapter matched the surface.
    NoAdapter,
    /// Surface creation from the raw handle failed (M18, ADR-0024): the
    /// structured [`wgpu::CreateSurfaceError`] rides the chain.
    SurfaceCreation(wgpu::CreateSurfaceError),
    /// Device or queue creation failed.
    NoDevice(wgpu::RequestDeviceError),
    /// Surface acquisition or other wgpu operation failed.
    Surface(wgpu::Error),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter => f.write_str(
                "no wgpu adapter for the surface (no GPU, or none the surface can use); \
                 set ALACER_SOFTWARE=1 for the softbuffer path",
            ),
            Self::SurfaceCreation(e) => write!(f, "surface creation failed: {e}"),
            Self::NoDevice(e) => write!(f, "wgpu device creation failed: {e}"),
            Self::Surface(e) => write!(f, "wgpu surface error: {e}"),
        }
    }
}

impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The wgpu failures keep their chain; `NoAdapter` is the platform's
        // own fact — the one honest leaf.
        match self {
            Self::NoAdapter => None,
            Self::SurfaceCreation(e) => Some(e),
            Self::NoDevice(e) => Some(e),
            Self::Surface(e) => Some(e),
        }
    }
}

/// The GPU glyph atlas: 8-bit coverage, shelf-packed, LRU eviction (§9.4's
/// pre-registered re-open, shipped — the M0 full-clear it replaces wiped a
/// hot working set at every overflow, the egui-shaped failure the section
/// names). Eviction order is the [`alacer_text::cache::Lru`] book's: every
/// hit refreshes, misses admit, the coldest placement goes first. When the
/// shelf itself exhausts, the survivors compact from the origin — the
/// retained coverage re-uploads, the freed glyphs stay gone — and the
/// scene's pack re-runs (the UVs handed out before a compaction are stale).
struct Atlas {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    entries: std::collections::HashMap<GlyphKey, GlyphSlot>,
    /// The eviction policy: the tick order and the eviction counter (the
    /// observability idiom the caches speak — hits, misses, evictions).
    book: alacer_text::cache::Lru<GlyphKey>,
    /// Area the resident placements cover, in pixels — the eviction
    /// target's arithmetic.
    used_area: u64,
    /// Whether the last [`Atlas::place`] compacted the shelf: the pack
    /// re-runs once when it did (bounded — see `PACK_ATTEMPTS`).
    rebuilt: bool,
}

/// One placed glyph: the atlas UV rect plus the coverage the placement
/// re-uploads from when the shelf compacts (the survivor's re-pack).
struct GlyphSlot {
    uv: [f32; 4],
    width: u32,
    height: u32,
    coverage: std::sync::Arc<[u8]>,
}

impl Atlas {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("alacer glyph atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST),
            view_formats: &[],
        });
        Self {
            texture,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            entries: std::collections::HashMap::new(),
            book: alacer_text::cache::Lru::new(),
            used_area: 0,
            rebuilt: false,
        }
    }

    /// The atlas UV rect at the current cursor, advancing it (the shelf
    /// bump the placement and the compaction's re-upload share).
    fn stamp(&mut self, w: u32, h: u32) -> [f32; 4] {
        let uv = [
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_x))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.width)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_y))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.height)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_x + w))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.width)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_y + h))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.height)),
        ];
        self.cursor_x += w;
        self.row_height = self.row_height.max(h);
        uv
    }

    fn advance_row_if_needed(&mut self, w: u32) {
        if self.cursor_x + w > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }
    }

    /// The shelf-bump write: one coverage bitmap at the cursor.
    fn upload(&self, queue: &wgpu::Queue, coverage: &[u8], w: u32, h: u32, x: u32, y: u32) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            coverage,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Re-packs every surviving placement from the origin (the LRU's
    /// compaction): the retained coverage re-uploads, the survivors' UVs
    /// refresh, the freed glyphs stay gone. The area ledger is unchanged —
    /// compaction moves, never evicts. Every UV handed out before this
    /// call is stale; the pack re-runs once when it happened.
    fn compact(&mut self, queue: &wgpu::Queue) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
        let survivors: Vec<(GlyphKey, GlyphSlot)> = self.entries.drain().collect();
        for (key, mut slot) in survivors {
            let (w, h) = (slot.width, slot.height);
            self.advance_row_if_needed(w);
            let (x, y) = (self.cursor_x, self.cursor_y);
            self.upload(queue, &slot.coverage, w, h, x, y);
            slot.uv = self.stamp(w, h);
            self.entries.insert(key, slot);
        }
        self.rebuilt = true;
    }

    fn place(
        &mut self,
        queue: &wgpu::Queue,
        key: GlyphKey,
        bitmap: &GlyphBitmap,
    ) -> Option<[f32; 4]> {
        if let Some(slot) = self.entries.get(&key) {
            let uv = slot.uv;
            self.book.touch(&key);
            return Some(uv);
        }
        let w = bitmap.width;
        let h = bitmap.height;
        if w == 0 || h == 0 || w > self.width || h > self.height {
            return None;
        }
        let needed = u64::from(w) * u64::from(h);
        let atlas_area = u64::from(self.width) * u64::from(self.height);

        // The LRU eviction: the coldest placements go until the new glyph
        // fits the atlas's area — the hot working set survives (its stamps
        // refresh every frame's hit), the cold tail pays the evictions.
        while atlas_area.saturating_sub(self.used_area) < needed && !self.book.is_empty() {
            if let Some(victim) = self.book.evict_lru()
                && let Some(slot) = self.entries.remove(&victim)
            {
                self.used_area -= u64::from(slot.width) * u64::from(slot.height);
            }
        }
        if atlas_area.saturating_sub(self.used_area) < needed {
            return None;
        }

        // The shelf itself exhausted: compact — the survivors re-pack from
        // the origin and the pack re-runs (the flag is read by the pack).
        if self.cursor_y + h > self.height {
            self.compact(queue);
        }
        self.advance_row_if_needed(w);
        let (x, y) = (self.cursor_x, self.cursor_y);
        self.upload(queue, &bitmap.coverage, w, h, x, y);
        let uv = self.stamp(w, h);
        self.used_area += needed;
        self.entries.insert(
            key,
            GlyphSlot {
                uv,
                width: w,
                height: h,
                coverage: std::sync::Arc::clone(&bitmap.coverage),
            },
        );
        self.book.admit(key);
        Some(uv)
    }
}

/// The GPU image atlas: premultiplied RGBA8, shelf-packed, full-clear on
/// overflow — the glyph atlas's own discipline (the shipped M0 eviction
/// policy; §9.4's LRU re-open rides the glyph atlas, never a shipped
/// promise here either). One placement per raster: the bytes ride by `Arc`
/// and every hit re-validates the identity, so a freed-and-reused pointer
/// never serves another image's pixels.
struct ImageAtlas {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    entries: std::collections::HashMap<usize, ImageSlot>,
}

/// One placed raster: the atlas UV rect and the `Arc` the placement is
/// keyed and validated by.
struct ImageSlot {
    uv: [f32; 4],
    width: u32,
    height: u32,
    bytes: std::sync::Arc<[u8]>,
}

impl ImageAtlas {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("alacer image atlas"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING.union(wgpu::TextureUsages::COPY_DST),
            view_formats: &[],
        });
        Self {
            texture,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            entries: std::collections::HashMap::new(),
        }
    }

    fn place(
        &mut self,
        queue: &wgpu::Queue,
        bytes: &std::sync::Arc<[u8]>,
        width: u32,
        height: u32,
    ) -> Option<[f32; 4]> {
        let key = std::sync::Arc::as_ptr(bytes).cast::<u8>() as usize;
        if let Some(slot) = self.entries.get(&key)
            && std::sync::Arc::ptr_eq(&slot.bytes, bytes)
            && slot.width == width
            && slot.height == height
        {
            return Some(slot.uv);
        }
        if width == 0 || height == 0 || width > self.width || height > self.height {
            return None;
        }
        let stride = u32::checked_mul(width, 4)?;
        if self.cursor_x + width > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }
        if self.cursor_y + height > self.height {
            // Full clear on overflow — the glyph atlas's own policy.
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.row_height = 0;
            self.entries.clear();
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: self.cursor_x,
                    y: self.cursor_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            bytes.as_ref(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let uv = [
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_x))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.width)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_y))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.height)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_x + width))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.width)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.cursor_y + height))
                / alacer_core::num::i32_to_f32(u32_to_i32(self.height)),
        ];
        self.entries.insert(
            key,
            ImageSlot {
                uv,
                width,
                height,
                bytes: std::sync::Arc::clone(bytes),
            },
        );
        self.cursor_x += width;
        self.row_height = self.row_height.max(height);
        Some(uv)
    }
}

/// The pack retry bound: each shelf compaction strictly grows the
/// contiguous room (survivors pack tight), so the pack converges for any
/// scene the atlas can hold. A scene whose glyph area exceeds the atlas
/// itself cannot converge and is served lossy — the degenerate class the
/// full-clear discipline the LRU replaced also served lossy (its cleared
/// glyphs' packed instances sampled whatever replaced them).
const PACK_ATTEMPTS: usize = 4;

/// Which atlas bind group a draw segment samples: the glyph coverage atlas
/// or the raster image atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TextureBind {
    Glyph,
    Image,
}

/// One scene's packed draw: instances in scene order plus the contiguous
/// segments that share a bind group — a segment switch is a bind group
/// switch, never a re-order (paint order is the correctness contract).
#[derive(Debug, Default)]
struct PackedScene {
    instances: Vec<Instance>,
    segments: Vec<(u32, u32, TextureBind)>,
}

impl PackedScene {
    /// Appends one instance to the segment its kind names: the open segment
    /// extends when the kind matches, otherwise a new one opens. Quads
    /// sample nothing and ride whatever segment is open (the glyph bind
    /// group works for them; a leading quad opens it).
    fn push(&mut self, instance: Instance, kind: TextureBind) {
        match self.segments.last_mut() {
            Some((_, count, open)) if *open == kind => *count += 1,
            _ => self
                .segments
                .push((u32::try_from(self.instances.len()).unwrap_or(0), 1, kind)),
        }
        self.instances.push(instance);
    }
}

/// The wgpu renderer: surface-bound, scene-consuming.
///
/// M3 partial paint (§9.3): the scene renders into an **offscreen frame
/// texture** that persists between frames; a partial frame loads it,
/// scissor-rects each damage region, repaints only the primitives reaching
/// into it, and blits to the swapchain. The software backend implements the
/// identical semantics over its persistent pixmap — the equivalence fuzz in
/// `alacer-testing` pins the two to the same pixel result (Q8).
pub struct WgpuRenderer {
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    /// The raster image atlas's bind group: same layout, the RGBA8 atlas
    /// texture behind binding 2 (the image fragment kind's sampler).
    image_bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    instances: wgpu::Buffer,
    instance_capacity: u64,
    atlas: Atlas,
    image_atlas: ImageAtlas,
    width: u32,
    height: u32,
    /// The persistent offscreen frame (GPU-side framebuffer, §9.3).
    frame_texture: Option<wgpu::Texture>,
    frame_view: Option<wgpu::TextureView>,
    /// The frame texture → swapchain blit (wgpu's util blitter: one
    /// fullscreen triangle sampling the source).
    blitter: wgpu::util::TextureBlitter,
    /// Whether the frame texture's contents are defined. Resize resets it;
    /// the first partial frame after a resize falls back to a full render.
    frame_valid: bool,
}

impl WgpuRenderer {
    /// Creates the renderer over an unconfigured surface.
    ///
    /// # Errors
    ///
    /// [`BackendError`] names the failing stage.
    #[expect(
        clippy::too_many_lines,
        reason = "GPU pipeline creation is one ordered setup narrative (instance, surface, device, pipeline, atlas); splitting it would scatter the setup state across helpers"
    )]
    pub fn new(
        surface: &wgpu::Surface<'_>,
        window_size: (u32, u32),
    ) -> Result<(Self, wgpu::SurfaceConfiguration), BackendError> {
        let instance = wgpu::Instance::new(instance_descriptor());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .map_err(|error| {
            // The typed contract stays the leaf (`NoAdapter` is the
            // platform's own fact about the host — the source chain ends
            // there), but the host's own answer is never dropped
            // silently: the clipboard seam's observability idiom (the
            // `tracing::warn!` naming the value it saw), at the adapter
            // seam.
            tracing::warn!(%error, "no wgpu adapter matched the surface; refusing the wgpu path");
            BackendError::NoAdapter
        })?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("alacer device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .map_err(BackendError::NoDevice)?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::default(),
            width: window_size.0.max(1),
            height: window_size.1.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("alacer scene shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alacer uniforms"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
            mapped_at_creation: false,
        });
        let instance_capacity = 64 * 32_768;
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alacer instances"),
            size: instance_capacity,
            usage: wgpu::BufferUsages::VERTEX.union(wgpu::BufferUsages::COPY_DST),
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("alacer bind layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The image path's crisp mode (the sampling enum's Nearest):
                // one filtering sampler, nearest-filtered.
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let atlas = Atlas::new(&device, 2048, 2048);
        let image_atlas = ImageAtlas::new(&device, 2048, 2048);
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("alacer bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&linear),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &atlas
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&nearest),
                },
            ],
        });
        let image_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("alacer image bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&linear),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &image_atlas
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&nearest),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("alacer pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("alacer scene pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 48,
                            shader_location: 3,
                        },
                    ],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let blitter = wgpu::util::TextureBlitter::new(&device, config.format);
        let renderer = Self {
            device,
            config: config.clone(),
            queue,
            pipeline,
            bind_group,
            image_bind_group,
            uniforms,
            instances,
            instance_capacity,
            atlas,
            image_atlas,
            width: window_size.0.max(1),
            height: window_size.1.max(1),
            frame_texture: None,
            frame_view: None,
            blitter,
            frame_valid: false,
        };
        Ok((renderer, config))
    }

    /// Resizes and reconfigures the surface. The offscreen frame is
    /// recreated lazily; its contents are undefined until the next full
    /// render (`frame_valid = false`).
    pub fn resize(&mut self, width: u32, height: u32, surface: &wgpu::Surface<'_>) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.config.width = self.width;
        self.config.height = self.height;
        surface.configure(&self.device, &self.config);
        self.frame_texture = None;
        self.frame_view = None;
        self.frame_valid = false;
    }

    /// The offscreen frame texture, created (or re-created) on demand.
    fn ensure_frame(&mut self) {
        if self.frame_texture.is_some() {
            return;
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("alacer frame"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                .union(wgpu::TextureUsages::TEXTURE_BINDING),
            view_formats: &[],
        });
        self.frame_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.frame_texture = Some(texture);
    }

    /// Renders one scene in full and presents (the [`alacer_core::damage::
    /// Damage::FULL`]-style path; also the fallback before the offscreen
    /// frame is valid).
    ///
    /// # Errors
    ///
    /// [`BackendError::Surface`] from acquisition or submission.
    pub fn render(
        &mut self,
        scene: &SceneSnapshot,
        glyphs: &mut dyn GlyphSource,
        surface: &wgpu::Surface<'_>,
    ) -> Result<(), BackendError> {
        let Some(frame) = Self::acquire(surface) else {
            return Ok(());
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let packed = self.pack_instances(scene, glyphs);
        self.ensure_frame();
        let Some(frame_view) = self.frame_view.clone() else {
            return Ok(());
        };
        // The budget covers 32k primitives; the 10k-node churn workload
        // fits with room to spare. Skipping beats unbounded buffers.
        if self.upload_instances(&packed.instances) {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("alacer frame"),
                });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("alacer scene pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(BACKGROUND),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                self.draw_all(&mut pass, &packed);
            }
            self.blitter
                .copy(&self.device, &mut encoder, &frame_view, &view);
            self.queue.submit([encoder.finish()]);
            self.frame_valid = true;
        }
        drop(frame); // present happens on drop (wgpu 30 contract)
        Ok(())
    }

    /// Repaints only `damage`'s regions and presents (§9.3, M3): the
    /// offscreen frame is loaded (`LoadOp::Load`), each region is cleared
    /// with a synthetic background quad and scissored, and the primitives
    /// intersecting the region are redrawn — then the frame blits to the
    /// swapchain. GPU damage semantics match the software path exactly.
    ///
    /// # Errors
    ///
    /// [`BackendError::Surface`] from acquisition or submission.
    pub fn render_partial(
        &mut self,
        scene: &SceneSnapshot,
        damage: &alacer_core::damage::Damage,
        glyphs: &mut dyn GlyphSource,
        surface: &wgpu::Surface<'_>,
    ) -> Result<(), BackendError> {
        let Some(frame) = Self::acquire(surface) else {
            return Ok(());
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.ensure_frame();
        let Some(frame_view) = self.frame_view.clone() else {
            return Ok(());
        };
        if !self.frame_valid {
            // Contents undefined (first frame / after resize): the full path
            // is the only correct fallback.
            drop(view);
            drop(frame);
            return self.render(scene, glyphs, surface);
        }
        let all = self.pack_instances(scene, glyphs);
        let viewport = alacer_core::geom::Size::new(
            alacer_core::num::i32_to_f32(u32_to_i32(self.width)),
            alacer_core::num::i32_to_f32(u32_to_i32(self.height)),
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("alacer partial frame"),
            });
        let mut drew_any = false;
        for region in damage.regions(viewport) {
            // Instances whose rect reaches into the region — the GPU twin of
            // the software path's scissor filter — grouped into the segments
            // their bind groups share, in scene order.
            let mut data: Vec<Instance> = Vec::new();
            let mut segments: Vec<(u32, u32, TextureBind)> = Vec::new();
            let mut open: Option<(usize, TextureBind)> = None;
            for &(start, count, kind) in &all.segments {
                let start = usize::try_from(start).unwrap_or(0);
                let end = start.saturating_add(usize::try_from(count).unwrap_or(0));
                for instance in all.instances.get(start..end).unwrap_or(&[]) {
                    let [x, y, w, h] = instance.rect;
                    let reaches = x < region.max_x()
                        && y < region.max_y()
                        && x + w > region.origin.x
                        && y + h > region.origin.y;
                    if !reaches {
                        continue;
                    }
                    match open {
                        Some((_, open_kind)) if open_kind == kind => {}
                        _ => {
                            open = Some((data.len(), kind));
                            segments.push((u32::try_from(data.len()).unwrap_or(0), 0, kind));
                        }
                    }
                    data.push(*instance);
                }
            }
            // The synthetic background quad leads the region's data (the
            // software path's clear_region twin); the first segment's count
            // closes over it. Segment counts derive from the starts.
            data.push(background_instance(&region));
            let total = u32::try_from(data.len()).unwrap_or(0);
            let mut ends: Vec<u32> = segments
                .iter()
                .skip(1)
                .map(|(start, _, _)| *start)
                .collect();
            ends.push(total);
            for ((start, count, _), end) in segments.iter_mut().zip(ends) {
                *count = end - *start;
            }
            if !self.upload_instances(&data) {
                continue;
            }
            let Some(scissor) = scissor_rect(&region, self.width, self.height) else {
                continue;
            };
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("alacer partial pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
            self.draw_segments(&mut pass, &segments);
            drew_any = true;
        }
        if drew_any {
            self.blitter
                .copy(&self.device, &mut encoder, &frame_view, &view);
            self.queue.submit([encoder.finish()]);
        }
        drop(frame); // present happens on drop (wgpu 30 contract)
        Ok(())
    }

    /// The swapchain texture, `None` when the surface is unavailable this
    /// frame (the caller skips the present — the frame stands).
    fn acquire(surface: &wgpu::Surface<'_>) -> Option<wgpu::SurfaceTexture> {
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => Some(texture),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                tracing::warn!("surface suboptimal; consider reconfiguring");
                Some(texture)
            }
            other => {
                tracing::debug!("surface unavailable this frame: {other:?}");
                None
            }
        }
    }

    /// Packs the scene, re-running the once-through pack when the glyph
    /// atlas compacted its shelf (every UV handed out before a compaction
    /// is stale, and the re-pack rides the CPU raster cache — the
    /// survivors' second pass is hits). Bounded by [`PACK_ATTEMPTS`].
    fn pack_instances(
        &mut self,
        scene: &SceneSnapshot,
        glyphs: &mut dyn GlyphSource,
    ) -> PackedScene {
        let mut packed = self.pack_instances_once(scene, glyphs);
        for _ in 0..PACK_ATTEMPTS {
            if !self.atlas.rebuilt {
                return packed;
            }
            self.atlas.rebuilt = false;
            packed = self.pack_instances_once(scene, glyphs);
        }
        packed
    }

    /// The once-through pack, in scene order: quads pack immediately,
    /// glyphs fill after the atlas resolves them, images after the image
    /// atlas places their raster (a refused raster — oversized, zero-size,
    /// or missing bytes — packs nothing, the honest skip the glyph path's
    /// own `None` shares).
    fn pack_instances_once(
        &mut self,
        scene: &SceneSnapshot,
        glyphs: &mut dyn GlyphSource,
    ) -> PackedScene {
        let mut packed = PackedScene {
            instances: Vec::with_capacity(scene.primitives.len()),
            segments: Vec::new(),
        };
        let mut glyph_slots: Vec<(usize, Glyph, GlyphBitmap)> = Vec::new();
        for primitive in &scene.primitives {
            match primitive {
                Primitive::Quad(quad) => packed.push(pack_quad(quad), TextureBind::Glyph),
                Primitive::Glyph(glyph) => {
                    // The placeholder's slot index rides the packing order —
                    // refused rasters skip their slot, so scene indices do
                    // not align with instance indices.
                    let slot = packed.instances.len();
                    packed.push(
                        Instance {
                            rect: [0.0; 4],
                            geom: [0.0; 4],
                            color: [0.0; 4],
                            uv: [0.0; 4],
                        },
                        TextureBind::Glyph,
                    );
                    if let Some(bitmap) =
                        glyphs.glyph_bitmap(glyph.font_id, glyph.glyph_id, glyph.size)
                    {
                        glyph_slots.push((slot, *glyph, bitmap));
                    }
                }
                Primitive::Image(image) => {
                    let Some(uv) = self.image_atlas.place(
                        &self.queue,
                        &image.bytes,
                        image.width,
                        image.height,
                    ) else {
                        continue;
                    };
                    packed.push(pack_image(image, image_uv(uv, image)), TextureBind::Image);
                }
            }
        }
        for (slot, glyph, bitmap) in glyph_slots {
            let key = atlas_key(&glyph);
            let Some(uv) = self.atlas.place(&self.queue, key, &bitmap) else {
                continue;
            };
            let x = f32_to_i32(glyph.position.x) + bitmap.left;
            let y = f32_to_i32(glyph.position.y) + bitmap.top;
            if let Some(slot) = packed.instances.get_mut(slot) {
                *slot = Instance {
                    rect: [
                        alacer_core::num::i32_to_f32(x),
                        alacer_core::num::i32_to_f32(y),
                        alacer_core::num::i32_to_f32(u32_to_i32(bitmap.width)),
                        alacer_core::num::i32_to_f32(u32_to_i32(bitmap.height)),
                    ],
                    geom: [
                        alacer_core::num::i32_to_f32(u32_to_i32(bitmap.width)),
                        alacer_core::num::i32_to_f32(u32_to_i32(bitmap.height)),
                        1.0,
                        0.0,
                    ],
                    color: [
                        f32::from(glyph.color.r) / 255.0,
                        f32::from(glyph.color.g) / 255.0,
                        f32::from(glyph.color.b) / 255.0,
                        f32::from(glyph.color.a) / 255.0,
                    ],
                    uv,
                };
            }
        }
        packed
    }

    /// Uploads the instance buffer; `false` when over budget (the frame is
    /// skipped — the budget beats unbounded buffers).
    fn upload_instances(&self, data: &[Instance]) -> bool {
        let used = u64::try_from(data.len()).unwrap_or(0) * 64;
        if used > self.instance_capacity {
            return false;
        }
        self.queue
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(data));
        self.queue.write_buffer(
            &self.uniforms,
            0,
            bytemuck::cast_slice(&[
                alacer_core::num::i32_to_f32(u32_to_i32(self.width)),
                alacer_core::num::i32_to_f32(u32_to_i32(self.height)),
                0.0,
                0.0,
            ]),
        );
        true
    }

    fn draw_all(&self, pass: &mut wgpu::RenderPass<'_>, packed: &PackedScene) {
        pass.set_pipeline(&self.pipeline);
        self.draw_segments(pass, &packed.segments);
    }

    /// Draws the packed segments: one bind group switch per kind change,
    /// the instance slice per segment — the scene order never re-orders.
    fn draw_segments(&self, pass: &mut wgpu::RenderPass<'_>, segments: &[(u32, u32, TextureBind)]) {
        for &(start, count, kind) in segments {
            let bind_group = match kind {
                TextureBind::Glyph => &self.bind_group,
                TextureBind::Image => &self.image_bind_group,
            };
            pass.set_bind_group(0, bind_group, &[]);
            let start_byte = u64::from(start) * 64;
            let used = u64::from(count) * 64;
            pass.set_vertex_buffer(0, self.instances.slice(start_byte..start_byte + used));
            pass.draw(0..6, 0..count);
        }
    }
}

/// The atlas key of a scene glyph.
#[must_use]
pub fn atlas_key(glyph: &Glyph) -> GlyphKey {
    GlyphKey {
        font_id: glyph.font_id,
        glyph_id: glyph.glyph_id,
        size_bits: glyph.size.to_bits(),
    }
}

/// The clear color: `alacer_widgets::theme::SURFACE` as raw wgpu RGB (the
/// platform crate cannot depend on the widget crate's constants inside WGSL
/// literals; the two must agree — a const test pins it in `softbuffer.rs`).
const BACKGROUND: wgpu::Color = wgpu::Color {
    r: 0.96,
    g: 0.96,
    b: 0.97,
    a: 1.0,
};

/// The synthetic background quad that erases one damage region before the
/// region's primitives redraw (the GPU twin of the software path's
/// `clear_region`): a sharp, unbordered quad covering the region.
fn background_instance(region: &alacer_core::geom::Rect) -> Instance {
    Instance {
        rect: [
            region.origin.x,
            region.origin.y,
            region.size.width,
            region.size.height,
        ],
        geom: [0.0, 0.0, 0.0, 0.0],
        color: [
            f32::from(BACKGROUND_COLOR_U8.0) / 255.0,
            f32::from(BACKGROUND_COLOR_U8.1) / 255.0,
            f32::from(BACKGROUND_COLOR_U8.2) / 255.0,
            1.0,
        ],
        uv: [region.size.width, region.size.height, 0.0, 0.0],
    }
}

/// `SURFACE` (245, 246, 248) as bytes — must equal [`BACKGROUND`].
const BACKGROUND_COLOR_U8: (u8, u8, u8) = (245, 246, 248);

/// The damage region as an integer scissor rect clamped to the target.
fn scissor_rect(
    region: &alacer_core::geom::Rect,
    width: u32,
    height: u32,
) -> Option<(u32, u32, u32, u32)> {
    let bound_w = alacer_core::num::u32_to_f32(width);
    let bound_h = alacer_core::num::u32_to_f32(height);
    let x0 = region.origin.x.max(0.0).floor();
    let y0 = region.origin.y.max(0.0).floor();
    let x1 = region.max_x().clamp(0.0, bound_w).floor();
    let y1 = region.max_y().clamp(0.0, bound_h).floor();
    let x = alacer_core::num::f32_to_u32(x0);
    let y = alacer_core::num::f32_to_u32(y0);
    let w = alacer_core::num::f32_to_u32(x1 - x0);
    let h = alacer_core::num::f32_to_u32(y1 - y0);
    if w == 0 || h == 0 || x >= width || y >= height {
        return None;
    }
    Some((x, y, w.min(width - x), h.min(height - y)))
}

fn u32_to_i32(v: u32) -> i32 {
    i32::try_from(v).unwrap_or(i32::MAX)
}

fn pack_quad(quad: &Quad) -> Instance {
    Instance {
        rect: [
            quad.rect.origin.x,
            quad.rect.origin.y,
            quad.rect.size.width,
            quad.rect.size.height,
        ],
        geom: [quad.corner_radius, quad.border_width, 0.0, 0.0],
        color: [
            f32::from(quad.color.r) / 255.0,
            f32::from(quad.color.g) / 255.0,
            f32::from(quad.color.b) / 255.0,
            f32::from(quad.color.a) / 255.0,
        ],
        uv: [quad.rect.size.width, quad.rect.size.height, 0.0, 0.0],
    }
}

/// Packs one image blit: the destination rect draws, the source window
/// rides the atlas UV, the sampling enum rides `geom.w` (1.0 = nearest),
/// and the color slot stays opaque white — the image fragment path reads
/// the atlas row verbatim, the premultiplied bytes carrying their own
/// colors.
fn pack_image(image: &Image, uv: [f32; 4]) -> Instance {
    Instance {
        rect: [
            image.dst.origin.x,
            image.dst.origin.y,
            image.dst.size.width,
            image.dst.size.height,
        ],
        geom: [
            alacer_core::num::u32_to_f32(image.width),
            alacer_core::num::u32_to_f32(image.height),
            2.0,
            match image.sampling {
                ImageSampling::Nearest => 1.0,
                ImageSampling::Bilinear => 0.0,
            },
        ],
        color: [1.0, 1.0, 1.0, 1.0],
        uv,
    }
}

/// The atlas UV window for an image's source rect: the atlas places the
/// whole bitmap, so the source window rides a sub-range of its UV rect,
/// clamped to the bitmap's pixel bounds (the software path's crop rule).
fn image_uv(full: [f32; 4], image: &Image) -> [f32; 4] {
    let width = alacer_core::num::u32_to_f32(image.width);
    let height = alacer_core::num::u32_to_f32(image.height);
    if width <= 0.0 || height <= 0.0 {
        return full;
    }
    let x0 = image.src.origin.x.clamp(0.0, width);
    let y0 = image.src.origin.y.clamp(0.0, height);
    let x1 = (image.src.origin.x + image.src.size.width).clamp(x0, width);
    let y1 = (image.src.origin.y + image.src.size.height).clamp(y0, height);
    let u_span = full[2] - full[0];
    let v_span = full[3] - full[1];
    [
        full[0] + x0 / width * u_span,
        full[1] + y0 / height * v_span,
        full[0] + x1 / width * u_span,
        full[1] + y1 / height * v_span,
    ]
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
    use super::{Image, ImageSampling, Instance, atlas_key, image_uv, pack_image, pack_quad};
    use alacer_core::color::Color;
    use alacer_core::geom::{Point, Rect, Size};
    use alacer_core::scene::{Glyph, Quad};

    #[test]
    fn quads_pack_into_instance_slots() {
        let quad = Quad {
            rect: Rect::new(Point::new(1.0, 2.0), Size::new(30.0, 10.0)),
            color: Color::rgb(255, 0, 0),
            corner_radius: 4.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        };
        let inst = pack_quad(&quad);
        assert_eq!(inst.rect, [1.0, 2.0, 30.0, 10.0]);
        assert_eq!(inst.geom[0], 4.0, "radius");
        assert_eq!(inst.geom[2], 0.0, "quad kind");
        assert_eq!(inst.color[0], 1.0);
        assert_eq!(inst.uv[0], 30.0, "quads carry w/h for the SDF via uv");
    }

    #[test]
    fn instance_is_64_bytes_pod() {
        assert_eq!(std::mem::size_of::<Instance>(), 64);
    }

    #[test]
    fn glyph_atlas_key_is_exact() {
        let glyph = Glyph {
            position: Point::zero(),
            size: 16.0,
            color: Color::BLACK,
            font_id: 7,
            glyph_id: 65,
            clip: None,
        };
        let key = atlas_key(&glyph);
        assert_eq!(key.font_id, 7);
        assert_eq!(key.size_bits, 16.0_f32.to_bits());
    }

    #[test]
    fn images_pack_into_instance_slots() {
        let image = Image {
            bytes: std::sync::Arc::from(vec![255_u8, 0, 0, 255].into_boxed_slice()),
            width: 4,
            height: 2,
            src: Rect::new(Point::new(0.0, 0.0), Size::new(4.0, 2.0)),
            dst: Rect::new(Point::new(1.0, 2.0), Size::new(8.0, 4.0)),
            clip: None,
            sampling: ImageSampling::Bilinear,
        };
        let inst = pack_image(&image, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(inst.rect, [1.0, 2.0, 8.0, 4.0], "the destination draws");
        assert_eq!(inst.geom[2], 2.0, "image kind");
        assert_eq!(inst.geom[3], 0.0, "bilinear rides geom.w");
        assert_eq!(inst.uv, [0.0, 0.0, 1.0, 1.0], "the atlas window verbatim");
    }

    #[test]
    fn image_uv_windows_the_atlas_rect() {
        let image = Image {
            bytes: std::sync::Arc::from(vec![255_u8, 0, 0, 255].into_boxed_slice()),
            width: 4,
            height: 1,
            src: Rect::new(Point::new(1.0, 0.0), Size::new(2.0, 1.0)),
            dst: Rect::new(Point::new(0.0, 0.0), Size::new(2.0, 1.0)),
            clip: None,
            sampling: ImageSampling::Nearest,
        };
        let window = image_uv([0.0, 0.0, 1.0, 1.0], &image);
        assert_eq!(
            window,
            [0.25, 0.0, 0.75, 1.0],
            "the source window is the atlas rect's sub-range"
        );
    }
}
