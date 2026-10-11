// atmosphere.wgsl — atmospheric scattering shader (AtmosphereModel).
//
// Ported from buildAtmosphereVertexShader + buildAtmosphereFragmentShader +
// AtmosphericEffects in Celestia/src/celengine/shadermanager.cpp.
//
// This is a multi-segment ray march through a Chapman atmosphere that
// accumulates Rayleigh + Mie scattered light and transmittance (ex). It is
// the physically-based limb-glow Celestia renders for Earth, Venus, etc.
//
// This file ports the MODERN "separate Rayleigh/Mie scale heights" variant
// (props.separateRayleighMieScaleHeights == true). The LEGACY variant
// (single extinctionCoeff) differs only in the per-segment accumulation block
// (see note in main) and the final `ex`.
//
// The vertex stage outputs a `centroid` position (MSAA-safe: centroid avoids
// MSAA extrapolating position past the shell silhouette, which over-brightens
// limb samples into dashes). WGSL has no `centroid` qualifier, so the
// equivalent is to enable the pipeline with `depth_clip_clamp` and mark the
// varying `@interpolant(centroid)`.
//
// WGSL notes: see ../README.md and common.wgsl (Chapman helpers). Single light.

// ----- shared matrix (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

// ----- per-light uniforms (light 0) -----
@group(0) @binding(1)
var<uniform> light0_direction : vec3<f32>;
@group(0) @binding(2)
var<uniform> light0_color : vec3<f32>;

// ----- scattering uniforms (ScatteringConstantDeclarations) -----
@group(0) @binding(3)
var<uniform> eye_position : vec3<f32>;
// atmosphereRadius: x = atmosphere radius, y = atmosphere radius ^2,
// z = planet radius ^2.
@group(0) @binding(4)
var<uniform> atmosphere_radius : vec3<f32>;
@group(0) @binding(5)
var<uniform> atmosphere_segment_count : i32;
@group(0) @binding(6)
var<uniform> atmosphere_extinction_threshold : f32>;
@group(0) @binding(7)
var<uniform> mie_coeff : f32>;
@group(0) @binding(8)
var<uniform> mie_h : f32>;
@group(0) @binding(9)
var<uniform> mie_k : f32>;
@group(0) @binding(10)
var<uniform> rayleigh_coeff : vec3<f32>;
@group(0) @binding(11)
var<uniform> rayleigh_h : f32>;
@group(0) @binding(12)
var<uniform> extinction_coeff : vec3<f32>;

struct VertexInput {
    @location(0) position : vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    // centroid in GLSL -> @interpolant(centroid) in WGSL (see header note).
    @interpolant(centroid) position : vec3<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;
    out.position = in.position.xyz;
    out.clip_position = calc_vp(vec4<f32>(in.position.xyz, 1.0));
    return out;
}

struct FragmentInput {
    @interpolant(centroid) position : vec3<f32>,
};

// Per-segment scattering accumulators. WGSL has no `out`/`inout` parameters, so
// the modern GLSL "accumulate in place" idiom is ported as copy-in + return.
struct ScatterAccum {
    mie_scattered : vec3<f32>,
    rayleigh_scattered : vec3<f32>,
}

// Per-segment scattering accumulation (modern separate-Rayleigh/Mie path).
fn scatter_segment(
    sample_radius : f32,
    sample_mu : f32,
    boundary_mu : f32,
    mie_to_space : f32,
    mie_extinction_coeff : vec3<f32>,
    rayleigh_extinction_coeff : vec3<f32>,
    rayleigh_boundary_column_scale : f32,
    atmosphere_boundary_grazing : f32,
    segment_depth : f32,
    rayleigh_segment_depth : f32,
    sun_visibility : f32,
    accum : ScatterAccum,
) -> ScatterAccum {
    // (SeparateAtmosphereScatteringAccumulation)
    var mie_scattered = accum.mie_scattered;
    var rayleigh_scattered = accum.rayleigh_scattered;
    let rayleigh_to_space =
        chapman_to_space_with_scale_height(sample_radius, sample_mu, rayleigh_h);
    let rayleigh_od_sun = max(0.0,
        rayleigh_to_space -
        chapman_from_atmosphere_boundary_with_scale_height(
            boundary_mu, rayleigh_boundary_column_scale, atmosphere_boundary_grazing));
    var od_sun = max(0.0,
        mie_to_space -
        chapman_from_atmosphere_boundary_with_scale_height(
            boundary_mu, rayleigh_boundary_column_scale, atmosphere_boundary_grazing));
    let sample_transmittance = exp(
        -mie_extinction_coeff * (od_sun)
        - rayleigh_extinction_coeff * (rayleigh_od_sun));
    let segment_tau =
        mie_extinction_coeff * segment_depth + rayleigh_extinction_coeff * rayleigh_segment_depth;
    let segment_integral = (vec3<f32>(1.0) - exp(-segment_tau)) / max(segment_tau, vec3<f32>(1.0e-18));
    let thin_integral =
        vec3<f32>(1.0) - segment_tau * 0.5 + segment_tau * segment_tau * (1.0 / 6.0);
    let thin_mask = vec3<f32>(1.0) - step(vec3<f32>(1.0e-3), segment_tau);
    let integral = mix(segment_integral, thin_integral, thin_mask);
    mie_scattered += sample_transmittance * (segment_depth * integral * sun_visibility);
    rayleigh_scattered += sample_transmittance * (rayleigh_segment_depth * integral * sun_visibility);
    return ScatterAccum(mie_scattered, rayleigh_scattered);
}

@fragment
fn fragment_main(in : FragmentInput) -> @location(0) vec4<f32> {
    let nposition = normalize(in.position);
    let eye_dir = normalize(eye_position - nposition);

    // Ray-sphere intersection of the view ray with the atmosphere shell.
    let rq = dot(eye_position, eye_dir);
    let qq = dot(eye_position, eye_position) - atmosphere_radius.y;
    let d = sqrt(max(rq * rq - qq, 0.0));
    let atm_enter = eye_position + min(0.0, -rq + d) * eye_dir;
    let atm_leave = nposition;

    let dist_atm = length(atm_enter - atm_leave);
    let view_direction = -eye_dir;
    let inverse_segment_count = 1.0 / f32(atmosphere_segment_count);
    let atm_step = (atm_leave - atm_enter) * inverse_segment_count;
    let step_length = dist_atm * inverse_segment_count;

    var segment_start = atm_enter;
    var mie_scattered_light : vec3<f32> = vec3<f32>(0.0);
    var rayleigh_scattered_light : vec3<f32> = vec3<f32>(0.0);

    let planet_radius_sq = atmosphere_radius.z * atmosphere_radius.z;
    let shadow_angular_width = min(0.2, sqrt(2.0 / (atmosphere_radius.z * mie_h)));
    let shadow_width = (atmosphere_radius.x + atmosphere_radius.z) * 0.5 * shadow_angular_width;
    let atmosphere_boundary_grazing = sqrt(1.5707963267948966 * atmosphere_radius.x * mie_h);

    var view_start_radius_sq = dot(segment_start, segment_start);
    var view_start_projection = dot(segment_start, view_direction);
    let view_start_radius = sqrt(view_start_radius_sq);
    let view_start_mu = view_start_projection / view_start_radius;

    let mie_extinction_coeff = max(vec3<f32>(mie_coeff), vec3<f32>(0.0));
    let rayleigh_extinction_coeff = max(extinction_coeff - mie_extinction_coeff, vec3<f32>(0.0));
    let mie_boundary_density = exp(min((atmosphere_radius.z - atmosphere_radius.x) * mie_h, 0.0));
    let mie_boundary_column_scale = (mie_boundary_density / mie_h) * atmosphere_boundary_grazing;
    let rayleigh_boundary_grazing = sqrt(1.5707963267948966 * atmosphere_radius.x * rayleigh_h);
    let rayleigh_boundary_density =
        exp(min((atmosphere_radius.z - atmosphere_radius.x) * rayleigh_h, 0.0));
    let rayleigh_boundary_column_scale =
        (rayleigh_boundary_density / rayleigh_h) * rayleigh_boundary_grazing;

    var od_atm = 0.0;
    var rayleigh_od_atm = 0.0;

    for (var i = 0; i < atmosphere_segment_count; i = i + 1) {
        let view_end_projection = view_start_projection + step_length;
        let view_end_radius_sq =
            max(1.0e-12, view_start_radius_sq + step_length * (2.0 * view_start_projection + step_length));
        let view_end_radius = sqrt(view_end_radius_sq);
        let view_end_mu = view_end_projection / view_end_radius;
        let view_inward = view_end_radius_sq < view_start_radius_sq;

        // Column-depth change over this segment (Chapman), cached across segments.
        let view_start_column = chapman_to_space(view_start_radius, view_inward ? -view_start_mu : view_start_mu);
        let view_end_column = chapman_to_space(view_end_radius, view_inward ? -view_end_mu : view_end_mu);
        let segment_depth = max(0.0, view_inward ? view_end_column - view_start_column
                                                : view_start_column - view_end_column);
        // RayleighViewOpticalDepth (separate scale height) collapses to the
        // same form with rayleigh_h here.
        let rayleigh_start_column =
            chapman_to_space_with_scale_height(view_start_radius, view_inward ? -view_start_mu : view_start_mu, rayleigh_h);
        let rayleigh_end_column =
            chapman_to_space_with_scale_height(view_end_radius, view_inward ? -view_end_mu : view_end_mu, rayleigh_h);
        let rayleigh_segment_depth = max(0.0, view_inward ? rayleigh_end_column - rayleigh_start_column
                                                          : rayleigh_start_column - rayleigh_end_column);

        // Sun visibility at the segment's midpoint.
        let sample = segment_start + atm_step * 0.5;
        let sample_rq = dot(sample, light0_direction);
        let sample_radius_sq = dot(sample, sample);
        let horizon_distance = sqrt(max(0.0, sample_radius_sq - planet_radius_sq));
        let sun_visibility = smoothstep(-shadow_width, shadow_width, sample_rq + horizon_distance);

        if (sun_visibility > 0.0) {
            let sample_radius = sqrt(sample_radius_sq);
            let sample_mu = sample_rq / sample_radius;
            let boundary_d = sqrt(max(sample_rq * sample_rq - (sample_radius_sq - atmosphere_radius.y), 0.0));
            let boundary_mu = boundary_d / atmosphere_radius.x;
            let mie_to_space = chapman_to_space(sample_radius, sample_mu);

            var accum = ScatterAccum(mie_scattered_light, rayleigh_scattered_light);
            accum = scatter_segment(
                sample_radius, sample_mu, boundary_mu, mie_to_space,
                mie_extinction_coeff, rayleigh_extinction_coeff,
                rayleigh_boundary_column_scale, atmosphere_boundary_grazing,
                segment_depth, rayleigh_segment_depth, sun_visibility,
                accum);
            mie_scattered_light = accum.mie_scattered;
            rayleigh_scattered_light = accum.rayleigh_scattered;
        }

        od_atm += segment_depth;
        rayleigh_od_atm += rayleigh_segment_depth;

        segment_start += atm_step;
        view_start_radius_sq = view_end_radius_sq;
        view_start_projection = view_end_projection;
    }

    let ex = exp(-mie_extinction_coeff * od_atm - rayleigh_extinction_coeff * rayleigh_od_atm);

    let light_color = light0_color;
    let cos_theta = dot(eye_dir, light0_direction);
    let phase = scattering_phase(cos_theta, mie_k);

    let integrated_light = light_color *
        (phase.y * rayleigh_coeff * rayleigh_scattered_light
         + phase.x * mie_coeff * mie_scattered_light);

    return vec4<f32>(integrated_light, 0.0);
}

// LEGACY variant (single extinctionCoeff) note:
//   ex = exp(-extinctionCoeff * odAtm)
//   integratedLight = (phRayleigh * rayleighCoeff + phMie * mieCoeff)
//                    * lightColor * scatteredLight   // single `scatteredLight`
// and the per-segment accumulation uses LegacyAtmosphereScatteringAccumulation
// (one `odSun`, one `extinctionCoeff`, no Rayleigh split).
