// globular.wgsl — globular-cluster star field
//
// Ported from Celestia/shaders/globular_{vert,frag}.glsl.
// Per-star point sprites whose sizes "morph" toward physical values as the
// observer closes in, dissolving the overdense core.
//
// WGSL notes:
//  * gl_PointSize -> @builtin(point_size); gl_PointCoord -> @builtin(point_coord)
//    (need the `point-size` feature).
//  * set_vp(x) == mvp_matrix * x.
//  * in_TexCoord0 is reused as (starSize, relStarDensity, colorIndex).

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> m : mat3x3<f32>;

@group(0) @binding(2)
var<uniform> offset : vec3<f32>;

@group(0) @binding(3)
var<uniform> brightness : f32;

@group(0) @binding(4)
var<uniform> pixel_weight : f32;

@group(0) @binding(5)
var<uniform> scale : f32;

@group(0) @binding(6)
var color_texture : texture_2d<f32>;

@group(0) @binding(7)
var color_sampler : sampler;

@group(0) @binding(8)
var star_texture : texture_2d<f32>;

@group(0) @binding(9)
var star_sampler : sampler;

// Observer distance [ly] at which star-sprite sizes start morphing toward
// physical values.
const clip_distance : f32 = 100.0;

struct VertexInput {
  @location(0) position : vec3<f32>,
  @location(2) tex_coord0 : vec3<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @builtin(point_size) point_size : f32,
  @interpolant(per-fragment) color : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  let star_size = in.tex_coord0.x;
  let rel_star_density = in.tex_coord0.y;
  let color_index = in.tex_coord0.z;

  let p = m * in.position;
  let br = 2.0 * brightness;
  let s = br * star_size * scale;

  // Morph the sprite sizes at close observer distance so the overdense core
  // dissolves as we close in.
  let obs_distance_to_star_ratio = length(p + offset) / clip_distance;
  out.point_size = s * min(obs_distance_to_star_ratio, 1.0);

  out.color = vec4<f32>(
    textureSample(color_texture, color_sampler, vec2<f32>(color_index, 0.0)).rgb,
    min(1.0, br * (1.0 - pixel_weight * rel_star_density)),
  );
  out.clip_position = mvp_matrix * vec4<f32>(p, 1.0);
  return out;
}

@fragment
fn fragment_main(
    in : VertexOutput,
    @builtin(point_coord) point_coord : vec2<f32>,
) -> @location(0) vec4<f32> {
  return vec4<f32>(
    in.color.rgb,
    in.color.a * textureSample(star_texture, star_sampler, point_coord).r,
  );
}
