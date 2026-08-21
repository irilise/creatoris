use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct SimClock {
    /// Total elapsed simulation time in seconds.
    pub elapsed: f64,
    /// Time warp multiplier (e.g., 1.0 = real time, 100_000.0 = fast forward).
    pub time_scale: f64,
    pub paused: bool,
}

impl Default for SimClock {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            time_scale: 100_000.0, // Default to high warp to see orbital motion
            paused: false,
        }
    }
}

pub struct SimClockPlugin;

impl Plugin for SimClockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimClock>()
            .add_systems(Update, update_clock);
    }
}

fn update_clock(time: Res<Time<Virtual>>, mut sim_clock: ResMut<SimClock>) {
    if !sim_clock.paused {
        let delta_f64 = time.delta_secs_f64();
        sim_clock.elapsed += delta_f64 * sim_clock.time_scale;
    }
}