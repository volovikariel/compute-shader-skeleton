/*
    1. Get GPUDevice (WHAT the code runs on)
       and
       GPUQueue (WHAT keeps track of WHAT to run **and** in WHAT order [e.g: please run the compute shader stage])
    2. Create GPUShaderModule (WHAT is ran on the GPU)
    3. Create the GPUComputePipeline
*/

use std::{env, fmt, fs};

use pollster::FutureExt;

#[derive(Debug)]
pub enum RequestDeviceError {
    AdapterCreationFailure,
    DeviceCreationFailure, // Add more error variants as needed
}

impl fmt::Display for RequestDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RequestDeviceError::AdapterCreationFailure => write!(f, "Adapter creation failure"),
            RequestDeviceError::DeviceCreationFailure => {
                write!(f, "Device or queue creation failure")
            }
        }
    }
}

async fn get_webgpu_device_and_queue() -> Result<(wgpu::Device, wgpu::Queue), RequestDeviceError> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = match instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
    {
        Some(adapter) => adapter,
        None => return Err(RequestDeviceError::AdapterCreationFailure),
    };

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = match adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WebGPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
    {
        Ok((device, queue)) => (device, queue),
        Err(_) => return Err(RequestDeviceError::DeviceCreationFailure),
    };

    Ok((device, queue))
}

fn create_shader_module(device: &wgpu::Device, shader_code: &str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader Module"),
        source: wgpu::ShaderSource::Wgsl(shader_code.into()),
    })
}

fn create_shader_pipeline(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(layout),
        module: shader_module,
        entry_point: "main",
    })
}

fn main() {
    let result = get_webgpu_device_and_queue().block_on();

    // Retrieve command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if a shader file path argument is provided
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <shader_file_path>");
        return;
    }

    // Extract the shader file path from the command-line arguments
    let shader_file_path = &args[1];

    // Read the contents of the shader file
    let shader_code = match fs::read_to_string(shader_file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading shader file: {}", err);
            return;
        }
    };

    let (device, queue) = result.unwrap();
    let shader_module = create_shader_module(&device, &shader_code);
}
