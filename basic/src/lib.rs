use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize, event_loop::EventLoop, platform::web::WindowBuilderExtWebSys,
    window::WindowBuilder,
};
mod camera;
use camera::PerspectiveCamera;
mod primitive;
use primitive::*;
mod color;

macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
struct UniformData {
    view_proj: [[f32; 4]; 4],
    mouse_move: [f32; 2],
    mouse_click: [f32; 2],
    resolution: [f32; 2],
    time: f32,
    _padding: [f32; 1],
}

#[derive(Debug)]
struct State {
    window: web_sys::Window,
    canvas: web_sys::HtmlCanvasElement,
    winit_window: winit::window::Window,
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
    animation_cb: Closure<dyn FnMut(f32)>,
}

static mut STATE: Option<State> = None;

impl State {
    async fn new() -> Self {
        // Our shapes
        let (vertices, indices) = shape::make_square(
            Vertex::new(&[-1.0, 1.0, 0.0], color::MAGENTA),
            Vertex::new(&[-1.0, -1.0, 0.0], color::BLUE),
            Vertex::new(&[1.0, 1.0, 0.0], color::YELLOW),
            Vertex::new(&[1.0, -1.0, 0.0], color::GREEN),
        );
        // window
        let window = web_sys::window().expect_throw("Failed to get the window");
        // canvas
        let canvas = State::init_canvas(&window).expect_throw("Failed to get the canvas");
        // winit window
        let winit_window = State::create_window(&canvas).expect_throw("Failed to create a window");
        // wgpu instance
        let instance = State::create_instance();
        // wgpu surface
        let surface = State::create_surface(&instance, &winit_window)
            .expect_throw("Failed to create a surface");
        // wgpu adapter
        let adapter = State::create_adapter(&instance, &surface)
            .await
            .expect_throw("Failed to create an adapter");
        // wgpu device and queue
        let (device, queue) = State::create_device_and_queue(&adapter)
            .await
            .expect_throw("Failed to create a device and a queue");
        // wgpu surface configuration
        let surface_config = State::create_surface_configuration(
            &surface,
            &adapter,
            &device,
            canvas.width(),
            canvas.height(),
        );
        // wgpu vertex buffer
        let vertex_buffer = State::create_vertex_buffer(&device, bytemuck::cast_slice(&vertices));
        // wgpu index buffer
        let index_buffer = State::create_index_buffer(&device, bytemuck::cast_slice(&indices));
        // camera
        let mut camera = PerspectiveCamera::new();
        let aspect = canvas.width() as f32 / canvas.height() as f32;
        camera.set_proj(None, Some(aspect), None, None);
        // wgpu uniform buffer
        let uniform_data = UniformData {
            view_proj: camera.to_view_proj(),
            resolution: [canvas.width() as f32, canvas.height() as f32],
            mouse_move: [std::f32::MIN, std::f32::MIN],
            mouse_click: [std::f32::MIN, std::f32::MIN],
            ..Default::default()
        };
        let (uniform_buffer, uniform_layout, uniform_bind_group) =
            State::create_uniform_buffer(&device, bytemuck::cast_slice(&[uniform_data][..]));
        // wgpu shader module
        let shader_module = State::create_shader_module(&device);
        // wgpu render pipeline
        let render_pipeline = State::create_render_pipeline(
            &device,
            &[&uniform_layout],
            &shader_module,
            &surface_config,
        );
        // animation_loop
        let animation_cb = State::create_animation_loop();

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
            animation_cb,
        }
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

    fn create_window(canvas: &web_sys::HtmlCanvasElement) -> Option<winit::window::Window> {
        // winit's window writes its physical and logical sizes in the HTML element
        // as attributes and inner style respectively
        // So, we should put "!important" in CSS to overwrite the inner style
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(
                canvas.client_width(),
                canvas.client_height(),
            ))
            .with_canvas(Some(canvas.clone()))
            .build(&EventLoop::new())
            .ok()?;
        log!(
            "Window size: ({}, {})",
            window.inner_size().width,
            window.inner_size().height
        );
        Some(window)
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
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
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
            usage: wgpu::BufferUsages::VERTEX,
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
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: "v_main",
                buffers: &[Vertex::layout()],
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

    fn create_animation_loop() -> Closure<dyn FnMut(f32)> {
        Closure::<dyn FnMut(f32)>::new(|time: f32| unsafe {
            let state = STATE.as_mut().unwrap_unchecked();
            state.render(time);
            state.request_animation_frame();
        })
    }

    #[inline(always)]
    fn request_animation_frame(&self) {
        self.window
            .request_animation_frame(self.animation_cb.as_ref().unchecked_ref())
            .expect_throw("Failed to request an animation frame");
    }

    fn add_event_listener(&mut self, type_: &str, f: impl Fn() + 'static) {
        let listener = Closure::<dyn Fn()>::new(f);
        self.window
            .add_event_listener_with_callback(type_, listener.as_ref().unchecked_ref())
            .expect("Failed to add an event listener");
        listener.forget(); // Leak, but it occurs just once
    }

    fn add_event_listener_with_mouseevent(
        &mut self,
        type_: &str,
        f: impl Fn(web_sys::MouseEvent) + 'static,
    ) {
        let listener = Closure::<dyn Fn(_)>::new(f);
        self.window
            .add_event_listener_with_callback(type_, listener.as_ref().unchecked_ref())
            .expect("Failed to add an event listener");
        listener.forget(); // Leak, but it occurs just once
    }

    fn resize(&mut self) {
        let new_width = self.canvas.client_width() as u32;
        let new_height = self.canvas.client_height() as u32;
        if new_width != self.canvas.width() || new_height != self.canvas.height() {
            // Synchronize manually
            self.canvas.set_width(new_width);
            self.canvas.set_height(new_height);
            self.winit_window
                .set_inner_size(PhysicalSize::new(new_width, new_height));
            self.surface_config.width = new_width;
            self.surface_config.height = new_height;
            self.surface.configure(&self.device, &self.surface_config);

            // Update uniform data
            self.uniform_data.resolution = [new_width as f32, new_height as f32];

            // Update camera aspect ratio
            let aspect = new_width as f32 / new_height as f32;
            self.camera.set_proj(None, Some(aspect), None, None);

            log!("Resized: ({}, {})", new_width, new_height);
        }
    }

    fn mousemove(&mut self, event: web_sys::MouseEvent) {
        // Update uniform data
        let br = self.canvas.get_bounding_client_rect();
        self.uniform_data.mouse_move = [event.client_x() as f32 - br.x() as f32, event.client_y() as f32 - br.y() as f32];
    }

    fn click(&mut self, event: web_sys::MouseEvent) {
        // Update uniform data
        let br = self.canvas.get_bounding_client_rect();
        self.uniform_data.mouse_click = [event.client_x() as f32 - br.x() as f32, event.client_y() as f32 - br.y() as f32];
    }

    fn render(&mut self, time: f32) {
        // Write uniform data to its buffer
        self.uniform_data.time = time * 0.001;
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

    fn print(&self) {
        log!("{self:?}");
    }

    fn set_camera(
        &mut self,
        eye: Option<(f32, f32, f32)>,
        center: Option<(f32, f32, f32)>,
        up: Option<(f32, f32, f32)>,
    ) {
        self.camera.set_view(eye, center, up);
        self.uniform_data.view_proj = self.camera.to_view_proj();
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
}

#[wasm_bindgen]
pub async fn run() {
    unsafe {
        STATE = Some(State::new().await);
        let state = STATE.as_mut().unwrap_unchecked();
        state.add_event_listener("resize", || STATE.as_mut().unwrap_unchecked().resize());
        state.add_event_listener_with_mouseevent("mousemove", |event: web_sys::MouseEvent| {
            STATE.as_mut().unwrap_unchecked().mousemove(event)
        });
        state.add_event_listener_with_mouseevent("click", |event: web_sys::MouseEvent| {
            STATE.as_mut().unwrap_unchecked().click(event)
        });
        state.request_animation_frame();
    };
}

#[wasm_bindgen]
pub fn print_self() {
    unsafe {
        STATE.as_ref().unwrap_unchecked().print();
    }
}

#[wasm_bindgen]
pub fn set_camera(eye_x: f32, eye_y: f32, eye_z: f32, center_x: f32, center_y: f32, center_z: f32) {
    unsafe {
        STATE.as_mut().unwrap_unchecked().set_camera(
            Some((eye_x, eye_y, eye_z)),
            Some((center_x, center_y, center_z)),
            None,
        );
    }
}
