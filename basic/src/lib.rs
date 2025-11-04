use std::{cell::RefCell, mem, ops};
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

thread_local! {
    static STATE: RefCell<State> = panic!();
}

#[wasm_bindgen]
pub async fn run() {
    // When panics, we can see error messages on the console.
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Creates a new state.
    let vertices = vec![
        // Top-left (magenta)
        Vertex {
            pos: [-1.0, 1.0, 0.0],
            color: [1.0, 0.0, 1.0],
        },
        // Bottom-left (blue)
        Vertex {
            pos: [-1.0, -1.0, 0.0],
            color: [0.0, 0.0, 1.0],
        },
        // Top-right (yello)
        Vertex {
            pos: [1.0, 1.0, 0.0],
            color: [1.0, 1.0, 0.0],
        },
        // Bottom-right (green)
        Vertex {
            pos: [1.0, -1.0, 0.0],
            color: [0.0, 1.0, 0.0],
        },
    ];
    // CCW
    let indices = vec![0, 1, 2, 2, 1, 3];
    let state = State::new(&vertices, &indices).await;

    // Registers event listerns.
    add_event_listener("", "resize", || {
        STATE.with_borrow_mut(|state| {
            state.resize();
        })
    });
    add_mouseevent_listener("#canvas0", "mousemove", |event| {
        STATE.with_borrow_mut(|state| {
            let x = scaled(event.offset_x() as f64) as f32;
            let y = scaled(event.offset_y() as f64) as f32;
            state.mousemove(x, y);
        })
    });
    add_mouseevent_listener("#canvas0", "click", |event| {
        STATE.with_borrow_mut(|state| {
            let x = scaled(event.offset_x() as f64) as f32;
            let y = scaled(event.offset_y() as f64) as f32;
            state.mouseclick(x, y);
        })
    });

    // Runs the animation loop.
    state.request_animation_frame();

    STATE.set(state);
}

struct State {
    canvas: Canvas,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_data: UniformData,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    animate_callback: Closure<dyn FnMut(f32)>,
}

impl State {
    async fn new(vertices: &[Vertex], indices: &[u32]) -> Self {
        // Creates a `wgpu::Instance`.
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        // Finds out the canvas.
        let canvas = Canvas::new("#canvas0");

        // Creates a `wgpu::Surface`.
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.element.clone()))
            .unwrap();

        // Creates a `wpgu::Adpater`.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .unwrap();

        // Creates a `wgpu::Device` and a `wgpu::Queue`.
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        // Configures the surface.
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: canvas.width(),
            height: canvas.height(),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Creates a `wgpu::Buffer` for the vertices.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Creates a `wgpu::Buffer` for the indices.
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        // Creates a `wgpu::Buffer` for the uniform data.
        let uniform_data = UniformData {
            resolution: [canvas.width() as f32, canvas.height() as f32],
            mouse_move: [f32::MIN, f32::MIN],
            mouse_click: [f32::MIN, f32::MIN],
            scale: web_sys::window().unwrap().device_pixel_ratio() as f32,
            time: 0.0,
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: bytemuck::cast_slice(&[uniform_data][..]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform bind group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Creates a `wgpu::ShaderModule`.
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("example.wgsl").into()),
        });

        // Creates a `wgpu::RenderPipeline`.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("v_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
                module: &shader_module,
                entry_point: Some("f_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        // Creates an animation loop.
        let animate_callback = Closure::<dyn FnMut(f32)>::new(|time: f32| {
            STATE.with_borrow_mut(|state| {
                state.render(time);
                state.request_animation_frame();
            })
        });

        Self {
            canvas,
            surface,
            device,
            queue,
            surface_config,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_data,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            animate_callback,
        }
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
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }

    fn request_animation_frame(&self) {
        web_sys::window()
            .unwrap()
            .request_animation_frame(self.animate_callback.as_ref().unchecked_ref())
            .expect_throw("Failed to request an animation frame");
    }

    fn resize(&mut self) {
        let new_width = scaled(self.canvas.client_width() as f64) as u32;
        let new_height = scaled(self.canvas.client_height() as f64) as u32;
        if new_width != self.canvas.width() || new_height != self.canvas.height() {
            self.canvas.set_width(new_width);
            self.canvas.set_height(new_height);
            self.surface_config.width = new_width;
            self.surface_config.height = new_height;
            self.surface.configure(&self.device, &self.surface_config);

            // Update uniform data
            self.uniform_data.resolution = [new_width as f32, new_height as f32];

            log!("Resized: ({new_width}, {new_height})");
        }
    }

    fn mousemove(&mut self, x: f32, y: f32) {
        self.uniform_data.mouse_move = [x, y];
    }

    fn mouseclick(&mut self, x: f32, y: f32) {
        self.uniform_data.mouse_click = [x, y];
    }
}

/// HTML canvas
#[derive(Debug, Clone)]
pub struct Canvas {
    element: web_sys::HtmlCanvasElement,
}

impl Canvas {
    pub fn new(selectors: &str) -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document.query_selector(selectors).unwrap().unwrap();
        let canvas = element.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let width = scaled(canvas.client_width() as f64) as u32;
        let height = scaled(canvas.client_height() as f64) as u32;
        canvas.set_width(width);
        canvas.set_height(height);

        Self { element: canvas }
    }
}

impl ops::Deref for Canvas {
    type Target = web_sys::HtmlCanvasElement;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

/// Simple vertex format.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    // pos
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    // color
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn add_event_listener(selectors: &str, type_: &str, f: impl Fn() + 'static) {
    let listener = Closure::<dyn Fn()>::new(f);
    _add_event_listener(selectors, type_, listener.as_ref().unchecked_ref());
    listener.forget(); // Leak, but it occurs just once
}

fn add_mouseevent_listener(
    selectors: &str,
    type_: &str,
    f: impl Fn(web_sys::MouseEvent) + 'static,
) {
    let listener = Closure::<dyn Fn(_)>::new(f);
    _add_event_listener(selectors, type_, listener.as_ref().unchecked_ref());
    listener.forget(); // Leak, but it occurs just once
}

fn _add_event_listener(selectors: &str, type_: &str, listener: &web_sys::js_sys::Function) {
    if selectors.is_empty() {
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback(type_, listener)
            .expect("Failed to add an event listener");
    } else {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector(selectors)
            .unwrap()
            .unwrap();
        element
            .add_event_listener_with_callback(type_, listener)
            .expect("Failed to add an event listener");
    }
}

fn scaled(value: f64) -> f64 {
    let scale = web_sys::window().unwrap().device_pixel_ratio();
    value * scale
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
struct UniformData {
    mouse_move: [f32; 2],
    mouse_click: [f32; 2],
    resolution: [f32; 2],
    scale: f32,
    time: f32,
}

/// Console log utility macro
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}
