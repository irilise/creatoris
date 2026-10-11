// srgb.wgsl — linear -> sRGB tone-map / display-encode pass
//
// Ported from Celestia/shaders/srgb_{vert,frag}.glsl.
// Full-screen pass that samples a linear HDR buffer and applies an exponential
// tone map (optional) followed by the IEC 61966-2-1 sRGB EOTF.
//
// WGSL notes:
//  * The GLSL `#ifdef TONE_MAP` / `uniform float exposure;` become a runtime
//    `use_tone_map` uniform flag; WGSL has no preprocessor.
//  * `vec3(1.055) * pow(c, 1/2.4)` uses the componentwise overload of pow.

@group(0) @binding(0)
var<uniform> use_tone_map : f32;

@group(0) @binding(1)
var<uniform> exposure : f32;

@group(0) @binding(2)
var tex : texture_2d<f32>;

@group(0) @binding(3)
var tex_sampler : sampler;

struct VertexInput {
  @location(0) position : vec2<f32>,
  @location(2) tex_coord0 : vec2<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) tex_coord : vec2<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;
  out.clip_position = vec4<f32>(in.position.xy, 0.0, 1.0);
  out.tex_coord = in.tex_coord0;
  return out;
}

// Apply the sRGB electro-optical transfer function (IEC 61966-2-1).
// Input is assumed to be linear light; output is gamma-encoded for display.
fn linear_to_srgb(c : vec3<f32>) -> vec3<f32> {
  // Clamp negatives to avoid undefined pow() behaviour.
  let clamped = max(c, vec3<f32>(0.0));
  let higher = vec3<f32>(1.055) * pow(clamped, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
  let lower = vec3<f32>(12.92) * clamped;
  // step(edge, x): 0 when x < edge, 1 when x >= edge.
  return mix(lower, higher, step(vec3<f32>(0.0031308), clamped));
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  let color = textureSample(tex, tex_sampler, in.tex_coord);
  if (use_tone_map > 0.5) {
    // Exponential tone mapping to roll off HDR highlights.
    let mapped = vec3<f32>(1.0) - exp(-exposure * color.rgb);
    return vec4<f32>(linear_to_srgb(mapped), color.a);
  }
  return vec4<f32>(linear_to_srgb(color.rgb), color.a);
}
