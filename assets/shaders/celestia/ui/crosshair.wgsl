// crosshair.wgsl — rotated crosshair overlay
//
// Ported from Celestia/shaders/crosshair_{vert,frag}.glsl.
// A 2D crosshair built from a unit quad, rotated by `angle` and placed at
// `radius` along the local X axis.
//
// WGSL notes:
//  * `mat3(c, s, 0, -s, c, 0, 0, 0, 1)` is column-major in GLSL -> the three
//    columns are (c,-s,0), (s,c,0), (0,0,1).
//  * `MVPMatrix * p` is written as `mvp_matrix * p` (GLSL column-major *
//    column-major is preserved in WGSL).

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var<uniform> radius : f32;

@group(0) @binding(2)
var<uniform> width : f32;

@group(0) @binding(3)
var<uniform> h : f32;

@group(0) @binding(4)
var<uniform> angle : f32;

@group(0) @binding(5)
var<uniform> color : vec4<f32>;

struct VertexInput {
  @location(0) position : vec4<f32>,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;

  let c = cos(angle);
  let s = sin(angle);
  let rot = mat3x3<f32>(
    vec3<f32>(c, -s, 0.0),
    vec3<f32>(s, c, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
  );

  let x = in.position.x * width + radius;
  let y = in.position.y * h;
  let p = rot * vec3<f32>(x, y, 0.0);
  out.clip_position = mvp_matrix * vec4<f32>(p, 1.0);
  return out;
}

@fragment
fn fragment_main() -> @location(0) vec4<f32> {
  return color;
}
