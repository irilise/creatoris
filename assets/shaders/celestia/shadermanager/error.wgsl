// error.wgsl — error / diagnostic program.
//
// Ported from CreateErrorProgram (errorVertexShaderSource /
// errorFragmentShaderSource) in Celestia/src/celengine/shadermanager.cpp.
// A trivial program that renders a flat red triangle, used to surface render
// failures. WGSL notes: see ../README.md.

// ----- shared matrix (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

struct VertexInput {
    @location(0) position : vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;
    out.clip_position = mvp_matrix * in.position;
    return out;
}

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
