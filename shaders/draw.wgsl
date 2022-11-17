struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    out.position = vec4<f32>(input.position.x, input.position.y, 0.0, 1.0);
    out.tex_coord = input.tex_coord;

    return out;
}

@group(0)
@binding(0)
var sim_texture: texture_2d<f32>;

@group(0)
@binding(1)
var sim_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(sim_texture, sim_sampler, input.tex_coord);

    if (sample.x > 0.0 || sample.y > 0.0 || sample.z > 0.0) {
        return vec4<f32>(sample.w, sample.w, sample.w, sample.w);
    }

    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
