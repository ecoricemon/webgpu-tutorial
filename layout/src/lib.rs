//! This is a test file to see what WGSL error messages are printed out on our browsers about an
//! invalid layout and how we can solve the problem.
//! This source code does not contain all the test cases about layout exhaustively. If you want to
//! test something, then please write code like [`uniform_offset::fail`] or [`uniform_offset::ok`].

use core::{cell::RefCell, fmt::Debug, slice};
use my_wgsl::WgslCompatible;
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

#[wasm_bindgen]
pub async fn test() {
    // When panics, we can see error messages on the console.
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Creates a `wgpu::Instance`, `wgpu::Adapter`, `wgpu::Device`, and `wgpu::Queue`.
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::BROWSER_WEBGPU,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();

    // Put the test cases here.
    // uniform_offset::fail(&device, &queue).await;
    uniform_offset::ok(&device, &queue).await;
}

/// # What layout rule is this about?
///
/// 'uniform' storage requires that the number of bytes between the start of the previous member of
/// type struct and the current member be a multiple of 16 bytes.
#[rustfmt::skip]
#[allow(dead_code)]
mod uniform_offset {
    use super::*;

    pub(super) async fn fail(device: &wgpu::Device, queue: &wgpu::Queue) {
        #[derive(Debug, PartialEq)]
        #[repr(C)]
        struct A { a0: i32 }

        #[derive(Debug, PartialEq)]
        #[repr(C)]
        struct B { b0: A, b1: i32 }

        let define = r"
            struct A { a0: i32 }
            struct B {
                b0: A,
                b1: i32 // Error occurs here. The previous member is of type struct, so we need at
                        // least 16 bytes between the starts of `b0` and `b1`.
            }
        ";

        let data = B { 
            b0: A { a0: 1 }, 
            b1: 2, 
        };
        let result = test_uniform_address_space(device, queue, data, Some(define), "B").await;
        if result.is_success {
            log!("TEST SUCCESS!");
        } else {
            log!("TEST FAILED: {}", result.message);
        }
    }

    // `WgslCompatible` let us know where we should fix to make our structs to be compatible with
    // WGSL.
    pub(super) async fn ok(device: &wgpu::Device, queue: &wgpu::Queue) {
        #[derive(WgslCompatible, Debug, PartialEq)]
        #[repr(C)]
        struct A { a0: i32, pad: [u8; 12] }

        #[derive(WgslCompatible, Debug, PartialEq)]
        #[wgsl(uniform)]
        #[repr(C)]
        struct B { b0: A, b1: i32 }

        let define = format!("{}{}", A::WGSL_DEFINE, B::WGSL_DEFINE);
        assert_eq(
            &define,
            "struct A { @size(16) a0: i32 }
            struct B { b0: A, b1: i32 }"
        );

        let data = B {
            b0: A { a0: 1, pad: [0; _] },
            b1: 2,
        };
        let result = test_uniform_address_space(device, queue, data, Some(&define), "B").await;
        if result.is_success {
            log!("TEST SUCCESS!");
        } else {
            log!("TEST FAILED: {}", result.message);
        }
    }
}

/// Tests if the gpu can read the given data correctly via storage buffer.
///
/// * deviec - A reference to a [`wgpu::Device`].
/// * queue - A reference to a [`wgpu::Queue`].
/// * data - The data that will be sent to GPU.
/// * wgsl_define_type - Type definition in WGSL source code. e.g. "struct A { .. }"
/// * wgsl_type - WGSL type of the given `data`.
#[allow(dead_code)]
async fn test_storage_address_space<T: Debug + PartialEq + 'static>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: T,
    wgsl_define_type: Option<&str>,
    wgsl_type: &str,
) -> TestResult {
    let ptr = &data as *const T as *const u8;
    let contents: &[u8] = unsafe { slice::from_raw_parts(ptr, size_of::<T>()) };

    // Creates a `wgpu::Buffer` for writing some data to the gpu.
    let init_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Init buffer"),
        contents,
        usage: wgpu::BufferUsages::STORAGE,
    });

    let wgsl_define_type = wgsl_define_type.unwrap_or_default();
    let shader_code = format!(
        "
        {wgsl_define_type}
        
        @group(0) @binding(0) var<storage, read> init_buffer: {wgsl_type};
        @group(0) @binding(1) var<storage, read_write> echo_buffer: {wgsl_type};

        @compute @workgroup_size(1)
        fn c_main() {{
            echo_buffer = init_buffer;
        }}
    "
    );

    run_on_gpu(device, queue, init_buffer, &shader_code, data).await
}

/// Tests if the gpu can read the given data correctly via uniform buffer.
///
/// * deviec - A reference to a [`wgpu::Device`].
/// * queue - A reference to a [`wgpu::Queue`].
/// * data - The data that will be sent to GPU.
/// * wgsl_define_type - Type definition in WGSL source code. e.g. "struct A { .. }"
/// * wgsl_type - WGSL type of the given `data`.
async fn test_uniform_address_space<T: Debug + PartialEq + 'static>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: T,
    wgsl_define_type: Option<&str>,
    wgsl_type: &str,
) -> TestResult {
    let ptr = &data as *const T as *const u8;
    let contents: &[u8] = unsafe { slice::from_raw_parts(ptr, size_of::<T>()) };

    // Creates a `wgpu::Buffer` for writing some data to the gpu.
    let init_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Init buffer"),
        contents,
        usage: wgpu::BufferUsages::UNIFORM,
    });

    let wgsl_define_type = wgsl_define_type.unwrap_or_default();
    let shader_code = format!(
        "
        {wgsl_define_type}
        
        @group(0) @binding(0) var<uniform> init_buffer: {wgsl_type};
        @group(0) @binding(1) var<storage, read_write> echo_buffer: {wgsl_type};

        @compute @workgroup_size(1)
        fn c_main() {{
            echo_buffer = init_buffer;
        }}
    "
    );

    run_on_gpu(device, queue, init_buffer, &shader_code, data).await
}

async fn run_on_gpu<T: Debug + PartialEq + 'static>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    init_buffer: wgpu::Buffer,
    shader_code: &str,
    data: T,
) -> TestResult {
    thread_local! {
        static READ_BUFFER: RefCell<wgpu::Buffer> = panic!();
    }

    // Creates a `wgpu::Buffer` for copying the data on the gpu.
    let echo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Echo buffer"),
        size: init_buffer.size(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Creates a `wgpu::Buffer` for reading the copied data from the gpu.
    let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Read buffer"),
        size: init_buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    READ_BUFFER.set(read_buffer.clone());

    // Creates a `wgpu::ShaderModule`.
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    });

    // Creates a `wgpu::ComputePipeline`.
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: None,
        module: &shader_module,
        entry_point: Some("c_main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind group"),
        layout: &compute_pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: init_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: echo_buffer.as_entire_binding(),
            },
        ],
    });

    // Creates a `wgpu::CommandEncoder`.
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute command encoder"),
    });

    // Writes a compute pass.
    let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("Compute pass"),
        timestamp_writes: None,
    });
    compute_pass.set_pipeline(&compute_pipeline);
    compute_pass.set_bind_group(0, Some(&bind_group), &[]);
    compute_pass.dispatch_workgroups(1, 1, 1);
    drop(compute_pass);

    // Copies the computation result.
    encoder.copy_buffer_to_buffer(&echo_buffer, 0, &read_buffer, 0, None);

    // Submits the command to the queue.
    queue.submit(std::iter::once(encoder.finish()));

    // Tests if the read buffer contains the data we put in.
    let (tx, rx) = futures::channel::oneshot::channel();
    read_buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        assert!(result.is_ok());

        READ_BUFFER.with_borrow(|buf| {
            let bytes = &buf.get_mapped_range(..)[..];
            let ptr = bytes.as_ptr();
            let read = unsafe { (ptr as *const T).as_ref().unwrap() };

            let is_success = read == &data;
            let message = if is_success {
                format!("test success: {read:?}")
            } else {
                format!("test failed: src: {data:?}, read: {read:?}")
            };
            tx.send(TestResult {
                is_success,
                message,
            })
            .unwrap();
        });
    });
    rx.await.unwrap()
}

#[derive(Debug)]
struct TestResult {
    is_success: bool,
    message: String,
}

fn assert_eq(a: &str, b: &str) {
    let a: String = a.chars().filter(|c| !c.is_whitespace()).collect();
    let b: String = b.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(a, b);
}

/// Console log utility macro
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into());
    }
}
