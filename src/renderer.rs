// renderer.rs
use std::sync::Arc;
use std::{error::Error, fmt};
use winit::window::Window;

const MAX_RECTANGLES: usize = 1024;

#[derive(Debug)]
pub enum RenderError {
    SurfaceValidation,
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurfaceValidation => formatter.write_str("surface validation failed"),
        }
    }
}

impl Error for RenderError {}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RectUniform {
    position: [f32; 2],
    size: [f32; 2],
    viewport: [f32; 2],
    antialias_padding: f32,
    _padding: f32,
    radii: [f32; 4],
    color: [f32; 4],
    border_color: [f32; 4],
    border: [f32; 4],
}

impl RectUniform {
    fn centered(
        viewport: [f32; 2],
        scale_factor: f32,
        logical_size: [f32; 2],
        logical_radius: f32,
        color: [f32; 4],
    ) -> Self {
        Self::centered_with_radii(
            viewport,
            scale_factor,
            logical_size,
            [logical_radius; 4],
            color,
        )
    }

    fn centered_with_radii(
        viewport: [f32; 2],
        scale_factor: f32,
        logical_size: [f32; 2],
        logical_radii: [f32; 4],
        color: [f32; 4],
    ) -> Self {
        Self::centered_with_style(
            viewport,
            scale_factor,
            logical_size,
            logical_radii,
            color,
            0.0,
            [0.0; 4],
        )
    }

    fn centered_with_style(
        viewport: [f32; 2],
        scale_factor: f32,
        logical_size: [f32; 2],
        logical_radii: [f32; 4],
        color: [f32; 4],
        logical_border_width: f32,
        border_color: [f32; 4],
    ) -> Self {
        let scale_factor = scale_factor.max(f32::EPSILON);
        let size = [
            logical_size[0].max(0.0) * scale_factor,
            logical_size[1].max(0.0) * scale_factor,
        ];
        let radii = crate::raster::normalize_corner_radii(
            size,
            logical_radii.map(|radius| radius * scale_factor),
        );

        Self {
            position: [(viewport[0] - size[0]) * 0.5, (viewport[1] - size[1]) * 0.5],
            size,
            viewport,
            antialias_padding: 2.0,
            _padding: 0.0,
            radii,
            color,
            border_color,
            border: [
                (logical_border_width.max(0.0) * scale_factor).min(size[0].min(size[1]) * 0.5),
                0.0,
                0.0,
                0.0,
            ],
        }
    }
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    rect_uniform: RectUniform,
    rect_buffer: wgpu::Buffer,
    rect_bind_group: wgpu::BindGroup,
    rectangle_count: u32,
    scale_factor: f32,
    diagnostics: bool,
    started_at: std::time::Instant,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rounded_rect_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rounded_rect.wgsl").into()),
        });

        let rect_uniform = RectUniform::centered(
            [config.width as f32, config.height as f32],
            window.scale_factor() as f32,
            [400.0, 180.0],
            45.0,
            [0.913, 0.332, 0.003, 1.0],
        );
        let rect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect_uniform"),
            size: (MAX_RECTANGLES * std::mem::size_of::<RectUniform>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&rect_buffer, 0, bytemuck::bytes_of(&rect_uniform));
        let rect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("rect_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let rect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rect_bind_group"),
            layout: &rect_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rect_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[Some(&rect_bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rounded_rect_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            rect_uniform,
            rect_buffer,
            rect_bind_group,
            rectangle_count: 1,
            scale_factor: window.scale_factor() as f32,
            diagnostics: std::env::var_os("MIO_GUI_DIAGNOSTICS").is_some(),
            started_at: std::time::Instant::now(),
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if self.diagnostics {
            eprintln!(
                "resize_event t_us={} event={}x{} configured={}x{} scale={}",
                self.started_at.elapsed().as_micros(),
                size.width,
                size.height,
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
        }
        if size.width == 0 || size.height == 0 {
            return;
        }
        if self.config.width == size.width && self.config.height == size.height {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.rect_uniform = RectUniform::centered(
            [size.width as f32, size.height as f32],
            self.scale_factor,
            [400.0, 180.0],
            45.0,
            [0.913, 0.332, 0.003, 1.0],
        );
        self.upload_rectangles(&[self.rect_uniform]);
        if self.diagnostics {
            eprintln!(
                "surface_configured t_us={} configured={}x{}",
                self.started_at.elapsed().as_micros(),
                self.config.width,
                self.config.height,
            );
        }
    }

    pub fn scale_factor_changed(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor as f32;
        self.rect_uniform = RectUniform::centered(
            [self.config.width as f32, self.config.height as f32],
            self.scale_factor,
            [400.0, 180.0],
            45.0,
            [0.913, 0.332, 0.003, 1.0],
        );
        self.upload_rectangles(&[self.rect_uniform]);
    }

    fn upload_rectangles(&mut self, rectangles: &[RectUniform]) {
        assert!(rectangles.len() <= MAX_RECTANGLES);
        if !rectangles.is_empty() {
            self.queue
                .write_buffer(&self.rect_buffer, 0, bytemuck::cast_slice(rectangles));
        }
        self.rectangle_count = rectangles.len() as u32;
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        let mut recovery_attempted = false;
        let (frame, reconfigure_after_present) = loop {
            match self.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame) => break (frame, false),
                wgpu::CurrentSurfaceTexture::Suboptimal(frame) => break (frame, true),
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost
                    if !recovery_attempted =>
                {
                    self.surface.configure(&self.device, &self.config);
                    recovery_attempted = true;
                }
                wgpu::CurrentSurfaceTexture::Outdated
                | wgpu::CurrentSurfaceTexture::Lost
                | wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded => return Ok(()),
                wgpu::CurrentSurfaceTexture::Validation => {
                    return Err(RenderError::SurfaceValidation);
                }
            }
        };
        if self.diagnostics {
            eprintln!(
                "frame_acquired t_us={} configured={}x{} texture={}x{}",
                self.started_at.elapsed().as_micros(),
                self.config.width,
                self.config.height,
                frame.texture.width(),
                frame.texture.height(),
            );
        }
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.11,
                            g: 0.11,
                            b: 0.13,
                            a: 1.0,
                        }),
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
            pass.set_bind_group(0, &self.rect_bind_group, &[]);
            pass.draw(0..6, 0..self.rectangle_count);
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        if self.diagnostics {
            eprintln!(
                "frame_presented t_us={} configured={}x{}",
                self.started_at.elapsed().as_micros(),
                self.config.width,
                self.config.height,
            );
        }
        if reconfigure_after_present {
            self.surface.configure(&self.device, &self.config);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::RectUniform;
    use crate::raster::RoundedRectMask;
    use wgpu::util::DeviceExt;

    static GPU_TEST_LOCK: Mutex<()> = Mutex::new(());

    async fn render_pixels(
        dimensions: [u32; 2],
        rect: RectUniform,
    ) -> Result<Vec<[u8; 4]>, String> {
        render_pixel_batch(dimensions, &[rect]).await
    }

    async fn render_pixel_batch(
        dimensions: [u32; 2],
        rectangles: &[RectUniform],
    ) -> Result<Vec<[u8; 4]>, String> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|error| error.to_string())?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|error| error.to_string())?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rounded_rect_test_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/rounded_rect.wgsl").into()),
        });
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rounded_rect_test_uniform"),
            contents: bytemuck::cast_slice(rectangles),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rounded_rect_test_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rounded_rect_test_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rounded_rect_test_pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rounded_rect_test_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let extent = wgpu::Extent3d {
            width: dimensions[0],
            height: dimensions[1],
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rounded_rect_test_texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let unpadded_bytes_per_row = dimensions[0] * 4;
        let padded_bytes_per_row = unpadded_bytes_per_row
            .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rounded_rect_test_readback"),
            size: u64::from(padded_bytes_per_row) * u64::from(dimensions[1]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("rounded_rect_test_encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rounded_rect_test_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..6, 0..rectangles.len() as u32);
        }
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            extent,
        );
        let submission = queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .map_err(|error| error.to_string())?;
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| error.to_string())?;
        let mut pixels = Vec::with_capacity((dimensions[0] * dimensions[1]) as usize);
        for row in mapped.chunks(padded_bytes_per_row as usize) {
            for pixel in row[..unpadded_bytes_per_row as usize].chunks_exact(4) {
                pixels.push(pixel.try_into().unwrap());
            }
        }
        drop(mapped);
        readback.unmap();

        Ok(pixels)
    }

    fn compare_gpu_to_cpu(
        dimensions: [u32; 2],
        logical_size: [f32; 2],
        logical_radii: [f32; 4],
        scale_factor: f32,
    ) -> (f32, f32, u8) {
        let rect = RectUniform::centered_with_radii(
            [dimensions[0] as f32, dimensions[1] as f32],
            scale_factor,
            logical_size,
            logical_radii,
            [1.0; 4],
        );
        let gpu = pollster::block_on(render_pixels(dimensions, rect)).unwrap();
        let cpu = RoundedRectMask::with_radii(rect.size, rect.radii).rasterize_at(
            dimensions,
            rect.position,
            32,
        );
        let mut total_difference = 0.0_f32;
        let mut maximum_difference = 0.0_f32;
        let mut maximum_symmetry_difference = 0_u8;

        for (gpu_pixel, cpu_alpha) in gpu.iter().zip(&cpu) {
            let difference = (f32::from(gpu_pixel[3]) / 255.0 - cpu_alpha).abs();
            total_difference += difference;
            maximum_difference = maximum_difference.max(difference);
        }

        for y in 0..dimensions[1] {
            for x in 0..dimensions[0] {
                let index = (y * dimensions[0] + x) as usize;
                let reflected_x = (y * dimensions[0] + dimensions[0] - x - 1) as usize;
                let reflected_y = ((dimensions[1] - y - 1) * dimensions[0] + x) as usize;
                maximum_symmetry_difference = maximum_symmetry_difference
                    .max(gpu[index][3].abs_diff(gpu[reflected_x][3]))
                    .max(gpu[index][3].abs_diff(gpu[reflected_y][3]));
            }
        }

        let mean_difference = total_difference / cpu.len() as f32;
        (
            maximum_difference,
            mean_difference,
            maximum_symmetry_difference,
        )
    }

    #[test]
    fn uniform_layout_has_expected_size() {
        assert_eq!(std::mem::size_of::<RectUniform>(), 96);
    }

    #[test]
    fn centers_rectangle_in_viewport() {
        let rect = RectUniform::centered([800.0, 600.0], 1.0, [400.0, 180.0], 45.0, [1.0; 4]);

        assert_eq!(rect.position, [200.0, 210.0]);
        assert_eq!(rect.size, [400.0, 180.0]);
    }

    #[test]
    fn clamps_radius_to_half_of_shortest_side() {
        let rect = RectUniform::centered([800.0, 600.0], 1.0, [400.0, 180.0], 200.0, [1.0; 4]);

        assert_eq!(rect.radii, [90.0; 4]);
    }

    #[test]
    fn preserves_size_outside_viewport() {
        let rect = RectUniform::centered([320.0, 120.0], 1.0, [400.0, 180.0], 45.0, [1.0; 4]);

        assert_eq!(rect.position, [-40.0, -30.0]);
        assert_eq!(rect.size, [400.0, 180.0]);
        assert_eq!(rect.radii, [45.0; 4]);
    }

    #[test]
    fn converts_logical_geometry_to_physical_pixels() {
        let rect = RectUniform::centered([1600.0, 1200.0], 2.0, [400.0, 180.0], 45.0, [1.0; 4]);

        assert_eq!(rect.position, [400.0, 420.0]);
        assert_eq!(rect.size, [800.0, 360.0]);
        assert_eq!(rect.radii, [90.0; 4]);
    }

    #[test]
    fn clamps_negative_geometry_to_zero() {
        let rect = RectUniform::centered([800.0, 600.0], 1.0, [-400.0, -180.0], -45.0, [1.0; 4]);

        assert_eq!(rect.position, [400.0, 300.0]);
        assert_eq!(rect.size, [0.0, 0.0]);
        assert_eq!(rect.radii, [0.0; 4]);
    }

    #[test]
    fn proportionally_clamps_overlapping_corner_radii() {
        let rect = RectUniform::centered_with_radii(
            [100.0, 100.0],
            1.0,
            [40.0, 20.0],
            [20.0, 30.0, 10.0, 10.0],
            [1.0; 4],
        );

        assert_eq!(rect.radii, [10.0, 15.0, 5.0, 5.0]);
    }

    #[test]
    fn clamps_border_width_to_half_of_shortest_side() {
        let rect = RectUniform::centered_with_style(
            [100.0, 100.0],
            1.0,
            [40.0, 20.0],
            [4.0; 4],
            [1.0; 4],
            30.0,
            [0.0; 4],
        );

        assert_eq!(rect.border[0], 10.0);
    }

    #[test]
    fn gpu_renders_inward_border_and_fill_colors() {
        let _guard = GPU_TEST_LOCK.lock().unwrap();
        let dimensions = [64, 32];
        let rect = RectUniform::centered_with_style(
            [dimensions[0] as f32, dimensions[1] as f32],
            1.0,
            [40.0, 20.0],
            [5.0; 4],
            [1.0, 0.0, 0.0, 1.0],
            3.0,
            [0.0, 1.0, 0.0, 1.0],
        );
        let pixels = pollster::block_on(render_pixels(dimensions, rect)).unwrap();
        let pixel = |x: u32, y: u32| pixels[(y * dimensions[0] + x) as usize];

        assert_eq!(pixel(32, 16), [255, 0, 0, 255]);
        assert_eq!(pixel(32, 7), [0, 255, 0, 255]);
        assert_eq!(pixel(32, 5)[3], 0);
    }

    #[test]
    fn gpu_batches_multiple_rectangles_in_one_draw_call() {
        let _guard = GPU_TEST_LOCK.lock().unwrap();
        let dimensions = [64, 32];
        let mut left = RectUniform::centered(
            [dimensions[0] as f32, dimensions[1] as f32],
            1.0,
            [16.0, 16.0],
            4.0,
            [1.0, 0.0, 0.0, 1.0],
        );
        left.position = [6.0, 8.0];
        let mut right = RectUniform::centered(
            [dimensions[0] as f32, dimensions[1] as f32],
            1.0,
            [16.0, 16.0],
            4.0,
            [0.0, 0.0, 1.0, 1.0],
        );
        right.position = [42.0, 8.0];
        let pixels = pollster::block_on(render_pixel_batch(dimensions, &[left, right])).unwrap();
        let pixel = |x: u32, y: u32| pixels[(y * dimensions[0] + x) as usize];

        assert_eq!(pixel(14, 16), [255, 0, 0, 255]);
        assert_eq!(pixel(50, 16), [0, 0, 255, 255]);
        assert_eq!(pixel(32, 16), [0, 0, 0, 0]);
    }

    #[test]
    fn gpu_quad_preserves_antialias_coverage_outside_shape_boundary() {
        let _guard = GPU_TEST_LOCK.lock().unwrap();
        let dimensions = [48, 32];
        let mut rect = RectUniform::centered(
            [dimensions[0] as f32, dimensions[1] as f32],
            1.0,
            [20.0, 12.0],
            4.0,
            [1.0; 4],
        );
        rect.position = [10.75, 10.0];
        let pixels = pollster::block_on(render_pixels(dimensions, rect)).unwrap();
        let alpha = |x: u32, y: u32| pixels[(y * dimensions[0] + x) as usize][3];

        assert_eq!(alpha(9, 16), 0);
        assert!(alpha(10, 16) > 0 && alpha(10, 16) < 255);
        assert_eq!(alpha(11, 16), 255);
    }

    #[test]
    fn gpu_coverage_matches_shape_radius_and_scale_matrices() {
        let _guard = GPU_TEST_LOCK.lock().unwrap();
        let mut worst_maximum = (0.0_f32, String::new());
        let mut worst_mean = (0.0_f32, String::new());
        let mut worst_symmetry = (0_u8, String::from("all cases"));
        let cases = [
            ([0.0, 0.0], [0.0; 4]),
            ([0.5, 0.5], [0.25; 4]),
            ([32.0, 16.0], [0.0; 4]),
            ([32.0, 16.0], [1.0; 4]),
            ([32.0, 16.0], [4.0; 4]),
            ([32.0, 16.0], [8.0; 4]),
            ([32.0, 16.0], [100.0; 4]),
            ([20.0, 20.0], [5.0; 4]),
            ([40.0, 12.0], [3.0; 4]),
            ([12.0, 28.0], [3.0; 4]),
            ([2.0, 2.0], [1.0; 4]),
            ([31.0, 15.0], [3.75; 4]),
            ([40.0, 20.0], [2.0, 5.0, 8.0, 0.0]),
        ];

        for (size, radii) in cases {
            let metrics = compare_gpu_to_cpu([64, 32], size, radii, 1.0);
            let label = format!("size={size:?} radii={radii:?} scale=1");
            if metrics.0 > worst_maximum.0 {
                worst_maximum = (metrics.0, label.clone());
            }
            if metrics.1 > worst_mean.0 {
                worst_mean = (metrics.1, label.clone());
            }
            if radii.iter().all(|radius| *radius == radii[0]) && metrics.2 > worst_symmetry.0 {
                worst_symmetry = (metrics.2, label);
            }
        }

        for scale_factor in [1.0, 1.25, 1.5, 2.0, 3.0] {
            let metrics = compare_gpu_to_cpu([96, 48], [24.0, 12.0], [3.0; 4], scale_factor);
            let label = format!("size=[24, 12] radius=3 scale={scale_factor}");
            if metrics.0 > worst_maximum.0 {
                worst_maximum = (metrics.0, label.clone());
            }
            if metrics.1 > worst_mean.0 {
                worst_mean = (metrics.1, label.clone());
            }
            if metrics.2 > worst_symmetry.0 {
                worst_symmetry = (metrics.2, label);
            }
        }

        assert!(
            worst_maximum.0 <= 0.22,
            "maximum={} case={}",
            worst_maximum.0,
            worst_maximum.1,
        );
        assert!(
            worst_mean.0 <= 0.001,
            "mean={} case={}",
            worst_mean.0,
            worst_mean.1,
        );
        assert!(
            worst_symmetry.0 <= 1,
            "symmetry={} case={}",
            worst_symmetry.0,
            worst_symmetry.1,
        );
        eprintln!(
            "gpu coverage matrix: maximum={} case={}; mean={} case={}; symmetry={} case={}",
            worst_maximum.0,
            worst_maximum.1,
            worst_mean.0,
            worst_mean.1,
            worst_symmetry.0,
            worst_symmetry.1,
        );
    }
}
