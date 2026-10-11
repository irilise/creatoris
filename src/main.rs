use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use std::path::PathBuf;

mod bodies;
mod camera;
mod cargo;
mod celestia;
mod celestia_bridge;
mod coordinates;
mod materials;
mod orbital_math;
mod render;
mod sim_clock;
mod ui;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Point the asset server at the project's `assets/` directory using an
                // absolute path so shaders load regardless of the working directory
                // the binary is launched from (e.g. `target/debug/assets/shaders/...`
                // would otherwise be missing).
                .set(AssetPlugin {
                    file_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("assets")
                        .to_string_lossy()
                        .into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Creatoris // Solar System Simulation".into(),
                        ..default()
                    }),
                    ..default()
                })
        )
        .add_plugins(sim_clock::SimClockPlugin)
        .add_plugins(coordinates::CoordinatesPlugin)
        .add_plugins(orbital_math::OrbitalMathPlugin)
        // Celestia port: framework-free engine core + Bevy bridge. The bridge
        // loads the embedded solar-system catalog and computes body positions
        // at the current simulation time.
        .add_plugins(celestia_bridge::CelestiaBridgePlugin)
        // Materials & Shaders
        .add_plugins(materials::MaterialsPlugin)
        // Gameplay & Entities
        .add_plugins(bodies::BodiesPlugin)
        .add_plugins(cargo::CargoPlugin)
        // Visuals, UI & Input
        .add_plugins(camera::CameraPlugin)
        .add_plugins(render::RenderPlugin)
        .add_plugins(ui::UiPlugin)
        .run();
}
