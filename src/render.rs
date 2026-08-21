use crate::coordinates::FloatingOrigin;
use crate::orbital_math::OrbitalElements;
use bevy::prelude::*;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_orbital_gizmos);
    }
}

fn draw_orbital_gizmos(
    mut gizmos: Gizmos,
    origin: Res<FloatingOrigin>,
    query: Query<&OrbitalElements>,
) {
    let segments = 64; // Higher count = smoother ellipse
    
    for elements in query.iter() {
        let period = std::f64::consts::TAU / elements.mean_motion;
        let dt = period / segments as f64;
        
        let mut previous_pt = None;

        for i in 0..=segments {
            let t = i as f64 * dt;
            let d_pos = elements.position_at_time(t);
            
            // Translate to rendering space
            let local_pos = (d_pos - origin.center).as_vec3();
            
            if let Some(prev) = previous_pt {
                // Draw path segment
                gizmos.line(prev, local_pos, Color::srgb(0.3, 0.3, 0.3));
            }
            previous_pt = Some(local_pos);
        }
    }
}