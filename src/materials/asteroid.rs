use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/asteroid/asteroid.wgsl";

#[derive(Debug, Clone)]
pub struct AsteroidMaterialConfig {
    pub rock_color: Color,
    pub roughness: f32,
}

impl Default for AsteroidMaterialConfig {
    fn default() -> Self {
        Self {
            rock_color: Color::srgb(0.48, 0.44, 0.40),
            roughness: 0.95,
        }
    }
}

pub fn create_asteroid_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<AsteroidMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.rock_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
