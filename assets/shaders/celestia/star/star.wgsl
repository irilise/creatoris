// star.wgsl — point-sprite star
//
// Ported from Celestia/shaders/star_{vert,frag}.glsl.
// Each star is one point sprite; its size comes from the vertex stream and the
// star-sprite texture is sampled with gl_PointCoord and tinted by the per-vertex
// color.
//
// WGSL notes:
//  * gl_PointSize  -> @builtin(point_size)  (needs the `point-size` feature)
//  * gl_PointCoord -> @builtin(point_coord) (needs the `point-size` feature)
//  * set_vp(x) == mvp_matrix * x in Celestia's "usual" projection mode.
//  * The GLSL `invariant gl_Position;` qualifier has no WGSL equivalent.

@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

@group(0) @binding(1)
var star_texture : texture_2d<f32>;

@group(0) @binding(2)
var star_sampler : sampler;

struct VertexInput {
  @location(0) position : vec3<f32>,
  @location(8) color : vec4<f32>,
  @location(7) point_size : f32,
};

struct VertexOutput {
  @builtin(position) clip_position : vec4<f32>,
  @builtin(point_size) point_size : f32,
  @interpolant(per-fragment) color : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
  var out : VertexOutput;
  out.point_size = in.point_size;
  out.color = in.color;
  out.clip_position = mvp_matrix * vec4<f32>(in.position, 1.0);
  return out;
}

@fragment
fn fragment_main(
    in : VertexOutput,
    @builtin(point_coord) point_coord : vec2<f32>,
) -> @location(0) vec4<f32> {
  return textureSample(star_texture, star_sampler, point_coord) * in.color;
}
