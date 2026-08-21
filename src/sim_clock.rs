use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct SimClock {
    /// Total elapsed simulation time in seconds.
    pub elapsed: f64,
    /// Time warp multiplier (e.g., 1.0 = real time, 100_000.0 = fast forward).
    pub time_scale: f64,
    pub paused: bool,
}

impl SimClock {
    pub const SECONDS_PER_DAY: f64 = 86_400.0;
    pub const DAYS_PER_YEAR: f64 = 365.25;
    pub const SECONDS_PER_YEAR: f64 = Self::SECONDS_PER_DAY * Self::DAYS_PER_YEAR;

    /// Elapsed days since simulation start.
    pub fn elapsed_days(&self) -> f64 {
        self.elapsed / Self::SECONDS_PER_DAY
    }

    /// Elapsed Earth years since simulation start.
    pub fn elapsed_years(&self) -> f64 {
        self.elapsed / Self::SECONDS_PER_YEAR
    }
}

impl Default for SimClock {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            time_scale: 500_000.0, // Default to a smooth warp where 1 year is ~63 seconds
            paused: false,
        }
    }
}

pub struct SimClockPlugin;

impl Plugin for SimClockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimClock>()
            .add_systems(Update, (handle_clock_input, update_clock));
    }
}

fn handle_clock_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sim_clock: ResMut<SimClock>,
) {
    // Space bar toggles pause / play
    if keyboard.just_pressed(KeyCode::Space) {
        sim_clock.paused = !sim_clock.paused;
    }

    // Time warp adjustments
    // Slow down: Left Bracket '[' or Minus '-'
    if keyboard.just_pressed(KeyCode::BracketLeft) || keyboard.just_pressed(KeyCode::Minus) {
        sim_clock.time_scale = (sim_clock.time_scale / 2.0).max(1.0);
    }
    // Speed up: Right Bracket ']' or Equal '='
    if keyboard.just_pressed(KeyCode::BracketRight) || keyboard.just_pressed(KeyCode::Equal) {
        sim_clock.time_scale = (sim_clock.time_scale * 2.0).min(50_000_000.0);
    }

    // Speed presets
    if keyboard.just_pressed(KeyCode::KeyZ) {
        sim_clock.time_scale = 1.0; // 1x Real-time
    } else if keyboard.just_pressed(KeyCode::KeyX) {
        sim_clock.time_scale = 10_000.0; // Fast
    } else if keyboard.just_pressed(KeyCode::KeyC) {
        sim_clock.time_scale = 500_000.0; // Normal orbital warp
    } else if keyboard.just_pressed(KeyCode::KeyV) {
        sim_clock.time_scale = 2_500_000.0; // High speed
    } else if keyboard.just_pressed(KeyCode::KeyB) {
        sim_clock.time_scale = 10_000_000.0; // Ultra warp
    }
}

fn update_clock(time: Res<Time<Virtual>>, mut sim_clock: ResMut<SimClock>) {
    if !sim_clock.paused {
        let delta_f64 = time.delta_secs_f64();
        sim_clock.elapsed += delta_f64 * sim_clock.time_scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_clock_defaults_and_conversion() {
        let mut clock = SimClock::default();
        assert!(!clock.paused);
        assert_eq!(clock.elapsed, 0.0);

        clock.elapsed = 86_400.0 * 365.25;
        assert!((clock.elapsed_days() - 365.25).abs() < 1e-4);
        assert!((clock.elapsed_years() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_sim_clock_pause_toggle() {
        let mut clock = SimClock::default();
        assert!(!clock.paused);
        clock.paused = !clock.paused;
        assert!(clock.paused);
        clock.paused = !clock.paused;
        assert!(!clock.paused);
    }
}