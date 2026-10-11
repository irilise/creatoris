// particle.wgsl — particle system shader (ParticleModel / ParticleDiffuseModel).
//
// Ported from buildParticleVertexShader + buildParticleFragmentShader in
// Celestia/src/celengine/shadermanager.cpp. Vertices are lit as infinitesimal
// spherical particles (no surface normal), optionally weighted by a Mie phase
// function when a light direction is available (PARTICLE_PHASE_PARAMETER).
//
// WGSL notes: see ../README.md. `gl_PointSize`/`gl_PointCoord` -> the
// `point-size` feature + `@builtin(point_size)`/`@builtin(point_coord)`.

// ----- shared matrix (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

// ----- particle uniforms -----
@group(0) @binding(1)
var<uniform> eye_position : vec3<f32>;
@group(0) @binding(2)
var<uniform> light0_direction : vec3<f32>;
@group(0) @binding(3)
var<uniform> point_scale : f32>;

// Optional diffuse texture.
@group(0) @binding(4)
var diffuse_texture : texture_2d<f32>;
@group(0) @binding(5)
var diffuse_sampler : sampler;

struct VertexInput {
    @location(0) position : vec4<f32>,
    @location(2) tex_coord0 : vec4<f32>,
    @location(8) color : vec4<f32>,
    @location(7) point_size : f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @builtin(point_size) point_size : f32>,
    @interpolant(per-fragment) v_color : vec4<f32>,
    @interpolant(per-fragment) v_tex_coord0 : vec2<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;

    // Mie phase weighting toward the light (PARTICLE_PHASE_PARAMETER == 1).
    let g = -0.4;
    let mie_k = 1.55 * g - 0.55 * g * g * g;
    let eye_dir = normalize(eye_position - in.position.xyz);
    var brightness = 0.0;
    let cos_theta = dot(light0_direction, eye_dir);
    let ph_mie = (1.0 - mie_k * mie_k) /
                 ((1.0 + mie_k * cos_theta) * (1.0 + mie_k * cos_theta));
    brightness += ph_mie;

    out.v_color = in.color * brightness;
    out.v_tex_coord0 = in.tex_coord0.xy;

    // PointSizeCalculation: ptSize = pointScale * in_PointSize / |ModelView*P|
    let pt_size = point_scale * in.point_size / length((mvp_matrix * in.position).xyz);
    out.point_size = pt_size;

    out.clip_position = calc_vp(in.position);
    return out;
}

struct FragmentInput {
    @interpolant(per-fragment) v_color : vec4<f32>,
    @interpolant(per-fragment) v_tex_coord0 : vec2<f32>,
};

@fragment
fn fragment_main(in : FragmentInput, @builtin(point_coord) pc : vec2<f32>)
    -> @location(0) vec4<f32> {
    // With a diffuse texture: fragColor = v_Color * texture(diffTex, gl_PointCoord)
    return in.v_color * vec4<f32>(
        textureSample(diffuse_texture, diffuse_sampler, pc).rgb, 1.0);
}
