use crate::camera::PerspectiveCamera;
use crate::input::InputState;
use eg_math::prelude::*;
use eg_primitive::prelude::*;
use eg_util::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    platform::web::{EventLoopExtWebSys, WindowBuilderExtWebSys},
    window::{WindowBuilder, WindowId},
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
struct UniformData {
    view_proj: Matrix4f,
    model: Matrix4f,
    mouse_move: [f32; 2],
    mouse_click: [f32; 2],
    resolution: [f32; 2],
    time: f32,
    _padding: [f32; 1],
}

#[derive(Debug)]
pub struct RenderState {
    window: web_sys::Window,
    canvas: web_sys::HtmlCanvasElement,
    pub winit_window: winit::window::Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_num: u32,
    camera: PerspectiveCamera,
    uniform_data: UniformData,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderState {
    pub async fn new() -> (Self, EventLoop<()>) {
        // Sample shape
        let (vertices, indices) = sample("cube").unwrap();
        // Window
        let window = web_sys::window().expect_throw("Failed to get the window");
        // Canvas
        let canvas = RenderState::init_canvas(&window).expect_throw("Failed to get the canvas");
        // winit event loop
        let event_loop =
            RenderState::create_event_loop().expect_throw("Failed to create an event loop");
        // winit window
        let winit_window = RenderState::create_window(&canvas, &event_loop)
            .expect_throw("Failed to create a window");
        // wgpu instance
        let instance = RenderState::create_instance();
        // wgpu surface
        let surface = RenderState::create_surface(&instance, &winit_window)
            .expect_throw("Failed to create a surface");
        // wgpu adapter
        let adapter = RenderState::create_adapter(&instance, &surface)
            .await
            .expect_throw("Failed to create an adapter");
        // wgpu device and queue
        let (device, queue) = RenderState::create_device_and_queue(&adapter)
            .await
            .expect_throw("Failed to create a device and a queue");
        // wgpu surface configuration
        let surface_config = RenderState::create_surface_configuration(
            &surface,
            &adapter,
            &device,
            canvas.width(),
            canvas.height(),
        );
        // wgpu vertex buffer
        let vertex_buffer =
            RenderState::create_vertex_buffer(&device, bytemuck::cast_slice(&vertices));
        // wgpu index buffer
        let index_buffer =
            RenderState::create_index_buffer(&device, bytemuck::cast_slice(&indices));
        // Camera
        let mut camera = PerspectiveCamera::new();
        let aspect = canvas.width() as f32 / canvas.height() as f32;
        camera.set_proj(None, Some(aspect), None, None);
        // wgpu uniform buffer
        let uniform_data = UniformData {
            view_proj: camera.view_proj,
            resolution: [canvas.width() as f32, canvas.height() as f32],
            mouse_move: [f32::MIN, f32::MIN],
            mouse_click: [f32::MIN, f32::MIN],
            ..Default::default()
        };
        let (uniform_buffer, uniform_layout, uniform_bind_group) =
            RenderState::create_uniform_buffer(&device, bytemuck::cast_slice(&[uniform_data][..]));
        // wgpu shader module
        let shader_module = RenderState::create_shader_module(&device);
        // wgpu render pipeline
        let render_pipeline = RenderState::create_render_pipeline(
            &device,
            &[&uniform_layout],
            &shader_module,
            &surface_config,
        );

        (
            Self {
                window,
                canvas,
                winit_window,
                surface,
                device,
                queue,
                surface_config,
                vertex_buffer,
                index_buffer,
                index_num: indices.len() as u32,
                camera,
                uniform_data,
                uniform_buffer,
                uniform_bind_group,
                render_pipeline,
            },
            event_loop,
        )
    }

    fn init_canvas(window: &web_sys::Window) -> Option<web_sys::HtmlCanvasElement> {
        let element = window.document()?.get_element_by_id("canvas0")?;
        let canvas = element.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;
        let (width, height) = (canvas.client_width() as u32, canvas.client_height() as u32);
        canvas.set_width(width);
        canvas.set_height(height);
        log!("Canvas size: ({}, {})", canvas.width(), canvas.height());
        Some(canvas)
    }

    fn create_event_loop() -> Option<EventLoop<()>> {
        EventLoop::new().ok()
    }

    fn create_window(
        canvas: &web_sys::HtmlCanvasElement,
        event_loop: &EventLoop<()>,
    ) -> Option<winit::window::Window> {
        // winit's window writes its physical and logical sizes in the HTML element
        // as attributes and inner style respectively
        // So, we should put "!important" in CSS to overwrite the inner style
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(
                canvas.client_width(),
                canvas.client_height(),
            ))
            .with_canvas(Some(canvas.clone()))
            .build(event_loop)
            .ok()?;
        log!(
            "Window size: ({}, {})",
            window.inner_size().width,
            window.inner_size().height
        );
        Some(window)
    }

    pub fn set_event_handlers(
        event_loop: EventLoop<()>,
        this_window_id: WindowId,
        input_state: Rc<RefCell<InputState>>,
    ) {
        // This is not an actual loop and hand events over to InputState
        event_loop.spawn(move |event, _, _control_flow| match event {
            Event::WindowEvent { event, window_id } if window_id == this_window_id => {
                input_state.borrow_mut().produce(event);
            }
            _ => (),
        });
    }

    fn create_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        })
    }

    fn create_surface(
        instance: &wgpu::Instance,
        winit_window: &winit::window::Window,
    ) -> Option<wgpu::Surface> {
        unsafe { instance.create_surface(winit_window).ok() }
    }

    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> Option<wgpu::Adapter> {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
    }

    async fn create_device_and_queue(
        adapter: &wgpu::Adapter,
    ) -> Option<(wgpu::Device, wgpu::Queue)> {
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .ok()
    }

    fn create_surface_configuration(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> wgpu::SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);
        surface_config
    }

    fn create_vertex_buffer(device: &wgpu::Device, contents: &[u8]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_index_buffer(device: &wgpu::Device, contents: &[u8]) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents,
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    fn create_uniform_buffer(
        device: &wgpu::Device,
        contents: &[u8],
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        (buffer, bind_group_layout, bind_group)
    }

    fn create_shader_module(device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("example.wgsl").into()),
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        shader_module: &wgpu::ShaderModule,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });
        let attributes = Vertex::vertex_attribute();
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "v_main",
                buffers: &[Vertex::layout(&attributes)],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: "f_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width != self.canvas.width() || new_height != self.canvas.height() {
            // Configuration of surface will make the canvas have the same size with it
            self.surface_config.width = new_width;
            self.surface_config.height = new_height;
            self.surface.configure(&self.device, &self.surface_config);
            debug_assert!(
                new_width == self.canvas.width() && new_height == self.canvas.height(),
                "Canvas couldn't resize itself"
            );
            debug_assert!(
                new_width == self.surface.get_current_texture().unwrap().texture.width()
                    && new_height == self.surface.get_current_texture().unwrap().texture.height(),
                "Surface couldn't resize itself"
            );

            // Update uniform data
            self.uniform_data.resolution = [new_width as f32, new_height as f32];

            // Update camera aspect ratio
            let aspect = new_width as f32 / new_height as f32;
            self.camera.set_proj(None, Some(aspect), None, None);

            log!("Resized: ({}, {})", new_width, new_height);
        }
    }

    pub fn mousemove(&mut self, x: f32, y: f32) {
        // Update uniform data
        self.uniform_data.mouse_move = [x, y];
    }

    pub fn click(&mut self) {
        // Update uniform data
        self.uniform_data.mouse_click = self.uniform_data.mouse_move;
    }

    pub fn render(&mut self, time: f32) {
        // Write uniform data to its buffer
        self.uniform_data.time = time * 0.001;

        // self.uniform_data.model = math::rotate_y(time * 0.001);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform_data][..]),
        );

        let surface_texture = self.surface.get_current_texture().unwrap();
        let texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.index_num, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }

    pub fn request_animation_frame(&self, cb: &Closure<dyn FnMut(f32)>) {
        self.window
            .request_animation_frame(cb.as_ref().unchecked_ref())
            .expect_throw("Failed to request an animation frame");
    }

    #[inline]
    pub fn get_scale_factor(&self) -> f64 {
        self.window.device_pixel_ratio()
    }

    #[inline]
    pub fn get_scaled_size(&self) -> (u32, u32) {
        let scale_factor = self.get_scale_factor() as f32;
        (
            (self.canvas.client_width() as f32 * scale_factor) as u32,
            (self.canvas.client_height() as f32 * scale_factor) as u32,
        )
    }

    #[inline]
    pub fn set_camera(
        &mut self,
        camera: Option<(f32, f32, f32)>,
        at: Option<(f32, f32, f32)>,
        up: Option<(f32, f32, f32)>,
    ) {
        self.camera.set_view(camera, at, up);
        self.uniform_data.view_proj = self.camera.view_proj;
    }
}
