// glcompat.wgsl — Lua GL-compatibility full-screen quad shader.
//
// Ported from kVertexShader / kFragmentShader in
// Celestia/src/celscript/lua/glcompat.cpp. This is the fixed full-screen-quad
// program the Lua GL-compat layer uses; it optionally passes through a color
// and/or a textured sample.
//
// The GLSL version injects `#define SHADER_COLOR` / `#define SHADER_TEXCOORD`
// at compile time to select two variants. WGSL has no preprocessor, so the two
// variants are kept as two functions and the pipeline picks one (or a runtime
// flag selects between them). The `point-size` feature is needed for
// gl_PointSize.

// ----- shared matrix (see ../shadermanager/common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;
@group(0) @binding(1)
var<uniform> point_size : f32>;

// Optional texture (SHADER_TEXCOORD variant).
@group(0) @binding(2)
var texture_2d : texture_2d<f32>;
@group(0) @binding(3)
var texture_sampler : sampler;

// SHADER_COLOR variant vertex stage.
struct ColorVertexInput {
    @location(0) position : vec2<f32>,
    @location(8) color : vec4<f32>,
};

struct ColorVertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @builtin(point_size) point_size : f32>,
    @interpolant(per-fragment) v_color : vec4<f32>,
};

@vertex
fn color_vertex_main(in : ColorVertexInput) -> ColorVertexOutput {
    var out : ColorVertexOutput;
    out.v_color = in.color;
    out.clip_position = mvp_matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.point_size = point_size;
    return out;
}

// SHADER_TEXCOORD variant vertex stage.
struct TexVertexInput {
    @location(0) position : vec2<f32>,
    @location(2) tex_coord0 : vec2<f32>,
};

struct TexVertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @builtin(point_size) point_size : f32>,
    @interpolant(per-fragment) v_tex_coord : vec2<f32>,
};

@vertex
fn tex_vertex_main(in : TexVertexInput) -> TexVertexOutput {
    var out : TexVertexOutput;
    out.v_tex_coord = in.tex_coord0;
    out.clip_position = mvp_matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.point_size = point_size;
    return out;
}

// SHADER_COLOR fragment stage.
@fragment
fn color_fragment_main(in : ColorVertexOutput) -> @location(0) vec4<f32> {
    return in.v_color;
}

// SHADER_TEXCOORD fragment stage.
@fragment
fn tex_fragment_main(in : TexVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_2d, texture_sampler, in.v_tex_coord);
}
