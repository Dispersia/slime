[[block]]
struct Globals {
    width: u32;
    height: u32;
    diffuse_rate: f32;
    decay_rate: f32;
};

[[block]]
struct Time {
    time: u32;
    delta_time: f32;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var<uniform> time: Time;
[[group(0), binding(2)]] var trail_map: [[access(read_write)]] texture_storage_2d<rgba16float>;
[[group(0), binding(3)]] var diffuse_trail_map: [[access(write)]] texture_storage_2d<rgba16float>;

struct ComputeInput {
    [[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>;
};

[[stage(compute), workgroup_size(8, 8)]]
fn cs_main(input: ComputeInput) {
    let id = input.global_invocation_id;

    if (id.x < 0u32 || id.x >= globals.width || id.y < 0u32 || id.y >= globals.height) {
        return;
    }

    let coords = vec2<i32>(i32(id.x), i32(id.y));

    var sum: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let original_col = textureLoad(trail_map, coords);
    for (var offset_x: i32 = -1; offset_x <= 1; offset_x = offset_x + 1) {
        for (var offset_y: i32 = -1; offset_y <= 1; offset_y = offset_y + 1) {
            let sample_x = min(globals.width - 1u32, max(0u32, id.x + u32(offset_x)));
            let sample_y = min(globals.height - 1u32, max(0u32, id.y + u32(offset_y)));

            let offset_coords = vec2<i32>(i32(sample_x), i32(sample_y));
            let texture_state = textureLoad(trail_map, offset_coords);
            sum = sum + texture_state;
        }
    }

    let blurred_col = sum / 9.0;
    let diffuse_weight = clamp(globals.diffuse_rate * time.delta_time, 0.0, 1.0);
    let blurred_col = original_col * (1.0 - diffuse_weight) + blurred_col * diffuse_weight;

    let output = max(vec4<f32>(0.0, 0.0, 0.0, 0.0), blurred_col - globals.decay_rate * time.delta_time);
    textureStore(diffuse_trail_map, coords, output);
}
