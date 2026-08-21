use bevy::prelude::*;

mod bodies;
mod camera;
mod cargo;
mod coordinates;
mod orbital_math;
mod render;
mod sim_clock;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creatoris // Solar System Simulation".into(),
                ..default()
            }),
            ..default()
        }))
        // Core Simulation
        .add_plugins(sim_clock::SimClockPlugin)
        .add_plugins(coordinates::CoordinatesPlugin)
        .add_plugins(orbital_math::OrbitalMathPlugin)
        // Gameplay & Entities
        .add_plugins(bodies::BodiesPlugin)
        .add_plugins(cargo::CargoPlugin)
        // Visuals, UI & Input
        .add_plugins(camera::CameraPlugin)
        .add_plugins(render::RenderPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}