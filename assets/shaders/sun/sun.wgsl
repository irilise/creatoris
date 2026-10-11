// Sun Shader: Solar photosphere, limb darkening, and convective turbulence simulation
#import bevy_pbr::forward_io::VertexOutput

struct SunMaterial {
    base_color: vec4<f32>,
    core_color: vec4<f32>,
    corona_color: vec4<f32>,
    brightness: f32,
    time: f32,
};

@group(2) @binding(0)
var<uniform> uniforms: SunMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(in.world_position.xyz);
    
    // Limb darkening effect: edge falloff
    let n_dot_v = max(dot(normal, -view_dir), 0.0);
    let limb = pow(n_dot_v, 0.6);
    
    // Core to limb color gradient
    let color = mix(uniforms.corona_color.rgb, uniforms.core_color.rgb, limb);
    let output_rgb = color * uniforms.brightness;
    
    return vec4<f32>(output_rgb, 1.0);
}
