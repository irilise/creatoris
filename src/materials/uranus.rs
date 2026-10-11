use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/uranus/uranus.wgsl";

#[derive(Debug, Clone)]
pub struct UranusMaterialConfig {
    pub ice_color: Color,
    pub roughness: f32,
}

impl Default for UranusMaterialConfig {
    fn default() -> Self {
        Self {
            ice_color: Color::srgb(0.48, 0.82, 0.86),
            roughness: 0.6,
        }
    }
}

pub fn create_uranus_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<UranusMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.ice_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
