struct Agent {
    position: vec2<f32>;
    angle: f32;
};

[[block]]
struct Agents {
    agents: [[stride(16)]] array<Agent>;
};

[[group(0), binding(0)]] var<storage, read> agents: Agents;
[[group(0), binding(1)]] var render_texture: texture_storage_2d<rgba16float, write>;

struct ComputeInput {
    [[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>;
};

[[stage(compute), workgroup_size(16)]]
fn cs_main(input: ComputeInput) {
    let id = input.global_invocation_id;
    let total_agents = arrayLength(&agents.agents);

    if (id.x >= total_agents) {
        return;
    }

    let agent = agents.agents[id.x];

    let x = i32(agent.position.x);
    let y = i32(agent.position.y);
    let coords = vec2<i32>(x, y);

    textureStore(render_texture, coords, vec4<f32>(1.0, 1.0, 1.0, 1.0));
}
