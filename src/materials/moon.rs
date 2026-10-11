use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/moon/moon.wgsl";

#[derive(Debug, Clone)]
pub struct MoonMaterialConfig {
    pub surface_color: Color,
    pub roughness: f32,
}

impl Default for MoonMaterialConfig {
    fn default() -> Self {
        Self {
            surface_color: Color::srgb(0.80, 0.80, 0.82),
            roughness: 0.95,
        }
    }
}

pub fn create_moon_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<MoonMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.surface_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
