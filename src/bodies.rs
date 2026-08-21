use crate::coordinates::UniversalPosition;
use crate::orbital_math::OrbitalElements;
use crate::sim_clock::SimClock;
use bevy::prelude::*;

#[derive(Component)]
pub struct CelestialBody {
    pub name: String,
    pub mass: f64,
    pub radius: f32,
}

pub struct BodiesPlugin;

impl Plugin for BodiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_solar_system)
            .add_systems(Update, propagate_orbits);
    }
}

fn spawn_solar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun
    commands.spawn((
        CelestialBody { name: "Sun".into(), mass: 1.989e30, radius: 696.0 },
        UniversalPosition(bevy::math::DVec3::ZERO),
        Mesh3d(meshes.add(Sphere::new(50.0))), // Scaled for visibility
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.9, 0.1),
            emissive: LinearRgba::rgb(2.0, 1.8, 0.2), // Glow
            ..default()
        })),
        Transform::default(),
    ));

    // Earth (mock elements scaled to fit a local viewport for demonstration)
    commands.spawn((
        CelestialBody { name: "Earth".into(), mass: 5.972e24, radius: 6.371 },
        OrbitalElements {
            semi_major_axis: 15_000.0, // Mock scaled distance
            eccentricity: 0.0167,
            inclination: 0.0,
            long_asc_node: 0.0,
            arg_periapsis: 1.99,
            mean_anomaly_0: 0.0,
            mean_motion: 0.0001, // Arbitrary speed
        },
        UniversalPosition(bevy::math::DVec3::ZERO),
        Mesh3d(meshes.add(Sphere::new(10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.3, 0.9))),
        Transform::default(),
    ));
}

fn propagate_orbits(
    clock: Res<SimClock>,
    mut query: Query<(&OrbitalElements, &mut UniversalPosition)>,
) {
    for (elements, mut pos) in query.iter_mut() {
        pos.0 = elements.position_at_time(clock.elapsed);
    }
}