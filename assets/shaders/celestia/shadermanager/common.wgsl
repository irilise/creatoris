// common.wgsl — shared helpers ported from Celestia's dynamic shader generator.
//
// Ported from the fragment/string constants and helper functions in
// Celestia/src/celengine/shadermanager.cpp. This is a "library" file: the GLSL
// generator concatenates these pieces into every generated program, so a real
// Bevy pipeline would either concatenate this with the per-program files or
// inline the needed functions. It is NOT a standalone pipeline.
//
// WGSL translation notes (also in ../README.md):
//  * Celestia's `set_vp(p)` / `calc_vp(p)` "usual" mode is `MVPMatrix * p`.
//  * `sampler2D` splits into `texture_2d` + `sampler`; `textureLod` ->
//    `textureSampleLevel`; `sampler2DShadow` -> `sampler_2d_compare`.

// ---------------------------------------------------------------------------
// VertexHeader: the matrix uniforms shared by every vertex stage.
// Celestia declares these as GLSL uniforms; WGSL gives each its own binding.
// ---------------------------------------------------------------------------
@group(0) @binding(0)
var<uniform> model_view_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> projection_matrix : mat4x4<f32>;

@group(0) @binding(2)
var<uniform> mvp_matrix : mat4x4<f32>;

// Celestia's "usual" projection: gl_Position = MVPMatrix * p.
fn calc_vp(in_position : vec4<f32>) -> vec4<f32> {
    return mvp_matrix * in_position;
}

// Celestia's optional fisheye projection (VPFunctionFishEye). Ported for
// completeness; use this instead of calc_vp when fisheye mode is enabled.
fn calc_vp_fisheye(in_position : vec4<f32>) -> vec4<f32> {
    const pid2 : f32 = 1.570796326794896619231322;
    var in_pos : vec4<f32> = model_view_matrix * in_position;
    let l = length(in_pos.xy);
    if (l != 0.0) {
        let phi = atan2(l, -in_pos.z);
        let lens_r = phi / pid2;
        in_pos.xy *= lens_r / l;
    }
    return projection_matrix * in_pos;
}

// ---------------------------------------------------------------------------
// CommonAttribs (GLSL) — fixed input layout used by most generated programs:
//
//   @location(0) in_Position  : vec4
//   @location(1) in_Normal    : vec3
//   @location(2) in_TexCoord0 : vec4
//   @location(3) in_TexCoord1 : vec4
//   @location(4) in_TexCoord2 : vec4
//   @location(5) in_TexCoord3 : vec4
//   @location(8) in_Color     : vec4
//
// Celestia injects a few more floats at non-fixed locations (e.g. in_Size,
// in_ColorIndex, in_Brightness, in_PointSize, in_Tangent, in_PositionNext,
// in_ScaleFactor, in_LineSide). Those are declared per-program in the files
// that use them.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Per-light uniform naming (DeclareLights / LightProperty).
//
//   light{i}_direction  : vec3   normalized light direction
//   light{i}_diffuse    : vec3   diffuse light color
//   light{i}_specular   : vec3   specular light color (when specular enabled)
//   light{i}_halfVector : vec3   Blinn-Phong half vector (when specular)
//   light{i}_color      : vec3   light color (atmosphere/scattering)
//   light{i}_brightness : f32    (when a night texture is used)
//
// This project uses a single star (the Sun at the origin), so the representative
// ports below instantiate light 0 only and note the generalization to N lights.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Chapman optical-depth functions (ChapmanOpticalDepth).
// Atmospheric scattering uses these to convert a radius/zenith into an
// slant optical depth through the Chapman layer.
//
// Required uniforms (ScatteringConstantDeclarations):
//   atmosphereRadius            : vec3   (x = atmosphere radius, y = ^2, z = planet ^2)
//   atmosphereExtinctionThreshold : f32
//   mieH                        : f32
// ---------------------------------------------------------------------------
@group(0) @binding(10)
var<uniform> atmosphere_radius : vec3<f32>;

@group(0) @binding(11)
var<uniform> atmosphere_extinction_threshold : f32;

@group(0) @binding(12)
var<uniform> mie_h : f32;

fn chapman_to_space_with_scale_height(
    r : f32,
    mu : f32,
    inverse_scale_height : f32,
) -> f32 {
    let m = clamp(mu, -1.0, 1.0);
    let x = r * inverse_scale_height;
    let density = exp(min((atmosphere_radius.z - r) * inverse_scale_height, 0.0));
    let grazing = sqrt(1.5707963267948966 * x);
    if (m >= 0.0) {
        return (density / inverse_scale_height) * grazing / ((grazing - 1.0) * m + 1.0);
    }
    let sin_zenith = sqrt(max(1.0e-8, 1.0 - m * m));
    let tangent_density = exp(min((atmosphere_radius.z - r * sin_zenith) * inverse_scale_height, 0.0));
    let upward = (density / inverse_scale_height) * grazing / ((grazing - 1.0) * (-m) + 1.0);
    return max(0.0,
        (2.0 * tangent_density / inverse_scale_height) * sqrt(1.5707963267948966 * x * sin_zenith) - upward);
}

fn chapman_to_space(r : f32, mu : f32) -> f32 {
    return chapman_to_space_with_scale_height(r, mu, mie_h);
}

fn chapman_from_atmosphere_boundary(mu : f32, grazing : f32) -> f32 {
    return (atmosphere_extinction_threshold / mie_h) * grazing /
           ((grazing - 1.0) * clamp(mu, 0.0, 1.0) + 1.0);
}

fn chapman_from_atmosphere_boundary_with_scale_height(
    mu : f32,
    column_scale : f32,
    grazing : f32,
) -> f32 {
    return column_scale / ((grazing - 1.0) * clamp(mu, 0.0, 1.0) + 1.0);
}

// ---------------------------------------------------------------------------
// Scattering phase functions (ScatteringPhaseFunctions): the Mie and Rayleigh
// phase functions as a function of cos(theta), the cosine of the angle between
// the view vector and the light vector.
// ---------------------------------------------------------------------------
fn scattering_phase(cos_theta : f32, mie_k : f32) -> vec2<f32> {
    let ph_mie = (1.0 - mie_k * mie_k) /
                 ((1.0 + mie_k * cos_theta) * (1.0 + mie_k * cos_theta));
    let ph_rayleigh = 0.75 * (1.0 + cos_theta * cos_theta);
    return vec2<f32>(ph_mie, ph_rayleigh);
}

// ---------------------------------------------------------------------------
// Eclipse shadow helpers (Shadow / ShadowsForLightSource).
//
// An eclipse shadow is a circular region of constant depth (maxDepth) ringed by
// a linear falloff to zero at the penumbra edge. The umbra/penumbra radii are
// computed accurately; the falloff is approximated as linear.
//
// Required uniforms (per light i, per shadow j):
//   shadowTexGenS_{i}_{j} : vec4
//   shadowTexGenT_{i}_{j} : vec4
//   shadowFalloff_{i}_{j} : f32
//   shadowMaxDepth_{i}_{j}: f32
// ---------------------------------------------------------------------------
// (Represented per-program where eclipse shadows are enabled; the single-light
//  representative port uses shadow_tex_gen_s0 / _t0 / _falloff0 / _max_depth0.)
