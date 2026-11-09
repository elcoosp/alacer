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
    Glyph, GlyphBitmap, GlyphKey, GlyphSource, Primitive, Quad, SceneSnapshot,
};
use std::num::NonZeroU64;

/// WGSL: one pipeline for both primitive kinds; `kind` selects the fragment
/// path (SDF rounded rect vs atlas sample).
const SHADER: &str = r"
struct Uniforms {
    view: vec2<f32>,
    pad: vec2<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var s: sampler;
@group(0) @binding(2) var t: texture_2d<f32>;

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
    @location(1) geom: vec4<f32>,   // radius, border_width, kind, 0
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,     // quads: (w, h, 0, 0) · glyphs: (u0, v0, u1, v1)
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
    out.uv = uv;
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
    if (kind > 0.5) {
        // Glyph: bilinear sample of the coverage atlas.
        let size = vec2<f32>(in.geom.x, in.geom.y);
        let uv = mix(in.uv.xy, in.uv.zw, in.local / max(size, vec2<f32>(1.0, 1.0)));
        let coverage = textureSample(t, s, uv).r;
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
    /// Surface creation from the raw handle failed.
    SurfaceCreation(String),
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
        // The wgpu failures keep their chain; NoAdapter and the surface
        // creation string are the platform's own facts — the leaves.
        match self {
            Self::NoAdapter | Self::SurfaceCreation(_) => None,
            Self::NoDevice(e) => Some(e),
            Self::Surface(e) => Some(e),
        }
    }
}

/// The GPU glyph atlas: 8-bit coverage, shelf-packed, full-clear on overflow
/// (the documented M0 eviction policy; LRU lands with M3, §9.4).
struct Atlas {
    texture: wgpu::Texture,
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    entries: std::collections::HashMap<GlyphKey, [f32; 4]>,
    clears: u64,
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
            clears: 0,
        }
    }

    fn place(
        &mut self,
        queue: &wgpu::Queue,
        key: GlyphKey,
        bitmap: &GlyphBitmap,
    ) -> Option<[f32; 4]> {
        if let Some(&uv) = self.entries.get(&key) {
            return Some(uv);
        }
        let w = bitmap.width;
        let h = bitmap.height;
        if w == 0 || h == 0 || w > self.width || h > self.height {
            return None;
        }
        if self.cursor_x + w > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }
        if self.cursor_y + h > self.height {
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.row_height = 0;
            self.entries.clear();
            self.clears += 1;
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
            bitmap.coverage.as_ref(),
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
        self.entries.insert(key, uv);
        self.cursor_x += w;
        self.row_height = self.row_height.max(h);
        Some(uv)
    }

    /// Number of full clears (eviction events).
    #[must_use]
    pub const fn clears(&self) -> u64 {
        self.clears
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
    uniforms: wgpu::Buffer,
    instances: wgpu::Buffer,
    instance_capacity: u64,
    atlas: Atlas,
    width: u32,
    height: u32,
    glyphs_rendered: u64,
    instance_frames_skipped: u64,
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
        .map_err(|_| BackendError::NoAdapter)?;
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
            ],
        });

        let atlas = Atlas::new(&device, 2048, 2048);
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
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            ..wgpu::SamplerDescriptor::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &atlas
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
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
            uniforms,
            instances,
            instance_capacity,
            atlas,
            width: window_size.0.max(1),
            height: window_size.1.max(1),
            glyphs_rendered: 0,
            instance_frames_skipped: 0,
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

    /// Glyphs blitted this session (instrumentation).
    #[must_use]
    pub const fn glyphs_rendered(&self) -> u64 {
        self.glyphs_rendered
    }

    /// Frames skipped for exceeding the instance budget.
    #[must_use]
    pub const fn instance_frames_skipped(&self) -> u64 {
        self.instance_frames_skipped
    }

    /// Atlas full-clears (eviction events).
    #[must_use]
    pub const fn atlas_clears(&self) -> u64 {
        self.atlas.clears()
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
        let data = self.pack_instances(scene, glyphs);
        self.ensure_frame();
        let Some(frame_view) = self.frame_view.clone() else {
            return Ok(());
        };
        if self.upload_instances(&data) {
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
                self.draw_all(&mut pass, &data);
            }
            self.blitter
                .copy(&self.device, &mut encoder, &frame_view, &view);
            self.queue.submit([encoder.finish()]);
            self.frame_valid = true;
        } else {
            // The budget covers 32k primitives; the 10k-node churn workload
            // fits with room to spare. Skipping beats unbounded buffers.
            self.instance_frames_skipped += 1;
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
            // the software path's scissor filter.
            let instances: Vec<Instance> = all
                .iter()
                .filter(|instance| {
                    let [x, y, w, h] = instance.rect;
                    x < region.max_x()
                        && y < region.max_y()
                        && x + w > region.origin.x
                        && y + h > region.origin.y
                })
                .copied()
                .collect();
            // The synthetic background quad: the region's erase step, drawn
            // first (the software path's clear_region twin).
            let mut data = Vec::with_capacity(instances.len() + 1);
            data.push(background_instance(&region));
            data.extend(instances);
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
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_scissor_rect(scissor.0, scissor.1, scissor.2, scissor.3);
            let used = u64::try_from(data.len()).unwrap_or(0) * 64;
            pass.set_vertex_buffer(0, self.instances.slice(..used));
            let count = u32::try_from(data.len()).unwrap_or(0);
            pass.draw(0..6, 0..count);
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

    /// Instance packing in scene order: quads pack immediately, glyphs fill
    /// after the atlas resolves them.
    fn pack_instances(
        &mut self,
        scene: &SceneSnapshot,
        glyphs: &mut dyn GlyphSource,
    ) -> Vec<Instance> {
        let mut data: Vec<Instance> = Vec::with_capacity(scene.primitives.len());
        let mut glyph_slots: Vec<(usize, Glyph, GlyphBitmap)> = Vec::new();
        for (index, primitive) in scene.primitives.iter().enumerate() {
            match primitive {
                Primitive::Quad(quad) => data.push(pack_quad(quad)),
                Primitive::Glyph(glyph) => {
                    data.push(Instance {
                        rect: [0.0; 4],
                        geom: [0.0; 4],
                        color: [0.0; 4],
                        uv: [0.0; 4],
                    });
                    if let Some(bitmap) =
                        glyphs.glyph_bitmap(glyph.font_id, glyph.glyph_id, glyph.size)
                    {
                        glyph_slots.push((index, *glyph, bitmap));
                    }
                }
            }
        }
        for (index, glyph, bitmap) in glyph_slots {
            let key = atlas_key(&glyph);
            let Some(uv) = self.atlas.place(&self.queue, key, &bitmap) else {
                continue;
            };
            let x = f32_to_i32(glyph.position.x) + bitmap.left;
            let y = f32_to_i32(glyph.position.y) + bitmap.top;
            if let Some(slot) = data.get_mut(index) {
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
                self.glyphs_rendered += 1;
            }
        }
        data
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

    fn draw_all(&self, pass: &mut wgpu::RenderPass<'_>, data: &[Instance]) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        let used = u64::try_from(data.len()).unwrap_or(0) * 64;
        pass.set_vertex_buffer(0, self.instances.slice(..used));
        let count = u32::try_from(data.len()).unwrap_or(0);
        pass.draw(0..6, 0..count);
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
    use super::{Instance, atlas_key, pack_quad};
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
}
