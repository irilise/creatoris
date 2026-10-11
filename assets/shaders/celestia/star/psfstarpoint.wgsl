// psfstarpoint.wgsl — PSF star "point / cone" pass (point-sprite variant)
//
// Ported from Celestia/shaders/psfstarpoint_{vert,frag}.glsl.
//
// Copyright (C) 2026-present, the Celestia Development Team
// Original rendering algorithm by Askaniy Anpilogov <aaskaniy@gmail.com>
// (https://github.com/Askaniy/CelestiaStarRenderer)
// GPL v2+ (see the source .glsl headers).
//
// The source has a `#ifdef BILLBOARD` quad variant and a point-sprite variant.
// WGSL has no preprocessor, so this file ports the point-sprite branch; the
// billboard analogue lives in `psfstarglowlarge.wgsl`.
//
// WGSL notes:
//  * gl_PointSize  -> @builtin(point_size);  gl_PointCoord -> @builtin(point_coord)
//  * set_vp(x) == mvp_matrix * x.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> point_radius : f32;

@group(0) @binding(2)
var<uniform> point_scale : f32;

struct VertexInput {
  @location(0) position : vec4<f32>,
  @location(8) color : vec3<f32>,
  @location(9) intensity : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @builtin(point_size) point_size : f32,
  @interpolant(per-fragment) color : vec3<f32>,
  @interpolant(per-fragment) peak_radiance : f32,
  @interpolant(per-fragment) point_size_out : f32,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;
  out.color = in.color;
  out.peak_radiance = in.intensity;

  let size = 2.0 * point_radius * point_scale;
  out.point_size = size;
  out.point_size_out = size;

  out.clip_position = mvp_matrix * in.position;
  return out;
}

@fragment
fn fragment_main(
    in : VertexOutput,
    @builtin(point_coord) point_coord : vec2<f32>,
) -> @location(0) vec4<f32> {
  let d = (point_coord - vec2<f32>(0.5)) * in.point_size_out;
  let px = length(d) / point_scale;

  let falloff = clamp(1.0 - px / point_radius, 0.0, 1.0);
  let radiance = in.color * (falloff * in.peak_radiance);

  return vec4<f32>(radiance, 1.0);
}
