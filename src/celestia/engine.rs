//! Celestia engine data model: bodies, stars, selection, and universe.
//!
//! Faithful Rust port of the core of Celestia's `celengine` library
//! (`body.h`, `star.h`, `selection.h`, `solarsys.h`, `universe.h`) plus the
//! `.ssc` catalog loader (`solarsys.cpp` / `parseobject.cpp`).
//!
//! This module is intentionally free of any Bevy dependency so the simulation
//! can be unit-tested in isolation and later rendered by the Bevy layer.
#![allow(missing_docs)]

use std::collections::BTreeMap;

use glam::{DQuat, DVec3};

use super::cel_parser::{self, AssociativeArray};
use super::ephemeris::{
    Orbit, RotationModel, UniformRotation, ConstantOrientation, EllipticalOrbit, HyperbolicOrbit,
};
use super::astro;

// --- Body classification (celengine/body.h: BodyClassification) ---------------

/// A body's classification, mirroring `BodyClassification`.
///
/// Celestia uses a bitset; here the common classes are separate variants for
/// ergonomic matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyClass {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
    Comet,
    Spacecraft,
    Barycenter,
    /// A body that is not clickable/renderable (e.g. a barycenter).
    Invisible,
}

/// Parse a body class name, mirroring the `Class` field in `.ssc` files.
pub fn parse_body_class(name: &str) -> Option<BodyClass> {
    match name {
        "Star" => Some(BodyClass::Star),
        "Planet" => Some(BodyClass::Planet),
        "DwarfPlanet" => Some(BodyClass::DwarfPlanet),
        "Moon" => Some(BodyClass::Moon),
        "Asteroid" => Some(BodyClass::Asteroid),
        "Comet" => Some(BodyClass::Comet),
        "Spacecraft" => Some(BodyClass::Spacecraft),
        "Barycenter" | "BaryCentre" => Some(BodyClass::Barycenter),
        "Invisible" => Some(BodyClass::Invisible),
        _ => None,
    }
}

// --- A star (celengine/star.h) -----------------------------------------------

/// An interstellar object, mirroring `celengine::Star` + `StarDetails`.
#[derive(Debug, Clone)]
pub struct Star {
    pub name: String,
    pub position: DVec3, // in light-years
    pub radius: f64, // km
    pub temperature: f64, // K
    pub bolometric_correction: f64,
    pub spectral_type: String,
    pub color: super::cel_parser::Color,
}

impl Star {
    pub fn new(name: impl Into<String>, position: DVec3) -> Self {
        Self {
            name: name.into(),
            position,
            radius: astro::SOLAR_RADIUS,
            temperature: astro::SOLAR_TEMPERATURE,
            bolometric_correction: 0.0,
            spectral_type: "G".to_string(),
            color: super::cel_parser::Color::default(),
        }
    }
}

// --- Selection (celengine/selection.h) ---------------------------------------

/// A reference to a single object in the universe, mirroring
/// `celengine::Selection`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Selection {
    #[default]
    None,
    Star(String),
    Body(String),
}

impl Selection {
    pub fn body(&self) -> Option<&str> {
        match self {
            Selection::Body(s) => Some(s),
            _ => None,
        }
    }

    pub fn star(&self) -> Option<&str> {
        match self {
            Selection::Star(s) => Some(s),
            _ => None,
        }
    }
}

// --- A body (celengine/body.h) -----------------------------------------------

/// A planet, moon, asteroid, comet, or spacecraft, mirroring `Body`.
#[derive(Debug)]
pub struct Body {
    pub name: String,
    pub class: BodyClass,
    pub mass: f64, // kg
    pub radius: f64, // km
    pub color: super::cel_parser::Color,
    /// The orbit of this body in its parent's frame.
    pub orbit: Box<dyn Orbit>,
    /// The rotation model of this body.
    pub rotation: Box<dyn RotationModel>,
    /// The parent body this orbits (None for the central star).
    pub parent: Option<String>,
    /// Whether the orbit uses planet units (AU/years) vs km/days.
    pub planet_units: bool,
    pub description: String,
}

impl Body {
    /// Position of this body at the given Julian date (TDB), in the frame of
    /// its parent.
    pub fn position_at_time(&self, tdb: f64) -> DVec3 {
        self.orbit.position_at_time(tdb)
    }

    /// Orientation of this body at the given Julian date.
    pub fn orientation_at_time(&self, tdb: f64) -> DQuat {
        self.rotation.orientation_at_time(tdb)
    }
}

// --- The solar system (celengine/solarsys.h) ---------------------------------

/// A planetary system: a central star and its orbiting bodies, mirroring
/// `PlanetarySystem`.
#[derive(Debug, Default)]
pub struct PlanetarySystem {
    pub star: String,
    /// Bodies in the order they are parsed.
    pub bodies: Vec<Body>,
    name_to_index: BTreeMap<String, usize>,
}

impl PlanetarySystem {
    pub fn new(star: impl Into<String>) -> Self {
        let star = star.into();
        let mut name_to_index = BTreeMap::new();
        let _ = &mut name_to_index;
        Self {
            star,
            bodies: Vec::new(),
            name_to_index,
        }
    }

    pub fn add(&mut self, body: Body) {
        self.name_to_index
            .insert(body.name.clone(), self.bodies.len());
        self.bodies.push(body);
    }

    pub fn get(&self, name: &str) -> Option<&Body> {
        self.name_to_index
            .get(name)
            .map(|&i| &self.bodies[i])
    }

    /// Position of a body relative to the system origin (the central star) at
    /// `tdb`, in kilometers. For a satellite, the parent's position is added
    /// recursively; the walk stops at the central star, which sits at the
    /// origin.
    pub fn body_position_at_time(&self, body_name: &str, tdb: f64) -> Option<DVec3> {
        let mut pos = DVec3::ZERO;
        let mut current = body_name;
        loop {
            // The central star is the system origin; it contributes nothing and
            // is not a body, so stop here.
            if current == self.star {
                break;
            }
            let b = self.get(current)?;
            pos += b.position_at_time(tdb);
            match &b.parent {
                Some(p) => current = p,
                None => break,
            }
        }
        Some(pos)
    }
}

/// A named solar system: a star and its planetary system, mirroring
/// `celengine::SolarSystem`.
#[derive(Debug)]
pub struct SolarSystem {
    pub name: String,
    pub star: Star,
    pub planetary_system: PlanetarySystem,
}

impl SolarSystem {
    /// The star's position in the interstellar frame (fixed).
    pub fn position_at_time(&self, _tdb: f64) -> DVec3 {
        self.star.position
    }

    /// Position of a body in the interstellar frame at `tdb`: the star's
    /// position plus the body's position relative to the system origin.
    pub fn body_position_at_time(&self, body_name: &str, tdb: f64) -> Option<DVec3> {
        let relative = self.planetary_system.body_position_at_time(body_name, tdb)?;
        Some(self.star.position + relative)
    }
}

// --- The universe (celengine/universe.h) -------------------------------------

/// The top-level container of all solar systems, mirroring `Universe`.
#[derive(Debug, Default)]
pub struct Universe {
    solar_systems: BTreeMap<String, SolarSystem>,
}

impl Universe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_solar_system(&mut self, ss: SolarSystem) {
        self.solar_systems.insert(ss.name.clone(), ss);
    }

    pub fn get_solar_system(&self, name: &str) -> Option<&SolarSystem> {
        self.solar_systems.get(name)
    }

    /// Find the nearest solar system to a position (in light-years).
    pub fn get_nearest_solar_system(&self, pos: DVec3) -> Option<&SolarSystem> {
        self.solar_systems
            .values()
            .min_by(|a, b| {
                (a.star.position - pos)
                    .length_squared()
                    .partial_cmp(&(b.star.position - pos).length_squared())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn count(&self) -> usize {
        self.solar_systems.len()
    }
}

// --- Orbit / rotation construction (parseobject.cpp) -------------------------

/// Build an orbit from a CEL `Orbit`/`EllipticalOrbit`/`HyperbolicOrbit`
/// group, mirroring `CreateKeplerianOrbit`.
///
/// Distance and time fields are scaled by their *explicit* unit when present
/// (e.g. `SemiMajorAxis 1.0 : AU`, `Period 365.256 : days`); only when no unit
/// is written do the `planet_units` defaults apply (AU/years for planets,
/// km/days otherwise), matching `GetDefaultUnits` + `getLength`/`getTime`.
pub fn create_orbit(orbit_data: &AssociativeArray, planet_units: bool) -> Option<Box<dyn Orbit>> {
    // Default scales, used only when a field has no explicit unit.
    let distance_default = if planet_units {
        astro::KM_PER_AU
    } else {
        1.0
    };
    let time_default = if planet_units {
        astro::DAYS_PER_YEAR
    } else {
        1.0
    };

    let mut elements = astro::KeplerElements::default();

    elements.eccentricity = orbit_data.get_number("Eccentricity").unwrap_or(0.0);
    if elements.eccentricity < 0.0 || elements.eccentricity == 1.0 {
        return None;
    }

    // A distance field: explicit unit wins, else the default scale.
    let read_distance = |key: &str| -> Option<f64> {
        match orbit_data.get_number_with_unit(key) {
            Some((value, Some(unit))) => astro::length_scale(&unit).map(|s| value * s),
            Some((value, None)) => Some(value * distance_default),
            None => None,
        }
    };
    // A time field: explicit unit wins, else the default scale.
    let read_time = |key: &str| -> Option<f64> {
        match orbit_data.get_number_with_unit(key) {
            Some((value, Some(unit))) => astro::time_scale(&unit).map(|s| value * s),
            Some((value, None)) => Some(value * time_default),
            None => None,
        }
    };

    // SemiMajorAxis and Period are required.
    let semi_major = if let Some(a) = read_distance("SemiMajorAxis") {
        a
    } else if let Some(p) = read_distance("PericenterDistance") {
        p / (1.0 - elements.eccentricity)
    } else {
        return None;
    };
    elements.semi_major_axis = semi_major;

    let period = read_time("Period")?;
    if period == 0.0 {
        return None;
    }
    elements.period = period;

    elements.inclination = orbit_data.get_number("Inclination").unwrap_or(0.0).to_radians();
    elements.long_ascending_node = orbit_data
        .get_number("AscendingNode")
        .unwrap_or(0.0)
        .to_radians();

    if let Some(arg) = orbit_data.get_number("ArgOfPericenter") {
        elements.arg_pericenter = arg.to_radians();
    } else if let Some(long) = orbit_data.get_number("LongOfPericenter") {
        elements.arg_pericenter = (long - elements.long_ascending_node.to_degrees()).to_radians();
    }

    // Epoch defaults to J2000.
    let mut epoch = astro::J2000;
    if let Some(e) = orbit_data.get_number("Epoch") {
        epoch = e;
    }

    // Mean anomaly or mean longitude.
    if let Some(m) = orbit_data.get_number("MeanAnomaly") {
        elements.mean_anomaly = m.to_radians();
    } else if let Some(l) = orbit_data.get_number("MeanLongitude") {
        elements.mean_anomaly =
            (l - (elements.arg_pericenter.to_degrees() + elements.long_ascending_node.to_degrees()))
                .to_radians();
    }

    if elements.eccentricity < 1.0 {
        EllipticalOrbit::new(&elements, epoch).map(|o| Box::new(o) as Box<dyn Orbit>)
    } else {
        HyperbolicOrbit::new(&elements, epoch).map(|o| Box::new(o) as Box<dyn Orbit>)
    }
}

/// Build a rotation model from a CEL `Rotation` group, mirroring
/// `CreateRotationModel`.
pub fn create_rotation_model(rot_data: &AssociativeArray) -> Option<Box<dyn RotationModel>> {
    let mut period = rot_data
        .get_number_with_unit("Period")
        .map(|(p, _)| p / 24.0) // hours -> days
        .unwrap_or(0.0);

    // Spin axis (defaults to +Z).
    let axis = match rot_data.get("SpinAxis") {
        Some(cel_parser::Value::Vector3(v)) => DVec3::new(v[0], v[1], v[2]).normalize(),
        _ => DVec3::Z,
    };

    if period == 0.0 {
        return Some(Box::new(ConstantOrientation::default()));
    }

    let _ = &mut period;
    Some(Box::new(UniformRotation::new(period, axis)))
}

// --- .ssc body table loader (solarsys.cpp / parseobject.cpp) -----------------

/// A parsed `.ssc` body entry.
#[derive(Debug)]
pub struct BodyDefinition {
    pub name: String,
    pub parent: String,
    pub class: BodyClass,
    pub mass: f64,
    pub radius: f64,
    pub color: super::cel_parser::Color,
    pub orbit: Option<Box<dyn Orbit>>,
    pub rotation: Option<Box<dyn RotationModel>>,
    pub planet_units: bool,
    pub description: String,
}

/// Parse one `Body`/`Location`/`AltSurface` entry from an `.ssc` group,
/// mirroring the body-parsing portion of `SolarSystemsBuilder::parseSsc`.
///
/// `planet_units` is chosen by the caller based on which catalog the body came
/// from (planets use AU/days, satellites/asteroids use km/days).
pub fn parse_body_definition(
    name: &str,
    parent: &str,
    group: &AssociativeArray,
    default_planet_units: bool,
) -> Option<BodyDefinition> {
    // Class may be specified inline; default depends on the catalog.
    let class = match group.get_string("Class").and_then(parse_body_class) {
        Some(c) => c,
        None => {
            if default_planet_units {
                BodyClass::Planet
            } else {
                BodyClass::Moon
            }
        }
    };

    // Determine planet units: planets and the primary use AU/days; moons and
    // asteroids use km/days unless overridden by the `Units` field.
    let mut planet_units = default_planet_units;
    if let Some(u) = group.get_string("Units") {
        planet_units = matches!(u.to_lowercase().as_str(), "au" | "planet");
    }

    let orbit_group = group.get_group("Orbit");
    let orbit = orbit_group.and_then(|g| create_orbit(g, planet_units));

    let rotation_group = group.get_group("Rotation");
    let rotation = rotation_group.and_then(|g| create_rotation_model(g));

    let color = group.get("Color").and_then(|v| v.as_color()).unwrap_or_default();

    Some(BodyDefinition {
        name: name.to_string(),
        parent: parent.to_string(),
        class,
        mass: group.get_number_with_unit("Mass").map(|(m, _)| m).unwrap_or(0.0),
        radius: group.get_number_with_unit("Radius").map(|(r, _)| r).unwrap_or(0.0),
        color,
        orbit,
        rotation,
        planet_units,
        description: group.get_string("Description").unwrap_or_default().to_string(),
    })
}

/// Load a full `.ssc` catalog (a sequence of body groups) into a planetary
/// system around the given central star.
pub fn parse_ssc_catalog(input: &str, star_name: &str, default_planet_units: bool) -> PlanetarySystem {
    let mut system = PlanetarySystem::new(star_name);

    // Each entry is parsed with `cel_parser::parse_ssc_entries`, which follows
    // Celestia's SSC grammar: the name and parent are read *before* the
    // property group, not from inside it.
    let entries = match cel_parser::parse_ssc_entries(input) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!("Failed to parse SSC catalog: {e}");
            return system;
        }
    };

    for entry in entries {
        if entry.name.is_empty() {
            continue;
        }
        let def = match parse_body_definition(
            &entry.name,
            &entry.parent,
            &entry.data,
            default_planet_units,
        ) {
            Some(def) => def,
            None => continue,
        };
        let orbit = def.orbit.unwrap_or_else(|| {
            // Bodies without an orbit are fixed at the origin.
            Box::new(FixedOrbit) as Box<dyn Orbit>
        });
        let rotation = def.rotation.unwrap_or_else(|| {
            Box::new(ConstantOrientation::default()) as Box<dyn RotationModel>
        });
        system.add(Body {
            name: def.name,
            class: def.class,
            mass: def.mass,
            radius: def.radius,
            color: def.color,
            orbit,
            rotation,
            parent: if def.parent.is_empty() {
                None
            } else {
                Some(def.parent)
            },
            planet_units: def.planet_units,
            description: def.description,
        });
    }

    system
}

/// A fixed (non-moving) orbit, used for the central star and for bodies that
/// specify no orbit. Mirrors the concept of a body at a constant position.
#[derive(Debug, Clone, Default)]
pub struct FixedOrbit;

impl Orbit for FixedOrbit {
    fn position_at_time(&self, _tdb: f64) -> DVec3 {
        DVec3::ZERO
    }
    fn get_period(&self) -> f64 {
        0.0
    }
    fn get_bounding_radius(&self) -> f64 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth_ssc() -> &'static str {
        r#"
            Body "Earth" "Sun"
            {
                Class Planet
                Mass 5.972e24 : kg
                Radius 6371.0 : km
                Color "0.18 0.46 0.88"
                Orbit
                {
                    Type Elliptical
                    SemiMajorAxis 1.0 : AU
                    Eccentricity 0.0167
                    Inclination 0.0
                    AscendingNode 0.0
                    ArgOfPericenter 102.9
                    MeanAnomaly 358.617
                    Period 365.256 : days
                }
            }
        "#
    }

    #[test]
    fn parse_earth_body() {
        let system = parse_ssc_catalog(earth_ssc(), "Sun", true);
        let earth = system.get("Earth").expect("earth present");
        assert_eq!(earth.class, BodyClass::Planet);
        assert!((earth.radius - 6371.0).abs() < 1e-3);
        assert!((earth.mass - 5.972e24).abs() < 1e17);
        assert_eq!(earth.parent.as_deref(), Some("Sun"));
        // Position at J2000 should be ~1 AU from the Sun.
        let p = earth.position_at_time(astro::J2000);
        assert!((p.length() - astro::KM_PER_AU).abs() < 1.0e7, "got {p:?}");
    }

    #[test]
    fn orbit_is_periodic() {
        let system = parse_ssc_catalog(earth_ssc(), "Sun", true);
        let earth = system.get("Earth").unwrap();
        assert!(earth.orbit.is_periodic());
        assert!((earth.orbit.get_period() - 365.256).abs() < 1e-3);
    }

    #[test]
    fn universal_position_uses_star_frame() {
        let system = parse_ssc_catalog(earth_ssc(), "Sun", true);
        // The star is at the origin, so the body position equals its orbit.
        let p = system.body_position_at_time("Earth", astro::J2000).unwrap();
        assert!((p.length() - astro::KM_PER_AU).abs() < 1.0e7);
    }

    #[test]
    fn universe_nearest_solar_system() {
        let mut u = Universe::new();
        let mut ss = SolarSystem {
            name: "Sol".to_string(),
            star: Star::new("Sun", DVec3::ZERO),
            planetary_system: PlanetarySystem::new("Sun"),
        };
        ss.planetary_system.add(Body {
            name: "Earth".to_string(),
            class: BodyClass::Planet,
            mass: 5.972e24,
            radius: 6371.0,
            color: super::super::cel_parser::Color::default(),
            orbit: Box::new(FixedOrbit) as Box<dyn Orbit>,
            rotation: Box::new(ConstantOrientation::default()) as Box<dyn RotationModel>,
            parent: None,
            planet_units: true,
            description: String::new(),
        });
        u.add_solar_system(ss);
        assert_eq!(u.count(), 1);
        let nearest = u.get_nearest_solar_system(DVec3::ZERO).unwrap();
        assert_eq!(nearest.name, "Sol");
    }

    #[test]
    fn selection_round_trip() {
        let s = Selection::Body("Earth".to_string());
        assert_eq!(s.body(), Some("Earth"));
        assert_eq!(Selection::default(), Selection::None);
    }
}
