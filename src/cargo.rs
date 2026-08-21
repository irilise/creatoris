use crate::sim_clock::SimClock;
use bevy::math::DVec3;
use bevy::prelude::*;

/// A lightweight, flat data structure for ungudided mass driver packets.
/// Does not need Transform or Mesh unless explicitly rendered.
#[derive(Component)]
#[allow(dead_code)]
pub struct CargoPacket {
    pub origin_entity: Entity,
    pub destination_entity: Entity,
    pub launch_epoch: f64,
    pub flight_duration: f64,
    
    // Orbital transfer parameters (e.g. Lambert problem solution vectors)
    pub transfer_start_pos: DVec3,
    pub transfer_end_pos: DVec3,
}

pub struct CargoPlugin;

impl Plugin for CargoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, simulate_cargo_streams);
    }
}

fn simulate_cargo_streams(
    clock: Res<SimClock>,
    mut cargo_query: Query<(Entity, &CargoPacket)>,
    mut commands: Commands,
) {
    // Iterate fast and flat. You can upgrade this to par_iter() as volume grows.
    for (entity, packet) in cargo_query.iter_mut() {
        let t_flight = clock.elapsed - packet.launch_epoch;
        
        if t_flight >= packet.flight_duration {
            // Reached destination, trigger delivery event / economy update
            commands.entity(entity).despawn();
        } else {
            // Evaluate intermediate position on transfer conic section
            // let current_pos = evaluate_transfer_orbit(...);
        }
    }
}