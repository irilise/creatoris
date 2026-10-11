// emissive.wgsl — emissive / unlit shader (EmissiveModel).
//
// Ported from buildEmissiveVertexShader + buildEmissiveFragmentShader in
// Celestia/src/celengine/shadermanager.cpp. The emissive model ignores all
// lighting and uses the material/diffuse color (optionally from a diffuse
// texture) as the final fragment color.
//
// WGSL notes: see ../README.md. `gl_PointCoord` (point-sprite variant) ->
// `@builtin(point_coord)` + the `point-size` feature.

// ----- shared matrix (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

// ----- emissive uniforms -----
@group(0) @binding(1)
var<uniform> opacity : f32>;
// The material color comes from light 0's diffuse property so emissive bodies
// interoperate with lit ones.
@group(0) @binding(2)
var<uniform> light0_diffuse : vec3<f32>;

// Optional diffuse texture.
@group(0) @binding(3)
var diffuse_texture : texture_2d<f32>;
@group(0) @binding(4)
var diffuse_sampler : sampler;

struct VertexInput {
    @location(0) position : vec4<f32>,
    @location(2) tex_coord0 : vec4<f32>,
    @location(8) color : vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @interpolant(per-fragment) v_color : vec4<f32>,
    @interpolant(per-fragment) v_tex_coord0 : vec2<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;
    // v_Color uses in_Color.rgb when vertex colors are enabled, else light0 color.
    out.v_color = vec4<f32>(in.color.rgb, opacity);
    out.v_tex_coord0 = in.tex_coord0.xy;
    out.clip_position = calc_vp(in.position);
    return out;
}

struct FragmentInput {
    @interpolant(per-fragment) v_color : vec4<f32>,
    @interpolant(per-fragment) v_tex_coord0 : vec2<f32>,
};

@fragment
fn fragment_main(in : FragmentInput) -> @location(0) vec4<f32> {
    // With a diffuse texture: fragColor = v_Color * texture(diffTex, v_TexCoord0.st)
    let tex = textureSample(diffuse_texture, diffuse_sampler, in.v_tex_coord0);
    return in.v_color * vec4<f32>(tex.rgb, 1.0);
}
