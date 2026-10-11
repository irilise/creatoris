use crate::coordinates::UniversalPosition;
use crate::materials::asteroid::create_asteroid_material;
use crate::materials::earth::create_earth_material;
use crate::materials::jupiter::create_jupiter_material;
use crate::materials::mars::create_mars_material;
use crate::materials::mercury::create_mercury_material;
use crate::materials::moon::create_moon_material;
use crate::materials::neptune::create_neptune_material;
use crate::materials::pluto::create_pluto_material;
use crate::materials::saturn::{create_saturn_material, create_saturn_rings_material};
use crate::materials::sun::create_sun_material;
use crate::materials::uranus::create_uranus_material;
use crate::materials::venus::create_venus_material;
use crate::orbital_math::OrbitalElements;
use crate::sim_clock::SimClock;
use bevy::math::DVec3;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CelestialType {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
}

#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct CelestialBody {
    pub name: String,
    pub celestial_type: CelestialType,
    pub mass: f64,    // kg
    pub radius: f32,  // km
    pub display_radius: f32, // visual rendering size
    pub color: Color,
    pub description: String,
}

/// Indicates that this body's orbit is relative to another entity (e.g., Moon around Earth).
#[derive(Component, Debug, Clone, Copy)]
pub struct OrbitParent(pub Entity);

/// Visual color for the orbital path gizmo.
#[derive(Component, Debug, Clone, Copy)]
pub struct OrbitPathColor(pub Color);

/// Marker for Saturn's rings.
#[derive(Component)]
pub struct SaturnRingVisual;

/// Resource tracking primary selectable celestial targets for the camera.
#[derive(Resource, Debug, Default)]
pub struct SolarSystemDirectory {
    pub targets: Vec<(Entity, String)>,
    pub current_target_index: usize,
}

impl SolarSystemDirectory {
    pub fn current_entity(&self) -> Option<Entity> {
        self.targets.get(self.current_target_index).map(|(e, _)| *e)
    }

    #[allow(dead_code)]
    pub fn current_name(&self) -> &str {
        self.targets
            .get(self.current_target_index)
            .map(|(_, name)| name.as_str())
            .unwrap_or("Unknown")
    }

    pub fn set_target_by_index(&mut self, index: usize) -> Option<Entity> {
        if index < self.targets.len() {
            self.current_target_index = index;
            Some(self.targets[index].0)
        } else {
            None
        }
    }

    pub fn cycle_next(&mut self) -> Option<Entity> {
        if self.targets.is_empty() {
            return None;
        }
        self.current_target_index = (self.current_target_index + 1) % self.targets.len();
        Some(self.targets[self.current_target_index].0)
    }

    pub fn cycle_prev(&mut self) -> Option<Entity> {
        if self.targets.is_empty() {
            return None;
        }
        if self.current_target_index == 0 {
            self.current_target_index = self.targets.len() - 1;
        } else {
            self.current_target_index -= 1;
        }
        Some(self.targets[self.current_target_index].0)
    }
}

pub struct BodiesPlugin;

impl Plugin for BodiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SolarSystemDirectory>()
            .add_systems(Startup, spawn_solar_system)
            .add_systems(Update, (propagate_orbits, update_saturn_rings));
    }
}

fn spawn_solar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut directory: ResMut<SolarSystemDirectory>,
) {
    let mut targets = Vec::new();

    // 0. Sun
    // Rendered via the ported Celestia star material (unlit + emissive
    // StandardMaterial), which stays reliably visible over a near-black clear color.
    let sun_mat = create_sun_material(&mut standard_materials, None);
    let sun_entity = commands
        .spawn((
            CelestialBody {
                name: "Sun".into(),
                celestial_type: CelestialType::Star,
                mass: 1.9885e30,
                radius: 696_340.0,
                display_radius: 120.0,
                color: Color::srgb(1.0, 0.92, 0.3),
                description: "G-type main-sequence star, containing 99.86% of the Solar System's total mass.".into(),
            },
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(120.0))),
            MeshMaterial3d(sun_mat),
            Transform::default(),
        ))
        .with_children(|parent| {
            // Bright point light at Sun's center illuminating all planets
            parent.spawn((
                PointLight {
                    color: Color::srgb(1.0, 0.98, 0.92),
                    intensity: 150_000_000_000.0,
                    range: 2_000_000.0,
                    ..default()
                },
                Transform::default(),
            ));
        })
        .id();
    targets.push((sun_entity, "Sun".to_string()));

    // 1. Mercury
    let mercury_color = Color::srgb(0.72, 0.70, 0.68);
    let mercury_mat = create_mercury_material(&mut standard_materials, None);
    let mercury_entity = commands
        .spawn((
            CelestialBody {
                name: "Mercury".into(),
                celestial_type: CelestialType::Planet,
                mass: 3.3011e23,
                radius: 2_439.7,
                display_radius: 14.0,
                color: mercury_color,
                description: "Smallest planet, scorched by proximity to the Sun with a cratered rocky surface.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                0.3871, 0.2056, 7.005, 48.331, 29.124, 87.969, 174.79,
            ),
            OrbitPathColor(Color::srgb(0.65, 0.65, 0.65)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(14.0))),
            MeshMaterial3d(mercury_mat),
            Transform::default(),
        ))
        .id();
    targets.push((mercury_entity, "Mercury".to_string()));

    // 2. Venus
    let venus_color = Color::srgb(0.92, 0.80, 0.52);
    let venus_mat = create_venus_material(&mut standard_materials, None);
    let venus_entity = commands
        .spawn((
            CelestialBody {
                name: "Venus".into(),
                celestial_type: CelestialType::Planet,
                mass: 4.8675e24,
                radius: 6_051.8,
                display_radius: 22.0,
                color: venus_color,
                description: "Terrestrial planet wrapped in thick clouds of sulfuric acid with a runaway greenhouse effect.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                0.7233, 0.0067, 3.394, 76.680, 54.884, 224.701, 50.115,
            ),
            OrbitPathColor(Color::srgb(0.85, 0.75, 0.4)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(22.0))),
            MeshMaterial3d(venus_mat),
            Transform::default(),
        ))
        .id();
    targets.push((venus_entity, "Venus".to_string()));

    // 3. Earth
    let earth_color = Color::srgb(0.18, 0.46, 0.88);
    let earth_mat = create_earth_material(&mut standard_materials, None);
    let earth_entity = commands
        .spawn((
            CelestialBody {
                name: "Earth".into(),
                celestial_type: CelestialType::Planet,
                mass: 5.9724e24,
                radius: 6_371.0,
                display_radius: 24.0,
                color: earth_color,
                description: "Our vibrant blue home world with abundant liquid oceans, nitrogen-oxygen atmosphere, and life.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                1.0000, 0.0167, 0.000, 0.0, 102.947, 365.256, 358.617,
            ),
            OrbitPathColor(Color::srgb(0.2, 0.55, 0.95)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(24.0))),
            MeshMaterial3d(earth_mat),
            Transform::default(),
        ))
        .id();
    targets.push((earth_entity, "Earth".to_string()));

    // 4. Moon (Earth's Moon)
    let moon_color = Color::srgb(0.80, 0.80, 0.82);
    let moon_mat = create_moon_material(&mut standard_materials, None);
    let moon_entity = commands
        .spawn((
            CelestialBody {
                name: "Moon".into(),
                celestial_type: CelestialType::Moon,
                mass: 7.342e22,
                radius: 1_737.4,
                display_radius: 8.0,
                color: moon_color,
                description: "Earth's only natural satellite, tidally locked and stabilizing Earth's axial wobble.".into(),
            },
            OrbitalElements::from_relative_orbit(
                1_400.0, 0.0549, 5.145, 125.08, 318.15, 27.321, 0.0,
            ),
            OrbitParent(earth_entity),
            OrbitPathColor(Color::srgba(0.7, 0.7, 0.8, 0.8)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(8.0))),
            MeshMaterial3d(moon_mat),
            Transform::default(),
        ))
        .id();
    targets.push((moon_entity, "Moon".to_string()));

    // 5. Mars
    let mars_color = Color::srgb(0.86, 0.38, 0.22);
    let mars_mat = create_mars_material(&mut standard_materials, None);
    let mars_entity = commands
        .spawn((
            CelestialBody {
                name: "Mars".into(),
                celestial_type: CelestialType::Planet,
                mass: 6.4171e23,
                radius: 3_389.5,
                display_radius: 16.0,
                color: mars_color,
                description: "The Red Planet, rich in iron oxide minerals, towering volcanoes, and polar ice caps.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                1.5237, 0.0934, 1.850, 49.578, 336.040, 686.980, 19.373,
            ),
            OrbitPathColor(Color::srgb(0.85, 0.4, 0.25)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(16.0))),
            MeshMaterial3d(mars_mat),
            Transform::default(),
        ))
        .id();
    targets.push((mars_entity, "Mars".to_string()));

    // Asteroid Belt: Spawn representative asteroid belt bodies between 2.2 and 3.2 AU
    let asteroid_mesh = meshes.add(Sphere::new(3.0));
    let asteroid_material = create_asteroid_material(&mut standard_materials, None);

    let asteroid_count = 140;
    for i in 0..asteroid_count {
        let frac = i as f64 / asteroid_count as f64;
        let pseudo_rand = (i as f64 * 73.189).sin().abs();
        let pseudo_rand2 = (i as f64 * 127.311).cos().abs();
        let pseudo_rand3 = (i as f64 * 231.777).sin().abs();

        let a_au = 2.2 + frac * 1.0 + pseudo_rand * 0.15;
        let e = 0.03 + pseudo_rand2 * 0.12;
        let inc = (pseudo_rand3 - 0.5) * 12.0;
        let long_node = frac * 360.0;
        let arg_peri = pseudo_rand * 360.0;
        let m0 = (i as f64 * 97.0) % 360.0;
        let period_days = (a_au.powf(1.5)) * 365.25;

        commands.spawn((
            CelestialBody {
                name: format!("Asteroid-{}", i + 1),
                celestial_type: CelestialType::Asteroid,
                mass: 1e18,
                radius: 50.0,
                display_radius: 3.0,
                color: Color::srgb(0.48, 0.44, 0.40),
                description: "Minor rocky body in the Main Asteroid Belt between Mars and Jupiter.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                a_au, e, inc, long_node, arg_peri, period_days, m0,
            ),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(asteroid_mesh.clone()),
            MeshMaterial3d(asteroid_material.clone()),
            Transform::default(),
        ));
    }

    // 6. Jupiter
    let jupiter_color = Color::srgb(0.85, 0.68, 0.48);
    let jupiter_mat = create_jupiter_material(&mut standard_materials, None);
    let jupiter_entity = commands
        .spawn((
            CelestialBody {
                name: "Jupiter".into(),
                celestial_type: CelestialType::Planet,
                mass: 1.8982e27,
                radius: 69_911.0,
                display_radius: 64.0,
                color: jupiter_color,
                description: "King of the planets, a massive gas giant with swirling storms and dozens of moons.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                5.2044, 0.0485, 1.303, 100.46, 273.867, 4332.59, 20.020,
            ),
            OrbitPathColor(Color::srgb(0.85, 0.65, 0.45)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(64.0))),
            MeshMaterial3d(jupiter_mat),
            Transform::default(),
        ))
        .id();
    targets.push((jupiter_entity, "Jupiter".to_string()));

    // 7. Saturn
    let saturn_color = Color::srgb(0.90, 0.82, 0.58);
    let saturn_mat = create_saturn_material(&mut standard_materials, None);
    // Ported Celestia ring renderer: a translucent banded disc.
    let saturn_rings_mat = create_saturn_rings_material(&mut standard_materials, None);
    let saturn_entity = commands
        .spawn((
            CelestialBody {
                name: "Saturn".into(),
                celestial_type: CelestialType::Planet,
                mass: 5.6834e26,
                radius: 58_232.0,
                display_radius: 52.0,
                color: saturn_color,
                description: "Jewel of the Solar System, famous for its magnificent and expansive ring system.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                9.5826, 0.0555, 2.485, 113.665, 339.392, 10759.22, 317.020,
            ),
            OrbitPathColor(Color::srgb(0.88, 0.80, 0.55)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(52.0))),
            MeshMaterial3d(saturn_mat),
            Transform::default(),
        ))
        .id();
    targets.push((saturn_entity, "Saturn".to_string()));

    // Saturn's Rings (Torus mesh flattened to simulate disk)
    let ring_mesh = meshes.add(Torus {
        minor_radius: 4.0,
        major_radius: 95.0,
    });

    commands.spawn((
        SaturnRingVisual,
        OrbitParent(saturn_entity),
        UniversalPosition(DVec3::ZERO),
        Mesh3d(ring_mesh),
        MeshMaterial3d(saturn_rings_mat),
        Transform::from_rotation(Quat::from_rotation_x(0.46)), // 26.7 deg axial tilt
    ));

    // 8. Uranus
    let uranus_color = Color::srgb(0.48, 0.82, 0.86);
    let uranus_mat = create_uranus_material(&mut standard_materials, None);
    let uranus_entity = commands
        .spawn((
            CelestialBody {
                name: "Uranus".into(),
                celestial_type: CelestialType::Planet,
                mass: 8.6810e25,
                radius: 25_362.0,
                display_radius: 36.0,
                color: uranus_color,
                description: "Cyan ice giant with a unique sideways rotation and faint ring system.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                19.2184, 0.0463, 0.773, 74.006, 96.998, 30685.4, 142.238,
            ),
            OrbitPathColor(Color::srgb(0.45, 0.8, 0.85)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(36.0))),
            MeshMaterial3d(uranus_mat),
            Transform::default(),
        ))
        .id();
    targets.push((uranus_entity, "Uranus".to_string()));

    // 9. Neptune
    let neptune_color = Color::srgb(0.20, 0.42, 0.92);
    let neptune_mat = create_neptune_material(&mut standard_materials, None);
    let neptune_entity = commands
        .spawn((
            CelestialBody {
                name: "Neptune".into(),
                celestial_type: CelestialType::Planet,
                mass: 1.0241e26,
                radius: 24_622.0,
                display_radius: 35.0,
                color: neptune_color,
                description: "Deep blue outermost giant, swept by the fastest supersonic winds in the Solar System.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                30.0700, 0.0095, 1.770, 131.784, 273.187, 60189.0, 256.228,
            ),
            OrbitPathColor(Color::srgb(0.25, 0.45, 0.92)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(35.0))),
            MeshMaterial3d(neptune_mat),
            Transform::default(),
        ))
        .id();
    targets.push((neptune_entity, "Neptune".to_string()));

    // 10. Pluto (Dwarf Planet)
    let pluto_color = Color::srgb(0.76, 0.66, 0.58);
    let pluto_mat = create_pluto_material(&mut standard_materials, None);
    let pluto_entity = commands
        .spawn((
            CelestialBody {
                name: "Pluto".into(),
                celestial_type: CelestialType::DwarfPlanet,
                mass: 1.303e22,
                radius: 1_188.3,
                display_radius: 10.0,
                color: pluto_color,
                description: "Kuiper Belt dwarf planet with a prominent heart-shaped plain of nitrogen ice.".into(),
            },
            OrbitalElements::from_keplerian_degrees(
                39.482, 0.2488, 17.16, 110.30, 113.83, 90560.0, 14.88,
            ),
            OrbitPathColor(Color::srgb(0.65, 0.55, 0.5)),
            UniversalPosition(DVec3::ZERO),
            Mesh3d(meshes.add(Sphere::new(10.0))),
            MeshMaterial3d(pluto_mat),
            Transform::default(),
        ))
        .id();
    targets.push((pluto_entity, "Pluto".to_string()));

    directory.targets = targets;
    directory.current_target_index = 0; // Start focused on the Sun
}

fn propagate_orbits(
    clock: Res<SimClock>,
    query_parents: Query<&UniversalPosition, Without<OrbitalElements>>,
    mut query_primary: Query<
        (&OrbitalElements, &mut UniversalPosition),
        Without<OrbitParent>,
    >,
    mut query_hierarchical: Query<
        (&OrbitalElements, &OrbitParent, &mut UniversalPosition),
        With<OrbitParent>,
    >,
) {
    // 1. Primary bodies (orbiting Sun at origin 0,0,0)
    for (elements, mut pos) in query_primary.iter_mut() {
        pos.0 = elements.position_at_time(clock.elapsed);
    }

    // 2. Hierarchical bodies (e.g. Moon orbiting Earth)
    for (elements, parent, mut pos) in query_hierarchical.iter_mut() {
        if let Ok(parent_pos) = query_parents.get(parent.0) {
            let relative_pos = elements.position_at_time(clock.elapsed);
            pos.0 = parent_pos.0 + relative_pos;
        }
    }
}

fn update_saturn_rings(
    query_saturn: Query<&UniversalPosition, (With<CelestialBody>, Without<SaturnRingVisual>)>,
    mut query_rings: Query<(&OrbitParent, &mut UniversalPosition), With<SaturnRingVisual>>,
) {
    for (parent, mut ring_pos) in query_rings.iter_mut() {
        if let Ok(saturn_pos) = query_saturn.get(parent.0) {
            ring_pos.0 = saturn_pos.0;
        }
    }
}