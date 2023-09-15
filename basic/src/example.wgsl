struct UniformData {
    view_proj: mat4x4<f32>,
    mouse_move: vec2<f32>,
    mouse_click: vec2<f32>,
    resolution: vec2<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> uni: UniformData;

struct VertexInput {
    @location(0) point: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) point: vec4<f32>,
    @location(1) color: vec4<f32>
}

@vertex
fn v_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.point = uni.view_proj * vec4f(in.point, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn f_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = select(0.0, 0.3, distance(in.point.xy, uni.mouse_move) < 25.0);
    let y = select(0.0, 0.3, distance(in.point.xy, uni.mouse_click) < 25.0);
    return vec4f(in.color.rgb + x - y, in.color.a);
}
