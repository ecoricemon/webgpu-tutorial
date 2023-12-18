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
    state: Option<State>,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { state: None }
    }

    #[wasm_bindgen]
    pub async fn init(&mut self) {
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
        let indices = vec![0, 1, 2, 2, 1, 3];

        // Creates our render state.
        self.state = Some(
            State::new(&vertices, &indices).await,
        );
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
    async fn new(vertices: &[Vertex], indices: &[u32]) -> Self {
        // Creates canvases.
        let canvas_a = Canvas::new("canvas_a", 1);
        let canvas_b = Canvas::new("canvas_b", 2);
        let canvas_c = Canvas::new("canvas_c", 3);

        // Creates `wgpu::Instance`.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        // Creates surface_a & surface_b.
        // Until wgpu 0.18, we should create at least one `wgpu::Surface` before making `wgpu::Adapter`.
        // wgpu implementation keeps WebGL context when we're creating `wgpu::Surface`.
        // and it uses the context information when it looks for adequate device.
        let surface_a = unsafe { instance.create_surface(&canvas_a).ok().unwrap() };

        // Make sure this shouldn't be dropped until we make an adapter.
        let surface_b = unsafe { instance.create_surface(&canvas_b).ok().unwrap() };

        // Creates `wgpu::Adapter`.
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: None,
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .unwrap();

        // Explicit drop.
        drop(surface_a);
        drop(surface_b);
        drop(canvas_a);
        drop(canvas_b);

        // Creates surface_c, which won't work as we expected.
        let surface_c = unsafe { instance.create_surface(&canvas_c).ok().unwrap() };
        let canvas = canvas_c;
        let surface = surface_c;

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
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

impl Canvas {
    fn new(id: &str, handle: u32) -> Self {
        let window = web_sys::window().unwrap();
        let element = window
            .document()
            .unwrap()
            .get_element_by_id(id)
            .unwrap();
        element
            .set_attribute("data-raw-handle", handle.to_string().as_str())
            .unwrap();

        Self {
            element: element
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .ok()
                .unwrap(),
            handle,
        }
    }
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
