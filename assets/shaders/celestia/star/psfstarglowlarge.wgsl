// psfstarglowlarge.wgsl — PSF glow batched-billboard fallback
//
// Ported from Celestia/shaders/psfstarglowlarge_{vert,frag}.glsl.
// Six vertices per star; per-star fields are replicated so the whole frame's
// glows draw in one batch when gl_PointSize would exceed the driver's
// GL_ALIASED_POINT_SIZE_RANGE.
//
// Copyright (C) 2026-present, the Celestia Development Team
// Original rendering algorithm by Askaniy Anpilogov <aaskaniy@gmail.com>
// (https://github.com/Askaniy/CelestiaStarRenderer)
// GPL v2+ (see the source .glsl headers).
//
// WGSL notes:
//  * This is the billboard analogue of psfstarglow.wgsl; the geometry is a
//    centred quad (in_Position in [-1,1]) offset around in_Center in pixels.
//  * set_vp(x) == mvp_matrix * x.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> psf_a : f32;

@group(0) @binding(2)
var<uniform> psf_b : f32;

@group(0) @binding(3)
var<uniform> psf_min_vis_rad : f32;

@group(0) @binding(4)
var<uniform> psf_point_scale : f32;

@group(0) @binding(5)
var<uniform> psf_viewport_rcp : vec2<f32>;

struct VertexInput {
  @location(0) position : vec2<f32>,
  @location(15) center : vec3<f32>,
  @location(2) tex_coord0 : vec2<f32>,
  @location(8) color : vec3<f32>,
  @location(9) intensity : f32,
  @location(13) alpha : f32,
  @location(14) limb_radius : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) uv : vec2<f32>,
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
  out.uv = in.tex_coord0;
  out.color = in.color;
  out.alpha = in.alpha;
  out.peak_radiance = in.intensity;
  out.limb_radius = in.limb_radius;

  let p04 = pow(in.intensity, 0.4);
  let r_full = p04 / psf_a;

  // Match psfstarglow.wgsl: tighten the rasterised radius to where the
  // post-alpha brightest channel falls below one framebuffer code.
  let t_val = psf_min_vis_rad / max(in.alpha, 1.0e-3);
  var r_eff = p04 / (psf_a + pow(t_val, 0.4) / psf_b);
  r_eff = min(r_eff, r_full);

  out.psf_radius = r_eff;
  out.p04 = p04;

  var fade_exp = 10.0;
  var val_limb = 0.0;
  if (in.limb_radius > 0.0) {
    let base_l = p04 / in.limb_radius - psf_a;
    let s = max(base_l, 0.0) * psf_b;
    val_limb = min(s * s * sqrt(s), in.intensity);
    if (val_limb > 0.0) {
      fade_exp = 1.0 + log2(in.intensity / val_limb);
    }
  }
  out.val_limb = val_limb;
  out.alpha_fade = pow(out.alpha, fade_exp);

  // set_vp(vec4(in_Center, 1.0)); then offset the quad corners by extent * w.
  let base = mvp_matrix * vec4<f32>(in.center, 1.0);
  let size_phys = 2.0 * r_eff * psf_point_scale;
  let extent = size_phys * psf_viewport_rcp;
  out.clip_position = vec4<f32>(
    base.xy + in.position * extent * base.w,
    base.zw,
  );
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  let r = in.psf_radius;
  let d = (in.uv - vec2<f32>(0.5)) * 2.0 * r;
  let px = length(d);
  if (px >= r) {
    discard;
  }

  let base = in.p04 / px - psf_a;
  let s = base * psf_b;
  var val = s * s * sqrt(s);
  val = min(val, in.peak_radiance);

  var brightness = val * in.alpha;
  if (in.limb_radius > 0.0 && px < in.limb_radius && in.peak_radiance > 1.0) {
    var t = 1.0 - px / in.limb_radius;
    t = sqrt(sqrt(sqrt(t)));
    let val_plateau = mix(in.val_limb, in.peak_radiance, t);
    brightness = max(val_plateau * in.alpha_fade, in.val_limb * in.alpha);
  }

  return vec4<f32>(in.color * brightness, 1.0);
}
