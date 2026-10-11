use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/pluto/pluto.wgsl";

#[derive(Debug, Clone)]
pub struct PlutoMaterialConfig {
    pub surface_color: Color,
    pub roughness: f32,
}

impl Default for PlutoMaterialConfig {
    fn default() -> Self {
        Self {
            surface_color: Color::srgb(0.76, 0.66, 0.58),
            roughness: 0.9,
        }
    }
}

pub fn create_pluto_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<PlutoMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.surface_color,
        perceptual_roughness: cfg.roughness,
        ..default()
    })
}
