//! Bevy bridge for the Celestia engine.
//!
//! This module connects the framework-free [`celestia`] simulation core to the
//! Bevy ECS. It exposes the parsed [`Universe`] as a Bevy resource and provides
//! a system that evaluates body positions at the current simulation time.
//!
//! The bridge keeps the two layers decoupled: the engine does all the
//! astronomy (Kepler propagation, frame math, data parsing); the bridge only
//! transfers results into Bevy-usable components.
#![allow(missing_docs)]

use bevy::prelude::*;

use crate::celestia::engine::Universe;
use crate::celestia::{astro, build_embedded_solar_system};

/// The full Celestia universe, held as a Bevy resource so any system can read
/// the parsed catalog and query body positions.
#[derive(Resource, Debug, Default)]
pub struct CelestiaUniverse {
    pub universe: Universe,
}

impl CelestiaUniverse {
    /// Build the universe from the embedded classical solar-system catalog.
    pub fn from_embedded_catalog() -> Self {
        Self {
            universe: build_embedded_solar_system(),
        }
    }
}

/// A body's position in the interstellar frame at the current simulation time,
/// in kilometers. Produced by [`compute_body_positions`] for inspection/rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct BodyJ2000Position {
    /// Position in kilometers, relative to the solar-system barycenter.
    pub position_km: glam::DVec3,
}

pub struct CelestiaBridgePlugin;

impl Plugin for CelestiaBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CelestiaUniverse>()
            .init_resource::<BodyPositions>()
            .add_systems(Startup, load_universe)
            .add_systems(Update, compute_body_positions);
    }
}

/// The computed positions of every body in the universe at the current time.
#[derive(Resource, Debug, Default)]
pub struct BodyPositions {
    /// Map of body name -> position in kilometers.
    pub positions: std::collections::HashMap<String, glam::DVec3>,
    /// Julian date (TDB) at which the positions were computed.
    pub julian_date: f64,
}

fn load_universe(mut universe: ResMut<CelestiaUniverse>) {
    universe.universe = build_embedded_solar_system();
    log::info!(
        "Loaded Celestia solar system: {} body(s) in the 'Sol' system",
        universe
            .universe
            .get_solar_system("Sol")
            .map(|s| s.planetary_system.bodies.len())
            .unwrap_or(0)
    );
}

/// Evaluate every body's position at the current simulation time and store the
/// results in [`BodyPositions`].
///
/// The simulation clock is measured in seconds of elapsed time since the
/// simulation start; Celestia's engine works in Julian dates, so we convert
/// with `tdb = J2000 + elapsed_days`.
fn compute_body_positions(
    clock: Res<crate::sim_clock::SimClock>,
    universe: Res<CelestiaUniverse>,
    mut positions: ResMut<BodyPositions>,
) {
    let tdb = astro::J2000 + clock.elapsed / astro::SECONDS_PER_DAY;
    positions.julian_date = tdb;
    positions.positions.clear();

    if let Some(sol) = universe.universe.get_solar_system("Sol") {
        for body in &sol.planetary_system.bodies {
            if let Some(p) = sol.body_position_at_time(&body.name, tdb) {
                positions.positions.insert(body.name.clone(), p);
            }
        }
    }
}

impl BodyPositions {
    /// Position of a body in kilometers, if known.
    pub fn get(&self, name: &str) -> Option<glam::DVec3> {
        self.positions.get(name).copied()
    }
}

impl BodyJ2000Position {
    /// Create a position from a kilometer vector.
    pub fn new(position_km: glam::DVec3) -> Self {
        Self { position_km }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_builds_universe() {
        let u = CelestiaUniverse::from_embedded_catalog();
        assert_eq!(u.universe.count(), 1);
        assert!(u.universe.get_solar_system("Sol").is_some());
    }

    #[test]
    fn positions_computed_at_j2000() {
        let clock = crate::sim_clock::SimClock::default();
        let universe = CelestiaUniverse::from_embedded_catalog();
        let mut positions = BodyPositions::default();
        // Simulate the system by calling the function's logic directly.
        let tdb = astro::J2000 + clock.elapsed / astro::SECONDS_PER_DAY;
        positions.julian_date = tdb;
        if let Some(sol) = universe.universe.get_solar_system("Sol") {
            for body in &sol.planetary_system.bodies {
                if let Some(p) = sol.body_position_at_time(&body.name, tdb) {
                    positions.positions.insert(body.name.clone(), p);
                }
            }
        }
        let earth = positions.get("Earth").unwrap();
        assert!((earth.length() - astro::KM_PER_AU).abs() < 5.0e6);
    }
}
