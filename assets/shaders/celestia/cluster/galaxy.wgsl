// galaxy.wgsl — galaxy sprite
//
// Ported from Celestia/shaders/galaxy_{vert,frag}.glsl.
// A point-anchored quad whose on-screen size decays with distance; the color
// lookup and the sprite texture are sampled independently.
//
// WGSL notes:
//  * set_vp(x) == mvp_matrix * x.
//  * `in_Size` / `in_ColorIndex` / `in_Brightness` are per-vertex floats whose
//    Celestia layout location is NOT fixed in shadermanager.cpp; locations 10/11/12
//    are placeholders — reconcile against the real vertex-buffer layout.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> m : mat4x4<f32>;

@group(0) @binding(2)
var<uniform> view_mat : mat3x3<f32>;

@group(0) @binding(3)
var<uniform> size : f32;

@group(0) @binding(4)
var<uniform> brightness : f32;

@group(0) @binding(5)
var color_texture : texture_2d<f32>;

@group(0) @binding(6)
var color_sampler : sampler;

@group(0) @binding(7)
var galaxy_texture : texture_2d<f32>;

@group(0) @binding(8)
var galaxy_sampler : sampler;

struct VertexInput {
  @location(0) position : vec4<f32>,
  @location(2) tex_coord0 : vec2<f32>,
  @location(10) star_size : f32,
  @location(11) color_index : f32,
  @location(12) vertex_brightness : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) color : vec4<f32>,
  @interpolant(per-fragment) tex_coord : vec2<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  let s = size * in.star_size;
  let p = m * in.position;
  let screen_frac = s / length(p);
  if (screen_frac >= 0.1) {
    // Discard degenerate (too-close) triangles: emit a coincident clip pos.
    out.clip_position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    return out;
  }

  let v = vec4<f32>(
    view_mat * vec3<f32>(
      in.tex_coord0.x * 2.0 - 1.0,
      in.tex_coord0.y * 2.0 - 1.0,
      0.0,
    ) * s,
    0.0,
  );
  let alpha = (0.1 - screen_frac) * in.vertex_brightness * brightness;
  out.color = vec4<f32>(
    textureSample(color_texture, color_sampler, vec2<f32>(in.color_index, 0.0)).rgb,
    alpha,
  );
  out.tex_coord = in.tex_coord0;
  out.clip_position = mvp_matrix * (p + v);
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(
    in.color.rgb,
    in.color.a * textureSample(galaxy_texture, galaxy_sampler, in.tex_coord).r,
  );
}
