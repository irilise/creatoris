// tidal.wgsl — tidal / tidal-stream sprite
//
// Ported from Celestia/shaders/tidal_{vert,frag}.glsl.
//
// WGSL notes:
//  * set_vp(x) == mvp_matrix * x.
//  * in_TexCoord0 is reused as (uv, colorIndex).

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> view_mat : mat3x3<f32>;

@group(0) @binding(2)
var<uniform> tidal_size : f32;

@group(0) @binding(3)
var<uniform> brightness : f32;

@group(0) @binding(4)
var<uniform> pixel_weight : f32;

@group(0) @binding(5)
var color_texture : texture_2d<f32>;

@group(0) @binding(6)
var color_sampler : sampler;

@group(0) @binding(7)
var tidal_texture : texture_2d<f32>;

@group(0) @binding(8)
var tidal_sampler : sampler;

struct VertexInput {
  @location(0) position : vec3<f32>,
  @location(2) tex_coord0 : vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) tex_coord : vec2<f32>,
  @interpolant(per-fragment) color : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  let p = view_mat * in.position * tidal_size;
  out.tex_coord = vec2<f32>(in.tex_coord0.x, in.tex_coord0.y);
  let color_index = in.tex_coord0.z;
  out.color = vec4<f32>(
    textureSample(color_texture, color_sampler, vec2<f32>(color_index, 0.0)).rgb,
    min(1.0, 2.0 * brightness * pixel_weight),
  );
  out.clip_position = mvp_matrix * vec4<f32>(p, 1.0);
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(
    in.color.rgb,
    in.color.a * textureSample(tidal_texture, tidal_sampler, in.tex_coord).r,
  );
}
