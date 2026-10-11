use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/jupiter/jupiter.wgsl";

#[derive(Debug, Clone)]
pub struct JupiterMaterialConfig {
    pub cloud_band_color: Color,
    pub roughness: f32,
}

impl Default for JupiterMaterialConfig {
    fn default() -> Self {
        Self {
            cloud_band_color: Color::srgb(0.85, 0.68, 0.48),
            roughness: 0.65,
        }
    }
}

pub fn create_jupiter_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<JupiterMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.cloud_band_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
