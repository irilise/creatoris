use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/earth/earth.wgsl";

#[derive(Debug, Clone)]
pub struct EarthMaterialConfig {
    pub ocean_color: Color,
    pub roughness: f32,
    pub metallic: f32,
}

impl Default for EarthMaterialConfig {
    fn default() -> Self {
        Self {
            ocean_color: Color::srgb(0.18, 0.46, 0.88),
            roughness: 0.6,
            metallic: 0.1,
        }
    }
}

pub fn create_earth_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<EarthMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.ocean_color,
        perceptual_roughness: cfg.roughness,
        metallic: cfg.metallic,
        reflectance: 0.5,
        ..default()
    })
}
