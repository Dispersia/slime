[[block]]
struct Globals {
    width: u32;
    height: u32;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;
[[group(0), binding(1)]] var input_texture: texture_storage_2d<rgba16float, read>;
[[group(0), binding(2)]] var output_texture: texture_storage_2d<rgba16float, write>;

struct ComputeInput {
    [[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>;
};

[[stage(compute), workgroup_size(8, 8)]]
fn cs_main(input: ComputeInput) {
    let id = input.global_invocation_id;

    if (id.x < 0u || id.x >= globals.width || id.y < 0u || id.y >= globals.height) {
        return;
    }

    let coords = vec2<i32>(i32(id.x), i32(id.y));

    let texture_state = textureLoad(input_texture, coords);
    textureStore(output_texture, coords, texture_state);
}