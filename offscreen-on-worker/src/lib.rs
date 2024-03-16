use std::{cell::RefCell, mem, rc::Rc};
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
mod worker;
use worker::*;
mod canvas;
use canvas::*;
mod message;
use message::*;

/// `App` is responsible for accessing window elements.
/// Also, it creates main worker and passes window events to the worker.
/// Main worker, on the other hand, does all works such as drawing.
#[allow(dead_code)]
#[wasm_bindgen]
pub struct App {
    canvas: Canvas,
    worker: Rc<RefCell<MainWorker>>,
}

#[wasm_bindgen]
impl App {
    /// `index.js` creates `App` using this constructor.
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Gets the canvas.
        let canvas = Canvas::new("#canvas0", 1);
        // Spawns main worker.
        let worker = MainWorker::spawn("main-worker", 1).unwrap();

        // Transfers offscreen canvas to the worker.
        let offscreen = OffscreenCanvas::from(&canvas);
        let (offscreen, handle) = offscreen.each();
        let msg = js_sys::Array::new_with_length(3);
        msg.set(0, JsMessage::INIT.into_jsvalue());
        msg.set(1, JsValue::from(offscreen.clone()));
        msg.set(2, JsValue::from(handle));
        let t = js_sys::Array::new_with_length(1);
        t.set(0, JsValue::from(offscreen));
        worker.post_message_with_transfer(&msg, &t).unwrap();
        let worker = Rc::new(RefCell::new(worker));

        // Registers "resize" event proxy.
        let worker_cloned = Rc::clone(&worker);
        let msg = js_sys::Array::new_with_length(JsResizeMessage::field_num() + 1);
        msg.set(0, JsMessage::WINDOW_RESIZE.into_jsvalue());
        let canvas_cloned = canvas.clone();
        let listener = Closure::<dyn Fn()>::new(move || {
            let window = web_sys::window().unwrap();
            let scale_factor = window.device_pixel_ratio();
            JsResizeMessage::set_js_array(&msg, &canvas_cloned, scale_factor, 1);
            worker_cloned.borrow_mut().post_message(&msg).unwrap();
        });
        let window = web_sys::window().unwrap();
        window
            .add_event_listener_with_callback("resize", listener.as_ref().unchecked_ref())
            .unwrap();
        listener.forget(); // Leak, but just once.

        // Registers "mousemove" event proxy.
        let worker_cloned = Rc::clone(&worker);
        let msg = js_sys::Array::new_with_length(JsMouseMessage::field_num() + 1);
        let listener = Closure::<dyn Fn(_)>::new(move |event: web_sys::MouseEvent| {
            let window = web_sys::window().unwrap();
            let scale_factor = window.device_pixel_ratio();
            msg.set(0, JsMessage::MOUSE_MOVE.into_jsvalue());
            JsMouseMessage::set_js_array(&msg, event, scale_factor, 1);
            worker_cloned.borrow_mut().post_message(&msg).unwrap();
        });
        canvas
            .add_event_listener_with_callback("mousemove", listener.as_ref().unchecked_ref())
            .unwrap();
        listener.forget(); // Leak, but just once.

        // Registers "click" event proxy.
        let worker_cloned = Rc::clone(&worker);
        let msg = js_sys::Array::new_with_length(JsMouseMessage::field_num() + 1);
        let listener = Closure::<dyn Fn(_)>::new(move |event: web_sys::MouseEvent| {
            let window = web_sys::window().unwrap();
            let scale_factor = window.device_pixel_ratio();
            msg.set(0, JsMessage::MOUSE_CLICK.into_jsvalue());
            JsMouseMessage::set_js_array(&msg, event, scale_factor, 1);
            worker_cloned.borrow_mut().post_message(&msg).unwrap();
        });
        canvas
            .add_event_listener_with_callback("click", listener.as_ref().unchecked_ref())
            .unwrap();
        listener.forget(); // Leak, but just once.

        Self { canvas, worker }
    }
}

thread_local! {
    /// Main render state.
    static STATE: RefCell<State> = panic!();
}

/// Initializes [`STATE`] from worker side, not in window context.
/// That's because window and worker don't share memory (We can use shared memory with some restrictions).
/// When initialization is over, JS replaces this event handler with [`main_onmessage`].
#[wasm_bindgen]
pub async fn main_onmessage_init(event: web_sys::MessageEvent) -> bool {
    let data = event.data();
    debug_assert!(data.is_array());
    let data: js_sys::Array = data.unchecked_into();
    match JsMessage::from_f64(data.get(0)).0 {
        JsMessage::INIT_INNER => {
            // Initializes State.
            let canvas: web_sys::OffscreenCanvas = data.get(1).unchecked_into();
            let handle = data.get(2).as_f64().unwrap() as u32;
            let canvas = OffscreenCanvas::new(canvas, handle);
            let state = State::new(canvas).await;
            STATE.set(state);

            // Registers animation callback to the State and activate it.
            let animation_cb = Closure::<dyn FnMut(f32)>::new(move |time: f32| {
                STATE.with_borrow_mut(|state| {
                    state.render(time);
                    state.request_animation_frame();
                })
            });
            STATE.with_borrow_mut(move |state| {
                state.animation_cb = animation_cb;
                state.request_animation_frame();
            });

            // Ready.
            true
        }
        _ => false, // Not ready yet.
    }
}

/// Main worker's message handler for various events.
#[wasm_bindgen]
pub fn main_onmessage(event: web_sys::MessageEvent) {
    let data = event.data();
    debug_assert!(data.is_array());
    let data: js_sys::Array = data.unchecked_into();
    match JsMessage::from_f64(data.get(0)).0 {
        JsMessage::WINDOW_RESIZE_INNER => {
            STATE.with_borrow_mut(|state| {
                state.resize(JsResizeMessage::from_js_array(data, 1));
            });
        }
        JsMessage::MOUSE_MOVE_INNER => {
            STATE.with_borrow_mut(|state| {
                state.mouse_move(JsMouseMessage::from_js_array(data, 1));
            });
        }
        JsMessage::MOUSE_CLICK_INNER => {
            STATE.with_borrow_mut(|state| {
                state.mouse_click(JsMouseMessage::from_js_array(data, 1));
            });
        }
        other => {
            crate::log!("unsupported message: {:?}", other);
        }
    }
}

/// Drawing relative data.
/// Note that this belongs to main worker.
struct State {
    canvas: OffscreenCanvas,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_num: u32,
    uniform_data: UniformData,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    animation_cb: Closure<dyn FnMut(f32)>,
}

impl State {
    pub async fn new(canvas: OffscreenCanvas) -> Self {
        // wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        // wgpu surface
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::OffscreenCanvas(
                web_sys::OffscreenCanvas::clone(&canvas),
            ))
            .unwrap();
        // wgpu adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .unwrap();
        // wgpu device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        // wgpu surface configuration
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
        log!(
            "suface size: {} x {}",
            surface_config.width,
            surface_config.height
        );
        surface.configure(&device, &surface_config);
        // wgpu vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // wgpu index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        // wgpu uniform buffer
        let uniform_data = UniformData {
            resolution: [canvas.width() as f32, canvas.height() as f32],
            mouse_move: [std::f32::MIN, std::f32::MIN],
            mouse_click: [std::f32::MIN, std::f32::MIN],
            ..Default::default()
        };
        let (uniform_buffer, uniform_layout, uniform_bind_group) =
            State::create_uniform_buffer(&device, bytemuck::cast_slice(&[uniform_data][..]));
        // wgpu shader module
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("example.wgsl").into()),
        });
        // wgpu render pipeline
        let render_pipeline = State::create_render_pipeline(
            &device,
            &[&uniform_layout],
            &shader_module,
            &surface_config,
        );
        // dummy animation callback.
        let animation_cb = Closure::<dyn FnMut(f32)>::new(|_| {});

        Self {
            canvas,
            surface,
            device,
            queue,
            surface_config,
            vertex_buffer,
            index_buffer,
            index_num: INDICES.len() as u32,
            uniform_data,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            animation_cb,
        }
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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
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

    pub fn request_animation_frame(&self) {
        let global = js_sys::global().unchecked_into::<web_sys::DedicatedWorkerGlobalScope>();
        global
            .request_animation_frame(self.animation_cb.as_ref().unchecked_ref())
            .unwrap();
    }

    fn resize(&mut self, msg: JsResizeMessage) {
        let new_width = (msg.width * msg.scale_factor) as u32;
        let new_height = (msg.height * msg.scale_factor) as u32;
        if new_width != self.canvas.width() || new_height != self.canvas.height() {
            self.surface_config.width = new_width;
            self.surface_config.height = new_height;
            self.surface.configure(&self.device, &self.surface_config);

            // Update uniform data
            self.uniform_data.resolution = [new_width as f32, new_height as f32];

            log!(
                "Resized: ({}, {}), scale: {}",
                new_width,
                new_height,
                msg.scale_factor
            );
        }
    }

    pub fn mouse_move(&mut self, msg: JsMouseMessage) {
        // Update uniform data
        let x = (msg.offset_x * msg.scale_factor) as f32;
        let y = (msg.offset_y * msg.scale_factor) as f32;
        self.uniform_data.mouse_move = [x, y];
    }

    pub fn mouse_click(&mut self, msg: JsMouseMessage) {
        // Update uniform data
        let x = (msg.offset_x * msg.scale_factor) as f32;
        let y = (msg.offset_y * msg.scale_factor) as f32;
        self.uniform_data.mouse_click = [x, y];
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

/// Vertex example.
const VERTICES: &[Vertex] = &[
    Vertex {
        pos: [-1.0, 1.0, 0.0],  // Top-left
        color: [1.0, 0.0, 1.0], // Magenta
    },
    Vertex {
        pos: [-1.0, -1.0, 0.0], // Bottom-left
        color: [0.0, 0.0, 1.0], // Blue
    },
    Vertex {
        pos: [1.0, 1.0, 0.0],   // Top-right
        color: [1.0, 1.0, 0.0], // Yellow
    },
    Vertex {
        pos: [1.0, -1.0, 0.0],  // Bottom-right
        color: [0.0, 1.0, 0.0], // Green
    },
];

const INDICES: &[u32] = &[0, 1, 2, 2, 1, 3]; // CCW, quad

/// Simple uniform data.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
struct UniformData {
    mouse_move: [f32; 2],
    mouse_click: [f32; 2],
    resolution: [f32; 2],
    time: f32,
    _padding: f32,
}

/// Boilerplate initialization for wasm debugging.
#[wasm_bindgen]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).unwrap();
}

/// Utility
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        #[cfg(debug_assertions)]
        {
            $crate::console_log(format!($($t)*));
        }
    }
}

/// Utility
pub fn console_log(s: String) {
    web_sys::console::log_1(&s.into());
}
