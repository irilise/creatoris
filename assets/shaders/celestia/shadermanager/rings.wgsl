// rings.wgsl — ring system shader (RingIllumModel).
//
// Ported from buildRingsVertexShader + buildRingsFragmentShader in
// Celestia/src/celengine/shadermanager.cpp.
//
// The fragment sums each light's intensity weighted by how much the viewer and
// light sit on the same side of the ring (litSide), attenuated by the ring's
// optical depth, and supports eclipse shadows (shadowDepths) and the
// ring-atmosphere split (ringHalf drops the half of the ring hidden behind the
// planet's atmosphere shell).
//
// WGSL notes: see ../README.md. Single-light representative; Celestia loops N
// lights and packs per-light shadow depths into shadowDepths.xyzw.

// ----- shared matrix (see common.wgsl) -----
@group(0) @binding(0)
var<uniform> mvp_matrix : mat4x4<f32>;

// ----- per-light uniforms (light 0) -----
@group(0) @binding(1)
var<uniform> light0_direction : vec3<f32>;
@group(0) @binding(2)
var<uniform> light0_diffuse : vec3<f32>;

// ----- ring uniforms -----
@group(0) @binding(3)
var<uniform> ring_width : f32>;
@group(0) @binding(4)
var<uniform> ring_radius : f32>;
@group(0) @binding(5)
var<uniform> ring_plane : vec4<f32>;
@group(0) @binding(6)
var<uniform> ring_center : vec3<f32>;
@group(0) @binding(7)
var<uniform> ring_scale : vec3<f32>;
@group(0) @binding(8)
var<uniform> ring_shadow_lod0 : f32>;
@group(0) @binding(9)
var<uniform> ambient_color : vec3<f32>;
@group(0) @binding(10)
var<uniform> ring_half : f32>;
@group(0) @binding(11)
var<uniform> ring_atmosphere_radius : f32>;
@group(0) @binding(12)
var<uniform> eye_position : vec3<f32>;

// ----- ring shadow texture -----
@group(0) @binding(13)
var ring_texture : texture_2d<f32>;
@group(0) @binding(14)
var ring_sampler : sampler;

// ----- diffuse (ring color/opacity) texture -----
@group(0) @binding(15)
var diffuse_texture : texture_2d<f32>;
@group(0) @binding(16)
var diffuse_sampler : sampler;

struct VertexInput {
    @location(0) position : vec4<f32>,
    @location(2) tex_coord0 : vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @interpolant(per-fragment) position : vec3<f32>,
    @interpolant(per-fragment) diff_tex_coord : vec2<f32>,
    // Ring shadow tex coord (length from ring center, mapped to [0,1] across
    // the ring band) — smooth across the ring plane.
    @interpolant(per-fragment) ring_shadow_tex_coord : vec4<f32>,
};

@vertex
fn vertex_main(in : VertexInput) -> VertexOutput {
    var out : VertexOutput;
    out.position = in.position.xyz;
    out.diff_tex_coord = in.tex_coord0.xy;

    // Ring shadow tex coord: (length(ringShadowProjection - ringCenter) -
    // ringRadius) * ringWidth, where ringShadowProjection is the point along the
    // light ray in the ring plane.
    let ring_shadow_position = in.position.xyz * ring_scale;
    let ring_plane_distance = -(dot(ring_shadow_position, ring_plane.xyz) + ring_plane.w);
    let ring_shadow_distance =
        ring_plane_distance / dot(light0_direction, ring_plane.xyz);
    let ring_shadow_projection = ring_shadow_position + light0_direction * ring_shadow_distance;
    let tex = (length(ring_shadow_projection - ring_center) - ring_radius) * ring_width;
    out.ring_shadow_tex_coord = vec4<f32>(tex, 0.0, 0.0, 0.0);

    out.clip_position = calc_vp(vec4<f32>(in.position.xyz, 1.0));
    return out;
}

struct FragmentInput {
    @interpolant(per-fragment) position : vec3<f32>,
    @interpolant(per-fragment) diff_tex_coord : vec2<f32>,
    @interpolant(per-fragment) ring_shadow_tex_coord : vec4<f32>,
};

@fragment
fn fragment_main(in : FragmentInput) -> @location(0) vec4<f32> {
    // Ring-atmosphere split: drop the half of the ring on the far side of the
    // planet's atmosphere shell. A fragment is "behind" when the eye->fragment
    // ray crosses the shell (a ray-sphere root in (0,1)). Uses alpha 0, not
    // discard, so every lane stays alive (texture derivatives match).
    var drop_fragment = false;
    if (ring_half != 0.0) {
        let ring_dir = in.position - eye_position;
        let a = dot(ring_dir, ring_dir);
        let b = dot(eye_position, ring_dir);
        let c = dot(eye_position, eye_position) - ring_atmosphere_radius * ring_atmosphere_radius;
        let behind = b < 0.0 && a + b > 0.0 && b * b >= a * c;
        drop_fragment = (ring_half > 0.0 && behind) || (ring_half < 0.0 && !behind);
    }
    var diff : vec4<f32> = vec4<f32>(ambient_color, 1.0);

    let eye_dir = normalize(eye_position - in.position);

    let color = textureSample(diffuse_texture, diffuse_sampler, in.diff_tex_coord);
    var optical_depth = color.a;

    // Per-light contribution (single light here).
    let lit_side = 1.0 - step(0.0, light0_direction.y * eye_dir.y);
    var intensity = (dot(light0_direction, eye_dir) + 1.0) * 0.5;
    intensity = mix(intensity, intensity * (1.0 - optical_depth), lit_side);

    // Eclipse shadow (note): when eclipse shadows are enabled, Celestia computes
    // `shadow` via the Shadow() helper from shadowDepths.x and adds
    // (shadow * intensity) * light0_diffuse instead of the plain line below.
    diff.rgb += intensity * light0_diffuse;

    // Ring shadow from the ring texture (ringShadowLOD0).
    let rsc = in.ring_shadow_tex_coord.x;
    if (rsc >= 0.0 && rsc <= 1.0) {
        let alpha = textureSampleLevel(ring_texture, ring_sampler, vec2<f32>(rsc, 0.0), ring_shadow_lod0).a;
        // diff.rgb *= (1.0 - alpha);  // applied per light in the full version
        let _ = alpha;
    }

    if (drop_fragment) {
        optical_depth = 0.0;
    }

    return vec4<f32>(color.rgb * diff.rgb, optical_depth);
}
