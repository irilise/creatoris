// depth.wgsl — depth-only prepass
//
// Ported from Celestia/shaders/depth_{vert,frag}.glsl.
// Writes only depth; the fragment stage is intentionally empty so no color
// output is produced.
//
// WGSL notes:
//  * set_vp(x) == mvp_matrix * x.
//  * The GLSL fragment main has an empty body; WGSL requires a return, so the
//    fragment stage is omitted and the pipeline should use a depth-only render
//    pass (no color attachment).

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
