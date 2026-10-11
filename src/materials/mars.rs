use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/mars/mars.wgsl";

#[derive(Debug, Clone)]
pub struct MarsMaterialConfig {
    pub rust_color: Color,
    pub roughness: f32,
}

impl Default for MarsMaterialConfig {
    fn default() -> Self {
        Self {
            rust_color: Color::srgb(0.86, 0.38, 0.22),
            roughness: 0.85,
        }
    }
}

pub fn create_mars_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<MarsMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.rust_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
