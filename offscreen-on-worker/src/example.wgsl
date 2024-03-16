struct UniformData {
    mouse_move: vec2<f32>,
    mouse_click: vec2<f32>,
    resolution: vec2<f32>,
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
    let fluc = sin(modf(uni.time).fract * 3.141592) * 0.3 + 0.7;
    out.color = in.color * fluc;
    return out;
}

@fragment
fn f_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = select(0.0, 0.3, distance(in.pos.xy, uni.mouse_move) < 25.0);
    let y = select(0.0, 0.3, distance(in.pos.xy, uni.mouse_click) < 25.0);
    return vec4f(in.color + x - y, 1.0);
}
