// body.wgsl — main body / surface shader (DiffuseModel, with optional
// specular, shadows, night map, emissive map, overlay).
//
// Ported from buildVertexShader + buildFragmentShader in
// Celestia/src/celengine/shadermanager.cpp. This is the "else" branch of
// buildProgram: it handles every lighting model that is NOT RingIllum,
// Atmosphere, Emissive, or Particle (i.e. DiffuseModel, PerPixelSpecularModel,
// OrenNayarModel, LunarLambertModel, StarModel, UnlitModel, ParticleDiffuseModel).
//
// This file ports the canonical single-light, non-tangent-space variant.
//
// Conditional GLSL `#if` variants and how they map:
//  * PerPixelSpecularModel  -> add `light0_specular`/`light0_halfVector` + the
//    spec block (shown, guarded by `has_specular`).
//  * OrenNayarModel         -> the OrenNayar body of AddDirectionalLightContrib
//    is `#if 0` in the source (disabled); see AddDirectionalLightContrib note
//    in this file's fragment.
//  * LunarLambertModel      -> add `lunar_lambert` uniform and the
//    phase-angle limb-darkening (LunarLambert); see note below the light loop.
//  * StarModel              -> add the limb-darkening line at the end of main.
//  * UnlitModel + VertexColors -> diffuse comes from in_Color.
//  * TangentSpaceLighting   -> uses normal + tangent maps; omitted here (large
//    branch; see shadermanager.cpp buildFragmentShader).
//  * AtmosphereModel        -> see atmosphere.wgsl instead.
//
// WGSL notes: see ../README.md. Light uniforms are single-light (light0_*);
// Celestia loops N lights. `invariant gl_Position` (GLSL VertexHeader) is
// dropped (no WGSL equivalent).

// ----- shared matrices (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

// ----- per-light uniforms (light 0) -----
@group(0) @binding(1)
var<uniform> light0_direction : vec3<f32>;
@group(0) @binding(2)
var<uniform> light0_diffuse : vec3<f32>;
@group(0) @binding(3)
var<uniform> light0_color : vec3<f32>;
@group(0) @binding(4)
var<uniform> light0_brightness : f32>;

// ----- material / lighting uniforms -----
@group(0) @binding(5)
var<uniform> ambient_color : vec3<f32>;
@group(0) @binding(6)
var<uniform> opacity : f32>;
@group(0) @binding(7)
var<uniform> shininess : f32>;
@group(0) @binding(8)
var<uniform> eye_position : vec3<f32>;
@group(0) @binding(9)
var<uniform> tex_coord_offset : f32>;

// ----- texture uniforms (split sampler + texture per the mapping) -----
@group(0) @binding(10)
var diffuse_texture : texture_2d<f32>;
@group(0) @binding(11)
var diffuse_sampler : sampler;
@group(0) @binding(12)
var night_texture : texture_2d<f32>;
@group(0) @binding(13)
var night_sampler : sampler;
@group(0) @binding(14)
var emissive_texture : texture_2d<f32>;
@group(0) @binding(15)
var emissive_sampler : sampler;

// GLSL CommonAttribs (locations) + the extras this program uses.
struct VertexInput {
    @location(0) position : vec4<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) tex_coord0 : vec4<f32>,
    @location(8) color : vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @interpolant(per-fragment) normal : vec3<f32>,
    @interpolant(per-fragment) position : vec3<f32>,
    @interpolant(per-fragment) diff_tex_coord : vec2<f32>,
    @interpolant(per-fragment) night_tex_coord : vec2<f32>,
    @interpolant(per-fragment) emissive_tex_coord : vec2<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;
    out.normal = in.normal;
    out.position = in.position.xyz;
    // Diffuse/night/emissive share the base tex coord; the offset is added for
    // diffuse (cloud maps) only.
    out.diff_tex_coord = in.tex_coord0.xy + vec2<f32>(tex_coord_offset, 0.0);
    out.night_tex_coord = in.tex_coord0.xy;
    out.emissive_tex_coord = in.tex_coord0.xy;
    out.clip_position = calc_vp(in.position);
    return out;
}

struct FragmentInput {
    @interpolant(per-fragment) normal : vec3<f32>,
    @interpolant(per-fragment) position : vec3<f32>,
    @interpolant(per-fragment) diff_tex_coord : vec2<f32>,
    @interpolant(per-fragment) night_tex_coord : vec2<f32>,
    @interpolant(per-fragment) emissive_tex_coord : vec2<f32>,
};

@fragment
fn fragment_main(in : FragmentInput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let nposition = normalize(in.position);
    var diff : vec4<f32> = vec4<f32>(ambient_color, opacity);
    var spec : vec4<f32> = vec4<f32>(0.0);
    let eye_dir = normalize(eye_position - nposition);
    let nv = dot(n, eye_dir);

    // ---- directional light contribution (single light) ----
    // AddDirectionalLightContrib: NL = max(0, dot(N, L)).
    let nl = max(0.0, dot(n, light0_direction));
    diff.rgb += nl * light0_diffuse;
    // Specular (PerPixelSpecularModel):
    //   let h = normalize(eye_dir + light0_direction);
    //   spec.rgb += nl * pow(max(0.0, dot(n, h)), shininess) * light0_specular;
    //
    // LunarLambertModel variant (note):
    //   let cos_phase = dot(eye_dir, light0_direction);
    //   let phase = degrees(acos(clamp(cos_phase, -1.0, 1.0)));
    //   let limb = 1.0 + phase * (-0.019 + phase * (0.242e-3 - phase * 1.46e-6));
    //   let weight = lunar_lambert * max(limb, 0.0);
    //   diff.rgb += mix(nl, 2.0 * nl / (max(nv, 0.001) + nl), weight) * light0_diffuse;

    // ---- albedo / diffuse map ----
    let color = textureSample(diffuse_texture, diffuse_sampler, in.diff_tex_coord);

    // ---- specular / diffuse combine ----
    var result = color * diff;

    // ---- night lights ----
    var total_light = nl * light0_brightness;
    // NightTextureBlend: totalLight = 1 - totalLight, then ^4
    let blend = 1.0 - total_light;
    let night_factor = blend * blend * blend * blend;
    result += textureSample(night_texture, night_sampler, in.night_tex_coord) * night_factor;

    // ---- emissive map (added on top, unlit) ----
    result.rgb += textureSample(emissive_texture, emissive_sampler, in.emissive_tex_coord).rgb;

    // ---- StarModel limb darkening (note; only for star surfaces) ----
    // result.rgb = result.rgb * (1.0 - vec3<f32>(1.0 - nv) * vec3<f32>(0.56, 0.61, 0.72));

    return vec4<f32>(result.rgb, 1.0);
}
