use bevy::prelude::*;

#[allow(dead_code)]
pub const SHADER_PATH: &str = "shaders/sun/sun.wgsl";

#[derive(Debug, Clone)]
pub struct SunMaterialConfig {
    pub base_color: Color,
    pub emissive_color: LinearRgba,
}

impl Default for SunMaterialConfig {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(1.0, 0.92, 0.3),
            emissive_color: LinearRgba::rgb(4.0, 3.6, 0.8),
        }
    }
}

pub fn create_sun_material(
    materials: &mut Assets<StandardMaterial>,
    config: Option<SunMaterialConfig>,
) -> Handle<StandardMaterial> {
    let cfg = config.unwrap_or_default();
    materials.add(StandardMaterial {
        base_color: cfg.base_color,
        emissive: cfg.emissive_color,
        unlit: true,
        ..default()
    })
}
