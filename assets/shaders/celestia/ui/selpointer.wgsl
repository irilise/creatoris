// selpointer.wgsl — selection-pointer quad
//
// Ported from Celestia/shaders/selpointer_{vert,frag}.glsl.
// Builds a small rotated quad at a pixel-anchored point (x0, y0) from two
// orthonormal pixel-space directions u, v.
//
// WGSL notes:
//  * set_vp(x) == mvp_matrix * x.
//  * `color` is a per-draw uniform (a flat vec4).

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> pixel_size : f32;

@group(0) @binding(2)
var<uniform> s : f32;

@group(0) @binding(3)
var<uniform> c : f32;

@group(0) @binding(4)
var<uniform> x0 : f32;

@group(0) @binding(5)
var<uniform> y0 : f32;

@group(0) @binding(6)
var<uniform> u : vec3<f32>;

@group(0) @binding(7)
var<uniform> v : vec3<f32>;

@group(0) @binding(8)
var<uniform> color : vec4<f32>;

struct VertexInput {
  @location(0) position : vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  let x = in.position.x * pixel_size;
  let y = in.position.y * pixel_size;
  let pos = (x * c - y * s + x0) * u + (x * s + y * c + y0) * v;
  out.clip_position = mvp_matrix * vec4<f32>(pos, 1.0);
  return out;
}

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
  return color;
}
