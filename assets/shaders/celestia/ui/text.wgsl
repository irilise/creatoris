// text.wgsl — glyph-atlas text
//
// Ported from Celestia/shaders/text_{vert,frag}.glsl.
// Per-instance glyph quads positioned by an instance position + per-glyph
// size; the fragment samples a glyph atlas and multiplies by the per-glyph
// color.
//
// WGSL notes:
//  * `MVPMatrix * p` is written as `mvp_matrix * p`.
//  * `tex_size * vec2(x, 1 - y)` scales only the y component of the tex coord.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var atlas_texture : texture_2d<f32>;

@group(0) @binding(2)
var atlas_sampler : sampler;

struct VertexInput {
  @location(0) vertex_pos : vec2<f32>,
  @location(1) instance_pos : vec2<f32>,
  @location(2) size : vec2<f32>,
  @location(3) tex_coord : vec2<f32>,
  @location(4) tex_size : vec2<f32>,
  @location(8) color : vec4<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) tex_coord : vec2<f32>,
  @interpolant(per-fragment) color : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  out.clip_position = mvp_matrix * vec4<f32>(
    in.instance_pos + in.size * in.vertex_pos,
    0.0,
    1.0,
  );
  out.tex_coord = in.tex_coord + in.tex_size * vec2<f32>(
    in.vertex_pos.x,
    1.0 - in.vertex_pos.y,
  );
  out.color = in.color;
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(
    in.color.rgb,
    textureSample(atlas_texture, atlas_sampler, in.tex_coord).r * in.color.a,
  );
}
