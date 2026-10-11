use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/mercury/mercury.wgsl";

#[derive(Debug, Clone)]
pub struct MercuryMaterialConfig {
    pub base_color: Color,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for MercuryMaterialConfig {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(0.72, 0.70, 0.68),
            roughness: 0.9,
            metallic: 0.05,
        }
    }
}

pub fn create_mercury_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<MercuryMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.base_color,
        perceptual_roughness: cfg.roughness,
        metallic: cfg.metallic,
        ..default()
    })
}
