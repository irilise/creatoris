use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/saturn/saturn.wgsl";
#[allow(dead_code)]
pub const RINGS_SHADER_PATH: &str = "shaders/saturn/saturn_rings.wgsl";

#[derive(Debug, Clone)]
pub struct SaturnMaterialConfig {
    pub body_color: Color,
    pub ring_color: Color,
    pub roughness: f32,
}

impl Default for SaturnMaterialConfig {
    fn default() -> Self {
        Self {
            body_color: Color::srgb(0.90, 0.82, 0.58),
            ring_color: Color::srgba(0.88, 0.80, 0.60, 0.85),
            roughness: 0.7,
        }
    }
}

pub fn create_saturn_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<SaturnMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.body_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}

pub fn create_saturn_rings_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<SaturnMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.ring_color,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.8,
        cull_mode: None,
        ..default()
    })
}
