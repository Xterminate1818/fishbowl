struct Uniforms {
  width: f32, height: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) offset: vec2<f32>,
  @location(3) radius: f32,
  @location(4) color: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

fn get_color(input: u32) -> vec4<f32> {
    let r = f32(input & 255u) / 255.0;
    let g = f32((input & (255u << 8u)) >> 8u) / 255.0;
    let b = f32((input & (255u << 16u)) >> 16u) / 255.0;
    let a = f32((input & (255u << 24u)) >> 24u) / 255.0;
    return vec4<f32>(r, g, b, a);
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    var x = (model.position.x * model.radius) - uniforms.width / 2.0;
    x += model.offset.x;
    var y = (model.position.y * model.radius) - uniforms.height / 2.0;
    y += model.offset.y;
    var norm_x = x / (uniforms.width / 2.0);
    var norm_y = -y / (uniforms.height / 2.0);
    out.clip_position = vec4<f32>(norm_x, norm_y, 0.0, 1.0);
    out.uv = model.uv;
    out.color = get_color(model.color);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var x = in.uv.x - 0.5;
    var y = in.uv.y - 0.5;
    var d = sqrt(x * x + y * y) * 2.0;
    if d > 1.1 {
      discard;
    } else if d > 1.0 {
        return vec4<f32>(in.color.xyz, 1.1 - d);
    } else {
        return in.color;
    }
}
