use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/venus/venus.wgsl";

#[derive(Debug, Clone)]
pub struct VenusMaterialConfig {
    pub cloud_color: Color,
    pub roughness: f32,
}

impl Default for VenusMaterialConfig {
    fn default() -> Self {
        Self {
            cloud_color: Color::srgb(0.92, 0.80, 0.52),
            roughness: 0.7,
        }
    }
}

pub fn create_venus_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<VenusMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.cloud_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
