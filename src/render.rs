use crate::bodies::{CelestialBody, CelestialType, OrbitParent, OrbitPathColor};
use crate::coordinates::{FloatingOrigin, UniversalPosition};
use crate::orbital_math::OrbitalElements;
use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct RenderSettings {
    pub show_orbits: bool,
    pub show_asteroid_orbits: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            show_orbits: true,
            show_asteroid_orbits: false, // Keep asteroid belts clean by default
        }
    }
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderSettings>()
            .insert_resource(ClearColor(Color::srgb(0.008, 0.008, 0.02)))
            .add_systems(Update, handle_render_input)
            .add_systems(PostUpdate, draw_orbital_gizmos);
    }
}

fn handle_render_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<RenderSettings>,
) {
    if keyboard.just_pressed(KeyCode::KeyO) {
        settings.show_orbits = !settings.show_orbits;
    }
    if keyboard.just_pressed(KeyCode::KeyI) {
        settings.show_asteroid_orbits = !settings.show_asteroid_orbits;
    }
}

fn draw_orbital_gizmos(
    mut gizmos: Gizmos,
    origin: Res<FloatingOrigin>,
    settings: Res<RenderSettings>,
    query_parents: Query<&UniversalPosition, Without<OrbitalElements>>,
    query: Query<(
        &OrbitalElements,
        Option<&CelestialBody>,
        Option<&OrbitPathColor>,
        Option<&OrbitParent>,
    )>,
) {
    if !settings.show_orbits {
        return;
    }

    let default_planet_color = Color::srgba(0.35, 0.5, 0.75, 0.7);

    for (elements, body, path_color, parent) in query.iter() {
        // Skip asteroid orbits unless explicitly toggled on to keep scene uncluttered
        let is_asteroid = body
            .map(|b| b.celestial_type == CelestialType::Asteroid)
            .unwrap_or(false);
        if is_asteroid && !settings.show_asteroid_orbits {
            continue;
        }

        let segments = if is_asteroid { 32 } else { 128 };
        let period = std::f64::consts::TAU / elements.mean_motion;
        let dt = period / segments as f64;

        // Base center offset in rendering space
        let center_offset = if let Some(parent_ent) = parent {
            if let Ok(parent_pos) = query_parents.get(parent_ent.0) {
                (parent_pos.0 - origin.center).as_vec3()
            } else {
                -origin.center.as_vec3()
            }
        } else {
            -origin.center.as_vec3()
        };

        let color = path_color
            .map(|c| c.0)
            .unwrap_or(default_planet_color);

        let mut previous_pt = None;

        for i in 0..=segments {
            let t = i as f64 * dt;
            let rel_dpos = elements.position_at_time(t);
            let local_pos = center_offset + rel_dpos.as_vec3();

            if let Some(prev) = previous_pt {
                gizmos.line(prev, local_pos, color);
            }
            previous_pt = Some(local_pos);
        }
    }
}