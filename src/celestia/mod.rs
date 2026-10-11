//! A faithful Rust port of Celestia's core simulation engine.
//!
//! This module mirrors the structure of Celestia's C++ libraries:
//!
//! | Celestia (C++)   | This crate            |
//! |------------------|-----------------------|
//! | `celastro`       | [`astro`]             |
//! | `celephem`       | [`ephemeris`]         |
//! | `celutil` (data) | [`cel_parser`]        |
//! | `celengine`      | [`engine`]            |
//!
//! The port is intentionally free of any Bevy dependency so the astronomy
//! simulation can be unit-tested in isolation and later rendered by the Bevy
//! layer (see `celestia_bridge`).
//!
//! # Units
//!
//! Celestia's internal units are kilometers (distance) and days (time), with
//! times expressed as Julian dates (J2000 = 2451545.0). See
//! [`astro`] for the full set of conversion functions.

#![allow(missing_docs)]

pub mod astro;
pub mod cel_parser;
pub mod celdmath;
pub mod ephemeris;
pub mod engine;

/// A small embedded solar-system catalog, written in Celestia's `.ssc` format.
///
/// This stands in for `data/solarsys.ssc` and lets the engine produce the
/// classical planets without shipping external data files. Distances are in AU
/// and periods in days (planet units).
pub const SOLAR_SYSTEM_CATALOG: &str = r#"
    # Creatoris embedded solar-system catalog (a subset of Celestia's solarsys.ssc).
    # Planet orbits use AU / day units.

    Body "Mercury" "Sun"
    {
        Class Planet
        Mass 3.3011e23 : kg
        Radius 2439.7 : km
        Color "0.72 0.70 0.68"
        Description "Smallest planet, scorched by proximity to the Sun."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 0.3871 : AU
            Eccentricity 0.2056
            Inclination 7.005
            AscendingNode 48.331
            ArgOfPericenter 29.124
            MeanAnomaly 174.79
            Period 87.969 : days
        }
    }

    Body "Venus" "Sun"
    {
        Class Planet
        Mass 4.8675e24 : kg
        Radius 6051.8 : km
        Color "0.92 0.80 0.52"
        Description "Terrestrial planet wrapped in thick clouds of sulfuric acid."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 0.7233 : AU
            Eccentricity 0.0067
            Inclination 3.394
            AscendingNode 76.680
            ArgOfPericenter 54.884
            MeanAnomaly 224.701
            Period 224.701 : days
        }
    }

    Body "Earth" "Sun"
    {
        Class Planet
        Mass 5.972e24 : kg
        Radius 6371.0 : km
        Color "0.18 0.46 0.88"
        Description "The third planet from the Sun; the only known world with life."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 1.0 : AU
            Eccentricity 0.0167
            Inclination 0.0
            AscendingNode 0.0
            ArgOfPericenter 102.947
            MeanAnomaly 358.617
            Period 365.256 : days
        }
        Rotation
        {
            Type FixedPeriod
            Period 23.9345 : hours
            SpinAxis "0.0 0.0 1.0"
        }
    }

    Body "Moon" "Earth"
    {
        Class Moon
        Mass 7.346e22 : kg
        Radius 1737.4 : km
        Color "0.75 0.75 0.75"
        Description "Earth's only natural satellite."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 384400.0 : km
            Eccentricity 0.0549
            Inclination 5.145
            AscendingNode 125.08
            ArgOfPericenter 318.15
            MeanAnomaly 0.0
            Period 27.321 : days
        }
    }

    Body "Mars" "Sun"
    {
        Class Planet
        Mass 6.4171e23 : kg
        Radius 3389.5 : km
        Color "0.85 0.30 0.15"
        Description "The red planet; a prime candidate for terraforming."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 1.5237 : AU
            Eccentricity 0.0934
            Inclination 1.850
            AscendingNode 49.578
            ArgOfPericenter 336.040
            MeanAnomaly 19.373
            Period 686.980 : days
        }
    }

    Body "Jupiter" "Sun"
    {
        Class Planet
        Mass 1.8986e27 : kg
        Radius 69911.0 : km
        Color "0.80 0.65 0.50"
        Description "The largest planet; a gas giant with 95 known moons."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 5.2044 : AU
            Eccentricity 0.0485
            Inclination 1.303
            AscendingNode 100.46
            ArgOfPericenter 273.867
            MeanAnomaly 20.020
            Period 4332.59 : days
        }
    }

    Body "Saturn" "Sun"
    {
        Class Planet
        Mass 5.6834e26 : kg
        Radius 58232.0 : km
        Color "0.90 0.80 0.55"
        Description "The ringed gas giant, the sixth planet from the Sun."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 9.5826 : AU
            Eccentricity 0.0555
            Inclination 2.485
            AscendingNode 113.665
            ArgOfPericenter 339.392
            MeanAnomaly 317.020
            Period 10759.22 : days
        }
    }

    Body "Uranus" "Sun"
    {
        Class Planet
        Mass 8.6810e25 : kg
        Radius 25362.0 : km
        Color "0.65 0.85 0.90"
        Description "An ice giant that rotates on its side."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 19.2184 : AU
            Eccentricity 0.0463
            Inclination 0.773
            AscendingNode 74.006
            ArgOfPericenter 96.998
            MeanAnomaly 142.238
            Period 30685.4 : days
        }
    }

    Body "Neptune" "Sun"
    {
        Class Planet
        Mass 1.0241e26 : kg
        Radius 24622.0 : km
        Color "0.30 0.45 0.90"
        Description "The most distant planet; a deep-blue ice giant."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 30.0700 : AU
            Eccentricity 0.0095
            Inclination 1.770
            AscendingNode 131.784
            ArgOfPericenter 273.187
            MeanAnomaly 256.228
            Period 60189.0 : days
        }
    }

    Body "Pluto" "Sun"
    {
        Class DwarfPlanet
        Mass 1.303e22 : kg
        Radius 1188.3 : km
        Color "0.75 0.65 0.55"
        Description "The most famous dwarf planet; an icy body in the Kuiper belt."
        Orbit
        {
            Type Elliptical
            SemiMajorAxis 39.482 : AU
            Eccentricity 0.2488
            Inclination 17.16
            AscendingNode 110.30
            ArgOfPericenter 113.83
            MeanAnomaly 14.88
            Period 90560.0 : days
        }
    }
"#;

/// Build a [`Universe`] populated with the embedded classical solar system.
///
/// This is the equivalent of loading `data/solarsys.ssc` into Celestia's
/// `Universe`, but from embedded data.
pub fn build_embedded_solar_system() -> engine::Universe {
    let mut universe = engine::Universe::new();
    let planetary_system = engine::parse_ssc_catalog(SOLAR_SYSTEM_CATALOG, "Sun", true);

    let mut star = engine::Star::new("Sun", glam::DVec3::ZERO);
    star.radius = astro::SOLAR_RADIUS;

    let ss = engine::SolarSystem {
        name: "Sol".to_string(),
        star,
        planetary_system,
    };
    universe.add_solar_system(ss);
    universe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_has_all_planets() {
        let universe = build_embedded_solar_system();
        assert_eq!(universe.count(), 1);
        let sol = universe.get_solar_system("Sol").expect("Sol");
        for name in [
            "Mercury",
            "Venus",
            "Earth",
            "Moon",
            "Mars",
            "Jupiter",
            "Saturn",
            "Uranus",
            "Neptune",
            "Pluto",
        ] {
            assert!(
                sol.planetary_system.get(name).is_some(),
                "missing {name}"
            );
        }
    }

    #[test]
    fn earth_orbits_the_sun() {
        let universe = build_embedded_solar_system();
        let sol = universe.get_solar_system("Sol").unwrap();
        let earth = sol.body_position_at_time("Earth", astro::J2000).unwrap();
        assert!((earth.length() - astro::KM_PER_AU).abs() < 5.0e6, "got {earth:?}");
    }

    #[test]
    fn moon_orbits_earth() {
        let universe = build_embedded_solar_system();
        let sol = universe.get_solar_system("Sol").unwrap();
        let earth = sol.body_position_at_time("Earth", astro::J2000).unwrap();
        let moon = sol.body_position_at_time("Moon", astro::J2000).unwrap();
        // The Moon should be within ~384,400 km of Earth.
        let separation = (moon - earth).length();
        assert!((separation - 384_400.0).abs() < 20_000.0, "got {separation}");
    }
}
