use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/neptune/neptune.wgsl";

#[derive(Debug, Clone)]
pub struct NeptuneMaterialConfig {
    pub azure_color: Color,
    pub roughness: f32,
}

impl Default for NeptuneMaterialConfig {
    fn default() -> Self {
        Self {
            azure_color: Color::srgb(0.20, 0.42, 0.92),
            roughness: 0.55,
        }
    }
}

pub fn create_neptune_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<NeptuneMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.azure_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
