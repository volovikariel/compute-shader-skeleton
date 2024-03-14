@group(0)
@binding(0)
var<storage, read_write> input_output_buffer: array<i32>;


@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) index_3d: vec3<u32>) {
    let index = index_3d.x % 5;
    input_output_buffer[index] += input_output_buffer[index];
}
