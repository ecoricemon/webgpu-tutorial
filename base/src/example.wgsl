struct UniformData {
    res: vec2<f32>,
    mouse: vec2<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> uni: UniformData;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(1) color: vec3<f32>
}

@vertex
fn v_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(in.pos, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn f_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = select(0.0, 0.3, distance(in.pos.xy, uni.mouse) < 25.0);
    return vec4f(in.color - x, 1.0);
}
