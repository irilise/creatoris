// line.wgsl — line-as-triangles anti-aliased line shader (LineAsTriangles).
//
// Ported from the LineVertexPosition / LineFragmentDeclaration fragments of
// Celestia/src/celengine/shadermanager.cpp. A line is drawn as two triangles;
// the fragment shader anti-aliases the edge analytically with fwidth(lineU).
//
// This file is the line-only geometry + AA; lighting/texture is supplied by the
// surrounding body shader it is combined with. WGSL notes: see ../README.md.

// ----- shared matrices (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

// ----- line uniforms -----
@group(0) @binding(1)
var<uniform> line_width_x : f32>;
@group(0) @binding(2)
var<uniform> line_width_y : f32>;
@group(0) @binding(3)
var<uniform> line_edge : f32>;

struct VertexInput {
    @location(0) position : vec4<f32>,
    @location(16) position_next : vec4<f32>,
    @location(17) scale_factor : f32>,
    @location(18) line_side : f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @interpolant(per-fragment) line_u : f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;

    // LineVertexPosition: inflate the line by one pixel on each side and pass
    // a perpendicular distance (lineU) to the fragment stage.
    let this_pos = calc_vp(in.position);
    let next_pos = calc_vp(in.position_next);
    let this_n = this_pos.xy / this_pos.w;
    let next_n = next_pos.xy / next_pos.w;
    let transform = normalize(next_n - this_n);
    let perp = vec2<f32>(transform.y * line_width_x, -transform.x * line_width_y)
               * in.scale_factor;
    out.clip_position = vec4<f32>((this_n + perp) * this_pos.w, this_pos.zw);
    out.line_u = in.line_side;
    return out;
}

struct FragmentInput {
    @interpolant(per-fragment) line_u : f32>,
};

@fragment
fn fragment_main(in : FragmentInput) -> @location(0) vec4<f32> {
    // Analytical edge anti-aliasing. Geometry is inflated 1px past the visible
    // edge; feather is 1px wide centered on the edge.
    let line_half_px = fwidth(in.line_u) * 0.5;
    let edge = 1.0 - smoothstep(line_edge - line_half_px, line_edge + line_half_px,
                                abs(in.line_u));
    return vec4<f32>(0.0, 0.0, 0.0, edge);
}
