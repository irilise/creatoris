// largestar.wgsl — batched textured-billboard fallback for oversized point sprites
//
// Ported from Celestia/shaders/largestar_{vert,frag}.glsl.
// Six vertices per star; per-star fields are replicated.
//
// WGSL notes:
//  * set_vp(x) == mvp_matrix * x.
//  * The GLSL `invariant gl_Position;` qualifier has no WGSL equivalent.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> viewport_rcp : vec2<f32>;

@group(0) @binding(2)
var star_texture : texture_2d<f32>;

@group(0) @binding(3)
var star_sampler : sampler;

struct VertexInput {
  @location(0) position : vec2<f32>,
  @location(15) center : vec3<f32>,
  @location(2) tex_coord0 : vec2<f32>,
  @location(8) color : vec4<f32>,
  @location(9) intensity : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) tex_coord : vec2<f32>,
  @interpolant(per-fragment) color : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;
  out.tex_coord = in.tex_coord0;
  out.color = in.color;

  let base = mvp_matrix * vec4<f32>(in.center, 1.0);
  let extent = in.intensity * viewport_rcp;
  out.clip_position = vec4<f32>(
    base.xy + in.position * extent * base.w,
    base.zw,
  );
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  return textureSample(star_texture, star_sampler, in.tex_coord) * in.color;
}
