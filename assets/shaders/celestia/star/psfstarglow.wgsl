// psfstarglow.wgsl — PSF star "glow (eye-PSF)" pass (point-sprite variant)
//
// Ported from Celestia/shaders/psfstarglow_{vert,frag}.glsl.
// Approximation of Greg Spencer et al. (1995) photopic PSF.
//
// Copyright (C) 2026-present, the Celestia Development Team
// Original rendering algorithm by Askaniy Anpilogov <aaskaniy@gmail.com>
// (https://github.com/Askaniy/CelestiaStarRenderer)
// GPL v2+ (see the source .glsl headers).
//
// WGSL notes:
//  * gl_PointSize  -> @builtin(point_size);  gl_PointCoord -> @builtin(point_coord)
//  * set_vp(x) == mvp_matrix * x.
//  * `valLimb` / `alphaFade` are per-sprite constants computed once in the
//    vertex stage (as in the source) to avoid redundant per-fragment work.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> psf_a : f32;

@group(0) @binding(2)
var<uniform> psf_b : f32;

@group(0) @binding(3)
var<uniform> psf_min_vis_rad : f32;

@group(0) @binding(4)
var<uniform> point_scale : f32;

struct VertexInput {
  @location(0) position : vec4<f32>,
  @location(8) color : vec3<f32>,
  @location(9) intensity : f32,
  @location(13) alpha : f32,
  @location(14) limb_radius : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @builtin(point_size) point_size : f32,
  @interpolant(per-fragment) color : vec3<f32>,
  @interpolant(per-fragment) alpha : f32,
  @interpolant(per-fragment) peak_radiance : f32,
  @interpolant(per-fragment) psf_radius : f32,
  @interpolant(per-fragment) p04 : f32,
  @interpolant(per-fragment) limb_radius : f32,
  @interpolant(per-fragment) val_limb : f32,
  @interpolant(per-fragment) alpha_fade : f32,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;
  out.color = in.color;
  out.alpha = in.alpha;
  out.peak_radiance = in.intensity;

  // PSF support radius depends on peak radiance: psf_radius = peak^0.4 / a.
  let p04 = pow(in.intensity, 0.4);
  let r_full = p04 / psf_a;

  // Tighten the rasterised radius to where the post-alpha brightest channel
  // falls below one framebuffer code; the PSF shape inside is unchanged.
  let t_val = psf_min_vis_rad / max(in.alpha, 1.0e-3);
  var r_eff = p04 / (psf_a + pow(t_val, 0.4) / psf_b);
  r_eff = min(r_eff, r_full);

  out.psf_radius = r_eff;
  out.p04 = p04;
  out.limb_radius = in.limb_radius;

  // Interior-fade exponent, evaluated once per sprite.
  var fade_exp = 10.0;
  var val_limb = 0.0;
  if (in.limb_radius > 0.0) {
    let base_l = p04 / in.limb_radius - psf_a;
    let s = max(base_l, 0.0) * psf_b;
    val_limb = min(s * s * sqrt(s), in.intensity); // s^2.5, cheaper than pow()
    if (val_limb > 0.0) {
      fade_exp = 1.0 + log2(in.intensity / val_limb);
    }
  }
  out.val_limb = val_limb;
  out.alpha_fade = pow(out.alpha, fade_exp);

  out.point_size = 2.0 * r_eff * point_scale;
  out.clip_position = mvp_matrix * in.position;
  return out;
}

@fragment
fn fragment_main(
    in : VertexOutput,
    @builtin(point_coord) point_coord : vec2<f32>,
) -> @location(0) vec4<f32> {
  // Distance from sprite centre in unscaled pixels (pointScale cancels out).
  let px = length(point_coord - vec2<f32>(0.5)) * 2.0 * in.psf_radius;

  // psf_radius now bounds the visible disc; discard mostly fires on the corners
  // of the point quad.
  if (px >= in.psf_radius) {
    discard;
  }

  let base = in.p04 / px - psf_a;
  let s = base * psf_b;
  var val = s * s * sqrt(s); // s^2.5
  val = min(val, in.peak_radiance);

  // Inside the geometric limb, crush the plateau and floor at the limb brightness.
  var brightness = val * in.alpha;
  if (in.limb_radius > 0.0 && px < in.limb_radius && in.peak_radiance > 1.0) {
    var t = 1.0 - px / in.limb_radius;
    t = sqrt(sqrt(sqrt(t))); // t^(1/8), kPsfCoreSharpness == 8
    let val_plateau = mix(in.val_limb, in.peak_radiance, t);
    brightness = max(val_plateau * in.alpha_fade, in.val_limb * in.alpha);
  }

  return vec4<f32>(in.color * brightness, 1.0);
}
