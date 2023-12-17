use std::mem;
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

#[allow(unused_macros)]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}

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

#[wasm_bindgen]
struct App {
    states: Vec<State>,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            states: vec![],
        }
    }

    #[wasm_bindgen]
    pub async fn init(&mut self) {
        // First canvas has single square.
        let vertices0 = vec![
            // Top-left (magenta)
            Vertex { pos: [-0.1, 1.0, 0.0], color: [1.0, 0.0, 1.0], },
            // Bottom-left (blue)
            Vertex { pos: [-0.1, -1.0, 0.0], color: [0.0, 0.0, 1.0], },
            // Top-right (yello)
            Vertex { pos: [0.1, 1.0, 0.0], color: [1.0, 1.0, 0.0], },
            // Bottom-right (green)
            Vertex { pos: [0.1, -1.0, 0.0], color: [0.0, 1.0, 0.0], },
        ];
        let indices0 = vec![0, 1, 2, 2, 1, 3];

        // Second canvas had two squares.
        let mut vertices1 = vertices0.clone();
        vertices1.extend(vertices0.clone());
        for i in 0..4 {
            vertices1[i].pos[0] -= 0.2;
        }
        for i in 4..8 {
            vertices1[i].pos[0] += 0.2;
        }
        let indices1 = vec![0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7];

        // Third canvas has three squares.
        let mut vertices2 = vertices0.clone();
        vertices2.extend(vertices0.clone());
        vertices2.extend(vertices0.clone());
        for i in 0..4 {
            vertices2[i].pos[0] -= 0.4;
        }
        for i in 8..12 {
            vertices2[i].pos[0] += 0.4;
        }
        let indices2 = vec![0, 1, 2, 2, 1, 3, 4, 5, 6, 6, 5, 7, 8, 9, 10, 10, 9, 11];

        // Creates render context of each canvas.
        self.states = vec![
            State::new("canvas0", 1, &vertices0, &indices0).await,
            State::new("canvas1", 2, &vertices1, &indices1).await,
            State::new("canvas2", 3, &vertices2, &indices2).await,
        ];
    }
}

#[allow(dead_code)]
struct State {
    canvas: Canvas,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_num: u32,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    async fn new(canvas_id: &str, canvas_handle: u32, vertices: &[Vertex], indices: &[u32]) -> Self {
        // Creates `Canvas` from web_sys::HtmlCanvasElement.
        let window = web_sys::window().unwrap();
        let element = window.document().unwrap().get_element_by_id(canvas_id).unwrap();
        element.set_attribute("data-raw-handle", canvas_handle.to_string().as_str()).unwrap();
        let canvas = Canvas {
            element: element.dyn_into::<web_sys::HtmlCanvasElement>().ok().unwrap(),
            handle: canvas_handle,
        };

        // Creates `wgpu::Instance` for each canvas.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // Creates `wgpu::Surace`.
        // Until wgpu 0.18, we should create at least one `wgpu::Surface` before making `wgpu::Adapter`.
        // wgpu implementation keeps WebGL context when we're creating `wgpu::Surface`.
        // and it uses the context information when it looks for adequate device.
        // This is why we're creating all of wgpu resources for each canvas.
        // But this approach may be inefficient.
        // See https://threejs.org/manual/#en/multiple-scenes and improve later.
        let surface = unsafe { instance.create_surface(&canvas).ok().unwrap() };

        // Creates `wgpu::Adapter`.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .unwrap();

        // Creates `wgpu::Device and wgpu::Queue`.
        let (device, queue) = adapter
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
            .unwrap();

        // Configures the surface.
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: canvas.element.width(),
            height: canvas.element.height(),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Creates `wgpu::Buffer` filled with the given vertices.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Creates `wgpu::Buffer` filled with the given indices.
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let index_num = indices.len() as u32;

        // Creates `wgpu::ShaderModule`.
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("example.wgsl").into()),
        });

        // Creates `wgpu::RenderPipeline`.
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
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
                module: &shader_module,
                entry_point: "f_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        // Render only once for simplicity.
        let surface_texture = surface.get_current_texture().unwrap();
        let texture_view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = device
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..index_num, 0, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Self {
            canvas,
            surface,
            device,
            queue,
            surface_config,
            vertex_buffer,
            index_buffer,
            index_num,
            render_pipeline,
        }
    }
}

struct Canvas {
    element: web_sys::HtmlCanvasElement,
    handle: u32,
}

unsafe impl raw_window_handle::HasRawWindowHandle for Canvas {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        use raw_window_handle::{RawWindowHandle, WebWindowHandle};
        let mut handle = WebWindowHandle::empty();
        handle.id = self.handle;
        RawWindowHandle::Web(handle)
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for Canvas {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        use raw_window_handle::{RawDisplayHandle, WebDisplayHandle};
        let handle = WebDisplayHandle::empty();
        RawDisplayHandle::Web(handle)
    }
}





//     fn create_render_pipeline(
//         device: &wgpu::Device,
//         bind_group_layouts: &[&wgpu::BindGroupLayout],
//         shader_module: &wgpu::ShaderModule,
//         surface_config: &wgpu::SurfaceConfiguration,
//     ) -> wgpu::RenderPipeline {
//     }

//     fn create_animation_loop() -> Closure<dyn FnMut(f32)> {
//         Closure::<dyn FnMut(f32)>::new(|time: f32| unsafe {
//             let state = STATE.as_mut().unwrap_unchecked();
//             state.render(time);
//             // state.request_animation_frame();
//         })
//     }

//     #[inline(always)]
//     fn request_animation_frame(&self) {
//         self.window
//             .request_animation_frame(self.animation_cb.as_ref().unchecked_ref())
//             .expect_throw("Failed to request an animation frame");
//     }

//     fn resize(&mut self) {
//         let new_width = self.canvas.client_width() as u32;
//         let new_height = self.canvas.client_height() as u32;
//         if new_width != self.canvas.width() || new_height != self.canvas.height() {
//             // Synchronize manually
//             self.canvas.set_width(new_width);
//             self.canvas.set_height(new_height);
//             self.winit_window
//                 .set_inner_size(PhysicalSize::new(new_width, new_height));
//             self.surface_config.width = new_width;
//             self.surface_config.height = new_height;
//             self.surface.configure(&self.device, &self.surface_config);

//             // Update uniform data
//             self.uniform_data.resolution = [new_width as f32, new_height as f32];

//             log!("Resized: ({}, {})", new_width, new_height);
//         }
//     }

//     fn mousemove(&mut self, event: web_sys::MouseEvent) {
//         // Update uniform data
//         self.uniform_data.mouse_move = [event.offset_x() as f32, event.offset_y() as f32];
//     }

//     fn click(&mut self, event: web_sys::MouseEvent) {
//         // Update uniform data
//         self.uniform_data.mouse_click = [event.offset_x() as f32, event.offset_y() as f32];
//     }

//     fn render(&mut self, time: f32) {
//         // Write uniform data to its buffer
//         self.uniform_data.time = time * 0.001;
//         self.queue.write_buffer(
//             &self.uniform_buffer,
//             0,
//             bytemuck::cast_slice(&[self.uniform_data][..]),
//         );

//         let surface_texture = self.surface.get_current_texture().unwrap();
//         log!("[lib] render(), surface_texture: {:#?}", surface_texture);
//         let texture_view = surface_texture.texture.create_view(&Default::default());
//         log!("[lib] render(), texture_view: {:#?}", texture_view);
//         let mut encoder = self
//             .device
//             .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                 label: Some("Render command encoder"),
//             });

//         {
//             let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                 label: Some("Render pass"),
//                 color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                     view: &texture_view,
//                     resolve_target: None,
//                     ops: wgpu::Operations {
//                         load: wgpu::LoadOp::Clear(wgpu::Color {
//                             r: 0.0,
//                             g: 0.0,
//                             b: 0.0,
//                             a: 1.0,
//                         }),
//                         store: wgpu::StoreOp::Store,
//                     },
//                 })],
//                 depth_stencil_attachment: None,
//                 timestamp_writes: None,
//                 occlusion_query_set: None,
//             });
//             render_pass.set_pipeline(&self.render_pipeline);
//             render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
//             render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
//             render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
//             render_pass.draw_indexed(0..self.index_num, 0, 0..1);
//         }
//         self.queue.submit(std::iter::once(encoder.finish()));
//         surface_texture.present();
//     }
// }

// #[wasm_bindgen(start)]
// pub fn main() {
//     std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//     console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
// }

// #[wasm_bindgen]
// pub async fn run() {
//     unsafe {
//         STATE = Some(State::new().await);
//         let state = STATE.as_mut().unwrap_unchecked();

//         // Sets resize event listener on window.
//         add_event_listener("", "resize", || STATE.as_mut().unwrap_unchecked().resize());

//         // Sets mousemove/click event listener on canvas
//         add_event_listener_with_mouseevent("canvas0", "mousemove", |event: web_sys::MouseEvent| {
//             STATE.as_mut().unwrap_unchecked().mousemove(event)
//         });
//         add_event_listener_with_mouseevent("canvas0", "click", |event: web_sys::MouseEvent| {
//             STATE.as_mut().unwrap_unchecked().click(event)
//         });
//         state.request_animation_frame();
//     };
// }

// fn get_window() -> web_sys::Window {
//     web_sys::window().expect_throw("Failed to get window")
// }

// fn get_document() -> web_sys::Document {
//     let window = get_window();
//     window.document().expect_throw("Failed to get document")
// }

// fn get_element_by_id(id: &str) -> web_sys::Element {
//     let document = get_document();
//     document
//         .get_element_by_id(id)
//         .expect_throw("Failed to get element")
// }

// fn add_event_listener(id: &str, type_: &str, f: impl Fn() + 'static) {
//     let listener = Closure::<dyn Fn()>::new(f);
//     if id.is_empty() {
//         get_window()
//             .add_event_listener_with_callback(type_, listener.as_ref().unchecked_ref())
//             .expect("Failed to add an event listener");
//     } else {
//         get_element_by_id(id)
//             .add_event_listener_with_callback(type_, listener.as_ref().unchecked_ref())
//             .expect("Failed to add an event listener");
//     }
//     listener.forget(); // Leak, but it occurs just once
// }

// fn add_event_listener_with_mouseevent(
//     id: &str,
//     type_: &str,
//     f: impl Fn(web_sys::MouseEvent) + 'static,
// ) {
//     let listener = Closure::<dyn Fn(_)>::new(f);
//     if id.is_empty() {
//         get_window()
//             .add_event_listener_with_callback(type_, listener.as_ref().unchecked_ref())
//             .expect("Failed to add an event listener");
//     } else {
//         get_element_by_id(id)
//             .add_event_listener_with_callback(type_, listener.as_ref().unchecked_ref())
//             .expect("Failed to add an event listener");
//     }
//     listener.forget(); // Leak, but it occurs just once
// }
