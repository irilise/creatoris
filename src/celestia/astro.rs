//! Celestia astronomy core: constants, unit conversions, and time.
//!
//! Faithful Rust port of Celestia's `celastro` library (`astro.h`/`astro.cpp`,
//! `date.h`/`date.cpp`). This module is intentionally free of any Bevy
//! dependency so it can be unit-tested in isolation and later reused by the
//! simulation engine and the rendering layer.
//!
//! Celestia's internal units are kilometers (distance) and days (time).
//! Angles are stored in radians. Times are expressed as Julian dates
//! (J2000 = 2451545.0).
#![allow(missing_docs)]

use std::f64::consts::PI;

// --- Angle / time helpers (celastro/astro.h) --------------------------------

/// Angle between the J2000 mean equator and the ecliptic plane:
/// 23° 26′ 21.448″ (Seidelmann 1992, eqn 3.222-1).
pub const J2000_OBLIQUITY: f64 = 23.4392911_f64.to_radians();

/// Speed of light in km/s (CODATA 2022).
pub const SPEED_OF_LIGHT: f64 = 299_792.458;

/// Newtonian constant of gravitation, N·m²/kg² (CODATA 2022).
pub const G: f64 = 6.67430e-11;

// IAU 2015 Resolution B3 + CODATA 2022 gravitation parameter (km³/day²):
//   GM_sun = 1.3271244e20 km³/s²  *  86400²  =  1.005401e15 km³/day²
pub const SUN_MASS: f64 = 1.3271244e20 / G; // kg
pub const EARTH_MASS: f64 = 3.986004e14 / G; // kg
pub const LUNAR_MASS: f64 = 7.346e22; // kg
pub const JUPITER_MASS: f64 = 1.2668653e17 / G; // kg

// Stellar constants
pub const SOLAR_ABSMAG: f64 = 4.81;
pub const SOLAR_TEMPERATURE: f64 = 5772.0;
pub const SOLAR_BOLOMETRIC_MAG: f64 = 4.74;
pub const SOLAR_IRRADIANCE: f64 = 1361.0; // W / m²
pub const SOLAR_POWER: f64 = 3.828e26; // W
pub const VEGAN_IRRADIANCE: f64 = 3.640e-11; // W / m²
pub const LN_MAG: f64 = 1.0857362; // 5 / ln(100)
pub const LOWEST_IRRADIATION: f64 = 1.0 / 255.0;
pub const LOWEST_IRRADIATION_SRGB: f64 = 1.0 / (255.0 * 12.92);

/// ~10 parsecs in light-years; used to decide when to switch apparent
/// magnitude to absolute magnitude.
pub const LY_PER_10PARSEC: f64 = 32.615637771674336;
pub const LY_PER_PARSEC: f64 = 3.26156377716743356213863970704550837409;
pub const KM_PER_LY: f64 = 9460_730_472_580.8;
pub const KM_PER_AU: f64 = 149_597_870.7;
pub const AU_PER_LY: f64 = 63241.077084266280268653583182317313558;
pub const KM_PER_PARSEC: f64 = 3.08567758149136727891393795779647161073e13;
pub const EARTH_RADIUS: f64 = 6378.1; // IAU 2015 B3, km
pub const JUPITER_RADIUS: f64 = 71_492.0; // IAU 2015 B3, km
pub const SOLAR_RADIUS: f64 = 695_700.0; // IAU 2015 B3, km

pub const MINUTES_PER_DEG: f64 = 60.0;
pub const SECONDS_PER_DEG: f64 = 3600.0;
pub const DEG_PER_HRA: f64 = 15.0;

// --- Distance / magnitude conversions (templated in C++) ---------------------

/// `ly / LY_PER_PARSEC`
pub const fn light_years_to_parsecs(ly: f64) -> f64 {
    ly / LY_PER_PARSEC
}

pub const fn parsecs_to_light_years(pc: f64) -> f64 {
    pc * LY_PER_PARSEC
}

pub const fn light_years_to_kilometers(ly: f64) -> f64 {
    ly * KM_PER_LY
}

pub const fn kilometers_to_light_years(km: f64) -> f64 {
    km / KM_PER_LY
}

pub const fn light_years_to_au(ly: f64) -> f64 {
    ly * AU_PER_LY
}

pub const fn au_to_light_years(au: f64) -> f64 {
    au / AU_PER_LY
}

pub const fn au_to_kilometers(au: f64) -> f64 {
    au * KM_PER_AU
}

pub const fn kilometers_to_au(km: f64) -> f64 {
    km / KM_PER_AU
}

pub const fn micro_light_years_to_kilometers(ly: f64) -> f64 {
    ly * (KM_PER_LY * 1e-6)
}

pub const fn kilometers_to_micro_light_years(km: f64) -> f64 {
    km / (KM_PER_LY * 1e-6)
}

pub const fn micro_light_years_to_au(ly: f64) -> f64 {
    ly * (AU_PER_LY * 1e-6)
}

pub const fn au_to_micro_light_years(au: f64) -> f64 {
    au / (KM_PER_LY * 1e-6)
}

/// `5 * log10(lyrs / LY_PER_PARSEC) - 5`
pub fn distance_modulus(lyrs: f64) -> f64 {
    5.0 * (lyrs / LY_PER_PARSEC).log10() - 5.0
}

/// `absMag + distanceModulus(lyrs)`
pub fn abs_to_app_mag(abs_mag: f64, lyrs: f64) -> f64 {
    abs_mag + distance_modulus(lyrs)
}

/// `absMag - distanceModulus(lyrs)`
pub fn app_to_abs_mag(app_mag: f64, lyrs: f64) -> f64 {
    app_mag - distance_modulus(lyrs)
}

// --- Kepler's equation (celastro/astro.cpp) ---------------------------------

/// Solve Kepler's equation `M = E - e·sin(E)` for the eccentric anomaly `E`,
/// and return the corresponding true anomaly.
///
/// Faithful port of Celestia's `celastro::anomaly`:
///
/// ```text
/// e = M - 2π·floor(M / 2π)            // reduce M into [0, 2π)
/// iterate: E <- E - (E - e·sin E - M) / (1 - e·cos E)
/// ```
///
/// # Arguments
/// * `mean_anomaly` - Mean anomaly `M` in radians.
/// * `eccentricity` - Orbital eccentricity `e` (`0 <= e < 1`).
///
/// # Returns
/// A `(true_anomaly, eccentric_anomaly)` pair in radians.
pub fn anomaly(mean_anomaly: f64, eccentricity: f64) -> (f64, f64) {
    const TOL: f64 = 1.745e-8;
    let mut iterations = 20usize; // hard cap on the Newton iterations

    // Reduce the mean anomaly into [0, 2π).
    let mut e = mean_anomaly - 2.0 * PI * (mean_anomaly / (2.0 * PI)).floor();
    let mut err = 1.0_f64;
    while err.abs() > TOL && iterations > 0 {
        err = e - eccentricity * e.sin() - mean_anomaly;
        let delta = err / (1.0 - eccentricity * e.cos());
        e -= delta;
        iterations -= 1;
    }

    let true_anomaly =
        2.0 * (((1.0 + eccentricity) / (1.0 - eccentricity)).sqrt() * (0.5 * e).tan()).atan();
    (true_anomaly, e)
}

// --- Keplerian elements (celastro/astro.h: KeplerElements) -------------------

/// Standard Keplerian orbital elements, mirroring
/// `celastro::KeplerElements`.
///
/// Distances are in kilometers and time in days (Celestia's internal units).
/// Angles are in radians.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct KeplerElements {
    pub semi_major_axis: f64, // km
    pub eccentricity: f64,
    pub inclination: f64, // rad
    pub long_ascending_node: f64, // rad
    pub arg_pericenter: f64, // rad
    pub mean_anomaly: f64, // rad
    pub period: f64, // days
}

impl KeplerElements {
    /// Build elements from planet-units values (AU / degrees / years), matching
    /// how Celestia's `.ssc` planet catalogs specify orbits.
    pub fn from_planet_units(
        semi_major_axis_au: f64,
        eccentricity: f64,
        inclination_deg: f64,
        long_ascending_node_deg: f64,
        arg_pericenter_deg: f64,
        mean_anomaly_deg: f64,
        period_years: f64,
    ) -> Self {
        Self {
            semi_major_axis: au_to_kilometers(semi_major_axis_au),
            eccentricity,
            inclination: inclination_deg.to_radians(),
            long_ascending_node: long_ascending_node_deg.to_radians(),
            arg_pericenter: arg_pericenter_deg.to_radians(),
            mean_anomaly: mean_anomaly_deg.to_radians(),
            period: period_years * DAYS_PER_YEAR,
        }
    }
}

/// A small numeric type used to tag which units a value is expressed in.
/// Celestia's `getLength`/`getTime`/`getAngle` conversions apply an
/// output-scale and a default-scale; these helpers capture the common
/// "convert from canonical units" operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceUnit {
    Kilometers,
    AU,
    LightYears,
}

/// Convert a distance to kilometers given its source unit.
pub fn distance_to_kilometers(value: f64, unit: DistanceUnit) -> f64 {
    match unit {
        DistanceUnit::Kilometers => value,
        DistanceUnit::AU => au_to_kilometers(value),
        DistanceUnit::LightYears => light_years_to_kilometers(value),
    }
}

// --- Unit scale tables (celastro/units.cpp) ----------------------------------

/// The scale factor (in kilometers) for a named length unit, mirroring
/// `celastro::getLengthScale`. Returns `None` for unknown units.
pub fn length_scale(unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        "km" | "kilometer" | "kilometers" => Some(1.0),
        "m" | "meter" | "meters" => Some(1e-3),
        "cm" | "centimeter" | "centimeters" => Some(1e-5),
        "earthradius" | "earthradii" | "earthr" => Some(EARTH_RADIUS),
        "jupiterradius" | "jupiterradii" => Some(JUPITER_RADIUS),
        "solarradius" | "solarradii" | "solar" => Some(SOLAR_RADIUS),
        "au" | "astronomicalunit" => Some(KM_PER_AU),
        "ly" | "lightyear" | "lightyears" => Some(KM_PER_LY),
        "pc" | "parsec" | "parsecs" => Some(KM_PER_PARSEC),
        "kpc" | "kiloparsec" | "kiloparsecs" => Some(1e3 * KM_PER_PARSEC),
        "mpc" | "megaparsec" | "megaparsecs" => Some(1e6 * KM_PER_PARSEC),
        _ => None,
    }
}

/// The scale factor (in days) for a named time unit, mirroring
/// `celastro::getTimeScale`. Returns `None` for unknown units.
pub fn time_scale(unit: &str) -> Option<f64> {
    match unit.to_lowercase().as_str() {
        "s" | "sec" | "second" | "seconds" => Some(1.0 / SECONDS_PER_DAY),
        "min" | "minute" | "minutes" => Some(1.0 / MINUTES_PER_DAY),
        "hr" | "hour" | "hours" => Some(1.0 / HOURS_PER_DAY),
        "day" | "days" => Some(1.0),
        "yr" | "year" | "years" | "jy" | "julianyear" | "julianyears" => Some(DAYS_PER_YEAR),
        _ => None,
    }
}

// --- Time (celastro/date.h) --------------------------------------------------

/// The J2000 epoch: 12 UT on 1 January 2000, as a Julian date.
pub const J2000: f64 = 2451545.0;

/// Length of a Julian year, in days.
pub const DAYS_PER_YEAR: f64 = 365.25;

/// Seconds in a day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const MINUTES_PER_DAY: f64 = 1440.0;
pub const HOURS_PER_DAY: f64 = 24.0;

/// Convert seconds to days.
pub const fn secs_to_days(s: f64) -> f64 {
    s * (1.0 / SECONDS_PER_DAY)
}

/// Convert days to seconds.
pub const fn days_to_secs(d: f64) -> f64 {
    d * SECONDS_PER_DAY
}

/// Convert seconds to Julian days.
pub const fn seconds_to_julian_date(sec: f64) -> f64 {
    sec / SECONDS_PER_DAY
}

/// Convert a Julian date to seconds.
pub const fn julian_date_to_seconds(jd: f64) -> f64 {
    jd * SECONDS_PER_DAY
}

/// A calendar date, mirroring `celastro::Date`.
///
/// Fields follow the C++ `Date` struct; only the fields needed by the current
/// port are populated by the constructors, but the layout is kept identical so
/// the type can grow to a full calendar/date formatter later.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Date {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub wday: i32,
    pub utc_offset: i32,
    pub tzname: String,
    pub seconds: f64,
}

impl Date {
    /// Create a date from a Julian date.
    ///
    /// Faithful port of Celestia's `celastro::Date(double jd)` constructor
    /// (`celastro/date.cpp`), which uses the Fliegel & van Flandern algorithm
    /// with 64-bit integer intermediate arithmetic.
    pub fn from_julian_date(jd: f64) -> Self {
        let a = (jd + 0.5).floor() as i64;
        let wday = ((a + 1) % 7) as i32;
        let c: i64;
        if a < 2299161 {
            c = a + 1524;
        } else {
            let b = ((a - 1867216) as f64 / 36524.25).floor() as i64;
            c = a + b - b / 4 + 1525;
        }

        let d = ((c - 122) as f64 / 365.25).floor() as i64;
        let e = (365.25 * d as f64).floor() as i64;
        let f = ((c - e) as f64 / 30.6001).floor() as i64;

        // Note: Celestia computes the fractional day by subtracting the
        // floored day count back from `jd + 0.5`.
        let dday =
            (c - e - ((30.6001 * f as f64).floor() as i64)) as f64 + ((jd + 0.5) - a as f64);

        // Celestia's comment: "This following used to be 14.0, but gcc was
        // computing it incorrectly, so it was changed to 14".
        let month = (f - 1 - 12 * (f / 14)) as i32;
        let year = (d - 4715 - ((7.0 + month as f64) / 10.0) as i64) as i32;
        let day = dday as i32;

        let dhour = (dday - day as f64) * 24.0;
        let hour = dhour as i32;

        let dminute = (dhour - hour as f64) * 60.0;
        let minute = dminute as i32;

        let seconds = (dminute - minute as f64) * 60.0;

        Self {
            year,
            month,
            day,
            hour,
            minute,
            wday,
            utc_offset: 0,
            tzname: "UTC".to_string(),
            seconds,
        }
    }

    /// Convert to a Julian date.
    ///
    /// Faithful port of Celestia's `celastro::Date::operator double()`
    /// (`celastro/date.cpp`), including the 1582 Gregorian-reform correction.
    pub fn to_julian_date(self) -> f64 {
        let y = self.year as i64;
        let m = self.month as i64;
        let (y, m) = if m <= 2 {
            (y - 1, m + 12)
        } else {
            (y, m)
        };

        // Correct for the lost days in Oct 1582 when the Gregorian calendar
        // replaced the Julian calendar.
        let b = if self.year > 1582
            || (self.year == 1582 && (self.month > 10 || (self.month == 10 && self.day >= 15)))
        {
            y / 400 - y / 100
        } else {
            -2
        };

        (365.25 * y as f64)
            .floor()
            + (30.6001 * (m + 1) as f64).floor()
            + b as f64
            + 1720996.5
            + self.day as f64
            + self.hour as f64 / HOURS_PER_DAY
            + self.minute as f64 / MINUTES_PER_DAY
            + self.seconds / SECONDS_PER_DAY
    }
}

/// Parse a date string of the form `YYYY-M-D` (or `YYYY/M/D`), matching the
/// most common date literal used in Celestia data files. Returns `None` on
/// failure.
pub fn parse_date(s: &str) -> Option<Date> {
    let parts: Vec<&str> = s.split(['-', '/', '.']).collect();
    if parts.len() < 3 {
        return None;
    }
    let year = parts[0].trim().parse::<i32>().ok()?;
    let month = parts[1].trim().parse::<i32>().ok()?;
    let day = parts[2].trim().parse::<f64>().ok()?;
    Some(Date {
        year,
        month,
        day: day as i32,
        hour: 0,
        minute: 0,
        wday: 0,
        utc_offset: 0,
        tzname: "UTC".to_string(),
        seconds: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anomaly_circular_orbit() {
        // e = 0 => true anomaly equals mean anomaly.
        let m = 1.2345_f64;
        let (ta, ea) = anomaly(m, 0.0);
        assert!((ta - m).abs() < 1e-9);
        assert!((ea - m).abs() < 1e-9);
    }

    #[test]
    fn anomaly_round_trips_mean_anomaly() {
        // For a range of eccentricities, the recovered mean anomaly
        // M = E - e·sin E should match the input.
        for &e in &[0.0_f64, 0.05, 0.2, 0.5, 0.8, 0.95] {
            for m in [0.0_f64, 1.0, 2.0, 4.0, 6.0] {
                let (_, ea) = anomaly(m, e);
                let m_recovered = ea - e * ea.sin();
                // The recovered anomaly is defined modulo 2π.
                let diff = (m_recovered - m).rem_euclid(2.0 * PI);
                let wrapped = diff.min(2.0 * PI - diff);
                assert!(
                    wrapped < 1e-6,
                    "e={e}, m={m}: recovered {m_recovered}, diff {diff}"
                );
            }
        }
    }

    #[test]
    fn distance_conversions_round_trip() {
        let km = au_to_kilometers(1.0);
        assert!((km - KM_PER_AU).abs() < 1e-3);
        assert!((kilometers_to_au(km) - 1.0).abs() < 1e-15);

        let km = light_years_to_kilometers(1.0);
        assert!((kilometers_to_light_years(km) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn j2000_date_round_trip() {
        let d = Date::from_julian_date(J2000);
        // J2000 = 2000-01-01 12:00.
        assert_eq!(d.year, 2000);
        assert_eq!(d.month, 1);
        assert_eq!(d.day, 1);
        assert_eq!(d.hour, 12);
        // Round trip.
        assert!((d.to_julian_date() - J2000).abs() < 1e-6);
    }

    #[test]
    fn parse_date_basic() {
        let d = parse_date("2000-1-1").expect("parse");
        assert_eq!(d.year, 2000);
        assert_eq!(d.month, 1);
        assert_eq!(d.day, 1);
    }
}
