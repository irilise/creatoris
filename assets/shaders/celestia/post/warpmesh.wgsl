// warpmesh.wgsl — warped full-screen mesh
//
// Ported from Celestia/shaders/warpmesh_{vert,frag}.glsl.
// A full-screen quad whose horizontal extent is stretched by `screen_ratio`;
// the sampled texture is tinted by a per-vertex intensity.
//
// WGSL notes:
//  * No set_vp / MVP: the source writes gl_Position directly in NDC.

@group(0) @binding(0)
var<uniform> screen_ratio : f32;

@group(0) @binding(1)
var tex : texture_2d<f32>;

@group(0) @binding(2)
var tex_sampler : sampler;

struct VertexInput {
  @location(0) position : vec2<f32>,
  @location(2) tex_coord0 : vec2<f32>,
  @location(9) intensity : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) tex_coord : vec2<f32>,
  @interpolant(per-fragment) intensity : f32,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  let offset = 0.5 - screen_ratio * 0.5;
  out.clip_position = vec4<f32>(
    in.position.x * screen_ratio,
    in.position.y,
    0.0,
    1.0,
  );
  out.tex_coord = vec2<f32>(
    in.tex_coord0.x * screen_ratio + offset,
    in.tex_coord0.y,
  );
  out.intensity = in.intensity;
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(
    textureSample(tex, tex_sampler, in.tex_coord).rgb * in.intensity,
    1.0,
  );
}
