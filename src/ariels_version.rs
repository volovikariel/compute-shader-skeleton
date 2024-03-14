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
        match self {
            RequestDeviceError::AdapterCreationFailure => write!(f, "Adapter creation failure"),
            RequestDeviceError::DeviceCreationFailure => {
                write!(f, "Device or queue creation failure")
            }
        }
    }
}

fn get_shader_code() -> String {
    let shader_code = match env::args().nth(1) {
        Some(shader_file_path) => {
            // Read the contents of the shader file
            match fs::read_to_string(&shader_file_path) {
                Ok(shader_file_content) => shader_file_content,
                Err(err) => {
                    eprintln!("Error reading shader file: {}", err);
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("Usage: cargo run -- <shader_file_path>");
            std::process::exit(1);
        }
    };
    shader_code
}

async fn get_webgpu_device_and_queue() -> Result<(wgpu::Device, wgpu::Queue), RequestDeviceError> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or(RequestDeviceError::AdapterCreationFailure)?;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("WebGPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .map_err(|_| RequestDeviceError::DeviceCreationFailure)?;

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
    let shader_code = get_shader_code();
    let (device, queue) = get_webgpu_device_and_queue().block_on().unwrap();
    let shader_module = create_shader_module(&device, &shader_code);
}
