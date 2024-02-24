use wgpu::util::DeviceExt;

async fn run() {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter = instance.request_adapter(&Default::default()).await.unwrap();
    let (device, queue) = adapter
        .request_device(&Default::default(), None)
        .await
        .unwrap();

    let compute_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let input = &[1.0f32, 2.0f32];
    let input_bytes: &[u8] = bytemuck::bytes_of(input);
    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: input_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: input_bytes.len() as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // This works if the buffer is initialized, otherwise reads all 0, for some reason.
    let query_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &[0; 16],
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader_module,
        entry_point: "main",
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: input_buffer.as_entire_binding(),
        }],
    });

    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(input.len() as u32, 1, 1);
    }
    encoder.copy_buffer_to_buffer(
        &input_buffer,
        0,
        &output_buffer,
        0,
        input_bytes.len() as u64,
    );
    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    let query_slice = query_buffer.slice(..);
    // Assume that both buffers become available at the same time. A more careful
    // approach would be to wait for both notifications to be sent.
    query_slice.map_async(wgpu::MapMode::Read, |_| ());
    device.poll(wgpu::Maintain::Wait);
    if let Some(Ok(())) = receiver.receive().await {
        let data_raw = &*buffer_slice.get_mapped_range();
        let data: &[f32] = bytemuck::cast_slice(data_raw);
        println!("data: {:?}", data);
    }
}

fn main() {
    pollster::block_on(run());
}
