use bevy::math::DVec3;
use bevy::prelude::*;

/// Absolute double-precision position in the solar system.
#[derive(Component, Debug, Clone, Copy)]
pub struct UniversalPosition(pub DVec3);

/// Dictates the anchor point for the f32 rendering space.
#[derive(Resource, Debug, Default)]
pub struct FloatingOrigin {
    pub center: DVec3,
    pub focus_entity: Option<Entity>,
}

pub struct CoordinatesPlugin;

impl Plugin for CoordinatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FloatingOrigin>()
            // Runs after orbits are calculated but before rendering
            .add_systems(PostUpdate, update_transforms_from_origin);
    }
}

fn update_transforms_from_origin(
    mut origin: ResMut<FloatingOrigin>,
    query_positions: Query<(Entity, &UniversalPosition)>,
    mut query_transforms: Query<&mut Transform>,
) {
    // 1. Update the origin center if tracking a specific entity
    if let Some(focus) = origin.focus_entity {
        if let Ok((_, pos)) = query_positions.get(focus) {
            origin.center = pos.0;
        }
    }

    // 2. Translate all universal coordinates to local f32 Transforms
    for (entity, pos) in query_positions.iter() {
        if let Ok(mut transform) = query_transforms.get_mut(entity) {
            let local_pos = pos.0 - origin.center;
            // Downcast to f32 for Bevy rendering.
            transform.translation = local_pos.as_vec3();
        }
    }
}