// comet.wgsl — comet glow
//
// Ported from Celestia/shaders/comet_{vert,frag}.glsl.
// Shade is the absolute view/normal dot product scaled by brightness and a
// fade factor; the result is emitted as a soft glow.
//
// WGSL notes:
//  * set_vp(x) == mvp_matrix * x.
//  * `in_Brightness` is a per-vertex float whose Celestia layout location is not
//    fixed in shadermanager.cpp; location 10 is a placeholder.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> color : vec3<f32>;

@group(0) @binding(2)
var<uniform> view_dir : vec3<f32>;

@group(0) @binding(3)
var<uniform> fade_factor : f32;

struct VertexInput {
  @location(0) position : vec4<f32>,
  @location(1) normal : vec3<f32>,
  @location(10) brightness : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @interpolant(per-fragment) shade : f32,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;
  out.shade = abs(dot(view_dir, in.normal) * in.brightness * fade_factor);
  out.clip_position = mvp_matrix * in.position;
  return out;
}

@fragment
fn fragment_main(in : VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(color, in.shade);
}
