//! The headless wgpu renderer (§9.3, Q8): the surface-free twin of
//! [`super::wgpu_backend::WgpuRenderer`] that renders a
//! [`SceneSnapshot`] into an offscreen texture and reads the pixels back
//! to a [`tiny_skia::Pixmap`] — the cross-backend equivalence fuzz's GPU
//! side.
//!
//! A4's software-first discipline made the wgpu backend compile-verified
//! only: no headless machine could prove the GPU path draws what the
//! software path draws, so Q8's "the equivalence fuzz runs against both
//! backends" stayed a plan demand. This module is the capability that
//! closes it: the same shader, the same scene packing, the same
//! premultiplied blending — over an adapter requested with **no
//! surface** (`Backends::VULKAN | Backends::GL`, both able to enumerate
//! headless devices — lavapipe and surfaceless EGL), a render into an
//! offscreen `Rgba8Unorm` target, and a
//! [`wgpu::util::DownloadBuffer`] readback that handles the 256-byte
//! `bytes_per_row` alignment GPUs mandate.
//!
//! Why `Rgba8Unorm` and not an sRGB target: the software backend blends
//! in the stored (encoded) byte space — `tiny_skia`'s premultiplied
//! arithmetic — and an `Unorm` texture blends the GPU's encoded values
//! the same way. The cross-backend comparison stays honest; the 1%
//! pixel-diff tolerance in the fuzz covers only the float-vs-fixed-point
//! rounding both paths are entitled to.

use super::wgpu_backend::{
    Atlas, BACKGROUND, BackendError, ImageAtlas, PackedScene, SHADER, TextureBind,
    instance_descriptor, pack_scene,
};
use alacer_core::num::i32_to_f32;
use alacer_core::scene::{GlyphSource, SceneSnapshot};
use std::num::NonZeroU64;
use std::sync::mpsc;

/// The readback staging format: one premultiplied RGBA8 row per texture
/// row, the exact pixel layout [`tiny_skia::Pixmap`] owns.
const READBACK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// GPU buffer-map row alignment: every `bytes_per_row` must be a multiple
/// of [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`] (256), so a width whose bytes
/// do not fill one leaves padding the de-pad step skips.
fn aligned_bytes_per_row(width: u32) -> u32 {
    let raw = width * 4;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    raw.div_ceil(alignment) * alignment
}

/// Renders scenes offscreen and reads the pixels back — the headless
/// twin of the surface [`super::wgpu_backend::WgpuRenderer`].
///
/// The pipeline is the surface renderer's verbatim (one instanced draw
/// over the scene journal's primitives, SDF quads, coverage-atlas glyphs,
/// premultiplied image blits); the adapter is requested without a
/// surface, so the module works on CI machines with a software Vulkan
/// driver and fails honestly with [`BackendError::NoAdapter`] where no
/// device exists at all.
pub struct HeadlessRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    image_bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    instances: wgpu::Buffer,
    /// The parallel clip column (one vec4 per instance slot): the shader's
    /// second instance buffer, the GPU side of the software path's rect
    /// masks.
    clips: wgpu::Buffer,
    instance_capacity: u64,
    atlas: Atlas,
    image_atlas: ImageAtlas,
    width: u32,
    height: u32,
}

impl HeadlessRenderer {
    /// Creates the renderer over a headless adapter.
    ///
    /// # Errors
    ///
    /// [`BackendError::NoAdapter`] when no headless device exists (the
    /// honest refusal A4's software-first discipline rides);
    /// [`BackendError::NoDevice`] when the adapter refuses the limits.
    #[expect(
        clippy::too_many_lines,
        reason = "GPU pipeline creation is one ordered setup narrative (instance, adapter, device, pipeline, atlases); splitting it would scatter the setup state across helpers"
    )]
    pub fn new(width: u32, height: u32) -> Result<Self, BackendError> {
        let mut descriptor = instance_descriptor();
        descriptor.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
        let instance = wgpu::Instance::new(descriptor);
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .map_err(|error| {
            tracing::warn!(%error, "no headless wgpu adapter; the equivalence fuzz cannot run here");
            BackendError::NoAdapter
        })?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("alacer headless device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        }))
        .map_err(BackendError::NoDevice)?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("alacer scene shader (headless)"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alacer headless uniforms"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM.union(wgpu::BufferUsages::COPY_DST),
            mapped_at_creation: false,
        });
        let instance_capacity = 64 * 32_768;
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alacer headless instances"),
            size: instance_capacity,
            usage: wgpu::BufferUsages::VERTEX.union(wgpu::BufferUsages::COPY_DST),
            mapped_at_creation: false,
        });
        let clips = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alacer headless clip+border aux"),
            size: 32 * 32_768,
            usage: wgpu::BufferUsages::VERTEX.union(wgpu::BufferUsages::COPY_DST),
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("alacer headless bind layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(32),
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
            label: Some("alacer headless glyph bind group"),
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
            label: Some("alacer headless image bind group"),
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
            label: Some("alacer headless pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("alacer headless scene pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
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
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: 32,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 4,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 5,
                            },
                        ],
                    }),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    // The readback twin of the software path's blend space
                    // (see the module header): encoded-value blending.
                    format: READBACK_FORMAT,
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

        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group,
            image_bind_group,
            uniforms,
            instances,
            clips,
            instance_capacity,
            atlas,
            image_atlas,
            width: width.max(1),
            height: height.max(1),
        })
    }

    /// The viewport this renderer reads back, in pixels.
    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Renders `scene` offscreen and reads the frame back as a
    /// premultiplied [`tiny_skia::Pixmap`] — the byte twin of the software
    /// backend's output.
    ///
    /// # Errors
    ///
    /// [`BackendError::Readback`] names the failing stage when the copy,
    /// map, or poll does not deliver the frame's bytes.
    pub fn render(
        &mut self,
        scene: &SceneSnapshot,
        glyphs: &mut dyn GlyphSource,
    ) -> Result<tiny_skia::Pixmap, BackendError> {
        let packed = self.pack(scene, glyphs);
        if !self.upload(&packed) {
            return Err(BackendError::Readback(
                "instance budget exceeded".to_owned(),
            ));
        }
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("alacer headless frame"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: READBACK_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::COPY_SRC),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("alacer headless frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("alacer headless scene pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
            pass.set_pipeline(&self.pipeline);
            for &(start, count, kind) in &packed.segments {
                let bind_group = match kind {
                    TextureBind::Glyph => &self.bind_group,
                    TextureBind::Image => &self.image_bind_group,
                };
                pass.set_bind_group(0, bind_group, &[]);
                let start_byte = u64::from(start) * 64;
                let used = u64::from(count) * 64;
                pass.set_vertex_buffer(0, self.instances.slice(start_byte..start_byte + used));
                let aux_start = u64::from(start) * 32;
                let aux_used = u64::from(count) * 32;
                pass.set_vertex_buffer(1, self.clips.slice(aux_start..aux_start + aux_used));
                pass.draw(0..6, 0..count);
            }
        }
        let bytes_per_row = aligned_bytes_per_row(self.width);
        let padded_size = u64::from(bytes_per_row) * u64::from(self.height);
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("alacer headless readback"),
            size: padded_size,
            // `COPY_SRC` is `DownloadBuffer::read_buffer`'s contract (it
            // stages its own MAP_READ copy); `COPY_DST` receives the
            // texture blit.
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let bytes = Self::download(&self.device, &self.queue, &readback, padded_size)?;
        Ok(depadded_pixmap(
            &bytes,
            bytes_per_row,
            self.width,
            self.height,
        ))
    }

    /// Blocks on the staging buffer's map callback: the copy is submitted,
    /// [`wgpu::Device::poll`] waits it out, and the callback hands the
    /// mapped bytes over the channel — the async readback's synchronous
    /// face, exactly the pattern the `DownloadBuffer` docs prescribe.
    fn download(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        size: u64,
    ) -> Result<Vec<u8>, BackendError> {
        let (sender, receiver) = mpsc::channel();
        wgpu::util::DownloadBuffer::read_buffer(
            device,
            queue,
            &buffer.slice(0..size),
            move |result| {
                // The callback runs on the poll below; a disconnected
                // receiver means the caller panicked first — dropping the
                // result is the only honest move then.
                let _ = sender.send(result);
            },
        );
        let _polled = device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| BackendError::Readback(format!("poll failed: {error}")))?;
        let download = receiver
            .recv()
            .map_err(|_| {
                BackendError::Readback("map callback never ran; the device is lost".to_owned())
            })?
            .map_err(|error| {
                BackendError::Readback(format!("staging buffer map failed: {error}"))
            })?;
        let bytes = download.to_vec();
        Ok(bytes)
    }

    /// Packs the scene, re-running the once-through pack when the glyph
    /// atlas compacted its shelf (the surface renderer's bounded re-pack).
    fn pack(&mut self, scene: &SceneSnapshot, glyphs: &mut dyn GlyphSource) -> PackedScene {
        let Self {
            queue,
            atlas,
            image_atlas,
            ..
        } = self;
        let mut packed = pack_scene(scene, glyphs, atlas, image_atlas, queue);
        for _ in 0..super::wgpu_backend::PACK_ATTEMPTS {
            if !atlas.rebuilt {
                return packed;
            }
            atlas.rebuilt = false;
            packed = pack_scene(scene, glyphs, atlas, image_atlas, queue);
        }
        packed
    }

    /// Uploads the instance buffer and its parallel clip column; `false`
    /// when over budget (the frame is refused — the budget beats unbounded
    /// buffers).
    fn upload(&self, packed: &PackedScene) -> bool {
        let used = u64::try_from(packed.instances.len()).unwrap_or(0) * 64;
        if used > self.instance_capacity {
            return false;
        }
        self.queue
            .write_buffer(&self.instances, 0, bytemuck::cast_slice(&packed.instances));
        self.queue
            .write_buffer(&self.clips, 0, bytemuck::cast_slice(&packed.aux));
        self.queue.write_buffer(
            &self.uniforms,
            0,
            bytemuck::cast_slice(&[
                i32_to_f32(super::wgpu_backend::u32_to_i32(self.width)),
                i32_to_f32(super::wgpu_backend::u32_to_i32(self.height)),
                0.0,
                0.0,
                super::wgpu_backend::BACKGROUND_RGBA_F32[0],
                super::wgpu_backend::BACKGROUND_RGBA_F32[1],
                super::wgpu_backend::BACKGROUND_RGBA_F32[2],
                super::wgpu_backend::BACKGROUND_RGBA_F32[3],
            ]),
        );
        true
    }
}

/// De-pads the aligned staging rows into a tight premultiplied
/// [`tiny_skia::Pixmap`]: row `y`'s pixels start at `y * bytes_per_row`;
/// the tail padding each 256-byte-aligned row carries is skipped, and the
/// GPU's premultiplied output IS the pixmap's own byte layout.
fn depadded_pixmap(bytes: &[u8], bytes_per_row: u32, width: u32, height: u32) -> tiny_skia::Pixmap {
    let stride = usize::try_from(bytes_per_row).unwrap_or(0);
    let row_bytes = usize::try_from(width).unwrap_or(0) * 4;
    let height_usize = usize::try_from(height).unwrap_or(0);
    let mut tight = Vec::with_capacity(row_bytes * height_usize);
    for row in 0..height_usize {
        let start = row * stride;
        let end = start + row_bytes;
        match bytes.get(start..end) {
            Some(slice) => tight.extend_from_slice(slice),
            // An undersized staging buffer is a `BackendError` upstream;
            // the zero-fill fallback keeps the pixmap allocation honest.
            None => tight.resize(row_bytes * height_usize, 0),
        }
    }
    // `tight` holds exactly `width × height × 4` premultiplied bytes by
    // construction — the layout `from_vec` requires.
    let size = tiny_skia::IntSize::from_wh(width, height).expect("non-zero viewport");
    tiny_skia::Pixmap::from_vec(tight, size)
        .expect("the de-padded bytes are exactly width × height × 4")
}

#[cfg(test)]
mod tests {
    use super::{aligned_bytes_per_row, depadded_pixmap};

    #[test]
    fn bytes_per_row_aligns_to_the_gpu_row_stride() {
        // 64 px × 4 B = 256 B: exactly one alignment unit, no padding.
        assert_eq!(aligned_bytes_per_row(64), 256);
        // 160 px × 4 B = 640 B: 2.5 alignment units — rounds up to the
        // next 256 boundary.
        assert_eq!(aligned_bytes_per_row(160), 768);
        // 161 px × 4 B = 644 B: rounds up to the next 256 boundary.
        assert_eq!(aligned_bytes_per_row(161), 768);
        // 1 px: the minimum aligned row.
        assert_eq!(aligned_bytes_per_row(1), 256);
    }

    #[test]
    fn depad_strips_row_padding_into_a_tight_pixmap() {
        // Two rows of 3 px each; the aligned stride is one 256-byte row.
        let mut staged = vec![0_u8; 256 * 2];
        for (x, r) in [(0_u32, 10_u8), (1, 20), (2, 30)] {
            staged[usize::try_from(x).unwrap_or(0) * 4 + 3] = r; // row 0: alpha column
        }
        let pixmap = depadded_pixmap(&staged, 256, 3, 2);
        assert_eq!(pixmap.width(), 3);
        assert_eq!(pixmap.height(), 2);
        assert_eq!(pixmap.data().len(), 3 * 2 * 4, "tight bytes, no padding");
        assert_eq!(pixmap.data()[3], 10);
        assert_eq!(pixmap.data()[7], 20);
        assert_eq!(pixmap.data()[11], 30);
        // Row 1 lives at the padded stride, not the tight one: its first
        // pixel is tight-pixel 3 (row 1, x 0), and the staging bytes the
        // de-pad copies for it are the row's own zeros.
        let row1_alpha = pixmap.data()[3 * 4 + 3];
        assert_eq!(row1_alpha, 0, "row 1 was never written");
    }
}
