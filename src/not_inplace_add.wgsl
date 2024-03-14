
@group(0)
@binding(0)
var<uniform> input_buffer: array<i32>;


@group(0)
@binding(1)
var<storage, read_write> output_buffer: array<i32>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) index_3d: vec3<u32>) {
    let index = index_3d.x % input_buffer.len();
    output_buffer[index] += input_buffer[index];
}
