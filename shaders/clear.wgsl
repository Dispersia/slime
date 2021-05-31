[[block]]
struct Globals {
    width: u32;
    height: u32;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var sim_texture: [[access(write)]] texture_storage_2d<rgba16float>;

struct ComputeInput {
    [[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>;
};

[[stage(compute), workgroup_size(64)]]
fn cs_main(input: ComputeInput) {
    let id = input.global_invocation_id;

    let x = id.x % globals.width;
    let y = u32(f32(id.x) / f32(globals.width));

    if (x < 0u32 || x > globals.width || y < 0u32 || y >= globals.height) {
        return;
    }

    let current_pos = vec2<i32>(i32(x), i32(y));

    textureStore(sim_texture, current_pos, vec4<f32>(0.0, 0.0, 0.0, 0.0));
}
